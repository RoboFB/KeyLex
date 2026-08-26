//! Generic keylex/v0 adapter (docs/protocol.md): newline-delimited JSON,
//! one `{"command": "..."}` object per line, over a local TCP socket. Any
//! target using `adapter = "socket"` in targets.toml is reached this way
//! (currently only the VS Code extension implements the listening side).

use std::io::Write;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use crate::config::Target;
use crate::dispatch::Adapter;

const CONNECT_TIMEOUT: Duration = Duration::from_millis(500);

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
}

impl Adapter for SocketAdapter {
    fn send(&self, target: &Target, native_command: &str) {
        let Some(address) = target.extra.get("address").and_then(|v| v.as_str()) else {
            eprintln!(
                "keylex: socket adapter target {:?} has no 'address' configured",
                target.program
            );
            return;
        };

        let Some(addr) = address.to_socket_addrs().ok().and_then(|mut it| it.next()) else {
            eprintln!("keylex: socket adapter could not resolve address {address:?}");
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
