//! `targets.toml`: where an action goes once it's resolved -- which
//! process identifies a target program, which transport reaches it, and
//! which actions it can carry out natively.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::error::ConfigError;

/// Which transport reaches a target, keying the adapter map built in
/// `src/cli.rs`. `Rpc` (Neovim) is declared but unimplemented: a target
/// using it parses fine and simply reports "unsupported" on dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AdapterKind {
    Socket,
    WebSocket,
    Rpc,
}

impl fmt::Display for AdapterKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdapterKind::Socket => f.write_str("socket"),
            AdapterKind::WebSocket => f.write_str("websocket"),
            AdapterKind::Rpc => f.write_str("rpc"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub program: String,
    /// Process names that identify this target as the focused app. Empty
    /// for an OS-wide listener, which is matched by `os` instead.
    #[serde(default)]
    pub match_process: Vec<String>,
    pub adapter: AdapterKind,
    /// `"linux"` / `"windows"`, matching `std::env::consts::OS`: marks this
    /// target as the OS-wide system listener rather than an app tied to a
    /// focused process. See `Registry::system_target`.
    pub os: Option<String>,
    /// Where the socket adapter connects out to.
    pub address: Option<String>,
    /// Where the websocket adapter listens for the target to connect in.
    pub port: Option<u16>,
    /// When set, the websocket adapter rejects any handshake whose `Origin`
    /// header doesn't match exactly
    /// (`docs/protocol.md#trust-model--authentication`).
    pub allowed_origin: Option<String>,
    /// This target's action -> native-command map. Declared inline only by
    /// a target with no `extensions/` folder yet (`neovim`); every other
    /// one owns it in its own `capabilities.toml`, named by `capabilities`
    /// below and loaded over this field at startup.
    #[serde(default)]
    pub supports: HashMap<String, String>,
    /// Path, relative to the config directory, to this target's own
    /// `capabilities.toml` -- see `docs/protocol.md#action-ids` for why
    /// that lives with the extension instead of here.
    capabilities: Option<PathBuf>,
    /// Exempts this target's command strings from the enforced
    /// `application.location.action` shape, for targets whose commands are
    /// an upstream API's own naming (VS Code) or a foreign scripting
    /// language (Neovim ex-commands), not ours to rename.
    #[serde(default)]
    exempt_command_grammar: bool,
}

#[derive(Debug, Deserialize, Default)]
struct CapabilitiesFile {
    #[serde(default)]
    supports: HashMap<String, String>,
}

impl Target {
    /// Replaces `supports` with what this target's own `capabilities.toml`
    /// declares, then checks every entry against the actions that exist and
    /// the command-string grammar.
    pub(crate) fn resolve(
        &mut self,
        config_dir: &Path,
        known_actions: &HashSet<&str>,
    ) -> Result<(), ConfigError> {
        if let Some(path) = &self.capabilities {
            let file: CapabilitiesFile = super::load_toml(&config_dir.join(path))?;
            self.supports = file.supports;
        }

        for (action_id, command) in &self.supports {
            if !known_actions.contains(action_id.as_str()) {
                return Err(ConfigError::new(format!(
                    "target {:?} supports unknown action {action_id:?}",
                    self.program
                )));
            }
            if !self.exempt_command_grammar && !fits_command_grammar(command) {
                return Err(ConfigError::new(format!(
                    "target {:?} action {action_id:?} has command {command:?}, expected application.location.action",
                    self.program
                )));
            }
        }
        Ok(())
    }
}

/// Whether `command` fits the enforced `application.location.action` shape:
/// exactly three dot-separated, non-empty tokens of lowercase letters and
/// underscores. See `docs/protocol.md#native-command-strings`.
fn fits_command_grammar(command: &str) -> bool {
    let mut parts = command.split('.');
    let fits =
        |part: &str| !part.is_empty() && part.chars().all(|c| c.is_ascii_lowercase() || c == '_');
    matches!(
        (parts.next(), parts.next(), parts.next(), parts.next()),
        (Some(a), Some(b), Some(c), None) if fits(a) && fits(b) && fits(c)
    )
}
