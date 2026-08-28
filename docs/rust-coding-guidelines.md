# Rust coding guidelines

Status: **active guidance**, applies to all Rust code in [`src/`](../src/)
and [`tests/`](../tests/). Unlike [prior-art.md](prior-art.md) and
[libraries.md](libraries.md), this isn't a research survey — it's what a
change to this repo's Rust code is expected to follow, and what a reviewer
(human or Claude) should check a diff against.

## Sources this builds on

This document doesn't re-explain what these already cover well; it links
to them and then says where Keylex's own code narrows the choice or adds a
project-specific rule on top:

- **[The Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/)**
  — the formatting rules `rustfmt` applies by default. This repo ships no
  `rustfmt.toml`, so the defaults are the whole story; run `cargo fmt`
  before committing.
- **[Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)** —
  naming, documentation, interoperability, and predictability conventions
  for anything exposed as a public API. Keylex has no external consumers
  yet (it's a binary, not a published crate), but `src/lib.rs` re-exports
  modules for the integration tests in `tests/`, so the same discipline
  applies to anything `pub`.
- **[RFC 430](https://rust-lang.github.io/rfcs/0430-finalizing-naming-conventions.html)**
  — the naming conventions (`UpperCamelCase` types, `snake_case`
  functions/modules, `SCREAMING_SNAKE_CASE` constants) that both of the
  above build on and that `rustc`'s own lints enforce.
- **[Clippy](https://doc.rust-lang.org/clippy/)** — run `cargo clippy`
  before committing. There's no CI enforcement of this yet (see "What's
  not enforced" below), so it's on the author, not a gate.
- **[The Rustonomicon](https://doc.rust-lang.org/nomicon/)** — the
  reference for what makes an `unsafe` block sound. See "Unsafe code"
  below for how this applies to Keylex's Windows FFI.
- **[ANSSI's Rust programming guidelines](https://anssi-fr.github.io/rust-guide/)**
  — a security-agency-authored guide aimed specifically at code with a
  security posture to defend, which describes Keylex's own situation
  well: it sits deep in the keyboard input path (see
  [README.md#privacy--security](../README.md#privacy--security)) and
  crosses an FFI boundary on Windows. Its sections on unsafe code, FFI,
  and dependency vetting are the most relevant here.
- **[The Rust Book, ch. 9 — Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)**
  — the baseline `Result`-vs-`panic!` model this codebase follows.

## What's not enforced (yet)

Being upfront about this rather than implying more automation than exists:
there is no `rustfmt.toml`, no `clippy.toml`, and no CI workflow running
either. `cargo build` and `cargo test` are the only checks a contributor is
forced through. Treat everything below as a review standard, not a linter
you can rely on to catch violations.

One check *is* worth running by hand and easy to forget, because this dev
environment is Linux-only and `cargo build` never touches the other
backend:

```bash
rustup target add x86_64-pc-windows-msvc   # once
cargo check --target x86_64-pc-windows-msvc
```

`cargo check` doesn't link, so it needs no MSVC toolchain — only the
target's standard library. This is not optional politeness: the Windows
capture backend was, at one point, committed in a state that did not
compile at all, and nothing in the Linux build could have said so.

## Module layout

One concern per module, and a directory (`config/`, `capture/`,
`spotlight/`, `adapters/`, `focus/`) as soon as a file grows past roughly
300 lines of substance. The parent `mod.rs` owns the type the rest of the
daemon talks to and re-exports what belongs to the module's surface; the
children hold the parts that can be understood on their own —
[`src/config/key.rs`](../src/config/key.rs) (what a trigger is),
[`src/config/target.rs`](../src/config/target.rs) (where an action goes),
[`src/config/error.rs`](../src/config/error.rs).

Platform-specific code lives behind a `cfg`-gated backend module with an
identical signature per platform (`capture::run`,
`focus::focused_process_name`), and anything the backends agree on is
lifted out rather than written twice: the chord state machine in
[`src/capture/chord.rs`](../src/capture/chord.rs) is shared by both,
with each backend supplying only the effects it performs differently
(re-emitting a real event through uinput vs. synthesizing one through
`SendInput`).

## Naming

Follow RFC 430 / API Guidelines' `C-CASE` as-is: `UpperCamelCase` for
types and traits (`KeyCombo`, `Chord`, `Registry`), `snake_case` for
functions, modules, and variables, `SCREAMING_SNAKE_CASE` for constants
(`MODIFIERS` in [`src/config/key.rs`](../src/config/key.rs)).

Don't repeat the module in the type: it's `spotlight::Entry` and
`dispatch::Outcome`, not `spotlight::SpotlightEntry` (API Guidelines
`C-WORD-ORDER`, and the stutter rule the standard library follows).

Don't confuse any of this with the *action-id* grammar (`modifier` or
`modifier.location`, e.g. `close.tab`) — that's a TOML-level vocabulary
constraint enforced by `Registry::load`, documented in
[CLAUDE.md](../CLAUDE.md) and [protocol.md](protocol.md), and unrelated to
Rust identifier naming rules.

## Types over strings

Config is parsed into types once, at load, and never re-interpreted
downstream. `fallback_tier = "notify_only"` becomes a
[`Fallback`](../src/config/action.rs) variant, `adapter = "websocket"`
becomes an `AdapterKind`, `key = "ctrl+w"` becomes a `KeyCombo`. Dispatch
then matches on those, rather than comparing strings and re-parsing combos
in each capture backend.

The same rule applies to a target's own fields: declare them on `Target`
(with `#[serde(deny_unknown_fields)]`, so a typo is a startup error rather
than a silently ignored line) instead of keeping a bag of
`toml::Value`s to be dug through at call sites.

## Documentation comments

Every module opens with a `//!` doc comment stating what the module is
for — see the top of [`src/dispatch.rs`](../src/dispatch.rs) or
[`src/capture/chord.rs`](../src/capture/chord.rs).

For comments on individual items, this repo already follows the same rule
[CLAUDE.md](../CLAUDE.md) states for the project as a whole: don't
document *what* the code does (the types and names should already say
that) — document the *why* when it's non-obvious. The doc comments on
`KeyCombo` vs. `Chord` in [`src/config/key.rs`](../src/config/key.rs) are
a good model: they don't restate the fields, they explain why two
similar-looking types both exist (one privileges a single trigger key, the
other deliberately doesn't).

## Error handling

Default to `Result` at any boundary that can fail — config parsing, I/O,
network. A bare `unwrap()` is reserved for `#[cfg(test)]` code, where it's
preferred over `?` for readability.

Outside tests, `expect()` is allowed only when the message states the
*invariant* being relied on — not a generic "should work" string, but the
specific reason the `Err`/`None` case can't happen here. The two remaining
call sites are the template:

- `src/capture/linux.rs` — `.expect("a device that reached
  discover_keyboard() must report supported keys")`
- `src/adapters/websocket.rs` — `.expect("a response built from constants
  is always well-formed")`

If you can't write a message that states *why* the invariant holds,
that's a sign the `expect()` should be a propagated `Result` instead — or
that the value should never have been looked up twice in the first place.

`Mutex::lock().unwrap()` (see
[`src/adapters/websocket.rs`](../src/adapters/websocket.rs)) is the one
accepted exception to "no bare `unwrap()` outside tests": a poisoned lock
means a prior panic already corrupted shared state, and there's no
sensible recovery, so propagating the panic is correct, not lazy.

Where a failure is expected and routine — a target that isn't running, a
focused process that can't be resolved — model it as `Option` and report
it, rather than as an error that unwinds a dispatch.

## Concurrency

Never hold a lock across blocking I/O. `WebSocketAdapter` gives each
accepted connection a thread that owns the socket outright and takes
commands over a channel, precisely so `send` — which runs on the keyboard
path — only ever does a lock-and-push. An earlier version shared the
socket behind a `Mutex` that the reader held across its blocking read; a
dispatch could then starve for tens of seconds waiting for it.

## Unsafe code

`unsafe` is confined to the Windows FFI boundary —
[`src/capture/windows.rs`](../src/capture/windows.rs) and
[`src/focus/windows.rs`](../src/focus/windows.rs), calling `SendInput`,
`SetTimer`, `GetForegroundWindow`, and friends via the `windows` crate.
Nothing on the Linux side needs `unsafe` (`evdev`/`uinput` access goes
through the safe `evdev` crate). Keep it that way: if a change to
capture/dispatch logic on Linux seems to need `unsafe`, that's worth a
second look rather than reaching for it.

Per the Nomicon's and ANSSI's guidance:

- Keep every `unsafe` block as small as the FFI call actually requires;
  don't wrap surrounding safe logic in it. Prefer a safe function with a
  small `unsafe` block inside over an `unsafe fn` whose whole body is
  unchecked.
- Put a `// SAFETY:` comment above each block stating the invariant that
  makes the call sound — why the pointer or handle is valid, why the
  buffer length matches. `focus::windows::image_name` is the model: the
  comment names the one thing the call actually depends on.
- The type-erased `HOOK_STATE` thread-local in `src/capture/windows.rs` is
  the one place where an invariant spans functions rather than sitting
  inside one. Its safety comment says who writes the pointer, when it is
  cleared, and why a non-null read is therefore always live; keep that
  comment true if you touch `run()`.

## Dependencies

Keep the dependency list minimal and justify anything non-obvious inline.
[`Cargo.toml`](../Cargo.toml) already does this: `rand` and `http` are
promoted from transitive to direct dependencies with a comment explaining
they're already pulled in via `tungstenite`'s handshake feature, so
declaring them directly adds no new supply-chain surface. Follow the same
pattern — a one-line comment in `Cargo.toml` — whenever a new dependency
is added, especially one that isn't obviously required by the feature
being built.

## Testing

Unit tests live inline in `#[cfg(test)] mod tests` blocks next to the code
they cover (`src/config/`, `src/auth.rs`, `src/spotlight/`). Cross-module
behavior (dispatch routing across the config/adapter boundary) belongs in
[`tests/dispatch.rs`](../tests/dispatch.rs) instead. Run `cargo test
<name>` to filter by substring when iterating on one test.

Tests run in parallel by default, so anything touching the filesystem must
pick a path unique to that test, not just to the process.
