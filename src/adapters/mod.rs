//! One transport per file, each reaching a target the way that target can
//! be reached: the daemon connects out over TCP for `socket`, and listens
//! for a client that can't itself listen for `websocket`. Both speak the
//! same keylex/v0 messages (`docs/protocol.md`).

mod socket;
mod websocket;

pub use socket::SocketAdapter;
pub use websocket::WebSocketAdapter;
