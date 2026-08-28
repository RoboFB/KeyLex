//! Generic keylex/v0 adapter (`docs/protocol.md`): newline-delimited JSON,
//! one `{"command": "..."}` object per line, over a local TCP socket. The
//! daemon is the client here; the target listens.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use crate::config::Target;
use crate::dispatch::Adapter;
use crate::spotlight::{ListActionsResponse, RemoteAction};

const CONNECT_TIMEOUT: Duration = Duration::from_millis(500);

/// How long `fetch_actions` waits for an answer. Generous next to
/// `CONNECT_TIMEOUT` because, unlike a plain dispatch, the target has real
/// work to do (cross-checking its command list) before it can reply.
const HANDSHAKE_READ_TIMEOUT: Duration = Duration::from_secs(2);

/// Carries the shared secret on every message
/// (`docs/protocol.md#trust-model--authentication`) so the listening target
/// can reject anything that isn't holding it.
pub struct SocketAdapter {
    token: String,
}

impl SocketAdapter {
    pub fn new(token: String) -> SocketAdapter {
        SocketAdapter { token }
    }

    /// Runs the `list_actions` handshake against `target`
    /// (`docs/protocol.md#action-catalog-handshake-list_actions`). Returns
    /// `None` on any failure -- unreachable, timed out, unparseable -- since
    /// this only ever enriches the spotlight catalog and is never needed for
    /// it to work.
    pub fn fetch_actions(&self, target: &Target) -> Option<Vec<RemoteAction>> {
        let mut stream = self.connect(target)?;
        if let Err(e) = stream.set_read_timeout(Some(HANDSHAKE_READ_TIMEOUT)) {
            eprintln!(
                "keylex: {}: could not set handshake read timeout: {e}",
                target.program
            );
            return None;
        }

        let request = serde_json::json!({ "type": "list_actions", "token": self.token });
        if let Err(e) = writeln!(stream, "{request}") {
            eprintln!("keylex: {}: handshake write failed: {e}", target.program);
            return None;
        }

        let mut line = String::new();
        if let Err(e) = BufReader::new(&stream).read_line(&mut line) {
            eprintln!(
                "keylex: {}: handshake read failed (target may not support it yet): {e}",
                target.program
            );
            return None;
        }

        match serde_json::from_str::<ListActionsResponse>(line.trim()) {
            Ok(response) => Some(response.actions),
            Err(e) => {
                eprintln!(
                    "keylex: {}: unparseable handshake response: {e}",
                    target.program
                );
                None
            }
        }
    }

    fn connect(&self, target: &Target) -> Option<TcpStream> {
        let Some(address) = target.address.as_deref() else {
            eprintln!(
                "keylex: target {:?} has no 'address' configured",
                target.program
            );
            return None;
        };
        let Some(resolved) = address.to_socket_addrs().ok().and_then(|mut it| it.next()) else {
            eprintln!(
                "keylex: target {:?}: could not resolve {address:?}",
                target.program
            );
            return None;
        };
        TcpStream::connect_timeout(&resolved, CONNECT_TIMEOUT)
            .map_err(|e| {
                eprintln!(
                    "keylex: target {:?} unreachable at {address}: {e}",
                    target.program
                )
            })
            .ok()
    }
}

impl Adapter for SocketAdapter {
    fn send(&self, target: &Target, native_command: &str) {
        let Some(mut stream) = self.connect(target) else {
            return;
        };
        let payload = serde_json::json!({ "command": native_command, "token": self.token });
        if let Err(e) = writeln!(stream, "{payload}") {
            eprintln!("keylex: target {:?}: write failed: {e}", target.program);
        }
    }
}
