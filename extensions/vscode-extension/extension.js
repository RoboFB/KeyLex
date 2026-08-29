// Reference implementation of the keylex/v0 adapter protocol
// (../../docs/protocol.md): newline-delimited JSON, one {"command": "..."}
// object per line, over a local TCP socket. Address must match
// src/keylex/config/targets.toml's vscode target ("127.0.0.1:7777").
//
// SECURITY NOTE: there is currently NO authentication on this socket at all
// (deliberately dropped for now -- see
// ../../docs/protocol.md#trust-model--authentication and CLAUDE.md's "Known
// gaps", a keypair-based scheme is planned to replace the old shared-secret
// token). Any local process able to open a TCP connection to 127.0.0.1:7777
// can drive commands here. Combined with the next paragraph, that means any
// local process can currently run *any* command registered in this VS Code
// window -- treat this the same as running an unauthenticated local server,
// and don't use it on a shared or untrusted machine yet.
//
// SECURITY NOTE, deliberately no allowlist here either (unlike an earlier
// version of this file): the command catalog below is discovered live from
// every installed extension's own contributed commands (see
// liveActionCatalog), specifically so a newly installed extension shows up
// in spotlight search automatically, with no hand-maintained list to keep
// in sync. The trade-off is real and worth stating plainly: anything that
// can reach this socket can invoke *any* command currently registered in
// this VS Code window -- including ones contributed by other installed
// extensions, some of which (running a task, executing terminal text,
// editing/deleting files) are considerably more dangerous than "close tab".
// If that trade-off isn't acceptable for your setup, reintroduce an
// explicit allowlist here (git blame this comment for the previous
// version).
const vscode = require("vscode");
const net = require("net");

const HOST = "127.0.0.1";
const PORT = 7777;

// The full live command catalog: every {command, title} an installed
// extension actually contributes (vscode.extensions.all's own parsed
// package.json is the same data VS Code's Command Palette itself is built
// from -- ext.packageJSON.contributes.commands), filtered down to whatever
// is *currently* registered (vscode.commands.getCommands()) so a disabled
// extension or a not-yet-activated command silently drops out instead of
// being offered. This is the only source of "valid options" for both the
// list_actions handshake
// (../../docs/protocol.md#action-catalog-handshake-list_actions) and the
// keylex.spotlight QuickPick below, and it needs zero maintenance when a
// new extension gets installed -- it's picked up on the next call.
async function liveActionCatalog() {
  const registered = new Set(await vscode.commands.getCommands(true));
  const byCommand = new Map();
  for (const ext of vscode.extensions.all) {
    const contributed = ext.packageJSON && ext.packageJSON.contributes && ext.packageJSON.contributes.commands;
    if (!Array.isArray(contributed)) continue;
    for (const entry of contributed) {
      if (!entry.command || !entry.title || !registered.has(entry.command)) continue;
      const title = entry.category ? `${entry.category}: ${entry.title}` : entry.title;
      byCommand.set(entry.command, { command: entry.command, title });
    }
  }
  return Array.from(byCommand.values());
}

// `keylex.spotlight`: a fuzzy-searchable QuickPick over the same live
// catalog the daemon gets via the list_actions handshake -- VS Code's
// built-in QuickPick already does fuzzy matching on label/description, so
// no separate matching logic is needed on this side.
async function runSpotlight() {
  const catalog = await liveActionCatalog();
  const items = catalog.map((entry) => ({
    label: entry.title,
    description: entry.command,
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
    // The list_actions handshake (../../docs/protocol.md#action-catalog-handshake-list_actions):
    // respond once, on this same connection, with the live catalog, then
    // close -- it's a request/response exchange, not the fire-and-forget
    // `command` messages below. `id` is just the native command string
    // itself: this extension has no notion of Keylex's cross-app action
    // ids (close.tab, save, ...) for the vast majority of what it
    // discovers, and doesn't need one -- the daemon-side merge
    // (spotlight::Index::merge_remote) recognizes an id that happens to
    // already be a known Keylex action and enriches that entry in place;
    // everything else it namespaces as a raw, VS-Code-only passthrough.
    if (message.type === "list_actions") {
      liveActionCatalog().then((catalog) => {
        const response =
          JSON.stringify({
            actions: catalog.map(({ command, title }) => ({ id: command, native_command: command, title })),
          }) + "\n";
        socket.end(response);
      });
      return;
    }

    if (!message.command) {
      console.error("keylex: message has no 'command' field:", message);
      return;
    }
    // No allowlist here -- see the SECURITY NOTE at the top of this file
    // for why, and what that trades away. This check is a liveness check,
    // not a safety filter: it only rejects a command string that isn't
    // (or is no longer) actually registered in this VS Code window.
    vscode.commands.getCommands(true).then((registered) => {
      if (!registered.includes(message.command)) {
        console.error("keylex: rejected unregistered command:", message.command);
        return;
      }
      console.log("keylex: executing command:", message.command);
      vscode.commands.executeCommand(message.command).then(undefined, (err) => {
        console.error(`keylex: command '${message.command}' failed:`, err);
      });
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
