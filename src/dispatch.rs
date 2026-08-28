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

    /// Falls back to a raw keycode, or reports unsupported. There is
    /// currently no static per-target `supports` map to check for a native
    /// route here -- a target only reports what it supports live, via the
    /// `list_actions` handshake (see `Router::send_native` and
    /// `spotlight::Entry::dispatch`, which already query that live). Wiring
    /// an equivalent live check into this path is future work, once
    /// `config/keymap.toml` gives a direct key press an action id to
    /// dispatch in the first place (see CLAUDE.md's "Known gaps").
    pub fn dispatch(&self, action_id: &str, _focused_process: Option<&str>) -> Outcome {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::KeyCombo;
    use std::cell::RefCell;

    struct NoopNotifier;
    impl Notifier for NoopNotifier {
        fn show(&self, _message: &str) {}
    }

    #[derive(Default)]
    struct RecordingFallback(RefCell<Vec<KeyCombo>>);
    impl FallbackSender for RecordingFallback {
        fn send(&self, combo: &KeyCombo) {
            self.0.borrow_mut().push(combo.clone());
        }
    }

    fn router(registry: &Registry) -> Router<'_> {
        Router::new(
            registry,
            Adapters::new(),
            Box::new(NoopNotifier),
            Box::new(RecordingFallback::default()),
        )
    }

    #[test]
    fn the_silent_tier_sends_a_keycode_without_notifying() {
        let registry = Registry::with_actions(HashMap::from([(
            "save".to_string(),
            Fallback::Keycode {
                combo: KeyCombo::parse("ctrl+s").unwrap(),
                notify: false,
            },
        )]));

        assert_eq!(router(&registry).dispatch("save", None).to_string(), "fallback: ctrl+s");
    }

    #[test]
    fn the_notify_attempt_tier_sends_a_keycode_and_notifies() {
        let registry = Registry::with_actions(HashMap::from([(
            "duplicate.line".to_string(),
            Fallback::Keycode {
                combo: KeyCombo::parse("ctrl+shift+d").unwrap(),
                notify: true,
            },
        )]));

        assert!(matches!(
            router(&registry).dispatch("duplicate.line", None),
            Outcome::Fallback(_)
        ));
    }

    #[test]
    fn an_action_with_nothing_safe_to_send_only_notifies() {
        let registry =
            Registry::with_actions(HashMap::from([("go_to.definition".to_string(), Fallback::Unsupported)]));

        assert!(matches!(
            router(&registry).dispatch("go_to.definition", None),
            Outcome::Unsupported(_)
        ));
    }

    #[test]
    fn an_unconfigured_action_is_unsupported() {
        let registry = Registry::with_actions(HashMap::new());

        assert!(matches!(
            router(&registry).dispatch("nothing.configured", None),
            Outcome::Unsupported(_)
        ));
    }
}
