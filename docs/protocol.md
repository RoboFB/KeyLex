# Keylex Adapter Protocol — `keylex/v0`

Status: **draft, unstable.** This document exists so a target adapter (an
editor extension, a browser extension host, a firmware author) can be
implemented against a written spec instead of reading the Python daemon's
source. Nothing here is a compatibility promise yet — expect breaking
changes until `v1`.

## Why a written protocol at all

Keylex's long-term goal isn't just a keycode remapper: it's a shared,
documented way for input devices *and* applications to talk about actions
("close tab", "go to definition") instead of raw keycodes — comparable to
what the [Language Server Protocol](https://microsoft.github.io/language-server-protocol/)
did for editors and language tooling. LSP's lesson is that this only works
if the wire format is specified independently of any one implementation.
This file is the first step in that direction for the app-adapter side of
Keylex; the device-input side (`devices.toml`) will get the same treatment
once more than one device-input mechanism exists.

## Transport

`keylex/v0` is transport-agnostic in principle; the only implemented
transport today is a **local TCP socket**, one JSON object per line
(newline-delimited JSON, not framed JSON-RPC). See
[src/keylex/adapters/vscode.py](../src/keylex/adapters/vscode.py) for the
reference client (`SocketAdapter`) and the target-side address configured
in [src/keylex/config/targets.toml](../src/keylex/config/targets.toml)
(`[target.address]`, e.g. `"127.0.0.1:7777"`).

## Message

One line, one JSON object, UTF-8, `\n`-terminated:

```json
{"command": "workbench.action.closeActiveEditor"}
```

| Field     | Type   | Required | Meaning                                                                 |
|-----------|--------|----------|--------------------------------------------------------------------------|
| `command` | string | yes      | The native command string, as declared in that target's `supports` map in `targets.toml`. Opaque to Keylex — only the target-side adapter/extension interprets it. |

There is currently no response/acknowledgement message — dispatch is
fire-and-forget. `SocketAdapter` logs (but does not retry) a connection
failure, which the router surfaces only as "the native adapter didn't run"
— it still counts as `status: "native"` in `DispatchResult`, since Keylex
has no way to distinguish "target unreachable" from "target
processed it" without a response message. **This is a known gap**, not a
design decision — an ack/error response is the most likely `v0` → `v1`
change.

## What Keylex guarantees vs. what the target owns

- Keylex guarantees the `command` string is only sent when
  `Router.dispatch` resolved a `target.supports[action_id]` entry for the
  currently focused process (see
  [src/keylex/core/router.py](../src/keylex/core/router.py)).
- Keylex does **not** validate that `command` means anything — that
  mapping (`"close.tab" → "workbench.action.closeActiveEditor"`) lives
  entirely in `targets.toml` and is the target adapter's contract to
  fulfil (e.g. the VS Code extension calling
  `vscode.commands.executeCommand(command)`).
- Action IDs themselves (the `close.tab` on the Keylex side) are
  constrained by the verb+object grammar declared in `actions.toml` and
  enforced by `Registry._validate_action_grammar` — see
  [../CLAUDE.md](../CLAUDE.md) for that vocabulary.

## Versioning

No version field exists on the wire yet — `v0` is implicit and singular
(there's exactly one consumer, the VS Code adapter). Before adding a second
adapter that uses this line-protocol (Chrome native messaging currently
plans to use its own framing, not this one), add an explicit
`{"protocol": "keylex/v0", ...}` envelope so multiple concurrent protocol
versions can be told apart during the `v0` → `v1` transition.

## Open questions (not yet decided)

- Should `v1` move to JSON-RPC 2.0 (matching LSP) for request/response and
  batching, or stay minimal newline-JSON? LSP's choice was driven by
  wanting standard tooling; Keylex's per-command payloads are much
  simpler, so this isn't a given.
- Chrome (Native Messaging) and Neovim (msgpack-RPC) adapters will each
  need their own framing/transport section here once implemented, since
  neither can use a raw TCP socket the way the VS Code adapter does.
