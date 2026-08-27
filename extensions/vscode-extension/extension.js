// Reference implementation of the keylex/v0 adapter protocol
// (../../docs/protocol.md): newline-delimited JSON, one {"command": "..."}
// object per line, over a local TCP socket. Address must match
// src/keylex/config/targets.toml's vscode target ("127.0.0.1:7777").
//
// Every message must also carry the shared secret from config/secret.token
// (../../docs/protocol.md#trust-model--authentication) -- without it, any local
// process able to open a TCP connection to this port could otherwise drive
// arbitrary VS Code commands. The allowlist below is a second, independent
// layer: even a correctly-tokened message can only trigger a command this
// extension already knows about, since some VS Code commands (e.g. running
// text in an open terminal) are far more dangerous than "close tab".
const vscode = require("vscode");
const net = require("net");
const fs = require("fs");

const HOST = "127.0.0.1";
const PORT = 7777;

// Mirrors this folder's capabilities.toml `[supports]` map -- an
// independent, hardcoded copy on purpose (see
// ../../docs/protocol.md#native-command-strings), so a compromised/rogue
// local client holding the token still can't invoke a command this
// extension wasn't built to expect. This is also the *only* source of
// "valid options" for both the `list_actions` handshake
// (../../docs/protocol.md#action-catalog-handshake-list_actions) and the
// `keylex.spotlight` QuickPick below -- neither reads a config/CSV file of
// commands, and both live-filter this list against
// `vscode.commands.getCommands()` before trusting an entry, so a command
// that's no longer actually registered (extension disabled, VS Code
// version drift) silently drops out instead of being offered.
const ACTION_CATALOG = [
  { id: "close.tab", command: "workbench.action.closeActiveEditor", title: "Close Editor" },
  { id: "close.window", command: "workbench.action.closeWindow", title: "Close Window" },
  { id: "close.sidebar", command: "workbench.action.closeSidebar", title: "Close Sidebar" },
  { id: "close.pane", command: "workbench.action.closePanel", title: "Close Panel" },
  { id: "close.terminal", command: "workbench.action.terminal.kill", title: "Kill Terminal" },
  { id: "duplicate.line", command: "editor.action.copyLinesDownAction", title: "Duplicate Line" },
  { id: "go_to.definition", command: "editor.action.revealDefinition", title: "Go to Definition" },
  { id: "comment.line", command: "editor.action.commentLine", title: "Toggle Line Comment" },
  { id: "save", command: "workbench.action.files.save", title: "Save File" },
];

const ALLOWED_COMMANDS = new Set(ACTION_CATALOG.map((entry) => entry.command));

// The daemon-facing catalog, live-checked against this running VS Code
// instance's actual command registry -- see the ACTION_CATALOG comment
// above for why this (never a static file) is the source of truth for
// "what can Keylex actually search/dispatch right now".
async function liveActionCatalog() {
  const registered = new Set(await vscode.commands.getCommands(true));
  return ACTION_CATALOG.filter((entry) => registered.has(entry.command));
}

// `keylex.spotlight`: a fuzzy-searchable QuickPick over the same live
// catalog the daemon gets via the list_actions handshake -- VS Code's
// built-in QuickPick already does fuzzy matching on label/description, so
// no separate matching logic is needed on this side.
async function runSpotlight() {
  const catalog = await liveActionCatalog();
  const items = catalog.map((entry) => ({
    label: entry.title,
    description: entry.id,
    detail: entry.command,
    command: entry.command,
  }));
  const picked = await vscode.window.showQuickPick(items, {
    placeHolder: "Keylex spotlight: search actions by name",
    matchOnDescription: true,
  });
  if (!picked) {
    return;
  }
  vscode.commands.executeCommand(picked.command).then(undefined, (err) => {
    vscode.window.showErrorMessage(`keylex: command '${picked.command}' failed: ${err}`);
  });
}

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
        handleLine(line, socket);
      }
    });
  });

  function handleLine(line, socket) {
    let message;
    try {
      message = JSON.parse(line);
    } catch (err) {
      console.error("keylex: could not parse message:", line, err);
      return;
    }
    if (message.token !== token) {
      console.error("keylex: rejected message with invalid/missing token:", message.command || message.type);
      return;
    }

    // The list_actions handshake (../../docs/protocol.md#action-catalog-handshake-list_actions):
    // respond once, on this same connection, with the live catalog, then
    // close -- it's a request/response exchange, not the fire-and-forget
    // `command` messages below.
    if (message.type === "list_actions") {
      liveActionCatalog().then((catalog) => {
        const response =
          JSON.stringify({
            actions: catalog.map(({ id, command, title }) => ({ id, native_command: command, title })),
          }) + "\n";
        socket.end(response);
      });
      return;
    }

    if (!message.command) {
      console.error("keylex: message has no 'command' field:", message);
      return;
    }
    if (!ALLOWED_COMMANDS.has(message.command)) {
      console.error("keylex: rejected non-allowlisted command:", message.command);
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

  context.subscriptions.push(vscode.commands.registerCommand("keylex.spotlight", runSpotlight));

  context.subscriptions.push({ dispose: () => server.close() });
}

function deactivate() {}

module.exports = { activate, deactivate };
