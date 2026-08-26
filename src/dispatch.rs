//! Decides how an abstract action is carried out for the currently
//! focused program: native adapter -> keycode fallback -> notify.

use crate::config::{Registry, Target};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DispatchStatus {
    Native,
    Fallback,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchResult {
    pub status: DispatchStatus,
    pub detail: String,
}

pub trait Adapter {
    fn send(&self, target: &Target, native_command: &str);
}

pub trait Notifier {
    fn show(&self, message: &str);
}

pub trait FallbackSender {
    fn send(&self, keycode: &str);
}

pub struct Router<'a> {
    pub registry: &'a Registry,
    pub adapters: std::collections::HashMap<String, Box<dyn Adapter>>,
    pub notifier: Box<dyn Notifier>,
    pub fallback_sender: Box<dyn FallbackSender>,
}

impl<'a> Router<'a> {
    pub fn dispatch(&self, action_id: &str, focused_process: &str) -> DispatchResult {
        let target = self.registry.target_for_process(focused_process);
        if let Some(target) = target {
            if target.supports.contains_key(action_id) {
                return self.dispatch_native(target, action_id);
            }
        }

        // Not scoped to (or not supported by) the focused app -- try the
        // OS-wide system listener before falling back to a keycode guess.
        if let Some(system_target) = self.registry.system_target() {
            if system_target.supports.contains_key(action_id) {
                return self.dispatch_native(system_target, action_id);
            }
        }

        let spec = self.registry.action_spec(action_id);
        self.dispatch_fallback(action_id, &spec.fallback_tier, spec.fallback_keycode.as_deref())
    }

    fn dispatch_native(&self, target: &Target, action_id: &str) -> DispatchResult {
        let native_command = target.supports.get(action_id).expect("checked by caller");
        match self.adapters.get(&target.adapter) {
            Some(adapter) => {
                adapter.send(target, native_command);
                DispatchResult {
                    status: DispatchStatus::Native,
                    detail: native_command.clone(),
                }
            }
            None => DispatchResult {
                status: DispatchStatus::Unsupported,
                detail: format!("adapter {} missing", target.adapter),
            },
        }
    }

    fn dispatch_fallback(
        &self,
        action_id: &str,
        fallback_tier: &str,
        fallback_keycode: Option<&str>,
    ) -> DispatchResult {
        let keycode = match (fallback_tier, fallback_keycode) {
            ("notify_only", _) | (_, None) => {
                self.notifier.show(&format!("Action not supported: {action_id}"));
                return DispatchResult {
                    status: DispatchStatus::Unsupported,
                    detail: action_id.to_string(),
                };
            }
            (_, Some(keycode)) => keycode,
        };

        self.fallback_sender.send(keycode);

        if fallback_tier == "notify_attempt" {
            self.notifier.show(&format!("Fallback attempted: {action_id}"));
        }

        DispatchResult {
            status: DispatchStatus::Fallback,
            detail: keycode.to_string(),
        }
    }
}
