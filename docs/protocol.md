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

`keylex/v0` is transport-agnostic in principle; two transports are
implemented today, both carrying the exact same message (see below).

### TCP socket

A **local TCP socket**, one JSON object per line (newline-delimited JSON,
not framed JSON-RPC). See [src/adapters/socket.rs](../src/adapters/socket.rs)
for the reference client (`SocketAdapter`) and the target-side address
configured in [config/targets.toml](../config/targets.toml)
(`[target.address]`, e.g. `"127.0.0.1:7777"`). Here Keylex is the TCP
*client*: it connects out to the target's server (e.g. the VS Code
extension), which is why this transport only works for targets whose
runtime can open a listening socket.

### WebSocket

For targets that can't listen on a socket at all -- a browser extension's
background script has no server capability, only client APIs (WebSocket,
fetch) -- the **roles are flipped**: Keylex runs the WebSocket *server*,
and the target's client connects in. The wire message is identical to the
TCP transport's, just carried as a single WS text frame instead of a
newline-terminated line. See
[src/adapters/websocket.rs](../src/adapters/websocket.rs) for the daemon
side and [extensions/chrome-extension/background.js](../extensions/chrome-extension/background.js)
for the reference client. A target using this transport sets
`adapter = "websocket"` and a `port` in `targets.toml`; only one connected
client is kept per target for now (a fresh connection supersedes an older
one). This transport keeps a single persistent connection open rather than
reconnecting per dispatch, since re-handshaking a WebSocket on every
keystroke would be wasteful on the keyboard-input path.

## Message

One line, one JSON object, UTF-8, `\n`-terminated:

```json
{"command": "workbench.action.closeActiveEditor", "token": "9f2c...ab"}
```

| Field     | Type   | Required | Meaning                                                                 |
|-----------|--------|----------|--------------------------------------------------------------------------|
| `command` | string | yes      | The native command string, as declared in that target's `supports` map in `targets.toml`. Opaque to Keylex — only the target-side adapter/extension interprets it. |
| `token`   | string | yes      | The shared secret from `<config-dir>/secret.token` (see "Trust model & authentication" below). Checked before `command` is ever acted on. |

There is currently no response/acknowledgement message — dispatch is
fire-and-forget. `SocketAdapter` logs (but does not retry) a connection
failure, which the router surfaces only as "the native adapter didn't run"
— it still counts as `status: "native"` in `DispatchResult`, since Keylex
has no way to distinguish "target unreachable" from "target
processed it" without a response message. **This is a known gap**, not a
design decision — an ack/error response is the most likely `v0` → `v1`
change.

## Trust model & authentication

Both transports run entirely over loopback (`127.0.0.1`), but loopback
alone isn't a trust boundary: any other local process can open a TCP
connection to a loopback port, and — for the WebSocket transport
specifically — a webpage open in the user's own browser can too (browsers
don't block a page's JS from opening `ws://127.0.0.1:<port>`). Without
anything more, either transport would let an unrelated local process or a
malicious webpage impersonate Keylex (socket transport) or the paired
extension (WebSocket transport), or — for the single-connection WebSocket
server — silently hijack the connection away from the legitimate client.

**Shared-secret token.** On first run, the daemon generates 32 random bytes,
hex-encodes them, and writes the result to `<config-dir>/secret.token`
(`src/auth.rs`) with owner-only file permissions on Unix. This file is
deliberately excluded from version control (see `.gitignore`) — it's a
per-install secret, not configuration. Every `command` message on the
socket transport must include a matching `token` field (checked on every
message, so a token change takes effect on the very next dispatch with no
stale connections to detect). The WebSocket transport instead requires the
very first frame on a freshly accepted connection to be an auth frame,
`{"token": "<hex>"}` — a connection is only promoted into the slot `send()`
uses once that token matches; an unauthenticated connection is closed and
never displaces whatever connection (if any) is already live. This is also
what fixes the WebSocket transport's "last-connect-wins" hijack risk: only
an *authenticated* connect wins now.

**Origin allowlisting (WebSocket only, defense-in-depth).** A target using
the websocket adapter can set an `allowed_origin` field (e.g.
`"chrome-extension://<the extension's actual id>"`) in `targets.toml`. When
set, the daemon inspects the handshake's `Origin` header and rejects the
handshake outright — before a `WebSocket` value even exists — if it doesn't
match exactly. If unset, the Origin header isn't checked at all (logged
once at startup so this isn't silently permissive); this keeps the field
optional for any future non-browser WebSocket client that wouldn't send an
`Origin` header in the first place.

**What the token is (and isn't) for.** It's a shared secret proving "this
connection/message came from something with local read access to the
daemon's config directory" — the same trust level as, say, an SSH agent
socket or a private key file. It stops an unrelated local process or a
malicious webpage from driving a target's native commands. It does **not**
defend against a fully compromised local user account with the same
filesystem access as the Keylex process (that account can just read
`secret.token` directly), nor against an already-compromised browser or
extension. TLS/`wss://` is deliberately not used here: the threat is
"another local process/webpage," not network eavesdropping, and a
loopback-only certificate wouldn't stop either.

## What Keylex guarantees vs. what the target owns

- Keylex guarantees the `command` string is only sent when
  `Router::dispatch` resolved a `target.supports[action_id]` entry for the
  currently focused process (see
  [src/dispatch.rs](../src/dispatch.rs)), and only alongside a `token`
  matching the per-install secret in `<config-dir>/secret.token` (see
  "Trust model & authentication" above).
- Keylex does **not** validate that `command` means anything — that
  mapping (`"close.tab" → "workbench.action.closeActiveEditor"`) lives
  entirely in `targets.toml` and is the target adapter's contract to
  fulfil (e.g. the VS Code extension calling
  `vscode.commands.executeCommand(command)`).
### Action IDs

Action IDs are **not** free-form strings — they're built from two
validated word lists in [config/vocabulary.toml](../config/vocabulary.toml),
a `modifiers` list (verbs, e.g. `close`, `save`, `move`) and a `locations`
list (objects, e.g. `tab`, `sidebar`, `left`). Every `[[action]]` in
[config/actions.toml](../config/actions.toml) declares a `modifier` and an
optional `location`; `Registry::load` (`src/config.rs`) rejects the whole
config at startup if either word isn't in `vocabulary.toml`, and derives
the actual id from them:

- `modifier` alone, e.g. `save`, `shutdown` — for actions with no natural
  object.
- `modifier.location`, e.g. `close.tab`, `move.left` — everything else.

This id is app-agnostic by design: the same `close.tab` is what a
`ctrl+w` key binding resolves to regardless of which app ends up
focused. Per-app translation only happens downstream, in each target's
`supports` map (see "Native command strings" below) — this is what lets
one physical key mean the same abstract thing everywhere, with the
native-adapter indirection doing the actual app-specific work.

### Native command strings

Each target's `supports` map (the values sent as `command` on the wire)
now lives in that target's own `capabilities.toml`, next to its adapter
code — `extensions/<name>/capabilities.toml` — rather than centrally in
`targets.toml`, which points at it via a `capabilities` path. This is
what lets an extension own the declaration of what it understands and
accepts, instead of that living as a second, easily-drifting copy of the
allowlist it already hardcodes on the receiving end (`ALLOWED_COMMANDS`
in `vscode-extension/extension.js`, the `switch` in
`chrome-extension/background.js`, the `COMMANDS` maps in the two
system-listener `listener.js` files).

For a command string Keylex invented itself — where it controls both the
daemon side and the target's handler — the shape is **enforced**, not
just recommended: `<application>.<location>.<action>`, three
lowercase/underscore dot-separated tokens, e.g. `chrome.tab.close`,
`os.window.move_left`. `Registry::load` rejects any non-conforming value.

The one thing this can't apply to is a command string that's actually
someone else's namespace: VS Code's real command IDs
(`workbench.action.closeActiveEditor`) are fixed by Microsoft, and
Neovim's ex-commands (`bd`, `w`) are fixed by Neovim — Keylex only
forwards them, it doesn't get to rename them. A target can opt out of the
grammar check with `exempt_command_grammar = true` in `targets.toml` for
exactly this reason; `vscode` and `neovim` are the only two that set it.

## Versioning

No version field exists on the wire yet — `v0` is implicit, now shared by
two consumers (the VS Code adapter over TCP, the Chrome extension over
WebSocket) that both speak the exact same `{"command": ...}` message, just
over different transports. Before adding a *third* adapter with a
genuinely different message shape, add an explicit
`{"protocol": "keylex/v0", ...}` envelope so multiple concurrent protocol
versions can be told apart during the `v0` → `v1` transition.

## Open questions (not yet decided)

- Should `v1` move to JSON-RPC 2.0 (matching LSP) for request/response and
  batching, or stay minimal newline-JSON/WS-text? LSP's choice was driven
  by wanting standard tooling; Keylex's per-command payloads are much
  simpler, so this isn't a given.
- Neovim (msgpack-RPC) and a terminal-emulator adapter will each need
  their own framing/transport section here once implemented, since neither
  can use the TCP-socket or WebSocket transports as-is.
- Unix-domain sockets (permission-based auth, no shared token needed) were
  considered as an alternative to a token for the socket transport, and
  rejected: they don't help the WebSocket transport at all, since browser
  JS can only open TCP/WS connections, never a Unix socket. One consistent
  auth mechanism across both transports beat a different one per transport.

## Chorded triggers

An action can bind to a **chord** instead of a single key(+modifier)
combo: an order-independent set of two or more keys that must all be held
down together. Configured in `actions.toml` with `chord` instead of `key`:

```toml
[[action]]
id = "some.action"
chord = ["ctrl", "d", "f"]
```

- `chord` is a TOML array of the same key tokens `key`/`fallback_keycode`
  already use (`ctrl`/`shift`/`alt`/`win` plus `a`-`z`/`0`-`9`/`prtsc`).
  Order in the array doesn't matter.
- `key` and `chord` are mutually exclusive on one action; `Registry::load`
  rejects an action with both set, a chord with fewer than two keys, a
  chord with a duplicate key, or a chord made of modifiers only (a chord
  is meant to express "keys held together", not just held-down modifiers).
- Two different actions' chords are allowed to share a member key (e.g.
  `["d","f"]` and `["d","g"]` can both be configured) — whichever complete
  set forms first wins.
- **Observable runtime behavior**: because a lone keystroke that happens to
  be a chord member can't be told apart from "the start of a chord" until
  either the rest of the chord follows or a short window elapses, pressing
  such a key alone (no chord) is delivered to the focused app with a small
  added delay (currently a fixed ~35ms debounce window) rather than
  instantly — this applies to any target's native adapter and to the
  keycode fallback path alike, since it happens in the capture backend
  before dispatch is even considered. A key released before the window
  elapses resolves immediately rather than waiting out the rest of it. See
  [src/capture/linux.rs](../src/capture/linux.rs) and
  [src/capture/windows.rs](../src/capture/windows.rs) for the platform
  implementations of this debounce/replay mechanism.
