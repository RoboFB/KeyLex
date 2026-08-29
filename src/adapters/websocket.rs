//! keylex/v0 over WebSocket (`docs/protocol.md`): the same
//! `{"command": "..."}` message as `SocketAdapter`, carried as a WS text
//! frame. Used for targets whose client can't open a listening socket --
//! browser extensions -- so the roles are flipped: the daemon is the
//! server and the target connects in. One connection is accepted and kept
//! open for every `send`, so dispatch never pays for a handshake.
//!
//! **No shared-secret authentication right now** -- see
//! `docs/protocol.md#trust-model--authentication` and CLAUDE.md's "Known
//! gaps" for why and what's planned instead (a keypair-based scheme).
//! `Origin` is still checked during the handshake when a target configures
//! `allowed_origin`, and a freshly accepted connection still becomes the
//! live one immediately (last-connect-wins) -- but nothing stops another
//! local process from connecting and displacing the real client, since
//! there's no secret left to prove it's the paired extension. Loopback-only
//! binding is the only current boundary.
//!
//! One thread owns each accepted socket outright, and `send` only hands it
//! a command through a channel. That matters: `send` sits on the keyboard
//! path and must never wait on socket I/O, which is exactly what sharing
//! the socket behind a mutex would make it do.

use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tungstenite::{Message, WebSocket};

use crate::config::Target;
use crate::dispatch::Adapter;

/// How long the owning thread waits for an incoming frame before checking
/// its outbox again. It bounds both how long a queued command can sit
/// there and how often an idle connection wakes up.
const READ_POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Where `send` leaves commands for the live connection's own thread. A
/// connection that has since died needs no cleanup: the channel's own
/// receiver is gone with it, so the next `send` fails and says so.
type Live = Arc<Mutex<Option<Sender<String>>>>;

pub struct WebSocketAdapter {
    live: Live,
}

impl WebSocketAdapter {
    /// Binds `port` and spawns the accept loop. The most recently accepted
    /// client (that passes the Origin check, if configured) is the live one.
    pub fn spawn(port: u16, allowed_origin: Option<String>) -> std::io::Result<WebSocketAdapter> {
        if allowed_origin.is_none() {
            eprintln!(
                "keylex: websocket adapter on port {port} has no 'allowed_origin' configured -- Origin header will not be checked"
            );
        }

        let listener = TcpListener::bind(("127.0.0.1", port))?;
        let live: Live = Arc::new(Mutex::new(None));
        let accepted = Arc::clone(&live);

        thread::spawn(move || {
            for stream in listener.incoming() {
                let stream = match stream {
                    Ok(stream) => stream,
                    Err(e) => {
                        eprintln!("keylex: websocket accept failed: {e}");
                        continue;
                    }
                };
                let Some(ws) = accept(stream, allowed_origin.as_deref()) else {
                    continue;
                };

                println!("keylex: websocket client connected on port {port}");
                let (commands, inbox) = mpsc::channel();
                *accepted.lock().unwrap() = Some(commands);
                thread::spawn(move || serve(ws, &inbox));
            }
        });

        Ok(WebSocketAdapter { live })
    }
}

/// Handshake and Origin check for one incoming client. `None` means it was
/// rejected, with the reason already reported.
// The handshake callback's error type is `ErrorResponse`, whose size is
// dictated by tungstenite's `Callback` trait rather than by us.
#[allow(clippy::result_large_err)]
fn accept(stream: TcpStream, allowed_origin: Option<&str>) -> Option<WebSocket<TcpStream>> {
    let ws = tungstenite::accept_hdr(stream, |request: &Request, response: Response| {
        check_origin(request, allowed_origin)?;
        Ok(response)
    })
    .map_err(|e| eprintln!("keylex: websocket handshake rejected: {e}"))
    .ok()?;

    ws.get_ref()
        .set_read_timeout(Some(READ_POLL_INTERVAL))
        .map_err(|e| eprintln!("keylex: websocket could not set read timeout: {e}"))
        .ok()?;

    Some(ws)
}

/// Rejects the handshake before a `WebSocket` value even exists if the
/// request's `Origin` doesn't match -- e.g. a browser tab other than the
/// paired extension opening `ws://127.0.0.1:<port>`. Not enforced at all
/// when no `allowed_origin` is configured for the target.
#[allow(clippy::result_large_err)]
fn check_origin(request: &Request, allowed_origin: Option<&str>) -> Result<(), ErrorResponse> {
    let Some(allowed) = allowed_origin else {
        return Ok(());
    };
    let origin = request
        .headers()
        .get("Origin")
        .and_then(|value| value.to_str().ok());
    if origin == Some(allowed) {
        return Ok(());
    }

    eprintln!("keylex: websocket rejected Origin {origin:?}, expected {allowed:?}");
    Err(http::Response::builder()
        .status(http::StatusCode::FORBIDDEN)
        .body(Some("origin not allowed".to_string()))
        .expect("a response built from constants is always well-formed"))
}

/// Owns one accepted connection until it breaks: writes whatever
/// `send` queues, and keeps reading so ping/pong keepalive is answered
/// (tungstenite only replies to pings while the app is reading) and a dead
/// connection is noticed promptly rather than at the next dispatch.
fn serve(mut ws: WebSocket<TcpStream>, commands: &Receiver<String>) {
    loop {
        for command in commands.try_iter() {
            let payload = serde_json::json!({ "command": command }).to_string();
            if let Err(e) = ws
                .send(Message::Text(payload.into()))
                .and_then(|()| ws.flush())
            {
                eprintln!("keylex: websocket write failed: {e}");
                return;
            }
        }

        match ws.read() {
            Ok(_) => {}
            Err(tungstenite::Error::Io(e))
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) => {}
            Err(e) => {
                eprintln!("keylex: websocket connection closed: {e}");
                return;
            }
        }
    }
}

impl Adapter for WebSocketAdapter {
    fn send(&self, _target: &Target, native_command: &str) {
        let live = self.live.lock().unwrap();
        let queued = live
            .as_ref()
            .is_some_and(|commands| commands.send(native_command.to_string()).is_ok());
        if !queued {
            eprintln!("keylex: no websocket client connected, dropping {native_command:?}");
        }
    }
}
