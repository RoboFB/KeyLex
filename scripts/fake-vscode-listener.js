// Stand-in for the real VS Code extension's socket server
// (../extensions/vscode-extension/extension.js). Run this next to
// `cargo run`/`cargo run -- --spotlight` to see a "native" dispatch and the
// list_actions handshake (docs/protocol.md#action-catalog-handshake-list_actions)
// actually arrive somewhere, without needing a real VS Code window.
const fs = require("fs");
const net = require("net");
const path = require("path");

const HOST = "127.0.0.1";
const PORT = 7777; // must match config/targets.toml's vscode target
const TOKEN_PATH = path.join(__dirname, "..", "config", "secret.token");

// The daemon generates this file on first run (src/auth.rs); every message
// it sends now carries this same token (docs/protocol.md#trust-model--authentication).
const token = fs.readFileSync(TOKEN_PATH, "utf8").trim();

// Mirrors the real extension's ACTION_CATALOG closely enough for local
// testing -- not the "live vscode.commands.getCommands()" check the real
// extension does, since there's no real VS Code here to check against.
const ACTION_CATALOG = [
  { id: "close.tab", command: "workbench.action.closeActiveEditor", title: "Close Editor" },
  { id: "save", command: "workbench.action.files.save", title: "Save File" },
];

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
        if (message.type === "list_actions") {
          console.log("received list_actions handshake request");
          const response =
            JSON.stringify({
              actions: ACTION_CATALOG.map(({ id, command, title }) => ({ id, native_command: command, title })),
            }) + "\n";
          socket.end(response);
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
