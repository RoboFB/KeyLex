# Prior art — general-purpose keyboard/input libraries

Status: **research notes**, companion to [prior-art.md](prior-art.md). That
file surveys whole *products* (remapping daemons, Stream Deck, gaming
tools); this one surveys the **libraries** application developers reach
for to listen to or simulate keyboard/mouse input from their own language
— npm/PyPI/crates.io/NuGet packages, not standalone daemons. Keylex
doesn't depend on any of these (its capture code is hand-rolled per
platform in [`src/capture/`](../src/capture/)), but understanding what
they can and can't do is useful for two reasons: it's the most direct
evidence available for *why* Keylex hand-rolls capture instead of taking a
dependency, and it's where a future adapter/extension author (a Neovim
plugin, a terminal adapter — both currently unimplemented per
[CLAUDE.md](../CLAUDE.md)) will most likely start looking.

## The pattern: one C core, thin bindings per language

The single most important fact this survey turned up: most of these
libraries are not independent implementations. **[libuiohook](https://github.com/kwhat/libuiohook)**
(by kwhat) is a small C library providing global keyboard/mouse hooks
from userland, and it is the shared native core behind:

- **[JNativeHook](https://github.com/kwhat/jnativehook)** (Java) — the
  oldest of the three, also by kwhat, predates libuiohook being split out
  as its own reusable project.
- **[SharpHook](https://github.com/TolikPylypchuk/SharpHook)** (.NET) —
  exposes libuiohook's functions almost 1:1 (`SharpHook.Native.UioHook`)
  plus a higher-level `IGlobalHook` interface on top.
- **[uiohook-napi](https://github.com/SnosMe/uiohook-napi)** (Node.js) —
  an N-API wrapper, the actively-maintained successor to the older,
  now largely abandoned **[iohook](https://github.com/wilix-team/iohook)**
  (last released ~5 years ago per its npm listing).

libuiohook's own architecture is platform-asymmetric in a way worth
noting: Windows gets **two separate hooks** (keyboard, mouse), while
macOS and Linux get **one combined hook** with keyboard/mouse events
filtered out of it after the fact. On Linux, that hook is built on **X11's
`XRecord` extension** — not evdev. That one architectural choice is the
root cause of the single biggest limitation shared by every library in
this document (see "The universal Wayland gap" below): none of them can
fall back to evdev the way Keylex, keyd, or xremap do, because none of
them were designed as an exclusive-grab system daemon — they're designed
to be linked into one ordinary desktop application that wants a global
shortcut.

## By language

### Python

- **[pynput](https://github.com/moses-palmer/pynput)** — the closest
  Python equivalent to Keylex's own `cfg(target_os)` split in
  [`src/capture/mod.rs`](../src/capture/mod.rs): a platform-agnostic
  `keyboard`/`mouse` API on top of three separate backends (Xlib on
  Linux, Win32 on Windows, Quartz on macOS), selectable at runtime via
  `$PYNPUT_BACKEND`. Its own docs are explicit that ["pynput's only
  support for Linux is via X11, which obviously won't work in Wayland
  environments"](https://pynput.readthedocs.io/en/latest/limitations.html) —
  stated as a permanent architectural limitation, not a to-do.
- **[keyboard](https://github.com/boppreh/keyboard)** (boppreh) — takes
  the opposite approach on Linux: it reads raw evdev directly (the same
  layer Keylex captures from), which is *why* its own `ensure_root()`
  check unconditionally demands root or `/dev/input/event*` access (the
  `input` group works in practice, per its issue tracker, though the
  library doesn't document that path). This is architecturally the
  closest of any library surveyed here to what Keylex does on Linux — and
  its own users hit the exact permission friction Keylex's README/setup
  presumably has to document too.

### Node.js / Electron

- **uiohook-napi** (above) is the current default choice for Electron
  apps wanting global shortcuts; its own issue tracker documents a
  standing, unresolved native-crash/silent-exit failure mode that the
  maintainers acknowledge has "no better alternatives in the market" —
  a useful data point on how much harder *reliability* is to get right
  in this space than the basic hook mechanism itself.
- **[robotjs](https://github.com/octalmage/robotjs)** — mouse/keyboard/
  screen automation (send input, not primarily listen), same X11-only
  ceiling on Linux as everything else here.

### Go

- **[robotgo](https://github.com/go-vgo/robotgo)** — for Linux global
  event hooks it uses `xcb`, `xkb`, and `libxkbcommon` rather than evdev
  or libuiohook; injection goes through XTest. A third independent
  implementation of the same X11-only pattern, which is itself the
  finding: three unrelated ecosystems (C/libuiohook, Python/pynput,
  Go/robotgo) converged on the identical X11-API ceiling rather than any
  of them reaching for evdev+uinput the way Keylex and the Linux-native
  remappers in [prior-art.md](prior-art.md) do.

### .NET

- **SharpHook** (above, wraps libuiohook, genuinely cross-platform) vs.
  the older **Gma.System.MouseKeyHook** — a pure `WH_KEYBOARD_LL`/
  `WH_MOUSE_LL` wrapper, Windows-only, architecturally identical to what
  Keylex's own [`src/capture/windows.rs`](../src/capture/windows.rs) does
  by hand. SharpHook's docs carry a warning directly relevant to Keylex's
  own single-hook design: *"you must use one instance of `IGlobalHook` at
  a time in the entire application... running a global hook when another
  is already running will corrupt libuiohook's internal global state"* —
  i.e. even a library built specifically to make this safe still can't
  make two hooks *within the same process* coexist, let alone across
  processes (see the hook-chaining latency risk already noted in
  [prior-art.md](prior-art.md#windows-user-mode-hooks-vs-a-kernel-filter-driver)
  for the cross-process case).

### Rust

- **[rdev](https://github.com/Narsil/rdev)** — already catalogued in
  [prior-art.md](prior-art.md#libraries-worth-knowing-about-not-adopted-just-catalogued);
  included here again because it's the one Rust crate in this whole
  survey that *does* use evdev+uinput on Linux (via the `evdev` crate)
  rather than X11 APIs, making it Keylex's nearest existing-crate
  relative on Linux specifically.
- **[global-hotkey](https://github.com/tauri-apps/global-hotkey)** (Tauri)
  and the older **tauri-hotkey** — solve a *narrower* problem than Keylex
  or any tool above: registering one specific combo as a system-wide
  shortcut (Windows `RegisterHotKey`-style, X11-only on Linux, no
  Wayland, no evdev). This class of API only ever receives combos nobody
  else has already claimed, and can't suppress/re-emit selectively the
  way an exclusive evdev grab or a `WH_KEYBOARD_LL` hook can — it's a
  fundamentally weaker primitive than what Keylex's capture layer needs,
  useful context for why "just use a hotkey-registration crate" isn't a
  viable simplification of `src/capture/`.
- **[keyberon](https://github.com/TeXitoi/keyberon)** — also already
  covered in prior-art.md; notable here only as the one library in this
  whole survey whose logic (layers, tap-hold) originated in *firmware*
  and was later generalized for desktop use (via kanata), rather than the
  other direction.

## The universal Wayland gap, confirmed from a second angle

[prior-art.md](prior-art.md#wayland-why-this-is-structurally-harder-not-just-unimplemented)
already establishes that no *remapping daemon* has a compositor-agnostic
Wayland answer. This survey adds a second, independent confirmation: no
general-purpose *automation/hotkey library* has one either, for a related
but distinct reason. Where compositors block it on security grounds
(unprivileged clients can't observe input meant for other windows),
these libraries are blocked on a *different* ground: they were built on
X11-specific extensions (`XRecord`, `XTest`, `xcb`/`xkb`) that simply have
no Wayland equivalent to bind to at all — there's no compositor
permission a Wayland session could grant that would make an `XRecord`
call work, because the extension itself doesn't exist there. The one
library here architecturally capable of working under Wayland today is
the one that skips X11 entirely and reads evdev directly (boppreh's
`keyboard`, and Rust's `rdev`/`evdev` crates) — exactly Keylex's own
choice, and further confirmation that evdev-or-nothing is the only route
to Wayland support in this whole space, not a Keylex-specific limitation
to be engineered around later.

## What this means for Keylex

- **Not taking a dependency on any of these was the right call for the
  core daemon.** Every library surveyed here solves "let one ordinary
  application listen to or inject a few global events," which is a
  materially weaker contract than what `src/capture/linux.rs` and
  `windows.rs` need: **exclusive** device ownership with **selective
  re-emission** of everything not matched. Only a library built around an
  exclusive evdev grab (which none of the general-purpose ones are) could
  do what Keylex's capture loop does; wrapping one of these instead would
  mean losing the "everything else passes through unchanged" guarantee
  that the whole capture-rule design in [CLAUDE.md](../CLAUDE.md#capture-rule)
  depends on.
- **The single-C-core-plus-thin-bindings shape (libuiohook) is a proven
  pattern worth remembering** if Keylex's own capture logic is ever
  extracted for reuse by, say, a future Neovim or terminal adapter that
  isn't Rust — the same shape (a small hand-tuned native core, thin
  per-language wrappers) that libuiohook/JNativeHook/SharpHook/
  uiohook-napi already validate at scale is a reasonable template to copy
  rather than reinvent.
- **SharpHook's single-hook-per-process warning is a real, generalizable
  constraint** — not just a Windows quirk — worth keeping in mind for
  Keylex's own architecture: nothing in this space tolerates two
  competing hook/grab layers gracefully, which argues against ever
  running Keylex's capture loop alongside another remapper (AutoHotkey,
  PowerToys, xremap, keyd) on the same keyboard device without a
  documented "don't do this" caveat.

## Sources

- [libuiohook](https://github.com/kwhat/libuiohook)
- [JNativeHook](https://github.com/kwhat/jnativehook)
- [SharpHook](https://github.com/TolikPylypchuk/SharpHook) /
  [SharpHook docs](https://sharphook.tolik.io/)
- [uiohook-napi](https://github.com/SnosMe/uiohook-napi) /
  [iohook](https://github.com/wilix-team/iohook)
- [pynput](https://github.com/moses-palmer/pynput) /
  [pynput platform limitations](https://pynput.readthedocs.io/en/latest/limitations.html)
- [keyboard (boppreh)](https://github.com/boppreh/keyboard)
- [robotjs](https://github.com/octalmage/robotjs)
- [robotgo](https://github.com/go-vgo/robotgo)
- [rdev](https://github.com/Narsil/rdev)
- [global-hotkey](https://github.com/tauri-apps/global-hotkey)
- [keyberon](https://github.com/TeXitoi/keyberon)
