// Reference client for the keylex/v0 WebSocket transport (../../docs/protocol.md):
// same {"command": "..."} message the VS Code extension (../vscode-extension/
// extension.js) gets over TCP, just delivered as a WS text frame instead.
//
// Unlike the VS Code side, roles are flipped here: a browser service worker
// has no server capability at all (no listening sockets), so the Keylex
// daemon runs the WebSocket *server* and this extension connects out to it
// as a client. Port must match config/targets.toml's chrome target.
//
// SECURITY NOTE: there is currently NO authentication on this connection at
// all (deliberately dropped for now -- see
// ../../docs/protocol.md#trust-model--authentication and CLAUDE.md's "Known
// gaps", a keypair-based scheme is planned to replace the old shared-secret
// token). Any local process, or any webpage's JS, that can open a
// ws://127.0.0.1:7778 connection can currently take this extension's place
// unless config/targets.toml's `allowed_origin` is set for the chrome
// target -- see extensions/chrome-extension/README.md.
const HOST = "127.0.0.1";
const PORT = 7778;
const RECONNECT_DELAY_MS = 1000;

let socket = null;
let sidePanelOpen = false;

function connect() {
  socket = new WebSocket(`ws://${HOST}:${PORT}`);

  socket.addEventListener("open", () => {
    console.log(`keylex: connected to daemon at ${HOST}:${PORT}`);
  });

  socket.addEventListener("message", (event) => {
    handleMessage(event.data);
  });

  socket.addEventListener("close", () => {
    console.log("keylex: daemon connection closed, retrying...");
    socket = null;
    setTimeout(connect, RECONNECT_DELAY_MS);
  });

  socket.addEventListener("error", (err) => {
    console.error("keylex: websocket error:", err);
  });
}

function handleMessage(data) {
  let message;
  try {
    message = JSON.parse(data);
  } catch (err) {
    console.error("keylex: could not parse message:", data, err);
    return;
  }
  if (!message.command) {
    console.error("keylex: message has no 'command' field:", message);
    return;
  }

  console.log("keylex: executing command:", message.command);
  runCommand(message.command).catch((err) => {
    console.error(`keylex: command '${message.command}' failed:`, err);
  });
}

async function activeTab() {
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  return tab;
}

async function runCommand(command) {
  switch (command) {
    case "chrome.tab.close": {
      const tab = await activeTab();
      if (tab) await chrome.tabs.remove(tab.id);
      break;
    }
    case "chrome.window.close": {
      const tab = await activeTab();
      if (tab) await chrome.windows.remove(tab.windowId);
      break;
    }
    case "chrome.sidepanel.toggle": {
      const tab = await activeTab();
      if (!tab) break;
      // chrome.sidePanel has no single toggle call, so state is tracked
      // here and applied via setOptions.
      sidePanelOpen = !sidePanelOpen;
      if (sidePanelOpen) {
        await chrome.sidePanel.setOptions({ tabId: tab.id, enabled: true });
        await chrome.sidePanel.open({ tabId: tab.id });
      } else {
        await chrome.sidePanel.setOptions({ tabId: tab.id, enabled: false });
      }
      break;
    }
    default:
      console.error("keylex: unknown command:", command);
  }
}

// MV3 service workers can be killed by Chrome after ~30s idle, and an open
// WebSocket doesn't reliably keep one alive. This alarm is a best-effort
// nudge to reduce how often that happens; when it does happen anyway, the
// worker simply re-runs this file from the top on the next wake, which
// reconnects for free via the connect() call below.
chrome.alarms.create("keylex-keepalive", { periodInMinutes: 0.4 });
chrome.alarms.onAlarm.addListener((alarm) => {
  if (alarm.name === "keylex-keepalive" && !socket) {
    connect();
  }
});

connect();
