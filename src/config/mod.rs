//! Loads `targets.toml` into the one lookup table the rest of the daemon
//! reads: which target (if any) matches a focused process, and what to fall
//! back to when nothing can carry an action out natively.
//!
//! There used to be a `vocabulary.toml`/`actions.toml` layer here too,
//! statically declaring every valid action id and its physical key/chord
//! trigger. That's gone in favor of live discovery (see CLAUDE.md's "Known
//! gaps"): a target now reports what it supports via the `list_actions`
//! handshake instead of Keylex holding a static copy, and no config file
//! populates the action/trigger/chord tables below yet -- `config/
//! keymap.toml` is a restored, unwired skeleton pending that redesign.

mod action;
mod error;
mod key;
mod target;

pub use action::Fallback;
pub use error::ConfigError;
pub use key::{is_modifier, Chord, KeyCombo};
pub use target::{AdapterKind, Target};

use std::collections::{HashMap, HashSet};
use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
struct TargetsFile {
    #[serde(default)]
    target: Vec<Target>,
}

fn load_toml<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, ConfigError> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| ConfigError::new(format!("could not read {}: {e}", path.display())))?;
    toml::from_str(&text)
        .map_err(|e| ConfigError::new(format!("could not parse {}: {e}", path.display())))
}

#[derive(Debug, Default)]
pub struct Registry {
    actions: HashMap<String, Fallback>,
    targets: Vec<Target>,
    triggers: HashMap<KeyCombo, String>,
    chords: HashMap<Chord, String>,
    /// Union of every token appearing in any configured chord, cached at
    /// load time so the capture backends can answer "is this key part of
    /// any chord?" in O(1) per keystroke instead of scanning `chords`.
    chord_members: HashSet<String>,
}

impl Registry {
    pub fn load(config_dir: &Path) -> Result<Registry, ConfigError> {
        let targets: TargetsFile = load_toml(&config_dir.join("targets.toml"))?;

        // actions/triggers/chords stay empty -- no config file populates
        // them yet, see this module's doc comment.
        Ok(Registry {
            targets: targets.target,
            ..Registry::default()
        })
    }

    /// Builds a `Registry` with a hand-picked action table and nothing
    /// else, for unit-testing `Router::dispatch`'s fallback behavior
    /// without a config file -- there's no TOML format left to declare a
    /// `Fallback` in (see this module's doc comment).
    #[cfg(test)]
    pub(crate) fn with_actions(actions: HashMap<String, Fallback>) -> Registry {
        Registry {
            actions,
            ..Registry::default()
        }
    }

    /// Same as `with_actions`, plus a key -> action id trigger table, for
    /// tests (e.g. `spotlight`'s) that also need `trigger_for_action` to
    /// resolve.
    #[cfg(test)]
    pub(crate) fn with_actions_and_triggers(
        actions: HashMap<String, Fallback>,
        triggers: HashMap<KeyCombo, String>,
    ) -> Registry {
        Registry {
            actions,
            triggers,
            ..Registry::default()
        }
    }

    pub fn has_action(&self, action_id: &str) -> bool {
        self.actions.contains_key(action_id)
    }

    /// What to do when no target can carry `action_id` out natively.
    /// `None` means no such action is configured at all.
    pub fn fallback(&self, action_id: &str) -> Option<&Fallback> {
        self.actions.get(action_id)
    }

    /// Every configured action id -- the baseline catalog
    /// `spotlight::Index` builds its search entries from.
    pub fn action_ids(&self) -> impl Iterator<Item = &str> {
        self.actions.keys().map(String::as_str)
    }

    pub fn targets(&self) -> &[Target] {
        &self.targets
    }

    pub fn target_for_process(&self, process_name: &str) -> Option<&Target> {
        self.targets
            .iter()
            .find(|target| target.match_process.iter().any(|p| p == process_name))
    }

    /// The OS-wide system listener for the platform this daemon is actually
    /// running on, independent of which app has focus -- used for actions
    /// like `shutdown` or `move.left` that aren't scoped to one app.
    pub fn system_target(&self) -> Option<&Target> {
        self.targets
            .iter()
            .find(|target| target.os.as_deref() == Some(std::env::consts::OS))
    }

    pub fn action_for_trigger(&self, combo: &KeyCombo) -> Option<&str> {
        self.triggers.get(combo).map(String::as_str)
    }

    pub fn action_for_chord(&self, chord: &Chord) -> Option<&str> {
        self.chords.get(chord).map(String::as_str)
    }

    /// Whether this key token belongs to any configured chord -- the
    /// hot-path check deciding whether a keystroke has to enter the
    /// pending/debounce state at all.
    pub fn is_chord_member(&self, token: &str) -> bool {
        self.chord_members.contains(token)
    }

    /// Whether `chord` could still grow into a configured one if more keys
    /// join it, i.e. whether it is a subset of some configured chord.
    pub fn is_chord_prefix(&self, chord: &Chord) -> bool {
        self.chords
            .keys()
            .any(|configured| chord.is_subset_of(configured))
    }

    /// The key or chord bound to `action_id`, for display in the spotlight
    /// catalog. Dispatch only ever needs the reverse lookup, so this scans
    /// the (small) trigger maps rather than keeping a second index in sync.
    pub fn trigger_for_action(&self, action_id: &str) -> Option<String> {
        let combo = self.triggers.iter().find(|(_, id)| *id == action_id);
        if let Some((combo, _)) = combo {
            return Some(combo.to_string());
        }
        self.chords
            .iter()
            .find(|(_, id)| *id == action_id)
            .map(|(chord, _)| chord.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load(name: &str, targets: &str) -> Result<Registry, ConfigError> {
        let dir = std::env::temp_dir().join(format!("keylex-config-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("targets.toml"), targets).unwrap();
        let result = Registry::load(&dir);
        let _ = std::fs::remove_dir_all(&dir);
        result
    }

    #[test]
    fn shipped_config_loads_and_resolves_lookups() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("config");
        let registry = Registry::load(&dir).expect("shipped config should load");

        let target = registry
            .target_for_process("Code.exe")
            .expect("vscode target");
        assert_eq!(target.program, "vscode");
        assert!(registry.target_for_process("unknown.exe").is_none());
    }

    #[test]
    fn a_target_with_no_supports_still_loads() {
        let registry = load(
            "no-supports",
            "[[target]]\nprogram = \"app\"\nadapter = \"socket\"\n",
        )
        .expect("target with no capabilities should still load");
        assert_eq!(registry.targets()[0].program, "app");
    }

    #[test]
    fn the_system_target_is_picked_by_os() {
        let registry = load(
            "system-target",
            &format!(
                "[[target]]\nprogram = \"system-{os}\"\nos = \"{os}\"\nadapter = \"socket\"\n",
                os = std::env::consts::OS
            ),
        )
        .expect("system target should load");
        assert!(registry.system_target().is_some());
    }
}
