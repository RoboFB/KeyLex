//! `targets.toml`: where an action goes once it's resolved -- which
//! process identifies a target program, and which transport reaches it.
//! Which actions a target can carry out natively is no longer declared
//! here or in a per-extension `capabilities.toml` -- a target reports that
//! live, via the `list_actions` handshake (see `docs/protocol.md`).

use std::fmt;

use serde::Deserialize;

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
}
