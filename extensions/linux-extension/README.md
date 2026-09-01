# Keylex Linux system listener

Reference client for the `keylex/v0` socket transport (see
[../../docs/protocol.md](../../docs/protocol.md)), backing the
`system-linux` target in [../../config/targets.toml](../../config/targets.toml).
Unlike `../vscode-extension` or `../chrome-extension`, this isn't tied to one
focused program -- it handles OS-wide actions (`shutdown`,
`show.desktop`, `move.left`, `move.right`) regardless of
what app currently has focus, via `wmctrl` / `xdotool` (X11 only, same
constraint as [../../src/focus/linux.rs](../../src/focus/linux.rs)) and
`systemctl poweroff`.

## Requirements

- Node.js
- `wmctrl` and `xdotool` on `PATH` (`apt install wmctrl xdotool` on
  Debian/Ubuntu)
- `systemctl poweroff` runnable without a password prompt for your user --
  true by default on most desktop Linux systems via polkit for the active
  session; if it isn't on yours, `shutdown` will just fail silently
  (its `keylex/v0` dispatch is fire-and-forget, per the protocol doc).
- For the GNOME-native actions below (workspace switching, volume,
  brightness, notifications, shell UI toggles, system actions): `wpctl`
  (part of `wireplumber`) and `gdbus`/`gsettings` (part of
  `glib2`/`libglib2.0-bin`) on `PATH`. All of these ship with any real
  GNOME/PipeWire desktop by default, same trust tier as `wmctrl`/`xdotool`
  above.

## Run

```bash
node listener.js
```

Run this alongside the daemon (`cargo run` from the repo root). It listens on
`127.0.0.1:7779`, matching the `system-linux` target's `address` in
`config/targets.toml`. **No authentication yet** -- see
[../../docs/protocol.md](../../docs/protocol.md#trust-model--authentication)
and [../../CLAUDE.md](../../CLAUDE.md)'s "Known gaps": any local process
that can reach that port can trigger `shutdown` and the window/desktop
commands below.

## Command mapping

| `command` (wire)                       | Action                    | Implementation                                                        |
|------------------------------------------|---------------------------|---------------------------------------------------------------------|
| `os.system.shutdown`                    | `shutdown`                | `systemctl poweroff`                                                  |
| `os.desktop.show`                       | `show.desktop`            | `wmctrl -k on`                                                        |
| `os.window.move_left`                   | `move.left`               | un-maximize + `wmctrl -r :ACTIVE: -e` to left half                    |
| `os.window.move_right`                  | `move.right`              | un-maximize + `wmctrl -r :ACTIVE: -e` to right half                   |
| `os.window.close`                       | `close.window`            | `wmctrl -c :ACTIVE:`                                                  |
| `os.desktop.create`                     | `create.desktop`          | `wmctrl -n <current desktop count + 1>` (see caveat below)            |
| `os.desktop.next`                       | `go_to.next_desktop`      | switch to the next desktop (`wmctrl -s`), doesn't move any window     |
| `os.desktop.previous`                   | `go_to.previous_desktop`  | switch to the previous desktop (`wmctrl -s`)                          |
| `os.window.move_to_next_desktop`        | `send.next_desktop`       | move the active window to the next desktop and follow it there        |
| `os.window.move_to_previous_desktop`    | `send.previous_desktop`   | move the active window to the previous desktop and follow it there    |

The desktop-switching/moving commands read `wmctrl -d`'s output to find the
current desktop and total count, so they work on however many virtual
desktops are actually configured, wrapping around at either end.

**`create.desktop` caveat:** this grows the desktop count via
`wmctrl -n` (EWMH `_NET_NUMBER_OF_DESKTOPS`), which only does something
useful under a fixed-workspace-count window manager. **GNOME Shell's default
config uses dynamic workspaces** (one is created/destroyed automatically the
moment you need it, no fixed count to grow) -- on stock GNOME this command
is effectively a no-op; disable dynamic workspaces in GNOME Settings, or use
a WM with a fixed workspace count, for it to do anything.

## Live GNOME-native actions

Beyond the ten static commands above, `listener.js` answers the
`list_actions` handshake
([../../docs/protocol.md](../../docs/protocol.md#action-catalog-handshake-list_actions))
with seven more categories that are re-checked against real system state on
*every* handshake, rather than a fixed list baked into source -- mirroring
how `vscode-extension/extension.js` queries VS Code's live command registry
every time instead of shipping a static catalog. Each one fails closed
(contributes nothing) if its underlying tool/service isn't available,
rather than breaking the rest of the catalog.

| Category | `command` (wire) | Live-checked at handshake time | Implementation |
|---|---|---|---|
| Workspace switching | `os.desktop.switch_to.<id>` (one per existing workspace other than the active one) | Real current workspace count/active index, via `wmctrl -d` | `wmctrl -s <id>` -- unlike `create.desktop`, switching *between* existing workspaces correctly drives EWMH `_NET_CURRENT_DESKTOP` under Mutter even in GNOME's dynamic-workspace mode |
| Volume | `os.audio.volume_up` / `os.audio.volume_down` / `os.audio.mute_toggle` | Current volume %/mute state, via `wpctl get-volume` | `wpctl set-volume`/`set-mute` against `@DEFAULT_AUDIO_SINK@` (PipeWire/WirePlumber, GNOME's default audio stack -- no `pactl`-only fallback) |
| Brightness | `os.display.brightness_up` / `os.display.brightness_down` | Current backlight %, via a D-Bus `Get` on `org.gnome.SettingsDaemon.Power.Screen`'s `Brightness` property | Read-then-write `gdbus call` pair, clamped 0-100; contributes nothing on hardware with no controllable backlight |
| Do Not Disturb | `os.notifications.toggle_dnd` | Current `org.gnome.desktop.notifications show-banners` value, via `gsettings get` | Read-then-flip `gsettings set`; the reported title ("Enable"/"Disable Do Not Disturb") always names the action that will actually run next |

### System actions -- the closest thing GNOME has to `getCommands()`

`os.shell.system_action.<name>` (one per currently-enabled action) is
sourced from `org.gnome.Shell`'s `/org/gnome/Shell/SystemActions` object,
which implements the standard `org.gtk.Actions` (GActionGroup-over-D-Bus)
interface -- the same live, introspectable registry GNOME's own system
menu/quick-settings reads from. `listener.js` calls `List()` to get every
action name this exact session currently exposes (varies by policy and
hardware -- `switch-user` disappears with a single account, `hibernate`
disappears if the kernel doesn't support it), then `Describe(name)` per
name to keep only the ones reporting `enabled`. Dispatch calls
`Activate(name, [], {})` directly -- a real callable D-Bus method, not a
synthesized keypress, so this category isn't subject to the
overview-toggle-style caveat below. Typical names: `lock-screen`,
`suspend`, `hibernate`, `restart`, `power-off`, `logout`, `switch-user`,
`lock-orientation`. A handful get a nicer title (see
`SYSTEM_ACTION_TITLES` in `listener.js`); anything else still gets a
readable auto-title-cased fallback, so a future GNOME version's new system
actions show up with no code change here.

### Shell UI keybinding toggles

`os.shell.toggle_overview` / `toggle_app_grid` / `toggle_message_tray` /
`toggle_quick_settings` each read the *live, currently-configured*
accelerator for one `org.gnome.shell.keybindings` key (`toggle-overview`,
`toggle-application-view`, `toggle-message-tray`, `toggle-quick-settings`
respectively) via `gsettings get`, convert it to `xdotool key` syntax, and
only advertise/dispatch the action when that conversion succeeds --
so a binding the user cleared, or remapped to something more complex than
one modifier plus one plain key (a named key like `Escape`, a multi-key
chord), simply doesn't show up rather than advertising something that
can't actually be triggered. Dispatch re-reads the binding fresh each time
rather than trusting whatever the last `list_actions` handshake saw.

**Best-effort, unverified**, same caveat category as `create.desktop`'s
documented no-op: the "proper" API, `org.gnome.Shell.Eval`, is gated behind
GNOME Shell's Looking Glass "Unsafe Mode" (off by default) -- the same
access-denied category [CLAUDE.md](../../CLAUDE.md)'s "Spotlight popup"
section already documents for `org.gnome.Shell.GrabAccelerator` being
refused to any external caller. Synthesizing the live-read key via
`xdotool key` is used instead -- plausible (Mutter's own accelerator
handling should react to a synthesized key event the same as a real one)
but never run against a real GNOME Shell session, since none exists in
this project's dev/CI environment.

## GNOME Shell search provider (`search-provider.js`)

**Untested outside a real GNOME Shell session** -- this was written against
the documented `org.gnome.Shell.SearchProvider2` D-Bus interface and its
class setup/method signatures were checked to load correctly against the
real `dbus-next` library, but there's no GNOME Shell (or even a session
D-Bus bus) available in this project's dev/CI environment to actually
register against and click through. Same caveat this repo already carries
for `src/capture/windows.rs` -- see [CLAUDE.md](../../CLAUDE.md)'s "Known
gaps".

Unlike `listener.js` (which speaks `keylex/v0` over a socket to receive
dispatches *from* the daemon), this is the other direction: it lets GNOME
Shell's own Activities search bar fuzzy-search and run Keylex actions,
without Keylex needing to build its own always-on-top launcher window --
"talking nicely to the window manager" instead of fighting it. It does this
by shelling out to the `keylex` binary itself (`--spotlight-query` /
`--spotlight-run`, see `src/spotlight/` and `src/cli.rs`) for every
search and activation, so the ranking is always the one real (pure-Rust,
`nucleo-matcher`) fuzzy engine -- this file is just D-Bus glue, not a second
search implementation.

### Setup

1. `cargo build --release` from the repo root (the search provider shells
   out to `target/release/keylex` by default; set `KEYLEX_BIN` in the
   `.service` file below to point elsewhere, e.g. a debug build, if you'd
   rather use one).
2. `npm install` in this directory (pulls in `dbus-next`, a pure-JS D-Bus
   client/service library -- no native build step).
3. Copy the two registration files GNOME Shell needs into your user data
   dirs and adjust paths:
   ```bash
   mkdir -p ~/.local/share/applications ~/.local/share/gnome-shell/search-providers ~/.local/share/dbus-1/services
   cp com.keylex.Spotlight.desktop ~/.local/share/applications/
   cp com.keylex.Spotlight.search-provider.ini ~/.local/share/gnome-shell/search-providers/
   sed "s#/REPLACE/WITH/ABSOLUTE/PATH/TO/KeyLex#$(cd ../.. && pwd)#" \
     com.keylex.SearchProvider.service.example > ~/.local/share/dbus-1/services/com.keylex.SearchProvider.service
   ```
4. Log out and back in (GNOME Shell only rescans these directories on
   startup). GNOME Shell then D-Bus-activates `search-provider.js`
   on-demand the first time you type in Activities search -- there's
   nothing to keep running manually.

If it doesn't show up: `busctl --user list | grep keylex` should show
`com.keylex.SearchProvider` once GNOME Shell has tried to activate it, and
`journalctl --user -f` while typing in Activities search will show this
script's `console.error` output (stdio from a D-Bus-activated process is
captured by the systemd user journal, not a visible terminal).
