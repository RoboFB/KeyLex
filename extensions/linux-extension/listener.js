// Reference implementation of the keylex/v0 adapter protocol
// (../../docs/protocol.md), for the "system-linux" target in
// config/targets.toml. Unlike vscode-extension/chrome-extension (which
// front one specific program), this listens for OS-wide actions that aren't
// scoped to whichever app is focused, and carries them out with `wmctrl` /
// `xdotool` and a `systemctl` shutdown -- same newline-delimited-JSON-over-
// TCP-socket transport and shared-secret auth as the VS Code adapter.
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

const COMMANDS = {
  "os.system.shutdown": () => run("systemctl", ["poweroff"]),
  "os.desktop.show": () => run("wmctrl", ["-k", "on"]),
  "os.window.move_left": () => moveActiveWindow("left"),
  "os.window.move_right": () => moveActiveWindow("right"),
};

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
  if (message.token !== token) {
    console.error("keylex: rejected message with invalid/missing token:", message.command);
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
