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
focus-independent actions like `shutdown` and `move.left`
via `wmctrl`/PowerShell respectively, see "Dispatch flow" below) all
exist; Neovim/terminal adapters are unimplemented stubs. macOS has no
capture backend at all yet, not even an untested one (see "Known gaps"
below) — Linux and Windows are the only supported platforms for now.

## Commands

```bash
cargo build              # compiles the daemon (Linux backend on this machine)
cargo run                 # real capture loop, blocks (needs evdev/uinput perms)
cargo run -- --demo        # two hardcoded dispatches, no capture/hardware needed
cargo run -- --config-dir <path>  # load actions.toml/targets.toml/vocabulary.toml from elsewhere

cargo run -- --spotlight                    # interactive fuzzy action search/dispatch (src/spotlight.rs)
cargo run -- --spotlight-query "<text>"      # non-interactive: print ranked matches as JSON, no dispatch
cargo run -- --spotlight-run "<action-id>"   # non-interactive: dispatch one action id, record frecency

cargo test                # unit tests (src/config.rs) + integration tests (tests/dispatch.rs)
cargo test <name>          # single test by substring
```

There is no separate lint step configured beyond `cargo clippy`.

## Architecture

### Three config layers (`config/*.toml` + per-extension `capabilities.toml`), loaded by `Registry` (`src/config.rs`)

1. **`vocabulary.toml`** — the one authoritative word list: a `modifiers`
   array (verbs, e.g. `close`, `save`, `move`) and a `locations` array
   (objects, e.g. `tab`, `sidebar`, `left`). Every action id in
   `actions.toml` is built only from these words; `Registry::load` refuses
   to start if it isn't.
2. **`actions.toml`** — the action vocabulary itself: one `[[action]]`
   entry per action, declaring a `modifier` and an optional `location`
   (each checked against `vocabulary.toml`) instead of a hand-typed id —
   the id itself is derived as `modifier` alone (e.g. `save`) or
   `modifier.location` (e.g. `close.tab`), a real enforced grammar, not
   just a naming convention. Also an optional `key` field (a `"ctrl+w"`-
   style combo — the same syntax used for `fallback_keycode`) that binds
   it to a physical key, a `fallback_tier` (`silent` / `notify_attempt` /
   `notify_only`), and an optional `fallback_keycode`. An action with no
   `key` simply isn't reachable from the keyboard yet (e.g.
   `go_to.definition` today) but can still be dispatched once something
   else triggers it. There is no separate device-binding layer.

   An action may instead (never both) bind a `chord`: an array of two or
   more key tokens (same vocabulary as `key`, including modifiers) that
   must all be held down together, order-independent — e.g.
   `chord = ["ctrl", "d", "f"]`. See
   [docs/protocol.md](docs/protocol.md#chorded-triggers) for the full
   syntax, validation rules, and the debounce/replay behavior this adds to
   capture.
3. **`targets.toml`** — output side, but transport wiring only: per target
   program, which process names identify it (`match_process`), which
   adapter reaches it, and a `capabilities` path pointing at that target's
   own `extensions/<name>/capabilities.toml`, which owns the actual
   `supports` whitelist mapping action ids to that program's native
   command strings — each extension declares what it understands and
   accepts, instead of that living as a second copy of the allowlist it
   already hardcodes on the receiving end (see
   [docs/protocol.md](docs/protocol.md#native-command-strings)). `neovim`
   is the one exception, still declaring `[target.supports]` inline since
   it has no `extensions/` folder yet (unimplemented stub). Also holds
   `[[system_action]]` entries — OS-level actions that don't depend on the
   focused app at all — and, as an ordinary `[[target]]` with an `os`
   field instead of `match_process` (`"system-linux"` / `"system-
   windows"`), the OS-wide system listeners under `extensions/`.
   `Registry::system_target` picks whichever one's `os` matches
   `std::env::consts::OS`, so only one is ever live per platform even
   though both entries are declared in the same file.

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

### Spotlight action search (`src/spotlight.rs`)

A fuzzy-searchable catalog of actions, cross-platform by construction: the
ranking engine (`nucleo-matcher`) and the interactive terminal launcher
(`crossterm`) are both pure computation/terminal-I/O crates with no
OS-specific code of their own, so `keylex --spotlight` behaves identically
on Linux, macOS, and Windows terminals (macOS still has no focused-process
resolution — see "macOS" below — so a dispatched action there always goes
through the keycode fallback, same as an unknown focused process on any
platform).

`spotlight::Index` never gets its "valid options" from a hand-maintained
file. It starts from every action id `Registry` already knows
(`action_ids()`), then enriches (never replaces) that with whatever a
socket-adapter target reports live via the `list_actions` handshake — a
small request/response extension to `keylex/v0`
(`SocketAdapter::fetch_actions`,
[docs/protocol.md](docs/protocol.md#action-catalog-handshake-list_actions)):
the daemon asks, a target answers with the commands it just verified still
exist, and the spotlight catalog reflects that live state, not a stale
snapshot. A target that doesn't answer (not running, doesn't implement the
handshake yet) just means its entries keep whatever
`actions.toml`/`targets.toml` already said.

`vscode-extension/extension.js` answers with a genuinely complete catalog,
not a hand-picked subset: `vscode.extensions.all` exposes every installed
extension's own parsed `package.json`, and `contributes.commands` there is
the exact same data VS Code's own Command Palette is built from, so a
newly installed extension's commands show up in spotlight search
automatically. Each candidate is still cross-checked live against
`vscode.commands.getCommands()` before being reported, but there is
deliberately **no allowlist** narrowing that down further any more — see
[docs/protocol.md](docs/protocol.md#trust-model--authentication)'s "No
per-command allowlist, by design" for the security trade-off that makes:
the shared token is the only remaining gate, and it now grants the ability
to run *any* command any installed VS Code extension contributes, not just
a vetted safe set.

An entry the handshake reports has no Keylex action id of its own for the
vast majority of what it discovers (only a handful, like `close.tab`,
happen to also be one of Keylex's curated cross-app actions). Those two
kinds are dispatched differently (`spotlight::dispatch_entry`): a real
Keylex action id goes through the normal focus-aware `Router::dispatch`
(native adapter for whatever's focused, keycode fallback otherwise) exactly
like a real key binding would; anything else is a raw native command with
no cross-app abstraction, namespaced internally as `"<target-
program>:<native-command>"` (`spotlight::Index::merge_remote`) and sent
straight to the target that reported it, regardless of what's currently
focused, since there is no abstract action to route by focus in the first
place.

Optional zoxide-style "last used" tracking (`spotlight::Frecency`) persists
a small per-action-id count/recency score to
`<config-dir>/spotlight_frecency.json` (git-ignored, runtime state) and adds
a bounded ranking bonus on top of the fuzzy score — never enough to let a
poor match outrank a strong one, only to break ties toward what's actually
used. It's only recorded on the Rust-driven dispatch paths (the interactive
launcher, `--spotlight-run`) — picking an action from the VS Code
extension's own `keylex.spotlight` QuickPick doesn't round-trip back to the
daemon to record a hit, since that would require the daemon to run a
listening server for the vscode target instead of being its TCP client (see
"Adapters" above); a known, deliberate scope cut, not an oversight.

Three consumers currently exist, all reusing this one engine rather than
reimplementing matching:
- `cargo run -- --spotlight` — the interactive terminal launcher.
- `extensions/vscode-extension/extension.js`'s `keylex.spotlight` command —
  a native VS Code `QuickPick` (which already does its own fuzzy filtering)
  populated from the *same* live, complete command catalog the
  `list_actions` handshake reports, so both consumers agree on what's
  "valid" for VS Code by construction, not by convention.
- `extensions/linux-extension/search-provider.js` — a best-effort, untested
  (no GNOME Shell in this dev environment) GNOME Shell search provider that
  shells out to `keylex --spotlight-query`/`--spotlight-run` so
  Activities search reuses the same Rust ranking instead of a JS
  reimplementation. See that folder's README.md for registration/setup and
  the exact untested caveat.

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
   of what's focused (used for actions like `shutdown` and
   `move.left`/`move.right` that don't belong to any one
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
- `vscode-extension/extension.js` has no per-command allowlist any more
  (see "Spotlight action search" above and
  [docs/protocol.md](docs/protocol.md#trust-model--authentication)'s "No
  per-command allowlist, by design") — this is a deliberate trade for
  complete, zero-maintenance command discovery, but it means
  `config/secret.token` alone now gates *every* command any installed VS
  Code extension contributes, not a vetted subset. Not a bug, but revisit
  this if Keylex's threat model ever needs to change.
- `extensions/linux-extension/search-provider.js` (the GNOME Shell search
  provider for spotlight, see "Spotlight action search" above) is untested
  outside a real GNOME Shell session, same caveat as the Windows capture
  backend — this dev environment has no session D-Bus bus or GNOME Shell to
  register against.
- Spotlight frecency (`spotlight::Frecency`) is only recorded on the
  Rust-driven dispatch paths (`--spotlight`, `--spotlight-run`) — an action
  picked from VS Code's own `keylex.spotlight` QuickPick doesn't report
  back to the daemon, since that would need the daemon to run a listening
  server for the vscode target rather than being its client. Deliberate
  scope cut for now, not a bug.
- The spotlight terminal launcher's chosen fuzzy library/UI crates
  (`nucleo-matcher`, `crossterm`) are pure computation/terminal-I/O with no
  OS bindings, so they compile and *should* behave identically on macOS —
  but macOS still has no focused-process resolution (see "macOS" below), so
  a dispatched action there always falls through to the keycode fallback,
  and this hasn't been run on an actual Mac either way.
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
