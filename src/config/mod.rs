//! Loads `vocabulary.toml`, `actions.toml`, and `targets.toml` into the one
//! lookup table the rest of the daemon reads: which action a key combo or
//! chord triggers, which target (if any) matches a focused process, and
//! what to fall back to when nothing can carry an action out natively.

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

use action::{RawAction, Trigger, Vocabulary};

#[derive(Debug, Deserialize, Default)]
struct ActionsFile {
    #[serde(default)]
    action: Vec<RawAction>,
}

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
        let vocabulary: Vocabulary = load_toml(&config_dir.join("vocabulary.toml"))?;
        let actions: ActionsFile = load_toml(&config_dir.join("actions.toml"))?;
        let targets: TargetsFile = load_toml(&config_dir.join("targets.toml"))?;

        let mut registry = Registry::default();
        for raw in actions.action {
            let action = raw.resolve(&vocabulary).map_err(ConfigError::new)?;
            match action.trigger {
                Some(Trigger::Key(combo)) => {
                    registry.triggers.insert(combo, action.id.clone());
                }
                Some(Trigger::Chord(chord)) => {
                    registry
                        .chord_members
                        .extend(chord.tokens().map(String::from));
                    registry.chords.insert(chord, action.id.clone());
                }
                None => {}
            }
            registry.actions.insert(action.id, action.fallback);
        }

        let known_actions: HashSet<&str> = registry.actions.keys().map(String::as_str).collect();
        let mut targets = targets.target;
        for target in &mut targets {
            target.resolve(config_dir, &known_actions)?;
        }
        registry.targets = targets;
        Ok(registry)
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

    /// Writes a throwaway config dir and loads it, so a test can state the
    /// TOML it cares about and nothing else.
    fn load(name: &str, actions: &str, targets: &str) -> Result<Registry, ConfigError> {
        let dir = std::env::temp_dir().join(format!("keylex-config-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("vocabulary.toml"),
            "modifiers = [\"close\", \"save\", \"toggle\"]\nlocations = [\"tab\", \"pane\"]\n",
        )
        .unwrap();
        std::fs::write(dir.join("actions.toml"), actions).unwrap();
        std::fs::write(dir.join("targets.toml"), targets).unwrap();
        let result = Registry::load(&dir);
        let _ = std::fs::remove_dir_all(&dir);
        result
    }

    #[test]
    fn shipped_config_loads_and_resolves_lookups() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("config");
        let registry = Registry::load(&dir).expect("shipped config should load");

        let combo = KeyCombo::parse("ctrl+w").unwrap();
        assert_eq!(registry.action_for_trigger(&combo), Some("close.tab"));
        assert_eq!(
            registry.fallback("close.tab"),
            Some(&Fallback::Keycode {
                combo,
                notify: false
            })
        );

        let target = registry
            .target_for_process("Code.exe")
            .expect("vscode target");
        assert_eq!(target.program, "vscode");
        assert_eq!(
            target.supports.get("close.tab").map(String::as_str),
            Some("workbench.action.closeActiveEditor")
        );
        assert!(registry.target_for_process("unknown.exe").is_none());
    }

    #[test]
    fn action_id_is_derived_from_modifier_and_location() {
        let registry = load(
            "derived-id",
            "[[action]]\nmodifier = \"close\"\nlocation = \"tab\"\n[[action]]\nmodifier = \"save\"\n",
            "",
        )
        .expect("valid vocabulary words should load");
        assert!(registry.has_action("close.tab"));
        assert!(registry.has_action("save"));
    }

    #[test]
    fn words_outside_the_vocabulary_are_rejected() {
        assert!(load(
            "unknown-modifier",
            "[[action]]\nmodifier = \"frobnicate\"\n",
            ""
        )
        .is_err());
        assert!(load(
            "unknown-location",
            "[[action]]\nmodifier = \"close\"\nlocation = \"nowhere\"\n",
            ""
        )
        .is_err());
    }

    #[test]
    fn a_chord_matches_whatever_order_its_keys_arrive_in() {
        let registry = load(
            "chord",
            "[[action]]\nmodifier = \"toggle\"\nlocation = \"pane\"\nchord = [\"f\", \"ctrl\", \"d\"]\n",
            "",
        )
        .expect("chord config should load");

        let chord: Chord = ["ctrl", "d", "f"].iter().map(|t| t.to_string()).collect();
        assert_eq!(registry.action_for_chord(&chord), Some("toggle.pane"));
        assert!(registry.is_chord_member("d"));
        assert!(!registry.is_chord_member("x"));
        assert_eq!(
            registry.trigger_for_action("toggle.pane").as_deref(),
            Some("ctrl+d+f")
        );
    }

    #[test]
    fn a_target_is_held_to_the_actions_and_command_grammar() {
        let supports = |command: &str| {
            format!("[[target]]\nprogram = \"app\"\nadapter = \"socket\"\n\n[target.supports]\n\"close.tab\" = \"{command}\"\n")
        };
        let action = "[[action]]\nmodifier = \"close\"\nlocation = \"tab\"\n";

        assert!(load("grammar-ok", action, &supports("app.tab.close")).is_ok());
        assert!(load("grammar-bad", action, &supports("closeIt")).is_err());
        assert!(load("unknown-action", "", &supports("app.tab.close")).is_err());
    }

    #[test]
    fn an_exempt_target_keeps_its_upstream_command_names() {
        let registry = load(
            "exempt",
            "[[action]]\nmodifier = \"close\"\nlocation = \"tab\"\n",
            "[[target]]\nprogram = \"vscode\"\nadapter = \"socket\"\nexempt_command_grammar = true\n\n[target.supports]\n\"close.tab\" = \"workbench.action.closeActiveEditor\"\n",
        )
        .expect("exempt target should skip the grammar check");
        assert_eq!(
            registry.targets()[0]
                .supports
                .get("close.tab")
                .map(String::as_str),
            Some("workbench.action.closeActiveEditor")
        );
    }
}
