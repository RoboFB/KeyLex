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
