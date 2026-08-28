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
  below for how this applies to Keylex's two `unsafe` call sites.
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

## Naming

Follow RFC 430 / API Guidelines' `C-CASE` as-is: `UpperCamelCase` for
types and traits (`KeyCombo`, `ChordSpec`, `Registry`), `snake_case` for
functions, modules, and variables, `SCREAMING_SNAKE_CASE` for constants
(`MODIFIER_NAMES` in [`src/config.rs`](../src/config.rs)).

Don't confuse this with the *action-id* grammar (`modifier` or
`modifier.location`, e.g. `close.tab`) — that's a TOML-level vocabulary
constraint enforced by `Registry::load`, documented in
[CLAUDE.md](../CLAUDE.md#three-config-layers-configtoml--per-extension-capabilitiestoml-loaded-by-registry-srcconfigrs)
and [protocol.md](protocol.md), and unrelated to Rust identifier
naming rules.

## Documentation comments

Every module in this repo opens with a `//!` doc comment stating what the
module is for — see the top of
[`src/config.rs`](../src/config.rs#L1-L3) or
[`src/dispatch.rs`](../src/dispatch.rs). Keep doing that for any new
module.

For comments on individual items, this repo already follows the same rule
[CLAUDE.md](../CLAUDE.md) states for the project as a whole: don't
document *what* the code does (the types and names should already say
that) — document the *why* when it's non-obvious. `src/config.rs`'s doc
comments on `KeyCombo` vs. `ChordSpec` are a good model: they don't
restate the fields, they explain why two similar-looking structs both
exist (one privileges a single trigger key, the other doesn't) — see
[`src/config.rs`](../src/config.rs#L44-L58).

## Error handling

Default to `Result` at any boundary that can fail — config parsing, I/O,
network. `bare unwrap()` is reserved for `#[cfg(test)]` code, where it's
preferred over `?` for readability (see the test modules in
[`src/config.rs`](../src/config.rs) and [`src/auth.rs`](../src/auth.rs)).

Outside tests, `expect()` is allowed only when the message states the
*invariant* being relied on — not a generic "should work" string, but the
specific reason the `Err`/`None` case can't happen here. This repo's
existing call sites are the template:

- `src/dispatch.rs:82` — `.expect("checked by caller")`
- `src/capture/linux.rs:497` — `.expect("a device that reached
  discover_keyboard() must report supported keys")`
- `src/adapters/websocket.rs:138` — `.expect("static error response is
  always well-formed")`

If you can't write a message that states *why* the invariant holds,
that's a sign the `expect()` should be a propagated `Result` instead.

`Mutex::lock().unwrap()` (see
[`src/adapters/websocket.rs`](../src/adapters/websocket.rs)) is the one
accepted exception to "no bare `unwrap()` outside tests": a poisoned lock
means a prior panic already corrupted shared state, and there's no
sensible recovery, so propagating the panic is correct, not lazy.

## Unsafe code

`unsafe` is confined to the Windows FFI boundary —
[`src/capture/windows.rs`](../src/capture/windows.rs) and
[`src/focus/windows.rs`](../src/focus/windows.rs), calling `SendInput`,
`SetTimer`, `GetForegroundWindow`, and friends via the `windows` crate.
Nothing on the Linux side needs `unsafe` (`evdev`/`uinput` access goes
through the safe `evdev` crate). Keep it that way: if a change to
capture/dispatch logic on Linux seems to need `unsafe`, that's worth a
second look rather than reaching for it.

Per the Nomicon's and ANSSI's guidance, keep every `unsafe` block as small
as the FFI call actually requires (don't wrap surrounding safe logic in
it), and add a `// SAFETY:` comment above new `unsafe` blocks stating the
invariant that makes the call sound (e.g. why the pointer/handle is
valid, why the buffer length matches). The existing `unsafe` blocks in
`windows.rs` predate this rule and haven't been retrofitted — this
codebase has no Windows machine to validate a refactor of that file
against (see [CLAUDE.md](../CLAUDE.md#known-gaps--deliberately-deferred-dont-fix-without-discussion)),
so treat that as a going-forward rule for new or touched `unsafe` code,
not a backlog item to chase on its own.

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
they cover (`src/config.rs`, `src/auth.rs`). Cross-module behavior
(dispatch routing across the config/adapter boundary) belongs in
[`tests/dispatch.rs`](../tests/dispatch.rs) instead. Run `cargo test
<name>` to filter by substring when iterating on one test.
