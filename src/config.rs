//! Loads `actions.toml`, `targets.toml`, and `vocabulary.toml`, and exposes
//! lookups: which action a key combo triggers, which target (if any)
//! matches a focused process, and an action's fallback behavior.

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

/// The action-id vocabulary loaded from `vocabulary.toml`: every action's
/// `modifier` and (if present) `location` must appear in these sets, or
/// `Registry::load` rejects the config outright. See
/// `docs/protocol.md#action-ids`.
#[derive(Debug, Clone, Deserialize)]
struct RawVocabulary {
    #[serde(default)]
    modifiers: Vec<String>,
    #[serde(default)]
    locations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Vocabulary {
    pub modifiers: HashSet<String>,
    pub locations: HashSet<String>,
}

impl From<RawVocabulary> for Vocabulary {
    fn from(raw: RawVocabulary) -> Vocabulary {
        Vocabulary {
            modifiers: raw.modifiers.into_iter().collect(),
            locations: raw.locations.into_iter().collect(),
        }
    }
}

/// Builds an action id from a validated `modifier`/`location` pair: just
/// `modifier` when there's no location, otherwise `modifier.location`.
/// The only place an action id string is ever assembled -- see
/// `docs/protocol.md#action-ids`.
fn action_id(modifier: &str, location: Option<&str>) -> String {
    match location {
        Some(location) => format!("{modifier}.{location}"),
        None => modifier.to_string(),
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Target {
    pub program: String,
    #[serde(default)]
    pub match_process: Vec<String>,
    pub adapter: String,
    /// Populated either directly from an inline `[target.supports]` table
    /// (the `neovim` target, which has no `extensions/` folder yet) or, far
    /// more commonly, by `Registry::load` reading the file named by
    /// `capabilities()` -- see `docs/protocol.md#action-ids`.
    #[serde(default)]
    pub supports: HashMap<String, String>,
    /// Adapter-specific fields not common to every target: `address` for
    /// the socket adapter; `port` and, optionally, `allowed_origin` (see
    /// `Target::allowed_origin`) for the websocket adapter; `capabilities`
    /// and `exempt_command_grammar` (see below).
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

    /// The optional `os` field from `extra` (`"linux"` / `"windows"`, matching
    /// `std::env::consts::OS`) that marks a target as the OS-wide system
    /// listener rather than an app tied to a focused process -- see
    /// `Registry::system_target`. A target with no `os` set is never picked
    /// by that lookup, so ordinary app targets don't need this field at all.
    pub fn os(&self) -> Option<&str> {
        self.extra.get("os").and_then(|v| v.as_str())
    }

    /// Path (relative to the config directory) to this target's own
    /// `capabilities.toml`, which owns its action -> native-command map --
    /// see `docs/protocol.md#action-ids`. Absent for a target (like
    /// `neovim`) that still declares `[target.supports]` inline.
    pub fn capabilities_path(&self) -> Option<&str> {
        self.extra.get("capabilities").and_then(|v| v.as_str())
    }

    /// Whether this target's `supports` values are exempt from the enforced
    /// `application.location.action` command-string shape -- for targets
    /// whose command strings are an upstream API's own naming (VS Code) or
    /// a foreign scripting language (Neovim ex-commands), not ours to
    /// rename. See `docs/protocol.md#native-command-strings`.
    pub fn exempt_command_grammar(&self) -> bool {
        self.extra
            .get("exempt_command_grammar")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
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
    modifier: String,
    location: Option<String>,
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

#[derive(Debug, Deserialize, Default)]
struct RawCapabilitiesFile {
    #[serde(default)]
    supports: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ConfigError {
    Io(String, std::io::Error),
    Toml(String, toml::de::Error),
    InvalidChord(String),
    /// An action's `modifier` or `location` isn't in `vocabulary.toml`.
    UnknownVocabulary(String),
    /// A target's `supports` map has a key that's not any known action id
    /// (typo, or the action was renamed/removed from `actions.toml`).
    UnknownAction(String),
    /// A non-exempt target's native command string doesn't fit the
    /// enforced `application.location.action` shape.
    BadCommandGrammar(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io(path, e) => write!(f, "could not read {path}: {e}"),
            ConfigError::Toml(path, e) => write!(f, "could not parse {path}: {e}"),
            ConfigError::InvalidChord(msg) => write!(f, "invalid chord binding: {msg}"),
            ConfigError::UnknownVocabulary(msg) => write!(f, "unknown vocabulary word: {msg}"),
            ConfigError::UnknownAction(msg) => write!(f, "unknown action id: {msg}"),
            ConfigError::BadCommandGrammar(msg) => write!(f, "invalid command string: {msg}"),
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

/// Validates one action's `modifier`/`location` against the vocabulary,
/// returning the derived id on success.
fn validate_action_vocabulary(
    vocabulary: &Vocabulary,
    modifier: &str,
    location: Option<&str>,
) -> Result<String, ConfigError> {
    if !vocabulary.modifiers.contains(modifier) {
        return Err(ConfigError::UnknownVocabulary(format!(
            "{modifier:?} is not a modifier in vocabulary.toml"
        )));
    }
    if let Some(location) = location {
        if !vocabulary.locations.contains(location) {
            return Err(ConfigError::UnknownVocabulary(format!(
                "{location:?} is not a location in vocabulary.toml"
            )));
        }
    }
    Ok(action_id(modifier, location))
}

/// Whether `command` fits the enforced `application.location.action` shape:
/// exactly three dot-separated, non-empty, lowercase-letters-and-underscores
/// tokens. See `docs/protocol.md#native-command-strings`.
fn fits_command_grammar(command: &str) -> bool {
    let parts: Vec<&str> = command.split('.').collect();
    parts.len() == 3
        && parts.iter().all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c == '_')
        })
}

impl Registry {
    pub fn load(config_dir: &Path) -> Result<Registry, ConfigError> {
        let vocabulary: Vocabulary =
            load_toml::<RawVocabulary>(&config_dir.join("vocabulary.toml"))?.into();
        let actions_file: RawActionsFile = load_toml(&config_dir.join("actions.toml"))?;
        let targets_file: RawTargetsFile = load_toml(&config_dir.join("targets.toml"))?;

        let mut actions = HashMap::new();
        let mut triggers = HashMap::new();
        let mut chord_triggers = HashMap::new();
        let mut chord_member_keys = HashSet::new();
        for raw in actions_file.action {
            if raw.key.is_some() && raw.chord.is_some() {
                return Err(ConfigError::InvalidChord(format!(
                    "action with modifier {:?} has both 'key' and 'chord' set; they are mutually exclusive",
                    raw.modifier
                )));
            }
            let id = validate_action_vocabulary(&vocabulary, &raw.modifier, raw.location.as_deref())?;
            if let Some(key) = &raw.key {
                triggers.insert(KeyCombo::parse(key), id.clone());
            }
            if let Some(chord) = &raw.chord {
                let keys = validate_chord(&id, chord)?;
                chord_member_keys.extend(keys.iter().cloned());
                chord_triggers.insert(keys, id.clone());
            }
            actions.insert(
                id.clone(),
                ActionSpec {
                    id,
                    fallback_tier: raw.fallback_tier,
                    fallback_keycode: raw.fallback_keycode,
                },
            );
        }

        let known_action_ids: HashSet<&str> = actions.keys().map(String::as_str).collect();

        let mut targets = targets_file.target;
        for target in &mut targets {
            if let Some(path) = target.capabilities_path() {
                let capabilities: RawCapabilitiesFile =
                    load_toml(&config_dir.join(path))?;
                target.supports = capabilities.supports;
            }

            for (action_id, command) in &target.supports {
                if !known_action_ids.contains(action_id.as_str()) {
                    return Err(ConfigError::UnknownAction(format!(
                        "target {:?} supports unknown action {action_id:?}",
                        target.program
                    )));
                }
                if !target.exempt_command_grammar() && !fits_command_grammar(command) {
                    return Err(ConfigError::BadCommandGrammar(format!(
                        "target {:?} action {action_id:?} has command {command:?}, expected application.location.action",
                        target.program
                    )));
                }
            }
        }

        let system_actions = targets_file
            .system_action
            .into_iter()
            .map(|sa| (sa.id.clone(), sa))
            .collect();

        Ok(Registry {
            actions,
            targets,
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

    /// The OS-wide system target for the platform this daemon is actually
    /// running on (an `extensions/linux-extension` or
    /// `extensions/windows-extension` listener, reached via `target.os`
    /// matching `std::env::consts::OS`) -- independent of which app, if any,
    /// currently has focus. Used by `Router::dispatch` for actions like
    /// `shutdown` or `move.left` that aren't scoped to a focused process at
    /// all.
    pub fn system_target(&self) -> Option<&Target> {
        self.targets
            .iter()
            .find(|t| t.os() == Some(std::env::consts::OS))
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

    /// Every configured action id -- the baseline catalog `spotlight::Index`
    /// builds its search entries from (`src/spotlight.rs`), before any
    /// target enriches it via the `list_actions` handshake.
    pub fn action_ids(&self) -> impl Iterator<Item = &str> {
        self.actions.keys().map(String::as_str)
    }

    /// The physical key or chord bound to `action_id`, rendered
    /// "ctrl+w"-style, if any -- a display hint for the spotlight catalog
    /// only. Dispatch itself only ever needs the reverse lookup
    /// (`action_for_trigger`/`action_for_chord`), so this scans the (small)
    /// trigger maps rather than keeping a second reverse index in sync.
    pub fn trigger_for_action(&self, action_id: &str) -> Option<String> {
        if let Some((combo, _)) = self.triggers.iter().find(|(_, id)| id.as_str() == action_id) {
            return Some(combo.to_string());
        }
        self.chord_triggers
            .iter()
            .find(|(_, id)| id.as_str() == action_id)
            .map(|(keys, _)| keys.iter().cloned().collect::<Vec<_>>().join("+"))
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

    const TEST_VOCABULARY: &str = r#"
modifiers = ["close", "some", "bad"]
locations = ["tab"]
"#;

    fn load_actions_only(name: &str, actions_toml: &str) -> Result<Registry, ConfigError> {
        let dir = temp_config_dir(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("vocabulary.toml"), TEST_VOCABULARY).unwrap();
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
            "[[action]]\nmodifier = \"some\"\nchord = [\"f\", \"ctrl\", \"d\"]\n",
        )
        .expect("chord config should load");

        let keys: BTreeSet<String> = ["ctrl", "d", "f"].into_iter().map(String::from).collect();
        assert_eq!(registry.action_for_chord(&keys), Some("some"));
        assert!(registry.is_chord_member("ctrl"));
        assert!(registry.is_chord_member("d"));
        assert!(registry.is_chord_member("f"));
        assert!(!registry.is_chord_member("x"));
    }

    #[test]
    fn chord_and_key_are_mutually_exclusive() {
        let err = load_actions_only(
            "chord-and-key",
            "[[action]]\nmodifier = \"bad\"\nkey = \"ctrl+w\"\nchord = [\"d\", \"f\"]\n",
        )
        .expect_err("both key and chord should be rejected");
        assert!(matches!(err, ConfigError::InvalidChord(_)));
    }

    #[test]
    fn chord_needs_at_least_two_keys() {
        let err = load_actions_only("chord-too-short", "[[action]]\nmodifier = \"bad\"\nchord = [\"d\"]\n")
            .expect_err("single-key chord should be rejected");
        assert!(matches!(err, ConfigError::InvalidChord(_)));
    }

    #[test]
    fn chord_rejects_duplicate_keys() {
        let err = load_actions_only(
            "chord-dupe",
            "[[action]]\nmodifier = \"bad\"\nchord = [\"d\", \"d\"]\n",
        )
        .expect_err("duplicate chord key should be rejected");
        assert!(matches!(err, ConfigError::InvalidChord(_)));
    }

    #[test]
    fn chord_rejects_modifiers_only() {
        let err = load_actions_only(
            "chord-mods-only",
            "[[action]]\nmodifier = \"bad\"\nchord = [\"ctrl\", \"shift\"]\n",
        )
        .expect_err("all-modifier chord should be rejected");
        assert!(matches!(err, ConfigError::InvalidChord(_)));
    }

    #[test]
    fn overlapping_chords_are_allowed() {
        let registry = load_actions_only(
            "chord-overlap",
            r#"[[action]]
modifier = "some"
chord = ["d", "f"]

[[action]]
modifier = "bad"
chord = ["d", "g"]
"#,
        )
        .expect("overlapping chords should be allowed");

        let df: BTreeSet<String> = ["d", "f"].into_iter().map(String::from).collect();
        let dg: BTreeSet<String> = ["d", "g"].into_iter().map(String::from).collect();
        assert_eq!(registry.action_for_chord(&df), Some("some"));
        assert_eq!(registry.action_for_chord(&dg), Some("bad"));
    }

    #[test]
    fn unknown_modifier_is_rejected() {
        let err = load_actions_only("unknown-modifier", "[[action]]\nmodifier = \"frobnicate\"\n")
            .expect_err("modifier not in vocabulary.toml should be rejected");
        assert!(matches!(err, ConfigError::UnknownVocabulary(_)));
    }

    #[test]
    fn unknown_location_is_rejected() {
        let err = load_actions_only(
            "unknown-location",
            "[[action]]\nmodifier = \"close\"\nlocation = \"nonexistent\"\n",
        )
        .expect_err("location not in vocabulary.toml should be rejected");
        assert!(matches!(err, ConfigError::UnknownVocabulary(_)));
    }

    #[test]
    fn action_id_is_derived_from_modifier_and_location() {
        let registry = load_actions_only(
            "derived-id",
            "[[action]]\nmodifier = \"close\"\nlocation = \"tab\"\nkey = \"ctrl+w\"\n",
        )
        .expect("valid modifier/location should load");
        let id = registry
            .action_for_trigger(&KeyCombo::parse("ctrl+w"))
            .expect("ctrl+w should be bound");
        assert_eq!(id, "close.tab");
    }

    #[test]
    fn bare_modifier_with_no_location_is_the_action_id() {
        let registry = load_actions_only("bare-modifier", "[[action]]\nmodifier = \"close\"\n")
            .expect("bare modifier should load");
        assert_eq!(registry.action_spec("close").id, "close");
    }

    fn write_full_config(name: &str, targets_toml: &str) -> PathBuf {
        let dir = temp_config_dir(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("vocabulary.toml"), TEST_VOCABULARY).unwrap();
        std::fs::write(
            dir.join("actions.toml"),
            "[[action]]\nmodifier = \"close\"\nlocation = \"tab\"\n",
        )
        .unwrap();
        std::fs::write(dir.join("targets.toml"), targets_toml).unwrap();
        dir
    }

    #[test]
    fn target_supports_unknown_action_is_rejected() {
        let dir = write_full_config(
            "unknown-action",
            r#"[[target]]
program = "vscode"
adapter = "socket"

  [target.supports]
  "close.nonexistent" = "workbench.action.closeActiveEditor"
"#,
        );
        let err = Registry::load(&dir).expect_err("unknown action in supports should be rejected");
        assert!(matches!(err, ConfigError::UnknownAction(_)));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn target_command_must_fit_grammar_unless_exempt() {
        let dir = write_full_config(
            "bad-grammar",
            r#"[[target]]
program = "chrome"
adapter = "websocket"

  [target.supports]
  "close.tab" = "not_enough_parts"
"#,
        );
        let err = Registry::load(&dir).expect_err("2-token command should be rejected");
        assert!(matches!(err, ConfigError::BadCommandGrammar(_)));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn exempt_target_skips_command_grammar_check() {
        let dir = write_full_config(
            "exempt-grammar",
            r#"[[target]]
program = "vscode"
adapter = "socket"
exempt_command_grammar = true

  [target.supports]
  "close.tab" = "workbench.action.closeActiveEditor"
"#,
        );
        let registry = Registry::load(&dir).expect("exempt target should skip grammar check");
        assert_eq!(
            registry.targets[0].supports.get("close.tab").map(String::as_str),
            Some("workbench.action.closeActiveEditor")
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn capabilities_file_is_loaded_and_merged_into_target_supports() {
        let dir = temp_config_dir("capabilities-file");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("ext")).unwrap();
        std::fs::write(dir.join("vocabulary.toml"), TEST_VOCABULARY).unwrap();
        std::fs::write(
            dir.join("actions.toml"),
            "[[action]]\nmodifier = \"close\"\nlocation = \"tab\"\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("ext").join("capabilities.toml"),
            "[supports]\n\"close.tab\" = \"app.tab.close\"\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("targets.toml"),
            r#"[[target]]
program = "app"
adapter = "socket"
capabilities = "ext/capabilities.toml"
"#,
        )
        .unwrap();

        let registry = Registry::load(&dir).expect("capabilities file should load");
        assert_eq!(
            registry.targets[0].supports.get("close.tab").map(String::as_str),
            Some("app.tab.close")
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
