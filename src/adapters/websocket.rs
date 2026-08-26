//! keylex/v0 adapter over WebSocket (docs/protocol.md): same
//! `{"command": "..."}` message as `SocketAdapter`, just carried as a WS
//! text frame instead of a newline-delimited TCP byte stream. Used for
//! targets whose client can't open a listening socket (browser extensions):
//! unlike `SocketAdapter`, where the daemon connects OUT to the target's
//! server, here the roles are flipped -- the daemon runs the WebSocket
//! *server* and the target's client connects IN, since browser JS has no
//! server capability at all. A single, persistent connection is kept open
//! (accepted once, reused for every `send`) so dispatch never pays a
//! handshake -- important since this is still on the keyboard-input path.
//!
//! Every accepted connection is authenticated (docs/protocol.md#trust-model--authentication)
//! before it's trusted: the `Origin` header is checked during the WS
//! handshake itself (if `allowed_origin` is configured), and the first frame
//! read afterwards must be `{"token": "<the shared secret>"}`. A connection
//! is only promoted into the live slot `send()` uses once that check passes
//! -- this is also what stops an unauthenticated connection from hijacking
//! ("last-connect-wins") the slot away from an already-authenticated one.

use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tungstenite::{Message, WebSocket};

use crate::config::Target;
use crate::dispatch::Adapter;

type Connection = Arc<Mutex<Option<WebSocket<TcpStream>>>>;

/// How long the reader thread's `ws.read()` call is allowed to block before
/// returning `WouldBlock`/`TimedOut` and releasing the connection mutex for
/// another cycle. Bounds `send()`'s worst-case wait for the lock.
const READ_POLL_INTERVAL: Duration = Duration::from_millis(20);

/// How long a freshly handshaken connection has to send its `{"token": ...}`
/// auth frame before it's dropped. Generous relative to `READ_POLL_INTERVAL`
/// since this is a one-time check, not the per-cycle send()-blocking budget.
const AUTH_TIMEOUT: Duration = Duration::from_secs(2);

pub struct WebSocketAdapter {
    connection: Connection,
}

impl WebSocketAdapter {
    /// Binds a TCP listener on `port` and spawns a background thread that
    /// accepts WebSocket clients, authenticates each one (Origin header +
    /// token), and keeps only the most recently *authenticated* one alive
    /// (an unauthenticated connection can never displace a live one).
    // The accept_hdr callback's Err type size is dictated by tungstenite's
    // `Callback` trait, not under our control here.
    #[allow(clippy::result_large_err)]
    pub fn spawn(
        port: u16,
        token: String,
        allowed_origin: Option<String>,
    ) -> std::io::Result<WebSocketAdapter> {
        if allowed_origin.is_none() {
            eprintln!(
                "keylex: websocket adapter on port {port} has no 'allowed_origin' configured -- Origin header will not be checked"
            );
        }

        let listener = TcpListener::bind(("127.0.0.1", port))?;
        let connection: Connection = Arc::new(Mutex::new(None));

        let accept_connection = Arc::clone(&connection);
        thread::spawn(move || {
            for stream in listener.incoming() {
                let stream = match stream {
                    Ok(stream) => stream,
                    Err(e) => {
                        eprintln!("keylex: websocket adapter accept failed: {e}");
                        continue;
                    }
                };

                let origin_for_check = allowed_origin.clone();
                let mut ws = match tungstenite::accept_hdr(stream, move |req: &Request, response: Response| {
                    check_origin(req, origin_for_check.as_deref())?;
                    Ok(response)
                }) {
                    Ok(ws) => ws,
                    Err(e) => {
                        eprintln!("keylex: websocket handshake rejected: {e}");
                        continue;
                    }
                };

                if let Err(e) = ws.get_ref().set_read_timeout(Some(AUTH_TIMEOUT)) {
                    eprintln!("keylex: websocket adapter could not set auth read timeout: {e}");
                    continue;
                }
                if let Err(e) = authenticate(&mut ws, &token) {
                    eprintln!("keylex: websocket adapter rejected unauthenticated connection: {e}");
                    continue;
                }

                if let Err(e) = ws.get_ref().set_read_timeout(Some(READ_POLL_INTERVAL)) {
                    eprintln!("keylex: websocket adapter could not set read timeout: {e}");
                    continue;
                }

                println!("keylex: websocket adapter client authenticated and connected on port {port}");
                *accept_connection.lock().unwrap() = Some(ws);
                spawn_reader(Arc::clone(&accept_connection));
            }
        });

        Ok(WebSocketAdapter { connection })
    }
}

/// Rejects the handshake outright (before a `WebSocket` value even exists)
/// if `allowed_origin` is configured and the request's `Origin` header
/// doesn't match -- e.g. a browser tab other than the paired extension
/// trying `new WebSocket("ws://127.0.0.1:<port>")`. Skipped entirely (not
/// enforced) when no `allowed_origin` is configured for this target.
// `ErrorResponse`'s size is dictated by tungstenite's `Callback` trait, not
// under our control here.
#[allow(clippy::result_large_err)]
fn check_origin(request: &Request, allowed_origin: Option<&str>) -> Result<(), ErrorResponse> {
    let Some(allowed) = allowed_origin else {
        return Ok(());
    };

    let origin = request.headers().get("Origin").and_then(|v| v.to_str().ok());
    if origin == Some(allowed) {
        return Ok(());
    }

    eprintln!("keylex: websocket adapter rejected connection with Origin {origin:?}, expected {allowed:?}");
    Err(http::Response::builder()
        .status(http::StatusCode::FORBIDDEN)
        .body(Some("origin not allowed".to_string()))
        .expect("static error response is always well-formed"))
}

/// Blocks (up to whatever read timeout is currently set on `ws`'s stream)
/// for the one auth frame a freshly connected client must send:
/// `{"token": "<the shared secret from config/secret.token>"}`.
fn authenticate(ws: &mut WebSocket<TcpStream>, token: &str) -> Result<(), String> {
    match ws.read() {
        Ok(Message::Text(text)) => {
            let parsed: serde_json::Value =
                serde_json::from_str(&text).map_err(|e| format!("invalid auth frame: {e}"))?;
            match parsed.get("token").and_then(|v| v.as_str()) {
                Some(provided) if provided == token => Ok(()),
                _ => Err("token mismatch".to_string()),
            }
        }
        Ok(other) => Err(format!("expected a text auth frame, got {other:?}")),
        Err(e) => Err(format!("no auth frame received: {e}")),
    }
}

/// Drains incoming frames on the current connection so ping/pong keepalive
/// is answered (tungstenite only responds to pings when the app calls
/// `read`) and so a closed/broken connection is noticed and cleared
/// promptly rather than left stale until the next failed `send`.
fn spawn_reader(connection: Connection) {
    thread::spawn(move || loop {
        let read_result = {
            let mut guard = connection.lock().unwrap();
            match guard.as_mut() {
                Some(ws) => ws.read(),
                None => return, // superseded by a newer connection
            }
        };
        match read_result {
            Ok(_) => continue,
            Err(tungstenite::Error::Io(ref e))
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                // Just a read-timeout poll cycle, not a real disconnect.
                continue;
            }
            Err(e) => {
                eprintln!("keylex: websocket adapter connection closed: {e}");
                *connection.lock().unwrap() = None;
                return;
            }
        }
    });
}

impl Adapter for WebSocketAdapter {
    fn send(&self, _target: &Target, native_command: &str) {
        let mut guard = self.connection.lock().unwrap();
        let Some(ws) = guard.as_mut() else {
            eprintln!("keylex: websocket adapter has no connected client, dropping {native_command:?}");
            return;
        };

        let payload = serde_json::json!({ "command": native_command }).to_string();
        if let Err(e) = ws.send(Message::Text(payload.into())) {
            eprintln!("keylex: websocket adapter write failed: {e}");
            *guard = None;
            return;
        }
        if let Err(e) = ws.flush() {
            eprintln!("keylex: websocket adapter flush failed: {e}");
            *guard = None;
        }
    }
}
