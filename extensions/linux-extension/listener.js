// Reference implementation of the keylex/v0 adapter protocol
// (../../docs/protocol.md), for the "system-linux" target in
// config/targets.toml. Unlike vscode-extension/chrome-extension (which
// front one specific program), this listens for OS-wide actions that aren't
// scoped to whichever app is focused -- shutdown, and window/virtual-desktop
// management (close/snap the active window, create a desktop, switch
// desktops, move a window to another desktop) -- and carries them out with
// `wmctrl` / `xdotool` and a `systemctl` shutdown -- same
// newline-delimited-JSON-over-TCP-socket transport as the VS Code adapter.
//
// SECURITY NOTE: there is currently NO authentication on this socket
// (deliberately dropped for now -- see
// ../../docs/protocol.md#trust-model--authentication and
// ../../CLAUDE.md's "Known gaps"). Any local process able to open a TCP
// connection to 127.0.0.1:7779 can trigger shutdown and window/desktop
// commands.
//
// Requires `wmctrl` and `xdotool` on PATH (X11 only, matching the rest of
// this repo's Linux focus/window handling -- see src/focus/linux.rs).
// The GNOME-native actions added below (workspace switching, volume,
// brightness, notifications, overview toggle) additionally need `wpctl`,
// `gdbus`, `gsettings`, and `pgrep` on PATH -- all of which ship with any
// real GNOME/PipeWire desktop, same trust tier as wmctrl/xdotool. None of
// these five have a GNOME Shell session to actually run against in this
// dev environment -- see the probe functions below for the untested
// caveat, same category as search-provider.js.
const net = require("net");
const { execFile } = require("child_process");
const util = require("util");
const execFileAsync = util.promisify(execFile);

const HOST = "127.0.0.1";
const PORT = 7779; // must match config/targets.toml's system-linux target

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

// Reads the live brightness (0-100) from GNOME Settings Daemon's Screen
// power interface via a D-Bus property Get, or throws if it's unavailable
// (no backlight, gnome-settings-daemon not running, ...).
async function currentBrightness() {
  const { stdout } = await execFileAsync("gdbus", [
    "call",
    "--session",
    "--dest",
    "org.gnome.SettingsDaemon.Power",
    "--object-path",
    "/org/gnome/SettingsDaemon/Power",
    "--method",
    "org.freedesktop.DBus.Properties.Get",
    "org.gnome.SettingsDaemon.Power.Screen",
    "Brightness",
  ]);
  const match = stdout.match(/int32\s+(-?\d+)/);
  if (!match) {
    throw new Error(`unexpected gdbus output: ${stdout.trim()}`);
  }
  return parseInt(match[1], 10);
}

// Reads-then-writes brightness rather than stepping blind, so concurrent
// changes (a hardware brightness key, another tool) aren't clobbered.
function stepBrightness(delta) {
  currentBrightness()
    .then((current) => {
      const next = Math.max(0, Math.min(100, current + delta));
      run("gdbus", [
        "call",
        "--session",
        "--dest",
        "org.gnome.SettingsDaemon.Power",
        "--object-path",
        "/org/gnome/SettingsDaemon/Power",
        "--method",
        "org.freedesktop.DBus.Properties.Set",
        "org.gnome.SettingsDaemon.Power.Screen",
        "Brightness",
        `<int32 ${next}>`,
      ]);
    })
    .catch((err) => console.error("keylex: could not read current brightness:", err.message));
}

// Reads-then-flips the do-not-disturb setting for the same reason
// stepBrightness reads before writing -- avoids re-asserting a stale state.
function toggleDoNotDisturb() {
  execFileAsync("gsettings", ["get", "org.gnome.desktop.notifications", "show-banners"])
    .then(({ stdout }) => {
      const bannersShown = stdout.trim() === "true";
      run("gsettings", ["set", "org.gnome.desktop.notifications", "show-banners", bannersShown ? "false" : "true"]);
    })
    .catch((err) => console.error("keylex: could not read notification banner state:", err.message));
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
  "os.audio.volume_up": () => run("wpctl", ["set-volume", "@DEFAULT_AUDIO_SINK@", "5%+"]),
  "os.audio.volume_down": () => run("wpctl", ["set-volume", "@DEFAULT_AUDIO_SINK@", "5%-"]),
  "os.audio.mute_toggle": () => run("wpctl", ["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"]),
  "os.display.brightness_up": () => stepBrightness(10),
  "os.display.brightness_down": () => stepBrightness(-10),
  "os.notifications.toggle_dnd": toggleDoNotDisturb,
  // See probeOverviewToggle below for why this is xdotool, not
  // org.gnome.Shell.Eval, and why it's still best-effort/unverified.
  "os.shell.toggle_overview": () => run("xdotool", ["key", "super"]),
};

// The static core: action id, wire command, and a human title for each,
// reported over the list_actions handshake
// (../../docs/protocol.md#action-catalog-handshake-list_actions) so
// spotlight search (../../src/spotlight/) shows something readable
// instead of the raw command string. Unlike vscode-extension/extension.js,
// there's no "installed extensions" style dynamic universe of commands to
// enumerate for *these* ten -- this listener only ever does exactly these
// ten things, so a flat hardcoded list is the whole story for them.
//
// The five probeXxx() functions below add a second, genuinely live layer
// on top of this static core (see buildActionCatalog): GNOME-native
// actions whose availability and title are re-checked against real system
// state on every list_actions handshake, the same "ask now, don't cache"
// approach extension.js's liveActionCatalog uses for VS Code commands --
// just against gdbus/wpctl/gsettings instead of the VS Code API.
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

// Live GNOME-native workspace switching: unlike createDesktop (a documented
// no-op under GNOME's default dynamic-workspace model, see README), moving
// *between* existing workspaces via `wmctrl -s` correctly drives EWMH
// _NET_CURRENT_DESKTOP under Mutter regardless of dynamic workspaces --
// this probe just reads the real current count/active index on every
// handshake (via wmctrl -d) and emits one switch action per *other*
// existing workspace, instead of a fixed next/previous pair.
async function probeWorkspaces() {
  try {
    const { stdout } = await execFileAsync("wmctrl", ["-d"]);
    const lines = stdout.trim().split("\n").filter(Boolean);
    const ids = lines.map((line) => parseInt(line.trim().split(/\s+/)[0], 10));
    const activeIndex = lines.findIndex((line) => line.trim().split(/\s+/)[1] === "*");
    return ids
      .map((id, index) => ({ id, index }))
      .filter(({ index }) => index !== activeIndex)
      .map(({ id, index }) => ({
        id: `go_to.desktop_${index + 1}`,
        command: `os.desktop.switch_to.${id}`,
        title: `Switch to Desktop ${index + 1}`,
      }));
  } catch (err) {
    return [];
  }
}

// Live volume/mute: reads the current PipeWire/WirePlumber sink state via
// `wpctl` (GNOME's current default audio stack) on every handshake so the
// title reflects the real percentage, not a stale guess. No `pactl`
// fallback for systems without `wpctl` -- this probe just contributes
// nothing there, a documented limitation rather than a second code path.
async function probeVolume() {
  try {
    const { stdout } = await execFileAsync("wpctl", ["get-volume", "@DEFAULT_AUDIO_SINK@"]);
    const match = stdout.match(/Volume:\s*([\d.]+)/);
    if (!match) return [];
    const percent = Math.round(parseFloat(match[1]) * 100);
    const muted = stdout.includes("[MUTED]");
    return [
      { id: "volume.up", command: "os.audio.volume_up", title: `Volume: ${percent}% (Volume Up)` },
      { id: "volume.down", command: "os.audio.volume_down", title: `Volume: ${percent}% (Volume Down)` },
      { id: "volume.mute_toggle", command: "os.audio.mute_toggle", title: muted ? "Unmute" : "Mute" },
    ];
  } catch (err) {
    return [];
  }
}

// Live brightness: reads the real current backlight level on every
// handshake; absent entirely on hardware with no controllable backlight
// (expected, not a bug -- currentBrightness() just throws and the probe
// contributes nothing).
async function probeBrightness() {
  try {
    const percent = await currentBrightness();
    return [
      { id: "brightness.up", command: "os.display.brightness_up", title: `Brightness: ${percent}% (Brightness Up)` },
      {
        id: "brightness.down",
        command: "os.display.brightness_down",
        title: `Brightness: ${percent}% (Brightness Down)`,
      },
    ];
  } catch (err) {
    return [];
  }
}

// Live do-not-disturb state: reads GNOME's actual current
// show-banners setting on every handshake so the reported title always
// names the action that will actually happen next ("Enable"/"Disable"),
// not a fixed label that could already be wrong.
async function probeNotifications() {
  try {
    const { stdout } = await execFileAsync("gsettings", ["get", "org.gnome.desktop.notifications", "show-banners"]);
    const bannersShown = stdout.trim() === "true";
    return [
      {
        id: "notifications.toggle_dnd",
        command: "os.notifications.toggle_dnd",
        title: bannersShown ? "Enable Do Not Disturb" : "Disable Do Not Disturb",
      },
    ];
  } catch (err) {
    return [];
  }
}

// Activities overview toggle -- BEST-EFFORT, UNVERIFIED. The "proper" API,
// org.gnome.Shell.Eval, is gated behind GNOME Shell's Looking Glass "Unsafe
// Mode" (off by default) -- the same access-denied category CLAUDE.md's
// "Spotlight popup" section documents for org.gnome.Shell.GrabAccelerator
// being refused to any caller outside the Shell process itself. `xdotool
// key super` is used instead: it synthesizes a real key event, which
// Mutter's own Super-tap overview detector reacts to directly (unlike the
// client passive-grab path GNOME blocks for global hotkeys like win+t) --
// but this has never been run against a real GNOME Shell session (none
// exists in this dev environment), so treat it the same as the documented
// create.desktop no-op: plausible, not confirmed. The live part of this
// probe is only confirming a GNOME session is actually running, so the
// action doesn't falsely show up on non-GNOME systems.
async function probeOverviewToggle() {
  if (!/gnome/i.test(process.env.XDG_CURRENT_DESKTOP || "")) return [];
  try {
    await execFileAsync("pgrep", ["-x", "gnome-shell"]);
  } catch (err) {
    return [];
  }
  return [{ id: "toggle.overview", command: "os.shell.toggle_overview", title: "Toggle Activities Overview" }];
}

// Runs the static catalog plus all five live probes fresh on every
// list_actions handshake (never cached, never computed at startup) --
// mirrors vscode-extension/extension.js's liveActionCatalog: ask what's
// really there right now, don't ship a fixed list. Promise.allSettled so
// one probe's failure (a missing tool, a non-GNOME session) can never take
// down the rest of the catalog, matching each probe's own fail-closed
// (empty array) contract.
async function buildActionCatalog() {
  const results = await Promise.allSettled([
    probeWorkspaces(),
    probeVolume(),
    probeBrightness(),
    probeNotifications(),
    probeOverviewToggle(),
  ]);
  const live = results.flatMap((result) => (result.status === "fulfilled" ? result.value : []));
  return [...ACTION_CATALOG, ...live];
}

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
  if (message.type === "list_actions") {
    buildActionCatalog().then((catalog) => {
      const response =
        JSON.stringify({
          actions: catalog.map(({ id, command, title }) => ({ id, native_command: command, title })),
        }) + "\n";
      socket.end(response);
    });
    return;
  }

  // Dynamic per-workspace switch commands (see probeWorkspaces) carry their
  // wmctrl desktop id in the command string itself, so they can't live in
  // the flat COMMANDS table above -- checked before it as a prefix match.
  const SWITCH_TO_DESKTOP_PREFIX = "os.desktop.switch_to.";
  if (typeof message.command === "string" && message.command.startsWith(SWITCH_TO_DESKTOP_PREFIX)) {
    const desktopId = message.command.slice(SWITCH_TO_DESKTOP_PREFIX.length);
    console.log("keylex: executing command:", message.command);
    run("wmctrl", ["-s", desktopId]);
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
