//! Decides how an abstract action is carried out for the currently focused
//! program: native adapter -> keycode fallback -> notify.

use std::collections::HashMap;
use std::fmt;

use crate::config::{AdapterKind, Fallback, KeyCombo, Registry, Target};

/// What a dispatch actually did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The target carried the action out through its own API, via the
    /// native command named here.
    Native(String),
    /// No native route, so this keycode was synthesized instead.
    Fallback(KeyCombo),
    /// Nothing was sent; the payload says why.
    Unsupported(String),
}

impl fmt::Display for Outcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Outcome::Native(command) => write!(f, "native: {command}"),
            Outcome::Fallback(combo) => write!(f, "fallback: {combo}"),
            Outcome::Unsupported(reason) => write!(f, "unsupported: {reason}"),
        }
    }
}

/// One transport reaching a target. Implementations live in
/// `src/adapters/`, one per `AdapterKind`.
pub trait Adapter {
    fn send(&self, target: &Target, native_command: &str);
}

pub trait Notifier {
    fn show(&self, message: &str);
}

/// Synthesizes a key combo system-wide, for actions no target can carry
/// out natively. Implemented by the capture backends, which already own
/// the virtual device / injection API needed to do it.
pub trait FallbackSender {
    fn send(&self, combo: &KeyCombo);
}

pub type Adapters = HashMap<AdapterKind, Box<dyn Adapter>>;

pub struct Router<'a> {
    registry: &'a Registry,
    adapters: Adapters,
    notifier: Box<dyn Notifier>,
    fallback_sender: Box<dyn FallbackSender>,
}

impl<'a> Router<'a> {
    pub fn new(
        registry: &'a Registry,
        adapters: Adapters,
        notifier: Box<dyn Notifier>,
        fallback_sender: Box<dyn FallbackSender>,
    ) -> Router<'a> {
        Router {
            registry,
            adapters,
            notifier,
            fallback_sender,
        }
    }

    pub fn registry(&self) -> &'a Registry {
        self.registry
    }

    /// The focused app's own target gets first refusal; the OS-wide system
    /// listener catches actions that belong to no single app (`shutdown`,
    /// `move.left`); anything left falls back to a raw keycode.
    pub fn dispatch(&self, action_id: &str, focused_process: Option<&str>) -> Outcome {
        let focused = focused_process.and_then(|name| self.registry.target_for_process(name));
        let native = [focused, self.registry.system_target()]
            .into_iter()
            .flatten()
            .find_map(|target| Some((target, target.supports.get(action_id)?)));

        if let Some((target, native_command)) = native {
            return self.send_native(target, native_command);
        }

        match self.registry.fallback(action_id) {
            Some(Fallback::Keycode { combo, notify }) => {
                self.fallback_sender.send(combo);
                if *notify {
                    self.notifier
                        .show(&format!("Fallback attempted: {action_id}"));
                }
                Outcome::Fallback(combo.clone())
            }
            Some(Fallback::Unsupported) | None => {
                self.notifier
                    .show(&format!("Action not supported: {action_id}"));
                Outcome::Unsupported(action_id.to_string())
            }
        }
    }

    /// Sends one native command straight to `target`, skipping action
    /// resolution -- also the path a raw spotlight entry takes, since a
    /// target's own command has no abstract action to route by focus.
    pub fn send_native(&self, target: &Target, native_command: &str) -> Outcome {
        match self.adapters.get(&target.adapter) {
            Some(adapter) => {
                adapter.send(target, native_command);
                Outcome::Native(native_command.to_string())
            }
            None => Outcome::Unsupported(format!("no {} adapter available", target.adapter)),
        }
    }
}
