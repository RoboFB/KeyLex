// Reference implementation of the keylex/v0 adapter protocol
// (../docs/protocol.md): newline-delimited JSON, one {"command": "..."}
// object per line, over a local TCP socket. Address must match
// src/keylex/config/targets.toml's vscode target ("127.0.0.1:7777").
const vscode = require("vscode");
const net = require("net");

const HOST = "127.0.0.1";
const PORT = 7777;

function activate(context) {
  const server = net.createServer((socket) => {
    let buffer = "";
    socket.on("data", (chunk) => {
      buffer += chunk.toString("utf8");
      let newlineIndex;
      while ((newlineIndex = buffer.indexOf("\n")) >= 0) {
        const line = buffer.slice(0, newlineIndex).trim();
        buffer = buffer.slice(newlineIndex + 1);
        if (!line) continue;
        handleLine(line);
      }
    });
  });

  function handleLine(line) {
    let message;
    try {
      message = JSON.parse(line);
    } catch (err) {
      console.error("keylex: could not parse message:", line, err);
      return;
    }
    if (!message.command) {
      console.error("keylex: message has no 'command' field:", message);
      return;
    }
    console.log("keylex: executing command:", message.command);
    vscode.commands.executeCommand(message.command).then(undefined, (err) => {
      console.error(`keylex: command '${message.command}' failed:`, err);
    });
  }

  server.on("error", (err) => {
    console.error("keylex: socket server error (is another instance already running?):", err);
  });

  server.listen(PORT, HOST, () => {
    console.log(`keylex: listening on ${HOST}:${PORT}`);
  });

  context.subscriptions.push({ dispose: () => server.close() });
}

function deactivate() {}

module.exports = { activate, deactivate };
