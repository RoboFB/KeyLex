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
Rust and is tested on Linux; the Windows capture backend compiles (check
it with `cargo check --target x86_64-pc-windows-msvc`, which needs no MSVC
toolchain) but cannot be *run* outside a Windows machine, and this dev
environment is Linux with no Windows box available. Application-side and
OS-side integrations each
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
cargo run -- --help        # every flag below, from src/cli.rs
cargo run -- --demo        # two hardcoded dispatches, no capture/hardware needed
cargo run -- --config-dir <path>  # load targets.toml/keymap.toml from elsewhere

cargo run -- --spotlight                    # interactive fuzzy action search/dispatch (src/spotlight/)
cargo run -- --spotlight-query "<text>"      # non-interactive: print ranked matches as JSON, no dispatch
cargo run -- --spotlight-run "<action-id>"   # non-interactive: dispatch one action id, record frecency

cargo test                # unit tests (src/config/, src/auth.rs, ...) + tests/dispatch.rs
cargo test <name>          # single test by substring

cargo check --target x86_64-pc-windows-msvc   # type-check the Windows backend
```

There is no separate lint step configured beyond `cargo clippy`. The
Windows cross-check is the one easy-to-forget check that matters: nothing
in a Linux build compiles `src/capture/windows.rs` or
`src/focus/windows.rs` at all.

## Coding guidelines

Rust code in this repo (`src/`, `tests/`) follows
[docs/rust-coding-guidelines.md](docs/rust-coding-guidelines.md) — naming,
doc-comment style, error-handling conventions (when `expect()` is
acceptable and what its message must say), where `unsafe` is allowed, and
dependency-vetting practice, each grounded in this codebase's existing
code with links to trusted external references (Rust API Guidelines, the
Rustonomicon, ANSSI's Rust guidelines, etc.). Check new or touched Rust
code against it.

**KISS is the first rule there, and it outranks the rest**: write the
boring version, solve only the problem in front of you, and let an
abstraction in only when it removes more than it adds. Less code is the
goal — a change that deletes more than it writes is usually the right one.
See that document's "Keep it simple" section for what that means in
review.

## Architecture

### Module map (`src/`)

```
cli.rs        argument parsing and the wiring each mode needs; main.rs is a 3-liner
config/       targets.toml -> Registry (key.rs, action.rs, target.rs, error.rs)
capture/      keystrokes -> action ids (linux.rs, windows.rs, shared chord.rs)
focus/        which app is focused (linux.rs, windows.rs)
dispatch.rs   Router: native adapter -> keycode fallback -> notify
adapters/     one transport per file (socket.rs, websocket.rs)
spotlight/    fuzzy action catalog (mod.rs), frecency.rs, terminal UI in ui.rs
auth.rs       the shared secret both transports require
```

Every string that means something is parsed into a type at load
(`Fallback`, `AdapterKind`, `KeyCombo`, `Chord`) and matched on
afterwards — nothing downstream re-interprets a config string.

### Config, loaded by `Registry` (`src/config/`)

There is no static action vocabulary or per-target capability list any
more — both were removed in favor of live discovery (see "Spotlight action
search" and [docs/protocol.md](docs/protocol.md#action-catalog-handshake-list_actions)).
What's left:

1. **`config/targets.toml`** — transport wiring only: per target program,
   which process names identify it (`match_process`), which adapter
   reaches it (`socket`/`websocket`/`rpc`), and connection details
   (`address`/`port`/`allowed_origin`). No `supports`/`capabilities` field
   any more — a target reports what it supports live, via the
   `list_actions` handshake, instead of Keylex holding a static copy. The
   OS-wide system listeners under `extensions/` are ordinary `[[target]]`
   entries too, identified by an `os` field instead of `match_process`
   (`"system-linux"` / `"system-windows"`); `Registry::system_target`
   picks whichever one's `os` matches `std::env::consts::OS`. Unknown keys
   are rejected at load, so a misspelled field is a startup error rather
   than a line that silently does nothing.
2. **`config/keymap.toml`** — a restored skeleton (every physical key on a
   US-en keyboard mapped to an empty "nice name") for the one thing live
   discovery can't cover: which physical key/chord should trigger an
   action, since capture has to grab a key before it knows anything about
   a target. **Not wired into `Registry::load` yet** — see "Known gaps"
   below. Until it is, `Registry`'s action/trigger/chord tables are always
   empty, and spotlight (`--spotlight`/`--spotlight-run`) is the only way
   to dispatch anything.
3. **`config/hotkeys-reference.csv`** — a per-app command catalog
   (VS Code, Chrome, GNOME, Neovim, Fusion 360 commands, one row each),
   kept purely as human reference for whoever designs `keymap.toml`'s
   eventual key→action wiring. Nothing in the daemon loads or parses it.

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

One implementation per transport, registered in `cli.rs`'s `adapters()`
map keyed by the `AdapterKind` parsed from `targets.toml`'s `adapter`
field (`socket`, `websocket`, and `rpc`, which is declared so a Neovim
target still loads but has no implementation — dispatching to it reports
"unsupported"). `adapters::SocketAdapter` (TCP, Keylex is
the client) backs the VS Code target; `adapters::WebSocketAdapter`
(Keylex runs the WebSocket *server*, since a browser extension can only be
a client) backs the Chrome target — see
[docs/protocol.md](docs/protocol.md) for the full spec of both transports.
Neovim/terminal adapters are unimplemented; each will need its own
transport.

Neither adapter authenticates messages right now — the shared-secret
`token` both used to require on every message has been deliberately
dropped for now (single-user local tool; see "Known gaps" below), so
anything able to open a loopback connection to a configured port can drive
either transport. Each accepted WebSocket connection is owned outright by
its own thread, and `send()` only queues a command onto a channel for it:
dispatch runs on the keyboard path and must never wait on socket I/O, which
is exactly what sharing the socket behind a mutex used to make it do. The
WebSocket adapter can still check the handshake's `Origin` header against a
per-target `allowed_origin` in `targets.toml` (defense-in-depth, doesn't
need a shared secret). See
[docs/protocol.md](docs/protocol.md#trust-model--authentication) for the
full wire-level contract, current exposure, and the planned keypair-based
replacement.

### Spotlight action search (`src/spotlight/`)

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
(`action_ids()` — empty today, since nothing populates it yet, see
"Config" above), then enriches (never replaces) that with whatever a
socket-adapter target reports live via the `list_actions` handshake — a
small request/response extension to `keylex/v0`
(`SocketAdapter::fetch_actions`,
[docs/protocol.md](docs/protocol.md#action-catalog-handshake-list_actions)):
the daemon asks, a target answers with the commands it just verified still
exist, and the spotlight catalog reflects that live state, not a stale
snapshot. A target that doesn't answer (not running, doesn't implement the
handshake yet) simply contributes nothing — there is no static catalog file
left to fall back to.

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
combined with the current lack of any authentication (see "Known gaps"
below), anything able to reach the socket at all can run *any* command any
installed VS Code extension contributes, not just a vetted safe set.

An entry the handshake reports has no Keylex action id of its own for the
vast majority of what it discovers (only a handful, like `close.tab`,
happen to also be one of Keylex's curated cross-app actions). Those two
kinds are dispatched differently (`spotlight::Entry::dispatch`): a real
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

`Router::dispatch(action_id, focused_process)` returns a `dispatch::Outcome`
(`Native` / `Fallback` / `Unsupported`). Since the static per-target
`supports` map is gone (see "Config" above), this currently always falls
through to **fallback**: the action's `Fallback` (empty today, since
nothing populates `Registry`'s action table yet) is either a `KeyCombo` to
send through the platform's `FallbackSender` (notifying too, on the
`notify_attempt` tier) or `Unsupported`, which only notifies. An action id
nothing declares is `Unsupported` as well.

`Router::send_native(target, native_command)` is the actual native-dispatch
primitive — it skips action-id resolution entirely and just hands
`native_command` to `target`'s adapter. This is what `spotlight::Entry::
dispatch` calls directly once it already has a live-reported command (see
"Spotlight action search" below); `Router::dispatch`'s own native path
(matching a focused/system target's `supports` against `action_id`) will
need an equivalent live `list_actions` query once `config/keymap.toml` is
wired up and direct key-triggered dispatch needs the same live capability
check spotlight already does.

`Notifier` is a log-only placeholder on both platforms — real OS
notifications are not implemented yet (a known, deferred gap). Fallback
keycode injection *is* real on both platforms: Linux writes the keycode
through the same `uinput` virtual device used for passthrough re-emission
(`src/capture/linux.rs`), Windows uses `SendInput`
(`src/capture/windows.rs`).

### Capture backends (`src/capture/`)

- `chord.rs` — the chord state machine both backends share: it decides
  what a chord-member key means (keep pending, consume and dispatch,
  replay), and each backend supplies only the effects it performs
  differently, through the small `chord::Keyboard` trait (press, release,
  arm the debounce timer). The decisions live in one place; only the
  emitting differs.
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
  callback). It type-checks under `cargo check --target
  x86_64-pc-windows-msvc`, but has never been run on a real Windows
  machine.
- `src/focus/` — resolves the focused process name needed by
  `Router::dispatch`. Linux: shells out to `xdotool` then reads
  `/proc/<pid>/comm` (X11 only; **Wayland focused-window detection is
  unimplemented**, logs a warning once and falls through to the keycode
  fallback instead of erroring). Windows: `GetForegroundWindow` +
  `QueryFullProcessImageNameW` via the `windows` crate.

`capture::run` is selected by `cfg(target_os)`, and each backend asks
`focus::focused_process_name()` for the focused app at dispatch time.
`None` from that (no backend, Wayland, an unreadable window) is a normal
answer, not an error: dispatch just falls through to the keycode path.

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

- **No action can be bound to a physical key yet.** `vocabulary.toml` and
  `actions.toml` are gone (superseded by live discovery, see "Config"
  above); `config/keymap.toml` is a restored skeleton with no loader code
  wired to it. `Registry`'s action/trigger/chord tables are always empty
  today, so `Router::dispatch`'s native/fallback path never actually runs
  from a real key press — spotlight (`--spotlight`/`--spotlight-run`,
  dispatching via `Router::send_native` with a live-reported command) is
  the only working entry point right now. Designing `keymap.toml`'s
  key→action wiring (and how `fallback_tier`/`fallback_keycode` come back)
  is deferred, deliberately — not an oversight.
- **Only the VS Code extension implements the `list_actions` live-discovery
  handshake.** Chrome, Neovim, and the Linux/Windows system listeners have
  no equivalent handler, and there is no static `capabilities.toml`
  fallback left for them either (that mechanism was removed entirely) — so
  none of them currently has any native-dispatch path at all. Extending the
  VS Code pattern (`liveActionCatalog`/`list_actions` in
  `extensions/vscode-extension/extension.js`) to the others is the natural
  next step, not started yet.
- **No authentication on either IPC transport right now.** The
  shared-secret `token` `SocketAdapter`/`WebSocketAdapter` used to require
  on every message (`docs/protocol.md#trust-model--authentication`) has
  been deliberately dropped, for now, in favor of shipping the core
  dispatch pipeline first — this is a single-user local tool today, not a
  multi-user or hardened deployment. Both transports are still
  loopback-only, and the WebSocket adapter's `allowed_origin` check still
  works without a token, but any other local process (or, for the
  WebSocket/Chrome transport, any webpage's JS) can currently connect and
  drive commands. The planned replacement is a keypair scheme — a public
  key per paired target held by the daemon, a private key held by the
  target/extension — rather than reintroducing a bearer secret; not started
  yet. Revisit before running Keylex on a shared machine or trusting it
  with anything more sensitive.
- No real OS notification — `Notifier` just logs, on both platforms.
- `vscode-extension/extension.js` has no per-command allowlist any more
  (see "Spotlight action search" above and
  [docs/protocol.md](docs/protocol.md#trust-model--authentication)'s "No
  per-command allowlist, by design") — this is a deliberate trade for
  complete, zero-maintenance command discovery, but combined with the lack
  of authentication above, it means anything that can reach the socket at
  all gates *every* command any installed VS Code extension contributes,
  not a vetted subset. Not a bug, but revisit this alongside the
  authentication gap above.
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
  code (plus the newer chord/timer logic). It type-checks (`cargo check
  --target x86_64-pc-windows-msvc` — run it after touching either file, a
  Linux build never compiles them), but has never been *run* on an actual
  Windows machine, so its behavior is still unverified.
- Chord debounce window (currently a hardcoded 35ms constant in both
  capture backends) isn't configurable yet.
- `rpc` is a declared `AdapterKind` with no implementation, so the
  `neovim` target in `targets.toml` loads but reports "unsupported" when
  dispatched to. That is deliberate: it keeps the config honest about the
  planned transport without pretending it exists.
- The keycode vocabulary the capture backends understand is still a-z,
  0-9, `prtsc` and the four modifier names. A `key`/`chord` naming
  anything else parses fine but never matches a real keystroke; full
  layout support is a later step.
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
