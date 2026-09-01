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
// brightness, notifications, shell UI toggles, system actions)
// additionally need `wpctl`, `gdbus`, and `gsettings` on PATH -- all of
// which ship with any real GNOME/PipeWire desktop, same trust tier as
// wmctrl/xdotool. None of these have a GNOME Shell session to actually run
// against in this dev environment -- see the probe functions below for the
// untested caveat, same category as search-provider.js.
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

const SYSTEM_ACTIONS_DEST = "org.gnome.Shell";
const SYSTEM_ACTIONS_PATH = "/org/gnome/Shell/SystemActions";

// GNOME's own live action registry for session-level actions (lock,
// suspend, restart, power-off, logout, switch-user, ...) -- the same
// org.gtk.Actions (GActionGroup-over-D-Bus) object GNOME's system menu
// itself calls into. See probeSystemActions below for why this is the
// closest real analog to vscode.commands.getCommands() this file has.
async function listSystemActions() {
  const { stdout } = await execFileAsync("gdbus", [
    "call",
    "--session",
    "--dest",
    SYSTEM_ACTIONS_DEST,
    "--object-path",
    SYSTEM_ACTIONS_PATH,
    "--method",
    "org.gtk.Actions.List",
  ]);
  return [...stdout.matchAll(/'([^']+)'/g)].map((match) => match[1]);
}

// Describe() replies look like "(true, '', @av [])" -- only the leading
// enabled/disabled boolean matters here.
async function systemActionEnabled(name) {
  const { stdout } = await execFileAsync("gdbus", [
    "call",
    "--session",
    "--dest",
    SYSTEM_ACTIONS_DEST,
    "--object-path",
    SYSTEM_ACTIONS_PATH,
    "--method",
    "org.gtk.Actions.Describe",
    name,
  ]);
  return /^\(\s*true\b/.test(stdout);
}

function activateSystemAction(name) {
  run("gdbus", [
    "call",
    "--session",
    "--dest",
    SYSTEM_ACTIONS_DEST,
    "--object-path",
    SYSTEM_ACTIONS_PATH,
    "--method",
    "org.gtk.Actions.Activate",
    name,
    "[]",
    "{}",
  ]);
}

// A handful of well-known names get a nicer title; anything else still
// gets a readable one auto-derived from its name, the same "don't drop
// what you don't recognize" fallback extension.js uses for uncontributed
// VS Code commands -- so a GNOME version that adds a new system action
// still surfaces it here with no code change.
const SYSTEM_ACTION_TITLES = {
  "lock-screen": "Lock Screen",
  suspend: "Suspend",
  hibernate: "Hibernate",
  restart: "Restart",
  "power-off": "Power Off",
  logout: "Log Out",
  "switch-user": "Switch User",
  "lock-orientation": "Lock Screen Orientation",
};

function titleForSystemAction(name) {
  return SYSTEM_ACTION_TITLES[name] || name.split("-").map((word) => word[0].toUpperCase() + word.slice(1)).join(" ");
}

// GNOME accelerator syntax ("<Super>a", "<Super>", "<Control><Alt>t") ->
// xdotool's "super+a" / "super" / "ctrl+alt+t". Returns null (rather than
// guessing) for anything that isn't this simple modifiers-plus-one-plain-
// key shape -- a named key like "Escape" or a multi-key chord is left
// unadvertised instead of risking a wrong keypress.
const ACCEL_MOD_NAMES = { Super: "super", Control: "ctrl", Primary: "ctrl", Alt: "alt", Shift: "shift" };

function acceleratorToXdotoolKey(accel) {
  const mods = [...accel.matchAll(/<(\w+)>/g)].map((match) => ACCEL_MOD_NAMES[match[1]]);
  if (mods.some((mod) => !mod)) return null;
  const key = accel.replace(/<\w+>/g, "");
  if (key && !/^[a-zA-Z0-9]$/.test(key)) return null;
  return [...mods, key.toLowerCase()].filter(Boolean).join("+");
}

// Reads the live accelerator array for one gsettings keybinding key (e.g.
// "['<Super>a']" or "[]" if cleared), returning the first binding or null.
// Fails closed (null) if the schema/key doesn't exist on this GNOME
// version -- gsettings itself errors in that case.
async function readAccelerator(schema, key) {
  try {
    const { stdout } = await execFileAsync("gsettings", ["get", schema, key]);
    const match = stdout.match(/'([^']*)'/);
    return match ? match[1] : null;
  } catch (err) {
    return null;
  }
}

// The GNOME Shell UI toggles worth surfacing beyond Activities overview --
// each backed by a named, user-configurable org.gnome.shell.keybindings
// key, so the actual key synthesized always matches this machine's real
// current binding instead of an assumed default.
const SHELL_KEYBINDING_TOGGLES = [
  {
    schema: "org.gnome.shell.keybindings",
    key: "toggle-overview",
    id: "toggle.overview",
    command: "os.shell.toggle_overview",
    title: "Toggle Activities Overview",
  },
  {
    schema: "org.gnome.shell.keybindings",
    key: "toggle-application-view",
    id: "toggle.app_grid",
    command: "os.shell.toggle_app_grid",
    title: "Toggle App Grid",
  },
  {
    schema: "org.gnome.shell.keybindings",
    key: "toggle-message-tray",
    id: "toggle.message_tray",
    command: "os.shell.toggle_message_tray",
    title: "Toggle Notification List",
  },
  {
    schema: "org.gnome.shell.keybindings",
    key: "toggle-quick-settings",
    id: "toggle.quick_settings",
    command: "os.shell.toggle_quick_settings",
    title: "Toggle Quick Settings",
  },
];

// Re-reads the live accelerator at dispatch time (not whatever the last
// list_actions probe saw) before synthesizing it, same read-then-act
// posture as stepBrightness/toggleDoNotDisturb above.
function dispatchShellKeybindingToggle(schema, key) {
  readAccelerator(schema, key)
    .then((accel) => {
      const xdotoolKey = accel && acceleratorToXdotoolKey(accel);
      if (!xdotoolKey) {
        console.error(`keylex: no usable live binding for ${schema} ${key}`);
        return;
      }
      run("xdotool", ["key", xdotoolKey]);
    })
    .catch((err) => console.error(`keylex: could not read binding for ${schema} ${key}:`, err.message));
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
  // See probeShellKeybindingToggles below for why these are live-read
  // xdotool key synthesis, not org.gnome.Shell.Eval, and why they're still
  // best-effort/unverified.
  "os.shell.toggle_overview": () => dispatchShellKeybindingToggle("org.gnome.shell.keybindings", "toggle-overview"),
  "os.shell.toggle_app_grid": () =>
    dispatchShellKeybindingToggle("org.gnome.shell.keybindings", "toggle-application-view"),
  "os.shell.toggle_message_tray": () =>
    dispatchShellKeybindingToggle("org.gnome.shell.keybindings", "toggle-message-tray"),
  "os.shell.toggle_quick_settings": () =>
    dispatchShellKeybindingToggle("org.gnome.shell.keybindings", "toggle-quick-settings"),
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
// The seven probeXxx() functions below add a second, genuinely live layer
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

// GNOME's own live action registry (see listSystemActions/
// systemActionEnabled/activateSystemAction above) -- the closest real
// analog to vscode.commands.getCommands() this file has: List() returns
// whatever this exact session currently exposes (varies by policy and
// hardware -- switch-user disappears with one account, hibernate
// disappears if the kernel doesn't support it), and Describe()'s enabled
// flag is checked live per action rather than assuming every listed name
// is currently usable.
async function probeSystemActions() {
  try {
    const names = await listSystemActions();
    const checks = await Promise.allSettled(
      names.map(async (name) => ({ name, enabled: await systemActionEnabled(name) })),
    );
    return checks
      .filter((check) => check.status === "fulfilled" && check.value.enabled)
      .map((check) => {
        const { name } = check.value;
        return {
          id: `system_action.${name}`,
          command: `os.shell.system_action.${name}`,
          title: titleForSystemAction(name),
        };
      });
  } catch (err) {
    return [];
  }
}

// GNOME Shell UI toggles (Activities overview, app grid, notification
// list, quick settings) -- each only advertised when its
// org.gnome.shell.keybindings key currently holds a binding this file
// knows how to safely synthesize (see readAccelerator/
// acceleratorToXdotoolKey above), so a cleared or unusually-remapped
// binding just doesn't appear instead of advertising something that can't
// actually be triggered.
//
// BEST-EFFORT, UNVERIFIED, same caveat category as create.desktop's
// documented no-op: the "proper" API, org.gnome.Shell.Eval, is gated
// behind GNOME Shell's Looking Glass "Unsafe Mode" (off by default) -- the
// same access-denied category CLAUDE.md's "Spotlight popup" section
// documents for org.gnome.Shell.GrabAccelerator being refused to any
// caller outside the Shell process itself. Synthesizing the live-read key
// via xdotool is used instead -- plausible (Mutter's own accelerator
// handling reacts to synthesized key events the same as real ones), but
// never run against a real GNOME Shell session, since none exists in this
// project's dev/CI environment.
async function probeShellKeybindingToggles() {
  const checks = await Promise.allSettled(
    SHELL_KEYBINDING_TOGGLES.map(async (toggle) => {
      const accel = await readAccelerator(toggle.schema, toggle.key);
      const xdotoolKey = accel && acceleratorToXdotoolKey(accel);
      if (!xdotoolKey) return null;
      return { id: toggle.id, command: toggle.command, title: toggle.title };
    }),
  );
  return checks
    .filter((check) => check.status === "fulfilled" && check.value)
    .map((check) => check.value);
}

// Runs the static catalog plus all seven live probes fresh on every
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
    probeSystemActions(),
    probeShellKeybindingToggles(),
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

  // Dynamic per-system-action commands (see probeSystemActions) carry the
  // org.gtk.Actions action name in the command string itself, same reason
  // the switch-to-desktop prefix above can't live in the flat COMMANDS
  // table.
  const SYSTEM_ACTION_PREFIX = "os.shell.system_action.";
  if (typeof message.command === "string" && message.command.startsWith(SYSTEM_ACTION_PREFIX)) {
    const actionName = message.command.slice(SYSTEM_ACTION_PREFIX.length);
    console.log("keylex: executing command:", message.command);
    activateSystemAction(actionName);
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
