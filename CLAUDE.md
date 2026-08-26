# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What Keylex is

Keylex intercepts every keystroke as deep in the OS as possible (evdev/
uinput on Linux, a WH_KEYBOARD_LL hook on Windows), resolves the bound
ones into abstract **actions** (`close.tab`, `go_to.definition`, `save`),
figures out which application is currently focused, and dispatches each
action through that app's **native API** — instead of just simulating a
keycode. A keycode is only sent as a fallback, when the focused app has
no native adapter or doesn't support that action. See
[README.md](README.md) for the German-language project pitch and the
architecture diagram.

The long-term ambition (not yet built) is for this action vocabulary to
become a shared, documented protocol that device firmware and application
plugins can target directly — see [docs/protocol.md](docs/protocol.md) for
the current (draft, unstable) wire format and its LSP-inspired rationale.

Everything here is early-stage. The core capture/dispatch pipeline
(config → registry → router/capture → adapter/fallback) is written in
Rust and is tested on Linux; the Windows capture backend exists but is
untestable outside a Windows machine (this dev environment is Linux, no
Windows box available). Application-side and OS-side integrations each
live in their own subfolder under [extensions/](extensions/), in whatever
language fits that target's ecosystem — the VS Code extension
(`extensions/vscode-extension/`, plain JS, using the official `vscode`
extension API), the Chrome extension (`extensions/chrome-extension/`,
Manifest V3, `chrome.tabs`/`chrome.windows`/`chrome.sidePanel`), and the
Linux/Windows OS-wide system listeners (`extensions/linux-extension/`,
`extensions/windows-extension/` — small Node scripts handling
focus-independent actions like `system.shutdown` and `window.move_left`
via `wmctrl`/PowerShell respectively, see "Dispatch flow" below) all
exist; Neovim/terminal adapters are unimplemented stubs. macOS has no
capture backend at all yet, not even an untested one (see "Known gaps"
below) — Linux and Windows are the only supported platforms for now.

## Commands

```bash
cargo build              # compiles the daemon (Linux backend on this machine)
cargo run                 # real capture loop, blocks (needs evdev/uinput perms)
cargo run -- --demo        # two hardcoded dispatches, no capture/hardware needed
cargo run -- --config-dir <path>  # load actions.toml/targets.toml from elsewhere

cargo test                # unit tests (src/config.rs) + integration tests (tests/dispatch.rs)
cargo test <name>          # single test by substring
```

There is no separate lint step configured beyond `cargo clippy`.

## Architecture

### Two config layers (`config/*.toml`), loaded by `Registry` (`src/config.rs`)

1. **`actions.toml`** — the action vocabulary itself: one `[[action]]`
   entry per action, with an optional `key` field (a `"ctrl+w"`-style
   combo — the same syntax used for `fallback_keycode`) that binds it to
   a physical key, a `fallback_tier` (`silent` / `notify_attempt` /
   `notify_only`), and an optional `fallback_keycode`. An action with no
   `key` simply isn't reachable from the keyboard yet (e.g.
   `go_to.definition` today) but can still be dispatched once something
   else triggers it. There is no separate device-binding layer and no
   enforced verb+object grammar — action IDs are plain strings; the
   dot-separated naming (`close.tab`) is a convention, not validated at
   load time.

   An action may instead (never both) bind a `chord`: an array of two or
   more key tokens (same vocabulary as `key`, including modifiers) that
   must all be held down together, order-independent — e.g.
   `chord = ["ctrl", "d", "f"]`. See
   [docs/protocol.md](docs/protocol.md#chorded-triggers) for the full
   syntax, validation rules, and the debounce/replay behavior this adds to
   capture.
2. **`targets.toml`** — output side. Per target program: which process
   names identify it (`match_process`), which adapter reaches it, and a
   `supports` whitelist mapping action IDs to that program's native
   command strings. Also holds `[[system_action]]` entries — OS-level
   actions that don't depend on the focused app at all — and, as an
   ordinary `[[target]]` with an `os` field instead of `match_process`
   (`"system-linux"` / `"system-windows"`), the OS-wide system listeners
   under `extensions/`. `Registry::system_target` picks whichever one's
   `os` matches `std::env::consts::OS`, so only one is ever live per
   platform even though both entries are declared in the same file.

### Capture rule

A key combo that matches a bound action's `key` is **always consumed**
(never reaches the OS/app directly, only indirectly via the fallback
path) and dispatched. Everything else is re-emitted unchanged. This one
rule is what "intercept every keycode as deeply as possible" means in
practice — there's no per-binding grab/observe mode to configure.

A `chord`-bound action extends this: any key that's a member of *some*
configured chord is held in a short-lived "pending" state instead of being
re-emitted immediately, since a lone keystroke can't yet be told apart
from the start of a chord. If the rest of the chord follows within the
debounce window, all of it is consumed and dispatched, same as a matched
`key`. If it doesn't (timeout, or a key that breaks every possible match
arrives), the pending key(s) are replayed as ordinary keystrokes instead —
from the app's perspective, as if the brief hold never happened, aside
from the added latency. Non-member keys are never buffered, so this adds
no overhead to ordinary typing.

### Adapters (`src/adapters/`)

One implementation per transport, registered in `main.rs`'s `build_adapters()`
map keyed by the `adapter` string used in `targets.toml` (`"socket"`,
`"websocket"`, later `"rpc"`, …). `adapters::SocketAdapter` (TCP, Keylex is
the client) backs the VS Code target; `adapters::WebSocketAdapter`
(Keylex runs the WebSocket *server*, since a browser extension can only be
a client) backs the Chrome target — see
[docs/protocol.md](docs/protocol.md) for the full spec of both transports.
Neovim/terminal adapters are unimplemented; each will need its own
transport.

Both adapters authenticate every message with a shared secret the daemon
generates on first run at `<config-dir>/secret.token` (`src/auth.rs`,
git-ignored, never committed): `SocketAdapter` sends it with every
`command`, and `WebSocketAdapter` requires it as the first frame on a
freshly accepted connection before promoting that connection into the slot
`send()` uses (this also fixes what would otherwise be a "last-connect-wins"
hijack risk on the single-connection WebSocket transport). The WebSocket
adapter can additionally check the handshake's `Origin` header against a
per-target `allowed_origin` in `targets.toml`. See
[docs/protocol.md](docs/protocol.md#trust-model--authentication) for the
full wire-level contract and threat model.

### Dispatch flow (`src/dispatch.rs`)

`Router::dispatch(action_id, focused_process)`:
1. `Registry::target_for_process` — does a target's `match_process` include
   the focused process? If yes and it `supports` this action → **native**:
   look up the adapter by `target.adapter` and call `adapter.send(target,
   native_command)`.
2. Otherwise, `Registry::system_target` — the OS-wide listener target for
   this platform (`extensions/linux-extension` or
   `extensions/windows-extension`, see "Two config layers" above). If it
   `supports` this action → **native** via that target instead, regardless
   of what's focused (used for actions like `system.shutdown` and
   `window.move_left`/`window.move_right` that don't belong to any one
   app).
3. Otherwise → **fallback**: based on `ActionSpec.fallback_tier`, either
   send `fallback_keycode` via the platform's `FallbackSender` (optionally
   also notifying), or — if there's no usable keycode / tier is
   `notify_only` — report **unsupported** and just notify.

`Notifier` is a log-only placeholder on both platforms — real OS
notifications are not implemented yet (a known, deferred gap). Fallback
keycode injection *is* real on both platforms: Linux writes the keycode
through the same `uinput` virtual device used for passthrough re-emission
(`src/capture/linux.rs`), Windows uses `SendInput`
(`src/capture/windows.rs`).

### Capture backends (`src/capture/`)

- `linux.rs` — grabs the physical `evdev` device exclusively and
  re-emits everything that isn't a matched trigger through a virtual
  `uinput` device, matching the interception-tools/evremap pattern (a raw
  evdev grab blinds the *whole* device, so anything not meant to be
  suppressed has to be manually re-emitted). The same virtual device is
  reused to inject fallback keycodes and chord replays. Capture is split
  across a reader thread (owns the grabbed device, forwards raw events
  over a channel) and the main thread (owns all state, blocks on that same
  channel), so a chord's debounce timer — a one-shot thread that sleeps
  then sends a tagged "timeout" message — can wake the loop without evdev
  needing any timeout support of its own.
- `windows.rs` — `WH_KEYBOARD_LL` hook via the `windows` crate.
  Suppression of a matched key is done by returning `1` from the hook
  instead of calling `CallNextHookEx`. Fallback keycodes and chord replays
  go out via `SendInput` (there's no re-emit queue here the way uinput
  provides on Linux); `SendInput`-injected events are recognized via
  `LLKHF_INJECTED` and passed straight through the hook without
  re-processing, or a chord replay would re-trigger its own matching
  logic. A chord's debounce window uses a `SetTimer`/`TIMERPROC` timer on
  the hook's own thread (no second thread needed, since `run()` already
  pumps the message loop that `DispatchMessageW` uses to invoke the timer
  callback). Untested outside a real Windows machine.
- `src/focus/` — resolves the focused process name needed by
  `Router::dispatch`. Linux: shells out to `xdotool` then reads
  `/proc/<pid>/comm` (X11 only; **Wayland focused-window detection is
  unimplemented**, logs a warning once and falls through to the keycode
  fallback instead of erroring). Windows: `GetForegroundWindow` +
  `QueryFullProcessImageNameW` via the `windows` crate.

`main.rs` picks the capture backend by `cfg(target_os)` and wires its
dispatch calls to `focus::focused_process_name()` for the currently-
focused app at dispatch time.

### macOS (planned, not implemented)

No macOS code exists anywhere in the repo — not even an untested stub like
the Windows backend has. `src/capture/mod.rs`'s fallback `run()` for any
`cfg(target_os)` other than Linux/Windows already covers this by returning
an `Unsupported` io error, so nothing there needs to change to keep that
true. The planned approach, when this is picked up, is a `CGEventTap`-based
global event tap via CoreGraphics, gated behind an Accessibility permission
grant — parallel in spirit to the Windows `WH_KEYBOARD_LL` hook, but not
started.

## Known gaps / deliberately deferred (don't "fix" without discussion)

- No real OS notification — `Notifier` just logs, on both platforms.
- No macOS capture backend at all (see "macOS" above) — Linux and Windows
  only, for now.
- Neovim (msgpack-RPC) and a terminal-emulator adapter are unimplemented;
  the terminal approach specifically needs its own research pass before
  designing it.
- Stream Deck / Space Mouse support was dropped along with the old
  per-device config layer — the daemon is single-keyboard-source only for
  now. Nothing architecturally blocks adding another capture source
  later (`src/capture/` is already backend-pluggable), it's just out of
  scope until there's an actual HID implementation to add.
- Wayland focused-window detection.
- The Windows capture backend (`src/capture/windows.rs`,
  `src/focus/windows.rs`) is a careful port of the previous Python/ctypes
  code (plus the newer chord/timer logic) but has never been compiled or
  run on an actual Windows machine — this dev environment is Linux-only.
- Chord debounce window (currently a hardcoded 35ms constant in both
  capture backends) isn't configurable yet.
- The privacy/GDPR posture described in [README.md](README.md#privacy--security)
  holds only because there is zero telemetry, crash-reporting, or
  cloud-sync anywhere in the codebase — that's a standing constraint, not
  an oversight. If any such feature is ever proposed, the README's privacy
  section (and this note) must be revisited before merging, since "all
  processing stays on-device" is the entire basis for the current
  GDPR-minimal claim.
- The WebSocket adapter keeps only one connection per target (last-connect
  wins) — no support yet for multiple simultaneous Chrome
  windows/profiles.
- `extensions/windows-extension/listener.js` (the `system-windows` target's
  listener) is untested outside a real Windows machine, same caveat as
  `src/capture/windows.rs`/`src/focus/windows.rs`.
- The OS-wide system listeners (`extensions/linux-extension`,
  `extensions/windows-extension`) have to be started manually alongside the
  daemon, same as `vscode-extension`/`chrome-extension` — there's no
  process supervision or auto-launch for any of the `extensions/` targets
  yet.
