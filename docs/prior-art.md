# Prior art — how everyone else remaps keys

Status: **research notes**, not a spec. This document exists so design
decisions in Keylex (the action vocabulary, the capture/dispatch split, the
fallback tiers) can be checked against what other remapping tools already
do — and, as importantly, where they've hit walls Keylex will hit too. See
[protocol.md](protocol.md) for Keylex's own wire format; this file is the
external survey that motivated some of its choices.

Every tool below is placed by **which layer it sits at**, because that's
the axis that determines both what it can do and what it costs in latency:
firmware → kernel/evdev → display-server/compositor → OS-level hook →
application-level API. Keylex spans three of these itself (evdev/uinput
capture, `WH_KEYBOARD_LL` on Windows, native per-app APIs as the preferred
dispatch target) — see [../CLAUDE.md](../CLAUDE.md) — so most of the tools
below are closer to Keylex's capture side than its dispatch side. Nothing
here does what Keylex's `targets.toml`/`capabilities.toml` split does:
resolve an action to a *native command in the focused application*, with a
keycode as fallback only. That combination looks to be genuinely rare;
where it does exist (Steam Input) it's discussed below because the parallel
is instructive even though the domain (game controllers) differs.

## Linux: kernel/evdev-layer remappers

These all sit below the display server, at the `evdev`/`uinput` layer
Keylex's own `src/capture/linux.rs` uses — which is exactly why they work
identically on X11, Wayland, and a bare VT, and why Keylex inherits the
same tradeoffs.

- **[keyd](https://github.com/rvaiya/keyd)** — a small C daemon, "a hand
  tuned input loop written in C that takes `<<1ms`" per its own README.
  INI config, layers with hybrid modifiers, oneshot modifiers. Its own
  pitch against `xmodmap`/`setxkbmap` is the same one Keylex implicitly
  makes against a bare `xdotool`/keycode-remap script: those are
  "display-server level tools with limited functionality," a system-level
  daemon supports layering, held-key logic, and consistent behavior in a
  VT. keyd has no concept of dispatching to an app's *native* API at
  all — everything it produces is still a keycode, re-emitted through
  `uinput`. That's the ceiling every evdev-layer tool on this list shares
  with Keylex's own fallback path (never with Keylex's native path).
- **[xremap](https://github.com/xremap/xremap)** ([architecture
  writeup](https://www.paolomainardi.com/posts/linux-remapping-keys-with-xremap/)) —
  Rust, evdev/uinput, YAML config. Notable for **application-scoped
  remaps** (`application.only`/`application.not`, regex over process name
  or window title) with per-compositor detection shims for GNOME, KDE,
  every wlroots compositor (Sway, Wayfire, River), Hyprland, niri, COSMIC,
  and Pantheon. This is the same problem Keylex's `focus::` module solves
  (`xdotool` + `/proc/<pid>/comm` on X11, no Wayland answer yet) —
  xremap's compositor-by-compositor detection list is a good proxy for
  how much per-compositor work Wayland focus-tracking actually costs once
  Keylex attempts it; there is no compositor-agnostic answer, only a
  matrix of protocol-specific ones (see the Wayland section below).
- **[evremap](https://github.com/wez/evremap)** — the smallest of the
  bunch, purpose-built for one thing: dual-role keys (tap CapsLock for
  Esc, hold it for Ctrl). Grabs one device exclusively, models pressed
  keys, remaps, re-emits via `uinput` — architecturally a minimal version
  of what Keylex's chord/debounce logic in `capture/linux.rs` already
  generalizes (evremap's dual-role key is a 2-outcome chord: tap vs.
  hold, where Keylex's `chord` construct is N-key, order-independent
  press-together).
- **[input-remapper](https://github.com/sezanzeb/input-remapper)**
  (formerly key-mapper) — the most-used Linux *GUI* for this, GTK
  front-end over the same evdev-capture/uinput-inject model, plus macros
  and per-device presets. Confirms there's real end-user demand for a GUI
  on top of what's otherwise all text-config daemons — a gap Keylex's own
  TOML-only config shares today.
- **[evsieve](https://github.com/KarsMulder/evsieve)** and
  **[kanata](https://github.com/jtroo/kanata)** — kanata is notable for
  being the one genuinely **cross-platform** entry (Linux via evdev/uinput,
  Windows via a low-level hook or the Interception driver, macOS via the
  Karabiner virtual-HID driver — see below), built on the `keyberon` Rust
  crate originally written for QMK-style firmware layers. It's the closest
  thing on this list to Keylex's own three-platform ambition, but it stops
  at the same ceiling as keyd: output is always a synthesized keycode,
  never a native app command.
- **[xwaykeyz](https://github.com/RedBearAK/xwaykeyz)** / the **Toshy**
  project — per-application keymapping specifically aimed at Wayland,
  built on top of the same evdev primitives. Its existence is itself
  evidence that "per-app remap on Wayland" is popular enough to be worth a
  dedicated project, not that the underlying problem (see below) is
  actually solved — it still depends on compositor-specific focus
  detection, same as xremap.

### The library split: evdev vs. libinput vs. uinput

Worth being precise about, since Keylex's own code touches this boundary
directly: **evdev** is the kernel's raw character-device interface
(`/dev/input/eventN`); **libinput** is a *higher-level* userspace library
(used by Wayland compositors and modern Xorg) that wraps evdev with
gesture recognition, pointer acceleration, palm rejection, etc. — Keylex
and every tool above talks to raw evdev directly and never touches
libinput, because libinput is a consumer of the device, not a place to
inject synthetic events back in. **uinput** is the separate kernel
mechanism for the other direction — creating a *virtual* device that
looks like real hardware to everything downstream, including libinput
itself. This is why grab-and-re-emit tools (Keylex, keyd, xremap, evremap)
are invisible to libinput-based compositors: from the compositor's
perspective a `uinput` virtual keyboard is indistinguishable from a
physical one. ([libinput's architecture
docs](https://wayland.freedesktop.org/libinput/doc/latest/architecture.html),
[Peter Hutterer's evdev-vs-uinput
writeup](http://who-t.blogspot.com/2016/05/the-difference-between-uinput-and-evdev.html).)

## Wayland: why this is structurally harder, not just unimplemented

Keylex's own gap list already says Wayland focus detection is
unimplemented; the research turned up *why* it's not just a missing
`xdotool`-equivalent:

- Wayland's security model deliberately gives only the **focused
  window's own client** access to its keyboard events — this is a
  design goal (anti-keylogging), not an oversight, so there is no
  compositor-agnostic "what process is focused" query the way
  `xdotool getactivewindow` provides on X11.
- Neither **Mutter** (GNOME) nor **KWin** (KDE) — the two dominant
  non-wlroots compositors — implements anything like `xmodmap`, and
  neither exposes application-scoped remapping natively; every tool above
  that offers per-app remapping on Wayland does its own focus-tracking
  per compositor family, there's no shared protocol for it yet.
- The one thing Wayland *does* standardize is the opposite
  direction — injecting synthetic input — via
  [`virtual-keyboard-unstable-v1`](https://wayland.app/protocols/virtual-keyboard-unstable-v1),
  which is how a compositor-level remapper can *emit* keys back in even
  though it can't cleanly *observe* which app is focused. This asymmetry
  (easy to inject, hard to observe-with-attribution) is exactly the
  asymmetry Keylex will hit if `focus::` ever grows a Wayland backend: it
  will need a separate implementation per compositor family, not one
  general fix, and evdev-layer capture (which Keylex already does) sidesteps
  the observation problem entirely by grabbing the raw device before any
  compositor-level focus concept applies — the same reason keyd/xremap/
  evremap all work today on Wayland already for the *keycode* remap case,
  while only the *native-command* case (which needs to know the focused
  app) is Wayland's outstanding hard problem for Keylex specifically.

## Windows: user-mode hooks vs. a kernel filter driver

- **AutoHotkey** implements essentially every hotkey that isn't a plain
  `RegisterHotkey` combo (any `#HotIf` context, any `~`/`*`/`$` modifier
  prefix, any custom multi-key combo) via a **`WH_KEYBOARD_LL`** low-level
  keyboard hook — the same primitive Keylex's own
  [`src/capture/windows.rs`](../src/capture/windows.rs) uses. Suppression
  works the same way in both: return non-zero instead of calling
  `CallNextHookEx`. A documented, non-hypothetical cost of this approach
  (true for AHK, PowerToys, and Keylex alike): `WH_KEYBOARD_LL` hooks from
  *every* process on the system are chained together and called
  in-process, synchronously, in registration order, on the thread that
  installed them — a slow or hung hook anywhere in that chain delays
  every hook after it, and Windows will silently drop a hook that's too
  slow to respond. This is a real, structural latency risk for a
  keyboard daemon that's architecturally invisible until another
  low-level hook happens to be running at the same time — worth a note
  next to Keylex's `windows.rs`, since it's untested on real hardware
  ([CLAUDE.md](../CLAUDE.md) already flags that gap for other reasons).
- **[PowerToys Keyboard
  Manager](https://github.com/microsoft/PowerToys/blob/main/doc/devdocs/modules/keyboardmanager/keyboardmanager.md)** —
  same `WH_KEYBOARD_LL` mechanism, running in a dedicated
  `PowerToys.KeyboardManagerEngine.exe` process so remaps stay live even
  with the settings UI closed. Its own issue tracker is a useful public
  record of what goes wrong with this approach in practice: GitHub issues
  [#18795](https://github.com/microsoft/PowerToys/issues/18795) and
  [#24285](https://github.com/microsoft/PowerToys/issues/24285) report
  input latency spikes (up to multi-second) correlated with the engine
  process pegging CPU — i.e. the failure mode of a user-mode hook isn't
  graceful degradation, it's the whole system's typing latency degrading
  with it. That's a strong argument for Keylex keeping its own hook/grab
  loop as lean as possible (see keyd's `<<1ms` claim above as the bar to
  clear) rather than doing heavier work (e.g. adapter I/O) synchronously
  on the capture thread.
- **[Interception](https://github.com/oblitum/Interception)** — a
  **kernel-mode filter driver** (built against WDK 7.1.0, dual LGPL/
  commercial license) rather than a user-mode hook. It sits in the
  device stack itself, below the point where per-process hook chaining
  can even become a factor, which is the reason AutoHotkey and kanata
  both offer it as an optional backend for cases where hook-chain jitter
  or hook-chain visibility (some anti-cheat/anti-keylogger software
  specifically detects `WH_KEYBOARD_LL` hooks) is unacceptable. No public
  latency benchmark for it was found — the tradeoff for the latency
  headroom is kernel-driver signing/installation friction that a
  pure-hook tool like Keylex's current Windows backend avoids entirely.

## macOS: Karabiner-Elements (the reference implementation for a platform Keylex doesn't have yet)

[Karabiner-Elements](https://github.com/pqrs-org/Karabiner-Elements) is
worth studying now even though Keylex has no macOS backend
([CLAUDE.md](../CLAUDE.md) already documents this as a real, not
theoretical, gap), because it's the most mature answer to "what does the
Windows-hook/Linux-evdev pattern become on macOS": IOKit HID event
interception (outside the Accessibility API, which is too limited/laggy
for this) paired with a **virtual HID device** implemented via Apple's
**DriverKit** — a System Extension, replacing what used to be a signed
kernel extension pre-Catalina. That DriverKit migration is itself a
cautionary data point for a future Keylex macOS backend: Apple has been
actively deprecating kernel extensions in favor of userspace System
Extensions industry-wide, so a macOS capture backend designed today should
target DriverKit/System Extensions from the start rather than a kext, or
it will need the same forced migration Karabiner already went through.
No independently-verified end-to-end latency figure for Karabiner is
published by the project itself — treat any specific millisecond number
you see quoted for it elsewhere as unverified.

## The Stream Deck ecosystem — and where `skate702`'s project fits

- **Elgato's own [Stream Deck
  SDK](https://docs.elgato.com/sdk/plugins/architecture)**: each plugin is
  a single long-lived process that speaks to the Stream Deck application
  over a dedicated **WebSocket**, following a registration handshake, then
  receives `keyDown`/`keyUp` events and can push settings back via
  `setSettings`/`didReceiveSettings`. Structurally this is exactly
  Keylex's own WebSocket adapter (`src/adapters/websocket.rs`) turned
  around: Elgato's app is the one thing that can't easily be a socket
  *server* reachable by arbitrary plugins, so — like Keylex's Chrome
  target — the roles are fixed with the hardware-facing app as the
  connection's "server" role from the plugin's point of view. See
  [protocol.md](protocol.md#websocket) for Keylex's own reasoning about
  which side of a WebSocket has to be the server given what each end is
  capable of.
- **[skate702](https://x.com/skate702)** (Sebastian Hahner, Twitch/X
  handle, GitHub `sebinside`) built
  **[HotkeylessAHK](https://github.com/sebinside/HotkeylessAHK)**: an
  AutoHotkey-side HTTP server (`localhost:42800/send/<FunctionName>`,
  parameters as query args) plus a companion Stream Deck plugin that
  calls it. The Stream Deck plugin queries a `list()` endpoint to
  auto-discover whatever functions the running AHK script has registered,
  then fires them with a plain HTTP GET per keypress. This is a genuinely
  different transport choice than either Elgato's own WebSocket protocol
  or Keylex's TCP-socket/newline-JSON protocol: **HTTP-over-loopback**,
  one connection per dispatch rather than one persistent connection.
  That's a real, measurable design tradeoff (TCP handshake + HTTP parsing
  overhead per keystroke, vs. Keylex's explicit choice — noted in
  [protocol.md](protocol.md) — to keep the WebSocket connection open
  specifically because "re-handshaking a WebSocket on every keystroke
  would be wasteful on the keyboard-input path"). HotkeylessAHK optimizes
  for zero-friction discoverability (any AHK function is instantly a
  Stream Deck action, no protocol implementation needed on the AHK side)
  at the cost of the per-keystroke connection overhead Keylex deliberately
  avoided. Related, less actively maintained prior art in the same space:
  [AutoHotStreamDeck](https://github.com/evilC/AutoHotStreamDeck).

## Gaming: input remapping as a first-class abstraction, not just a keycode swap

This corner of the ecosystem is the closest external parallel to what
Keylex's action vocabulary (`actions.toml`, `hotkeys-reference.csv`) is trying to
do, even though the domain is game controllers rather than keyboards +
apps:

- **[Steam Input](https://partner.steamgames.com/doc/api/isteaminput)** —
  in "Native mode," a game never reads raw buttons at all; it declares
  **actions** and **action sets** (a named, swappable group of bindings —
  conceptually close to Keylex's `[[action]]` entries) and Steam resolves
  the player's actual physical controller into those actions, including
  runtime **Action Set Layers** that overlay/modify a base set. The game
  can query which physical control is currently bound to a given action
  purely to draw the correct on-screen button glyph — i.e. Valve solved
  "the app needs to know what a user-remappable action *is*, independent
  of the physical input" for controllers in essentially the same shape
  Keylex is solving it for keyboard+app actions. This is validation that
  the "middle vocabulary" architectural bet (an action layer between raw
  input and app-specific commands, per [protocol.md](protocol.md)'s LSP
  comparison) is a proven pattern elsewhere, not a novel risk.
- **[reWASD](https://www.rewasd.com/)** and **DS4Windows** — both build a
  **virtual controller device** and remap onto it, architecturally
  parallel to Keylex's uinput virtual device on Linux. reWASD's own forum
  documentation states its remapping engine adds "5ms" when synthesizing a
  virtual controller, and explicitly "no latency when remapping mouse or
  keyboard to virtual mouse or keyboard inputs" — treat the 5ms figure as
  vendor-reported, not independently benchmarked, but it's a useful order-
  of-magnitude anchor: a well-optimized virtual-device remap layer targets
  low single-digit milliseconds, matching keyd's `<<1ms` claim on the
  keyboard side.
- Kernel-level anti-cheat is the adjacent reason gaming remap tools
  sometimes avoid the "cleanest" architecture: a kernel filter driver
  (Interception-style) can look identical to a cheat-input injector from
  an anti-cheat's point of view, which is part of why some remap tools
  stick to user-mode hooks or virtual-HID emulation even when a kernel
  driver would be architecturally simpler. Not directly a Keylex concern
  today (Keylex doesn't target games), but worth remembering if a future
  target ever needs one.

## Firmware-level remapping (QMK/ZMK) — the layer below all of the above

Every tool discussed so far remaps *after* the keyboard has already
reported a scancode over USB/BLE. QMK and ZMK remap **inside the keyboard
itself**, before that report is ever sent, which puts a hard floor under
every number quoted above:

- USB polling interval alone contributes up to **8ms at 125Hz** vs. **~1ms
  at 1000Hz** — this is pure protocol latency, independent of any software
  remap layer sitting downstream. [QMK's own docs](https://beta.docs.qmk.fm/developing-qmk/qmk-reference/config_options)
  expose `USB_POLLING_INTERVAL_MS` precisely because this floor is
  configurable per-keyboard.
- A widely-cited independent measurement (Michael Stapelberg's
  photodiode-based QMK latency testing on a Kinesis Advantage) is
  frequently referenced as showing real-world total keyboard latency in
  the high single-digit-to-low-double-digit millisecond range once
  switch debounce, matrix scan, and polling are all accounted for — cited
  here as a pointer to the methodology (measure with a physical sensor,
  not software timestamps) rather than as a number Keylex should quote,
  since the source page couldn't be re-fetched to confirm exact figures
  during this research pass.
- ZMK's BLE split-keyboard transport adds its own extra hop (reported in
  the ZMK issue tracker as averaging a few milliseconds) on top of the
  polling-rate floor — relevant only as a reminder that *every* layer
  between a physical switch and Keylex's own capture code adds latency
  Keylex has no visibility into or control over.

**Why this matters for Keylex specifically:** every millisecond Keylex's
own `capture/linux.rs` reader-thread/main-thread handoff or its 35ms chord
debounce constant adds is stacked on top of this firmware/USB floor, not
instead of it. There's no realistic way to make Keylex's own pipeline
"feel" faster than a keyboard with a 125Hz polling rate permits — a useful
sanity bound when reasoning about whether a reported latency complaint is
Keylex's fault at all.

## What this resolves and what it doesn't — mapped to Keylex's own known gaps

Cross-referencing this survey against [CLAUDE.md](../CLAUDE.md)'s "Known
gaps" section:

- **Wayland focused-window detection** — confirmed structurally hard, not
  just unimplemented: no tool surveyed here has a compositor-agnostic
  answer either (see "Wayland" above); xremap's and xwaykeyz's
  per-compositor detection code is the realistic shape any future Keylex
  fix will have to take, not a single shim.
- **No macOS capture backend** — Karabiner-Elements is the concrete
  reference architecture (IOKit HID tap + DriverKit virtual HID device)
  for when this is picked up; the kext→DriverKit migration Karabiner
  already went through is a warning to design against DriverKit from day
  one.
- **Chord debounce window hardcoded at 35ms** — sits comfortably above the
  firmware-level floor discussed above (sub-millisecond to ~8ms depending
  on polling rate), so 35ms is a UX/reliability choice, not something
  constrained by hardware latency; no tool surveyed publishes a
  "canonical" debounce constant to benchmark against, evremap and keyd
  both treat this as a tunable rather than documenting a specific default.
- **No real OS notification (`Notifier` is log-only)** — orthogonal to
  everything surveyed here; none of these tools' notification stories
  were in scope for this pass.
- **Windows backend untested on real hardware** — the `WH_KEYBOARD_LL`
  hook-chaining latency risk described above (a slow hook elsewhere in
  the chain delaying Keylex's own hook) is a concrete new risk this
  research surfaces that wasn't previously called out; worth a manual
  test pass (running alongside AutoHotkey/PowerToys simultaneously)
  whenever real Windows hardware becomes available.
- **The native-command/keycode-fallback split itself** — not something
  any Linux/Windows/macOS remapper surveyed here does (they all terminate
  at "produce a keycode"); Steam Input is the one clear precedent, and
  only in the controller-input domain. This remains Keylex's most
  distinctive architectural bet, not a known gap — but it also means
  there's no existing benchmark or war story to borrow latency numbers
  from for that specific code path (`Router::dispatch`'s native branch);
  if that path ever needs a latency budget, it'll have to be measured
  fresh rather than sourced from prior art.

## Libraries worth knowing about (not adopted, just catalogued)

For general-purpose keyboard/mouse *libraries* (as opposed to the
standalone products above) — pynput, libuiohook and everything built on
it, robotgo, global-hotkey, and why none of them can replace Keylex's own
hand-rolled capture layer — see the dedicated companion doc,
[libraries.md](libraries.md). The Rust crates listed below are repeated
here because they're closer to Keylex's own domain (remapping daemons)
than that survey's app-hotkey-library focus.

- **[rdev](https://github.com/Narsil/rdev)** — cross-platform (Linux via
  evdev, Windows, macOS) Rust listen/grab/send library, MIT-licensed,
  serde-friendly event types. The closest existing crate to what Keylex's
  own `src/capture/` modules hand-roll per-platform; not adopted by
  Keylex today, but a candidate to revisit if/when a macOS backend is
  built, since it already has a macOS event-tap implementation to compare
  against a hand-written `CGEventTap` backend.
- **[evdev](https://github.com/emberian/evdev)** /
  **[evdevil](https://github.com/SludgePhD/evdevil)** — lower-level Rust
  evdev/uinput bindings than rdev; evdevil specifically claims broader
  coverage (force-feedback, multitouch) than Keylex needs today.
- **[keyberon](https://github.com/TeXitoi/keyberon)** — the Rust crate
  originally built for QMK-style firmware keyboard layers, now reused
  as kanata's cross-platform remapping core. Notable as a rare case of
  firmware-side remapping logic (layers, tap-hold) being generalized into
  a reusable library good enough for a desktop-daemon use case.

## Sources

- [keyd](https://github.com/rvaiya/keyd)
- [xremap](https://github.com/xremap/xremap) /
  [xremap architecture writeup](https://www.paolomainardi.com/posts/linux-remapping-keys-with-xremap/)
- [evremap](https://github.com/wez/evremap)
- [input-remapper](https://github.com/sezanzeb/input-remapper)
- [kanata](https://github.com/jtroo/kanata)
- [xwaykeyz](https://github.com/RedBearAK/xwaykeyz)
- [libinput architecture docs](https://wayland.freedesktop.org/libinput/doc/latest/architecture.html)
- [Peter Hutterer — evdev vs. uinput](http://who-t.blogspot.com/2016/05/the-difference-between-uinput-and-evdev.html)
- [virtual-keyboard-unstable-v1 protocol](https://wayland.app/protocols/virtual-keyboard-unstable-v1)
- [PowerToys Keyboard Manager dev docs](https://github.com/microsoft/PowerToys/blob/main/doc/devdocs/modules/keyboardmanager/keyboardmanager.md),
  [issue #18795](https://github.com/microsoft/PowerToys/issues/18795),
  [issue #24285](https://github.com/microsoft/PowerToys/issues/24285)
- [Interception driver](https://github.com/oblitum/Interception)
- [Karabiner-Elements](https://github.com/pqrs-org/Karabiner-Elements)
- [Stream Deck SDK architecture](https://docs.elgato.com/sdk/plugins/architecture)
- [HotkeylessAHK](https://github.com/sebinside/HotkeylessAHK) /
  [skate702 (X/Twitter)](https://x.com/skate702/status/1569018321093414912) /
  [AutoHotStreamDeck](https://github.com/evilC/AutoHotStreamDeck)
- [Steam Input (ISteamInput)](https://partner.steamgames.com/doc/api/isteaminput),
  [Action Set Layers](https://partner.steamgames.com/doc/features/steam_controller/action_set_layers)
- [reWASD](https://www.rewasd.com/)
- [QMK config options](https://beta.docs.qmk.fm/developing-qmk/qmk-reference/config_options)
- [ZMK USB polling rate issue](https://github.com/zmkfirmware/zmk/issues/432)
- [rdev](https://github.com/Narsil/rdev),
  [evdev (Rust)](https://github.com/emberian/evdev),
  [evdevil](https://github.com/SludgePhD/evdevil),
  [keyberon](https://github.com/TeXitoi/keyberon)
