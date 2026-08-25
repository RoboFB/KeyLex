# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What Keylex is

Keylex turns physical input (keyboard shortcuts, Stream Deck buttons, a
Space Mouse, …) into abstract **verb+object actions** (`close.tab`,
`go_to.definition`, `save`) and dispatches each action through the
**native API of whatever application is currently focused**, instead of
just simulating a keycode. A keycode is only sent as a fallback, when the
focused app has no native adapter or doesn't support that action. See
[README.md](README.md) for the German-language project pitch and the
architecture diagram.

The long-term ambition (not yet built) is for this action vocabulary to
become a shared, documented protocol that device firmware and application
plugins can target directly — see [docs/protocol.md](docs/protocol.md) for
the current (draft, unstable) wire format and its LSP-inspired rationale.
This is *not* about literal Linux kernel source — no kernel code is
planned; `evdev`/`uinput` on Linux and the low-level keyboard hook on
Windows already provide everything needed for input capture.

Everything here is early-stage. The core dispatch pipeline
(config → registry → router → adapter/fallback) works and is tested; the
Windows/Linux input listeners exist but are effectively untested outside
this Windows dev environment (no evdev available here), and most target
adapters beyond VS Code are unimplemented stubs.

## Commands

```bash
python -m venv .venv
source .venv/bin/activate        # Windows: .venv\Scripts\activate
pip install -e ".[dev]"          # + ".[linux,dev]" on Linux, for the evdev listener

python -m keylex.daemon          # real platform input listener, blocks
python -m keylex.daemon --demo   # two hardcoded dispatches, no listener/hardware needed

pytest                           # tests/test_registry.py, tests/test_router.py
pytest tests/test_router.py -k native   # single test
```

There is no separate build or lint step configured yet.

## Architecture

### Three config layers (`src/keylex/config/*.toml`), loaded by `Registry`

1. **`devices.toml`** — input side. Which physical devices are listened to
   and how their raw signals map to internal events. Keyboard bindings are
   `key + modifiers → event`; button devices (Stream Deck, Space Mouse) map
   `button → event`. Each binding resolves to a `mode`:
   `"grab"` (consume the key — OS/app never sees it unless re-emitted) or
   `"observe"` (additive, normal OS behavior untouched). Resolution order:
   binding-level `mode` → device-level `default_mode` → global default
   `"observe"` (`registry.DEFAULT_BINDING_MODE`).
2. **`actions.toml`** — the verb+object vocabulary itself (`[[verb]]`
   entries with a `name` and allowed `objects`) plus per-action fallback
   overrides (`[[action]]`: `fallback_tier` and optional
   `fallback_keycode`). `Registry._validate_action_grammar` enforces this
   at load time: every action ID referenced anywhere (actions.toml,
   targets.toml `supports`, devices.toml `event`) must either be a
   declared bare word (like `save`, no dot), a declared `system_action`
   (`system.*`), or resolve to a declared `verb.object` pair — anything
   else raises `GrammarError` and the daemon refuses to start. This is the
   concrete enforcement of "verb + word logic."
3. **`targets.toml`** — output side. Per target program: which process
   names identify it (`match_process`), which adapter reaches it, and a
   `supports` whitelist mapping action IDs to that program's native
   command strings. Also holds `[[system_action]]` entries — OS-level
   actions that don't depend on the focused app at all.

### Dispatch flow (`src/keylex/core/router.py`)

`Router.dispatch(action_id, focused_process)`:
1. `Registry.target_for_process` — does a target's `match_process` include
   the focused process? If yes and it `supports` this action → **native**:
   look up the adapter by `target.adapter` and call `adapter.send(target,
   native_command)`.
2. Otherwise → **fallback**: based on `ActionSpec.fallback_tier`
   (`silent` / `notify_attempt` / `notify_only`), either send
   `fallback_keycode` via `FallbackSender` (optionally also notifying), or
   — if there's no usable keycode / tier is `notify_only` — report
   **unsupported** and just notify.

`Notifier`/`FallbackSender` (`src/keylex/core/system.py`) are currently
logging-only placeholders on both platforms — real keycode injection and
OS notifications are not implemented yet (tracked as deferred work, not
forgotten).

### Adapters (`src/keylex/adapters/`)

One class per target program, registered in `daemon.build_router`'s
`adapters` dict keyed by the `adapter` string used in `targets.toml`
(`"socket"`, later `"native_messaging"`, `"rpc"`, …). Only
`vscode.SocketAdapter` exists so far: newline-delimited JSON
(`{"command": ...}`) over a local TCP socket — see
[docs/protocol.md](docs/protocol.md) for the full spec of this wire
format. Chrome/Neovim/terminal adapters are unimplemented; each will need
its own transport, not necessarily this socket protocol.

### Input listeners (`src/keylex/input/`)

- `events.py` — the platform-independent `InputEvent` (device_id,
  action_id, phase, key/button, modifiers). Listeners only call
  `on_event` once `Registry.binding_for` has already matched a binding —
  `action_id` comes from the listener, callers don't re-resolve it.
- `base.py` — `InputListener` ABC (`start()` blocks, `stop()`).
- `windows.py` — `WH_KEYBOARD_LL` hook via raw `ctypes.windll` (no
  `pywin32` dependency). Suppression of a `"grab"` binding is done by
  returning `1` from the hook instead of calling `CallNextHookEx`.
- `linux.py` — grabs the physical `evdev` device exclusively and
  re-emits everything that isn't a `"grab"` binding through a virtual
  `uinput` device, matching the interception-tools/evremap pattern (a raw
  evdev grab blinds the *whole* device, so anything not meant to be
  suppressed has to be manually re-emitted). Needs the `linux` extra
  (`python-evdev`) — the one new runtime dependency introduced so far.
- `active_window.py` — resolves the focused process name needed by
  `Router.dispatch`. Windows: `ctypes` (`GetForegroundWindow` +
  `QueryFullProcessImageNameW`). Linux: shells out to `xdotool`/`ps`
  (X11 only; **Wayland focused-window detection is unimplemented**, logs a
  warning once and falls through to the keycode fallback instead of
  erroring).

`daemon.py` picks the listener by `sys.platform` in `start_input_listener`
and wires its `on_event` to `Router.dispatch`, using `focused_process_name()`
for the currently-focused app at dispatch time.

## Known gaps / deliberately deferred (don't "fix" without discussion)

- No real Windows/Linux fallback keycode sender or OS notification —
  `FallbackSender`/`Notifier` just log.
- Chrome (Native Messaging), Neovim (msgpack-RPC), terminal-emulator
  adapters are unimplemented; terminal approach specifically needs its own
  research pass before designing it.
- Stream Deck / Space Mouse are configured as input sources in
  `devices.toml` but have no actual HID-reading implementation.
- Wayland focused-window detection.
- The Rust rewrite of the core is an intentional *later* milestone (after
  the config schema and protocol stabilize) — the prototype is Python on
  purpose for now.
