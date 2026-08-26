// Stand-in for the (not yet built) VS Code extension's socket server.
//
// There is no real VS Code-side listener yet, so SocketAdapter always times
// out and every action falls through to the keycode fallback. Run this next
// to `cargo run` to see a "native" dispatch actually arrive somewhere, per
// the wire format in docs/protocol.md -- without needing the real extension.
const fs = require("fs");
const net = require("net");
const path = require("path");

const HOST = "127.0.0.1";
const PORT = 7777; // must match config/targets.toml's vscode target
const TOKEN_PATH = path.join(__dirname, "..", "config", "secret.token");

// The daemon generates this file on first run (src/auth.rs); every message
// it sends now carries this same token (docs/protocol.md#trust-model--authentication).
const token = fs.readFileSync(TOKEN_PATH, "utf8").trim();

const server = net.createServer((socket) => {
  let buffer = "";
  socket.on("data", (chunk) => {
    buffer += chunk.toString("utf8");
    let newlineIndex;
    while ((newlineIndex = buffer.indexOf("\n")) >= 0) {
      const line = buffer.slice(0, newlineIndex).trim();
      buffer = buffer.slice(newlineIndex + 1);
      if (!line) continue;
      try {
        const message = JSON.parse(line);
        if (message.token !== token) {
          console.error("rejected message with invalid/missing token:", message);
          continue;
        }
        console.log("received command:", message.command);
      } catch (err) {
        console.error("bad message:", line, err);
      }
    }
  });
});

server.listen(PORT, HOST, () => {
  console.log(`fake VS Code listener on ${HOST}:${PORT} (Ctrl+C to stop)`);
});
