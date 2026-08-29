# Keylex Adapter Protocol — `keylex/v0`

Status: **draft, unstable.** This document exists so a target adapter (an
editor extension, a browser extension host, a firmware author) can be
implemented against a written spec instead of reading the Python daemon's
source. Nothing here is a compatibility promise yet — expect breaking
changes until `v1`. For the coding conventions the Rust reference
implementations linked below (`src/adapters/`) follow, see
[rust-coding-guidelines.md](rust-coding-guidelines.md).

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
{"command": "workbench.action.closeActiveEditor"}
```

| Field     | Type   | Required | Meaning                                                                 |
|-----------|--------|----------|--------------------------------------------------------------------------|
| `command` | string | yes      | The native command string, as declared in that target's `supports` map in `targets.toml`. Opaque to Keylex — only the target-side adapter/extension interprets it. |

There is no `token`/auth field on the wire right now — see "Trust model &
authentication" below for why, and what's planned to replace it.

There is currently no response/acknowledgement message — dispatch is
fire-and-forget. `SocketAdapter` logs (but does not retry) a connection
failure, which the router surfaces only as "the native adapter didn't run"
— it still counts as `status: "native"` in `DispatchResult`, since Keylex
has no way to distinguish "target unreachable" from "target
processed it" without a response message. **This is a known gap**, not a
design decision — an ack/error response is the most likely `v0` → `v1`
change.

## Trust model & authentication

**Status: no authentication on the wire right now.** Earlier versions of
`keylex/v0` required a shared-secret `token` on every message; that's been
deliberately dropped for now (see CLAUDE.md's "Known gaps") to keep the
protocol and every adapter simple while the core dispatch pipeline is still
being built out. This section describes both the resulting exposure and the
mitigations that are still in place, so anyone running Keylex today knows
exactly what it does and doesn't protect against.

Both transports run entirely over loopback (`127.0.0.1`), but loopback
alone isn't a trust boundary: any other local process can open a TCP
connection to a loopback port, and — for the WebSocket transport
specifically — a webpage open in the user's own browser can too (browsers
don't block a page's JS from opening `ws://127.0.0.1:<port>`). With no
authentication at all, either transport currently lets *any* local process
or webpage impersonate Keylex (socket transport) or the paired extension
(WebSocket transport), and the WebSocket server's single connection slot is
last-connect-wins — a second, unrelated client can silently displace the
real one. This is the accepted trade-off **for a single-user machine with
no other untrusted local processes** — see the "single-user local tool"
framing in CLAUDE.md. It is not an acceptable posture for a shared machine,
a machine also running untrusted software, or a browser routinely visiting
untrusted pages, until authentication is back.

**What's still enforced.**

- Both adapters bind to `127.0.0.1` only (`SocketAdapter`'s target
  addresses are configured that way in `targets.toml`;
  `WebSocketAdapter::spawn` hardcodes `TcpListener::bind(("127.0.0.1",
  port))`) — nothing here is reachable from another machine on the network.
- **Origin allowlisting (WebSocket only, defense-in-depth).** A target
  using the websocket adapter can still set an `allowed_origin` field
  (e.g. `"chrome-extension://<the extension's actual id>"`) in
  `targets.toml`. When set, the daemon inspects the handshake's `Origin`
  header and rejects the handshake outright — before a `WebSocket` value
  even exists — if it doesn't match exactly. If unset, the Origin header
  isn't checked at all (logged once at startup so this isn't silently
  permissive). This doesn't require any shared secret and is unaffected by
  dropping the token, so it's still worth setting (see
  `extensions/chrome-extension/README.md`).

**What's gone, and why it mattered.** The old shared-secret token proved
"this connection/message came from something with local read access to the
daemon's config directory," which stopped an unrelated local process or a
malicious webpage from driving a target's native commands — it did **not**
defend against a fully compromised local user account (which could just
read the token file directly) or an already-compromised browser/extension,
and TLS/`wss://` was deliberately never used, since the threat was "another
local process/webpage," not network eavesdropping. All of that protection
is currently absent.

**Planned replacement: asymmetric keys, not a shared secret.** The next
iteration is expected to give the daemon a public key per paired target
(recorded in `targets.toml` or a dedicated paired-keys file) while each
target/extension holds the matching private key — e.g. signing a
challenge, or each message, rather than presenting a bearer secret that
grants full impersonation the moment anything else obtains it. This also
opens the door to per-target revocation (drop one public key without
affecting the others) instead of one global secret shared by every
adapter. Not implemented yet — treat this paragraph as direction, not a
spec.

**No per-command allowlist, by design.** An earlier version of
`vscode-extension/extension.js` kept a second, independent allowlist
(`ALLOWED_COMMANDS`) so that even an authenticated message could only
trigger a command the extension was specifically built to expect. That's
gone now: the extension's command catalog is discovered live from every
installed extension's own contributed commands (see "Native command
strings" and `docs/protocol.md#action-catalog-handshake-list_actions`
below) specifically so a newly installed extension shows up in spotlight
search with no hand-maintained list to keep in sync — and a per-command
allowlist can't coexist with that goal, since the whole point is not
knowing the set of valid commands in advance. The trade-off this makes is
real, and compounds with the current lack of authentication above: with no
token *and* no allowlist, anything that can reach the socket at all can
invoke *any* command currently registered in the target VS Code window,
including ones contributed by other installed extensions (running a task,
executing terminal text, editing/deleting files) that are considerably
more dangerous than "close tab". A target that wants a narrower guarantee
than "anyone who can connect can do anything this app can do" is free to
keep its own allowlist (as `extensions/linux-extension/listener.js` and
`extensions/windows-extension/listener.js` still do, since neither has an
analogous "installed extensions" universe to discover in the first place —
see "Native command strings" below).

## What Keylex guarantees vs. what the target owns

- Keylex guarantees the `command` string sent to a target came from that
  same target's own `list_actions` response (see "Action-catalog handshake"
  below) — never from a static file Keylex maintains a second copy of. It
  does **not** currently guarantee the message came from the daemon itself
  rather than an impersonating local process — see "Trust model &
  authentication" above.
- Keylex does **not** validate that `command` means anything — interpreting
  it is entirely the target adapter's contract to fulfil (e.g. the VS Code
  extension calling `vscode.commands.executeCommand(command)`).

### Action IDs

Action IDs are plain, free-form strings — there is no static vocabulary or
grammar checked at startup any more. An id is whatever the reporting target
calls it in its `list_actions` response (see "Action-catalog handshake"
below); `close.tab`/`modifier.location`-shaped ids are a convention some
targets choose to follow, not something `Registry::load` enforces.
`config/hotkeys-reference.csv` still catalogs each app's own commands for
human reference, but nothing in the daemon reads it.

Physical-key binding (which key/chord triggers which action id) is a
separate, currently-deferred concern — see `config/keymap.toml` and
CLAUDE.md's "Known gaps".

This id is app-agnostic by design: the same `close.tab` is what a
`ctrl+w` key binding resolves to regardless of which app ends up
focused. Per-app translation only happens downstream, in each target's
`supports` map (see "Native command strings" below) — this is what lets
one physical key mean the same abstract thing everywhere, with the
native-adapter indirection doing the actual app-specific work.

### Native command strings

There is no static `capabilities.toml`/`supports` map any more, on either
side. A target's entire command catalog — action id, native command
string, and title — comes from its own `list_actions` response (see below),
generated live from whatever it can actually do right now. This is a
deliberate step further than the previous design: instead of an extension
declaring its capabilities once in a file that can drift out of sync,
it's asked, and answers fresh, every time.

`vscode-extension/extension.js` implements this today (see "No per-command
allowlist, by design" under "Trust model & authentication" above). Chrome,
Neovim, and the Linux/Windows system listeners don't yet — see CLAUDE.md's
"Known gaps" — so until each grows its own `list_actions` handler, Keylex
has no native command string to send them for any action, and dispatch to
them always falls through to the keycode-fallback/unsupported path.

There is no enforced command-string grammar any more either — a target's
`list_actions` response can shape its `native_command` however its own
adapter code expects (an upstream API's own command ids, a foreign
scripting language's own syntax, or Keylex's own `<application>.<location>.
<action>` convention if a target chooses to follow it), since Keylex never
invents or checks these strings itself now.

## Action-catalog handshake (`list_actions`)

`keylex/v0`'s original message is one-directional and fire-and-forget: the
daemon sends `command`s, nothing ever replies. The spotlight action search
(`keylex --spotlight`, `src/spotlight/`) needs the opposite direction too:
it wants to know, at query time, exactly which actions a target can *actually*
carry out right now -- not a copy of that list baked into a config file that
can drift out of sync with what the target really has installed/enabled.

This adds one small request/response exchange to the TCP-socket transport
(the only transport where the daemon is already the connecting party, so a
synchronous "write request, read one response line, close" round trip fits
naturally into the existing per-`send()` connection lifecycle):

**Request** (daemon -> target, same line format as a `command` message):

```json
{"type": "list_actions"}
```

**Response** (target -> daemon, one line, same connection, before it closes):

```json
{
  "actions": [
    {
      "id": "close.tab",
      "native_command": "workbench.action.closeActiveEditor",
      "title": "Close Editor"
    }
  ]
}
```

| Field            | Type   | Meaning                                                                                   |
|------------------|--------|---------------------------------------------------------------------------------------------|
| `id`             | string | Either a real Keylex action id already known locally (e.g. one bound to a key in `config/keymap.toml`, once that's wired up) *or*, when this native command has no such cross-app abstraction, the same value as `native_command` -- a target never needs to invent an id of its own. `spotlight::Index::merge_remote` (`src/spotlight/mod.rs`) tells the two cases apart by checking whether `id` is already a known Keylex action id: if so, it enriches that entry in place; otherwise it namespaces the raw command as `"<target-program>:<id>"` so it can never collide with a real action id or another target's raw command. |
| `native_command` | string | The literal command string to send back to this target on dispatch (via `SocketAdapter::send`/`WebSocketAdapter::send`, bypassing action-id/`supports` lookup entirely for a namespaced/raw entry -- see `spotlight::Entry::dispatch`). |
| `title`          | string | A human-readable label for the spotlight list (e.g. "Close Editor"), the target's own choosing. |

A target should only report an entry here if it just verified, live, that
the underlying native command still exists/works in the running instance
(e.g. `vscode-extension/extension.js` cross-checks every command every
installed extension contributes against `vscode.commands.getCommands(true)`
before including it) -- that's the whole point of the handshake: the
spotlight catalog reflects what's *actually* available in the target right
now, not a static snapshot, and (for `vscode`) is not bounded to a
hand-picked subset at all -- see "No per-command allowlist, by design"
under "Trust model & authentication" above for what that trades away.

A raw, namespaced entry (no matching Keylex action id) has no cross-app
routing behavior: `keylex --spotlight`/`--spotlight-run` sends its
`native_command` straight to the target that reported it, regardless of
which app is currently focused, since there is no abstract action for
`Router::dispatch` to route by focus in the first place -- unlike a real
action id (`close.tab`), which still goes through the normal
focus-aware/keycode-fallback path.

This is best-effort and non-fatal: a target that doesn't implement
`list_actions` at all (older extension version, or a target that hasn't
added support yet) simply won't answer, the connection attempt times out or
the response fails to parse, and `spotlight::bootstrap` just leaves that
target's entries out -- there is no static `actions.toml`/`targets.toml`
catalog to fall back to any more; `list_actions` is the only source of
entries beyond whatever `config/keymap.toml` binds locally.
`SocketAdapter::fetch_actions` (`src/adapters/socket.rs`) is the
reference client for this exchange; it's not implemented for the WebSocket
transport yet (no target using it needs a spotlight catalog today).

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
  Worth revisiting once authentication actually returns (see "Trust model &
  authentication" above): a Unix domain socket for the TCP-socket transport
  specifically would need no keypair/token distribution at all, at the cost
  of the two transports no longer sharing one mechanism.

## Chorded triggers

An action can bind to a **chord** instead of a single key(+modifier)
combo: an order-independent set of two or more keys that must all be held
down together. The capture-side mechanism below is fully implemented and
tested; what's currently missing is a config file to declare a chord from
(the old `actions.toml`'s `chord` field is gone along with the rest of the
static action list -- see CLAUDE.md's "Known gaps"). Once a successor
exists, the shape would look like:

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
