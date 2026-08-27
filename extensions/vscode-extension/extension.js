// Reference implementation of the keylex/v0 adapter protocol
// (../../docs/protocol.md): newline-delimited JSON, one {"command": "..."}
// object per line, over a local TCP socket. Address must match
// src/keylex/config/targets.toml's vscode target ("127.0.0.1:7777").
//
// Every message must also carry the shared secret from config/secret.token
// (../../docs/protocol.md#trust-model--authentication) -- without it, any local
// process able to open a TCP connection to this port could otherwise drive
// arbitrary VS Code commands. There is deliberately no second allowlist layer
// here: any correctly-tokened message can invoke any VS Code command,
// including ones far more dangerous than "close tab" (e.g. sending text into
// an open terminal). Fine for local dev/testing; revisit before this is used
// anywhere the token can leak.
const vscode = require("vscode");
const net = require("net");
const fs = require("fs");

const HOST = "127.0.0.1";
const PORT = 7777;

function loadToken() {
  const tokenPath = vscode.workspace.getConfiguration("keylex").get("tokenPath");
  if (!tokenPath) {
    throw new Error(
      "keylex.tokenPath is not set -- point it at the secret.token file the " +
        "daemon generated in its config directory (see ../../docs/protocol.md#trust-model--authentication)"
    );
  }
  return fs.readFileSync(tokenPath, "utf8").trim();
}

function activate(context) {
  const token = loadToken();

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
    if (message.token !== token) {
      console.error("keylex: rejected message with invalid/missing token:", message.command);
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
