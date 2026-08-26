//! Loads `actions.toml` and `targets.toml` and exposes lookups: which
//! action a key combo triggers, which target (if any) matches a focused
//! process, and an action's fallback behavior.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt;
use std::path::Path;

use serde::Deserialize;

/// A "ctrl+shift+w"-style combo: one main key plus zero or more modifiers,
/// order-independent. The same syntax is used both for a key that triggers
/// an action and for the keycode sent by the fallback path.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyCombo {
    pub key: String,
    pub modifiers: BTreeSet<String>,
}

impl KeyCombo {
    pub fn parse(raw: &str) -> KeyCombo {
        let mut tokens: Vec<String> = raw
            .split('+')
            .map(|t| t.trim().to_lowercase())
            .filter(|t| !t.is_empty())
            .collect();
        let key = tokens.pop().unwrap_or_default();
        KeyCombo {
            key,
            modifiers: tokens.into_iter().collect(),
        }
    }
}

impl fmt::Display for KeyCombo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for m in &self.modifiers {
            write!(f, "{m}+")?;
        }
        write!(f, "{}", self.key)
    }
}

/// Token names recognized as modifiers, shared with the capture backends'
/// own modifier tables (`src/capture/linux.rs`, `src/capture/windows.rs`).
/// Used here only to reject a chord made of modifiers alone -- a chord is
/// meant to express "keys held together", not just held-down modifiers.
pub const MODIFIER_NAMES: &[&str] = &["ctrl", "shift", "alt", "win"];

/// A set of key tokens (modifiers and/or plain keys) that must all be held
/// down together, order-independent, to trigger an action. Distinct from
/// `KeyCombo`, whose `key`/`modifiers` split privileges one token as "the
/// one whose down-edge triggers the check" -- a real chord has no such
/// privileged member.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChordSpec {
    pub keys: BTreeSet<String>,
}

#[derive(Debug, Clone)]
pub struct ActionSpec {
    pub id: String,
    pub fallback_tier: String, // silent | notify_attempt | notify_only
    pub fallback_keycode: Option<String>,
}

impl ActionSpec {
    fn unbound(id: &str) -> ActionSpec {
        ActionSpec {
            id: id.to_string(),
            fallback_tier: "notify_attempt".to_string(),
            fallback_keycode: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Target {
    pub program: String,
    #[serde(default)]
    pub match_process: Vec<String>,
    pub adapter: String,
    #[serde(default)]
    pub supports: HashMap<String, String>,
    /// Adapter-specific fields not common to every target: `address` for
    /// the socket adapter; `port` and, optionally, `allowed_origin` (see
    /// `Target::allowed_origin`) for the websocket adapter.
    #[serde(flatten)]
    pub extra: HashMap<String, toml::Value>,
}

impl Target {
    /// The websocket adapter's optional `allowed_origin` field from `extra`
    /// -- when set, `WebSocketAdapter` rejects any handshake whose `Origin`
    /// header doesn't match exactly (docs/protocol.md#trust-model--authentication).
    pub fn allowed_origin(&self) -> Option<&str> {
        self.extra.get("allowed_origin").and_then(|v| v.as_str())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SystemAction {
    pub id: String,
    #[serde(flatten)]
    pub extra: HashMap<String, toml::Value>,
}

#[derive(Debug, Deserialize)]
struct RawAction {
    id: String,
    key: Option<String>,
    /// Order-independent multi-key chord trigger, mutually exclusive with
    /// `key`. See `ChordSpec`.
    chord: Option<Vec<String>>,
    #[serde(default = "default_fallback_tier")]
    fallback_tier: String,
    fallback_keycode: Option<String>,
}

fn default_fallback_tier() -> String {
    "notify_attempt".to_string()
}

#[derive(Debug, Deserialize, Default)]
struct RawActionsFile {
    #[serde(default)]
    action: Vec<RawAction>,
}

#[derive(Debug, Deserialize, Default)]
struct RawTargetsFile {
    #[serde(default)]
    target: Vec<Target>,
    #[serde(default)]
    system_action: Vec<SystemAction>,
}

#[derive(Debug)]
pub enum ConfigError {
    Io(String, std::io::Error),
    Toml(String, toml::de::Error),
    InvalidChord(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io(path, e) => write!(f, "could not read {path}: {e}"),
            ConfigError::Toml(path, e) => write!(f, "could not parse {path}: {e}"),
            ConfigError::InvalidChord(msg) => write!(f, "invalid chord binding: {msg}"),
        }
    }
}

impl std::error::Error for ConfigError {}

fn load_toml<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, ConfigError> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| ConfigError::Io(path.display().to_string(), e))?;
    toml::from_str(&text).map_err(|e| ConfigError::Toml(path.display().to_string(), e))
}

#[derive(Debug)]
pub struct Registry {
    actions: HashMap<String, ActionSpec>,
    pub targets: Vec<Target>,
    pub system_actions: HashMap<String, SystemAction>,
    triggers: HashMap<KeyCombo, String>,
    chord_triggers: HashMap<BTreeSet<String>, String>,
    /// Union of every token appearing in any configured chord (modifiers and
    /// plain keys alike), cached at load time so the capture backends can do
    /// an O(1) "is this key part of any chord?" check per keystroke instead
    /// of scanning `chord_triggers` on every event.
    chord_member_keys: HashSet<String>,
}

fn validate_chord(action_id: &str, tokens: &[String]) -> Result<BTreeSet<String>, ConfigError> {
    if tokens.len() < 2 {
        return Err(ConfigError::InvalidChord(format!(
            "action {action_id:?} chord needs at least 2 keys, got {}",
            tokens.len()
        )));
    }

    let mut keys = BTreeSet::new();
    for token in tokens {
        let normalized = token.trim().to_lowercase();
        if !keys.insert(normalized.clone()) {
            return Err(ConfigError::InvalidChord(format!(
                "action {action_id:?} chord has duplicate key {normalized:?}"
            )));
        }
    }

    if keys.iter().all(|k| MODIFIER_NAMES.contains(&k.as_str())) {
        return Err(ConfigError::InvalidChord(format!(
            "action {action_id:?} chord must include at least one non-modifier key, got only modifiers {keys:?}"
        )));
    }

    Ok(keys)
}

impl Registry {
    pub fn load(config_dir: &Path) -> Result<Registry, ConfigError> {
        let actions_file: RawActionsFile = load_toml(&config_dir.join("actions.toml"))?;
        let targets_file: RawTargetsFile = load_toml(&config_dir.join("targets.toml"))?;

        let mut actions = HashMap::new();
        let mut triggers = HashMap::new();
        let mut chord_triggers = HashMap::new();
        let mut chord_member_keys = HashSet::new();
        for raw in actions_file.action {
            if raw.key.is_some() && raw.chord.is_some() {
                return Err(ConfigError::InvalidChord(format!(
                    "action {:?} has both 'key' and 'chord' set; they are mutually exclusive",
                    raw.id
                )));
            }
            if let Some(key) = &raw.key {
                triggers.insert(KeyCombo::parse(key), raw.id.clone());
            }
            if let Some(chord) = &raw.chord {
                let keys = validate_chord(&raw.id, chord)?;
                chord_member_keys.extend(keys.iter().cloned());
                chord_triggers.insert(keys, raw.id.clone());
            }
            actions.insert(
                raw.id.clone(),
                ActionSpec {
                    id: raw.id,
                    fallback_tier: raw.fallback_tier,
                    fallback_keycode: raw.fallback_keycode,
                },
            );
        }

        let system_actions = targets_file
            .system_action
            .into_iter()
            .map(|sa| (sa.id.clone(), sa))
            .collect();

        Ok(Registry {
            actions,
            targets: targets_file.target,
            system_actions,
            triggers,
            chord_triggers,
            chord_member_keys,
        })
    }

    pub fn action_spec(&self, action_id: &str) -> ActionSpec {
        self.actions
            .get(action_id)
            .cloned()
            .unwrap_or_else(|| ActionSpec::unbound(action_id))
    }

    pub fn target_for_process(&self, process_name: &str) -> Option<&Target> {
        self.targets
            .iter()
            .find(|t| t.match_process.iter().any(|p| p == process_name))
    }

    /// The action id bound to this physical key combo, if any.
    pub fn action_for_trigger(&self, combo: &KeyCombo) -> Option<&str> {
        self.triggers.get(combo).map(|s| s.as_str())
    }

    /// The action id bound to this exact chord (order-independent key set),
    /// if any.
    pub fn action_for_chord(&self, keys: &BTreeSet<String>) -> Option<&str> {
        self.chord_triggers.get(keys).map(|s| s.as_str())
    }

    /// Whether this key token is a member of any configured chord -- the
    /// hot-path check capture backends use to decide whether a keystroke
    /// needs to enter the pending/debounce state at all.
    pub fn is_chord_member(&self, token: &str) -> bool {
        self.chord_member_keys.contains(token)
    }

    /// Whether `keys` could still become a configured chord if more of its
    /// members get held down together, i.e. `keys` is a (non-strict) subset
    /// of some configured chord's key-set. Capture backends use this to
    /// decide whether to keep a pending chord waiting or give up on it.
    pub fn is_chord_prefix(&self, keys: &BTreeSet<String>) -> bool {
        self.chord_triggers.keys().any(|chord_keys| keys.is_subset(chord_keys))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn key_combo_parses_modifiers_order_independently() {
        let a = KeyCombo::parse("ctrl+shift+w");
        let b = KeyCombo::parse("shift+ctrl+w");
        assert_eq!(a, b);
        assert_eq!(a.key, "w");
        assert_eq!(a.modifiers.len(), 2);
    }

    #[test]
    fn key_combo_handles_bare_key() {
        let c = KeyCombo::parse("prtsc");
        assert_eq!(c.key, "prtsc");
        assert!(c.modifiers.is_empty());
    }

    #[test]
    fn loads_config_dir_and_resolves_lookups() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("config");
        let registry = Registry::load(&dir).expect("config should load");

        let spec = registry.action_spec("close.tab");
        assert_eq!(spec.fallback_tier, "silent");
        assert_eq!(spec.fallback_keycode.as_deref(), Some("ctrl+w"));

        let action_id = registry
            .action_for_trigger(&KeyCombo::parse("ctrl+w"))
            .expect("ctrl+w should be bound");
        assert_eq!(action_id, "close.tab");

        let target = registry.target_for_process("Code.exe").expect("vscode target");
        assert_eq!(target.program, "vscode");
        assert_eq!(
            target.supports.get("close.tab").map(String::as_str),
            Some("workbench.action.closeActiveEditor")
        );

        assert!(registry.target_for_process("unknown.exe").is_none());
    }

    #[test]
    fn unbound_action_id_falls_back_to_default_spec() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("config");
        let registry = Registry::load(&dir).expect("config should load");
        let spec = registry.action_spec("nonexistent.action");
        assert_eq!(spec.fallback_tier, "notify_attempt");
        assert!(spec.fallback_keycode.is_none());
    }

    fn temp_config_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "keylex-config-test-{name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ))
    }

    fn load_actions_only(name: &str, actions_toml: &str) -> Result<Registry, ConfigError> {
        let dir = temp_config_dir(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("actions.toml"), actions_toml).unwrap();
        std::fs::write(dir.join("targets.toml"), "").unwrap();
        let result = Registry::load(&dir);
        let _ = std::fs::remove_dir_all(&dir);
        result
    }

    #[test]
    fn chord_matches_regardless_of_declared_order() {
        let registry = load_actions_only(
            "chord-order",
            "[[action]]\nid = \"some.action\"\nchord = [\"f\", \"ctrl\", \"d\"]\n",
        )
        .expect("chord config should load");

        let keys: BTreeSet<String> = ["ctrl", "d", "f"].into_iter().map(String::from).collect();
        assert_eq!(registry.action_for_chord(&keys), Some("some.action"));
        assert!(registry.is_chord_member("ctrl"));
        assert!(registry.is_chord_member("d"));
        assert!(registry.is_chord_member("f"));
        assert!(!registry.is_chord_member("x"));
    }

    #[test]
    fn chord_and_key_are_mutually_exclusive() {
        let err = load_actions_only(
            "chord-and-key",
            "[[action]]\nid = \"bad\"\nkey = \"ctrl+w\"\nchord = [\"d\", \"f\"]\n",
        )
        .expect_err("both key and chord should be rejected");
        assert!(matches!(err, ConfigError::InvalidChord(_)));
    }

    #[test]
    fn chord_needs_at_least_two_keys() {
        let err = load_actions_only("chord-too-short", "[[action]]\nid = \"bad\"\nchord = [\"d\"]\n")
            .expect_err("single-key chord should be rejected");
        assert!(matches!(err, ConfigError::InvalidChord(_)));
    }

    #[test]
    fn chord_rejects_duplicate_keys() {
        let err = load_actions_only(
            "chord-dupe",
            "[[action]]\nid = \"bad\"\nchord = [\"d\", \"d\"]\n",
        )
        .expect_err("duplicate chord key should be rejected");
        assert!(matches!(err, ConfigError::InvalidChord(_)));
    }

    #[test]
    fn chord_rejects_modifiers_only() {
        let err = load_actions_only(
            "chord-mods-only",
            "[[action]]\nid = \"bad\"\nchord = [\"ctrl\", \"shift\"]\n",
        )
        .expect_err("all-modifier chord should be rejected");
        assert!(matches!(err, ConfigError::InvalidChord(_)));
    }

    #[test]
    fn overlapping_chords_are_allowed() {
        let registry = load_actions_only(
            "chord-overlap",
            r#"[[action]]
id = "first"
chord = ["d", "f"]

[[action]]
id = "second"
chord = ["d", "g"]
"#,
        )
        .expect("overlapping chords should be allowed");

        let df: BTreeSet<String> = ["d", "f"].into_iter().map(String::from).collect();
        let dg: BTreeSet<String> = ["d", "g"].into_iter().map(String::from).collect();
        assert_eq!(registry.action_for_chord(&df), Some("first"));
        assert_eq!(registry.action_for_chord(&dg), Some("second"));
    }
}
