// Reference implementation of the keylex/v0 adapter protocol
// (../../docs/protocol.md), for the "system-linux" target in
// config/targets.toml. Unlike vscode-extension/chrome-extension (which
// front one specific program), this listens for OS-wide actions that aren't
// scoped to whichever app is focused -- shutdown, and window/virtual-desktop
// management (close/snap the active window, create a desktop, switch
// desktops, move a window to another desktop) -- and carries them out with
// `wmctrl` / `xdotool` and a `systemctl` shutdown -- same
// newline-delimited-JSON-over-TCP-socket transport and shared-secret auth as
// the VS Code adapter.
//
// Requires `wmctrl` and `xdotool` on PATH (X11 only, matching the rest of
// this repo's Linux focus/window handling -- see src/focus/linux.rs).
const fs = require("fs");
const net = require("net");
const path = require("path");
const { execFile } = require("child_process");

const HOST = "127.0.0.1";
const PORT = 7779; // must match config/targets.toml's system-linux target
const TOKEN_PATH = path.join(__dirname, "..", "..", "config", "secret.token");

const token = fs.readFileSync(TOKEN_PATH, "utf8").trim();

function run(cmd, args) {
  execFile(cmd, args, (err, _stdout, stderr) => {
    if (err) {
      console.error(`keylex: '${cmd} ${args.join(" ")}' failed:`, stderr || err.message);
    }
  });
}

function getDisplayGeometry(callback) {
  execFile("xdotool", ["getdisplaygeometry"], (err, stdout) => {
    if (err) {
      console.error("keylex: xdotool getdisplaygeometry failed:", err.message);
      return;
    }
    const [width, height] = stdout.trim().split(/\s+/).map(Number);
    callback(width, height);
  });
}

function moveActiveWindow(half) {
  getDisplayGeometry((width, height) => {
    const halfWidth = Math.floor(width / 2);
    const x = half === "left" ? 0 : halfWidth;
    // Un-maximize first -- wmctrl can't resize a maximized window.
    run("wmctrl", ["-r", ":ACTIVE:", "-b", "remove,maximized_vert,maximized_horz"]);
    run("wmctrl", ["-r", ":ACTIVE:", "-e", `0,${x},0,${halfWidth},${height}`]);
  });
}

function closeActiveWindow() {
  run("wmctrl", ["-c", ":ACTIVE:"]);
}

// Parses `wmctrl -d`'s output (one line per virtual desktop, e.g.
// "0  *  DG: 1920x1080  VP: 0,0  WA: 0,27x1920x1053  Workspace 1") into the
// list of real desktop ids (in on-screen order) and which one is currently
// active, for the desktop-switching/window-moving/desktop-creating helpers
// below. `ids` is used (rather than just array indices) since wmctrl's
// desktop numbering isn't guaranteed contiguous from 0 on every WM.
function listDesktops(callback) {
  execFile("wmctrl", ["-d"], (err, stdout) => {
    if (err) {
      console.error("keylex: wmctrl -d failed:", err.message);
      return;
    }
    const lines = stdout.trim().split("\n").filter(Boolean);
    const ids = lines.map((line) => parseInt(line.trim().split(/\s+/)[0], 10));
    const activeIndex = lines.findIndex((line) => line.trim().split(/\s+/)[1] === "*");
    callback(ids, activeIndex);
  });
}

// Grows the desktop count by one via `wmctrl -n` (EWMH _NET_NUMBER_OF_DESKTOPS).
// NOTE: GNOME Shell's *default* config uses dynamic workspaces (one is
// created/destroyed automatically as needed, no fixed count to grow) --
// this only does something useful under a WM using a fixed workspace count
// (most non-GNOME EWMH window managers), or GNOME with dynamic workspaces
// turned off in Settings.
function createDesktop() {
  listDesktops((ids) => {
    run("wmctrl", ["-n", String(ids.length + 1)]);
  });
}

// Switches which desktop is currently shown (`delta` = +1/-1), wrapping
// around at either end. Does not move any window.
function switchDesktop(delta) {
  listDesktops((ids, activeIndex) => {
    if (activeIndex < 0 || ids.length === 0) {
      console.error("keylex: could not determine the active desktop from wmctrl -d");
      return;
    }
    const targetIndex = (activeIndex + delta + ids.length) % ids.length;
    run("wmctrl", ["-s", String(ids[targetIndex])]);
  });
}

// Moves the focused window to the adjacent desktop (`delta` = +1/-1,
// wrapping around) and follows it there, matching the common WM
// "move window to workspace left/right" behavior (as opposed to
// switchDesktop, which only changes which desktop is shown).
function sendWindowToDesktop(delta) {
  listDesktops((ids, activeIndex) => {
    if (activeIndex < 0 || ids.length === 0) {
      console.error("keylex: could not determine the active desktop from wmctrl -d");
      return;
    }
    const targetId = String(ids[(activeIndex + delta + ids.length) % ids.length]);
    run("wmctrl", ["-r", ":ACTIVE:", "-t", targetId]);
    run("wmctrl", ["-s", targetId]);
  });
}

const COMMANDS = {
  "os.system.shutdown": () => run("systemctl", ["poweroff"]),
  "os.desktop.show": () => run("wmctrl", ["-k", "on"]),
  "os.window.move_left": () => moveActiveWindow("left"),
  "os.window.move_right": () => moveActiveWindow("right"),
  "os.window.close": closeActiveWindow,
  "os.desktop.create": createDesktop,
  "os.desktop.next": () => switchDesktop(1),
  "os.desktop.previous": () => switchDesktop(-1),
  "os.window.move_to_next_desktop": () => sendWindowToDesktop(1),
  "os.window.move_to_previous_desktop": () => sendWindowToDesktop(-1),
};

// Mirrors this folder's capabilities.toml -- action id, wire command, and a
// human title for each, reported over the list_actions handshake
// (../../docs/protocol.md#action-catalog-handshake-list_actions) so
// spotlight search (../../src/spotlight/) shows something readable
// instead of the raw command string. Unlike vscode-extension/extension.js,
// there's no "installed extensions" style dynamic universe of commands
// here to auto-discover -- this listener only ever does exactly these ten
// things, so a small hardcoded catalog is the whole story, not a stand-in
// for a real discovery mechanism.
const ACTION_CATALOG = [
  { id: "shutdown", command: "os.system.shutdown", title: "Shut Down" },
  { id: "show.desktop", command: "os.desktop.show", title: "Show Desktop" },
  { id: "move.left", command: "os.window.move_left", title: "Snap Window Left" },
  { id: "move.right", command: "os.window.move_right", title: "Snap Window Right" },
  { id: "close.window", command: "os.window.close", title: "Close Window" },
  { id: "create.desktop", command: "os.desktop.create", title: "New Desktop" },
  { id: "go_to.next_desktop", command: "os.desktop.next", title: "Switch to Next Desktop" },
  { id: "go_to.previous_desktop", command: "os.desktop.previous", title: "Switch to Previous Desktop" },
  { id: "send.next_desktop", command: "os.window.move_to_next_desktop", title: "Move Window to Next Desktop" },
  {
    id: "send.previous_desktop",
    command: "os.window.move_to_previous_desktop",
    title: "Move Window to Previous Desktop",
  },
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

  if (message.type === "list_actions") {
    const response =
      JSON.stringify({
        actions: ACTION_CATALOG.map(({ id, command, title }) => ({ id, native_command: command, title })),
      }) + "\n";
    socket.end(response);
    return;
  }

  const handler = COMMANDS[message.command];
  if (!handler) {
    console.error("keylex: rejected unknown command:", message.command);
    return;
  }
  console.log("keylex: executing command:", message.command);
  handler();
}

server.on("error", (err) => {
  console.error("keylex: socket server error (is another instance already running?):", err);
});

server.listen(PORT, HOST, () => {
  console.log(`keylex linux system listener on ${HOST}:${PORT} (Ctrl+C to stop)`);
});
