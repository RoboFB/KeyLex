// Reference client for the keylex/v0 WebSocket transport (../../docs/protocol.md):
// same {"command": "..."} message the VS Code extension (../vscode-extension/
// extension.js) gets over TCP, just delivered as a WS text frame instead.
//
// Unlike the VS Code side, roles are flipped here: a browser service worker
// has no server capability at all (no listening sockets), so the Keylex
// daemon runs the WebSocket *server* and this extension connects out to it
// as a client. Port must match config/targets.toml's chrome target.
//
// The daemon won't trust this connection until it sees the shared secret
// from config/secret.token as the very first frame
// (../../docs/protocol.md#trust-model--authentication) -- since a browser
// extension can't read an arbitrary filesystem path, that token is pasted
// once into this extension's options page (options.html/options.js) and
// kept in chrome.storage.local from then on.
const HOST = "127.0.0.1";
const PORT = 7778;
const RECONNECT_DELAY_MS = 1000;

let socket = null;
let sidePanelOpen = false;
let token = null;

async function loadToken() {
  const { keylexToken } = await chrome.storage.local.get("keylexToken");
  return keylexToken || null;
}

function connect() {
  socket = new WebSocket(`ws://${HOST}:${PORT}`);

  socket.addEventListener("open", () => {
    if (!token) {
      console.error("keylex: no token configured yet -- set one via the extension's Options page");
      socket.close();
      return;
    }
    socket.send(JSON.stringify({ token }));
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

async function connectWithToken() {
  token = await loadToken();
  if (!token) {
    console.error("keylex: no token stored yet -- open this extension's Options page and paste one in");
    return;
  }
  connect();
}

chrome.storage.onChanged.addListener((changes, area) => {
  if (area === "local" && changes.keylexToken) {
    token = changes.keylexToken.newValue || null;
    if (token && !socket) connectWithToken();
  }
});

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
// reconnects for free via the connectWithToken() call below.
chrome.alarms.create("keylex-keepalive", { periodInMinutes: 0.4 });
chrome.alarms.onAlarm.addListener((alarm) => {
  if (alarm.name === "keylex-keepalive" && !socket) {
    connectWithToken();
  }
});

connectWithToken();
