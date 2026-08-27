//! Generic keylex/v0 adapter (docs/protocol.md): newline-delimited JSON,
//! one `{"command": "..."}` object per line, over a local TCP socket. Any
//! target using `adapter = "socket"` in targets.toml is reached this way
//! (currently only the VS Code extension implements the listening side).

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use crate::config::Target;
use crate::dispatch::Adapter;
use crate::spotlight::{ListActionsResponse, RemoteAction};

const CONNECT_TIMEOUT: Duration = Duration::from_millis(500);
/// How long `fetch_actions` waits for a target to answer the `list_actions`
/// handshake (docs/protocol.md#action-catalog-handshake-list_actions) before
/// giving up. Generous relative to `CONNECT_TIMEOUT` since, unlike a plain
/// dispatch, the target actually has to do work (e.g. cross-check its
/// command list) before replying.
const HANDSHAKE_READ_TIMEOUT: Duration = Duration::from_secs(2);

/// Sent alongside every `command` (docs/protocol.md#trust-model--authentication)
/// so the listening target (e.g. the VS Code extension) can reject any
/// connection that isn't holding the shared secret from `config/secret.token`.
pub struct SocketAdapter {
    token: String,
}

impl SocketAdapter {
    pub fn new(token: String) -> SocketAdapter {
        SocketAdapter { token }
    }

    /// Runs the `list_actions` handshake against `target`
    /// (docs/protocol.md#action-catalog-handshake-list_actions): connects,
    /// sends the request, reads back one response line on the same
    /// connection, and parses it. Returns `None` on any failure -- no
    /// address configured, unreachable, timed out, or an unparseable
    /// response -- since this is always a best-effort enrichment, never a
    /// requirement for the spotlight catalog to work at all.
    pub fn fetch_actions(&self, target: &Target) -> Option<Vec<RemoteAction>> {
        let (address, addr) = resolve_address(target, "spotlight handshake")?;

        let mut stream = TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT)
            .map_err(|e| eprintln!("keylex: spotlight handshake unreachable at {address}: {e}"))
            .ok()?;
        if let Err(e) = stream.set_read_timeout(Some(HANDSHAKE_READ_TIMEOUT)) {
            eprintln!("keylex: spotlight handshake could not set read timeout for {address}: {e}");
            return None;
        }

        let request = serde_json::json!({ "type": "list_actions", "token": &self.token }).to_string() + "\n";
        if let Err(e) = stream.write_all(request.as_bytes()) {
            eprintln!("keylex: spotlight handshake write failed for {address}: {e}");
            return None;
        }

        let mut line = String::new();
        if let Err(e) = BufReader::new(&stream).read_line(&mut line) {
            eprintln!("keylex: spotlight handshake read failed for {address} (target may not support it yet): {e}");
            return None;
        }

        match serde_json::from_str::<ListActionsResponse>(line.trim()) {
            Ok(response) => Some(response.actions),
            Err(e) => {
                eprintln!("keylex: spotlight handshake got an unparseable response from {address}: {e}");
                None
            }
        }
    }
}

impl Adapter for SocketAdapter {
    fn send(&self, target: &Target, native_command: &str) {
        let Some((address, addr)) = resolve_address(target, "socket adapter") else {
            return;
        };

        match TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT) {
            Ok(mut stream) => {
                let payload = serde_json::json!({ "command": native_command, "token": &self.token })
                    .to_string()
                    + "\n";
                if let Err(e) = stream.write_all(payload.as_bytes()) {
                    eprintln!("keylex: socket adapter write failed for {address}: {e}");
                }
            }
            Err(e) => eprintln!("keylex: socket adapter unreachable at {address}: {e}"),
        }
    }
}

/// Shared by `send` and `fetch_actions`: pulls `target.extra["address"]` and
/// resolves it to a concrete `SocketAddr`, logging (and returning `None`)
/// on either being missing/unresolvable.
fn resolve_address<'a>(target: &'a Target, context: &str) -> Option<(&'a str, std::net::SocketAddr)> {
    let Some(address) = target.extra.get("address").and_then(|v| v.as_str()) else {
        eprintln!("keylex: {context}: target {:?} has no 'address' configured", target.program);
        return None;
    };
    let Some(addr) = address.to_socket_addrs().ok().and_then(|mut it| it.next()) else {
        eprintln!("keylex: {context}: could not resolve address {address:?}");
        return None;
    };
    Some((address, addr))
}
