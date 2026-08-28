//! What happens when no target can carry an action out natively. Currently
//! unused in practice -- nothing populates `Registry`'s action table yet,
//! since the static `vocabulary.toml`/`actions.toml` layer was removed in
//! favor of live discovery (see CLAUDE.md's "Known gaps"). The type stays so
//! `Router::dispatch`'s fallback path has something to match on once
//! `config/keymap.toml` grows a loader.

use super::key::KeyCombo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fallback {
    /// Nothing safe to synthesize -- either the action is `notify_only`, or
    /// it named no `fallback_keycode` to send.
    Unsupported,
    Keycode {
        combo: KeyCombo,
        notify: bool,
    },
}
