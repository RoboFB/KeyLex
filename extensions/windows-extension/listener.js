// Reference implementation of the keylex/v0 adapter protocol
// (../../docs/protocol.md), for the "system-windows" target in
// config/targets.toml. Windows counterpart of extensions/linux-extension --
// same OS-wide-action idea, same newline-delimited-JSON-over-TCP-socket
// transport and shared-secret auth as the VS Code adapter, but carries
// commands out via PowerShell (Win32 SetWindowPos/ShowWindow through inline
// C#, and Shell.Application for the desktop toggle) instead of wmctrl.
//
// Untested outside a real Windows machine -- this repo's dev environment is
// Linux-only, same caveat as src/capture/windows.rs and src/focus/windows.rs.
const fs = require("fs");
const net = require("net");
const path = require("path");
const { execFile } = require("child_process");

const HOST = "127.0.0.1";
const PORT = 7780; // must match config/targets.toml's system-windows target
const TOKEN_PATH = path.join(__dirname, "..", "..", "config", "secret.token");

const token = fs.readFileSync(TOKEN_PATH, "utf8").trim();

function runPowerShell(script) {
  execFile(
    "powershell.exe",
    ["-NoProfile", "-NonInteractive", "-Command", script],
    (err, _stdout, stderr) => {
      if (err) {
        console.error("keylex: powershell command failed:", stderr || err.message);
      }
    }
  );
}

// Shared P/Invoke declarations for the two window-move commands below.
const WINDOW_HELPER = `
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class KeylexWin32 {
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
  [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr hWnd, IntPtr hWndInsertAfter, int X, int Y, int cx, int cy, uint uFlags);
  [DllImport("user32.dll")] public static extern int GetSystemMetrics(int nIndex);
}
"@
$hwnd = [KeylexWin32]::GetForegroundWindow()
[KeylexWin32]::ShowWindow($hwnd, 9) | Out-Null  # SW_RESTORE, so a maximized window can be resized
$screenWidth = [KeylexWin32]::GetSystemMetrics(0)
$screenHeight = [KeylexWin32]::GetSystemMetrics(1)
$halfWidth = [int]($screenWidth / 2)
`;

function moveActiveWindow(half) {
  const x = half === "left" ? 0 : "$halfWidth";
  runPowerShell(
    `${WINDOW_HELPER}[KeylexWin32]::SetWindowPos($hwnd, [IntPtr]::Zero, ${x}, 0, $halfWidth, $screenHeight, 0x0040) | Out-Null`
  );
}

const COMMANDS = {
  shutdown: () => runPowerShell("shutdown /s /t 0"),
  show_desktop: () => runPowerShell("(New-Object -ComObject Shell.Application).ToggleDesktop()"),
  move_left: () => moveActiveWindow("left"),
  move_right: () => moveActiveWindow("right"),
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
  console.log(`keylex windows system listener on ${HOST}:${PORT} (Ctrl+C to stop)`);
});
