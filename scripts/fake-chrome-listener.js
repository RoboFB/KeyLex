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
const fs = require("fs");
const path = require("path");
const WebSocket = require("ws");

const HOST = "127.0.0.1";
const PORT = 7778; // must match config/targets.toml's chrome target
const TOKEN_PATH = path.join(__dirname, "..", "config", "secret.token");

// The daemon won't promote this connection into its live slot until it sees
// this token as the very first frame (docs/protocol.md#trust-model--authentication).
const token = fs.readFileSync(TOKEN_PATH, "utf8").trim();

function connect() {
  const socket = new WebSocket(`ws://${HOST}:${PORT}`);

  socket.on("open", () => {
    socket.send(JSON.stringify({ token }));
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
