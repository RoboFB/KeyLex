// Stand-in for the real Chrome extension's WebSocket client, for testing
// the daemon's WebSocketAdapter without loading Chrome at all.
//
// Unlike scripts/fake-vscode-listener.js (a fake *server*, since the real
// VS Code extension listens), this is a fake *client*: the daemon runs the
// WebSocket server (see src/adapters/websocket.rs and docs/protocol.md),
// so this just connects to it and logs whatever commands arrive.
//
// Needs the `ws` package: run `npm install ws` in this directory first, or
// wherever you invoke this script from.
const WebSocket = require("ws");

const HOST = "127.0.0.1";
const PORT = 7778; // must match config/targets.toml's chrome target

function connect() {
  const socket = new WebSocket(`ws://${HOST}:${PORT}`);

  socket.on("open", () => {
    console.log(`fake Chrome listener connected to ${HOST}:${PORT}`);
  });

  socket.on("message", (data) => {
    try {
      const message = JSON.parse(data.toString("utf8"));
      console.log("received command:", message.command);
    } catch (err) {
      console.error("bad message:", data, err);
    }
  });

  socket.on("close", () => {
    console.log("connection closed, retrying in 1s (Ctrl+C to stop)...");
    setTimeout(connect, 1000);
  });

  socket.on("error", (err) => {
    console.error("websocket error:", err.message);
  });
}

connect();
