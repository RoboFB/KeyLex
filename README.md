# Keylex

Keylex generalizes keystrokes into **abstract actions** ("close tab",
"go_to definition", "save") and dispatches those actions through the
focused application's **native API** — instead of just simulating
keycodes. A keycode is only sent as a fallback, when an app has no native
interface of its own.

The goal is for a physical key to do the same sensible thing on every
system and in every program, regardless of whether VS Code, Chrome,
Neovim, or a terminal is currently focused. The core (keyboard
interception, focus resolution, dispatch) is written in Rust, to be able
to intervene as deep in the OS as possible; the app-side adapters (e.g.
the VS Code extension) each live in whichever language makes the most
sense for that app — currently JavaScript.

## Architecture

```
Physical keyboard                 Rust daemon                     Targets
                          (config/targets.toml wiring)        (config/targets.toml)
──────────────────      ──────────────────────────────      ──────────────
Every keystroke   ─→  Capture (evdev/uinput | WH_KEYBOARD_LL)
                              │
                              ├─ no match → passed through unchanged
                              │
                              └─ match → internal action           ┌─→  VS Code adapter (extension/socket)
                                 e.g. "close.tab"             ─────┼─→  Chrome adapter (WebSocket)
                                                                    ├─→  Neovim (RPC, unimplemented)
                                                                    ├─→  OS-wide system action (socket)
                                                                    └─→  Keycode fallback (generic)
```

There is no static action vocabulary or per-target capability list any
more — both were replaced by live discovery:

1. **`config/targets.toml`** — transport wiring only: per target program,
   which process identifies it, which adapter reaches it
   (`socket`/`websocket`/`rpc`), and connection details. A target reports
   what it supports live, via the `list_actions` handshake, instead of
   Keylex holding a static copy.
2. **`config/keymap.toml`** — the skeleton for the one thing live
   discovery can't cover: which physical key/chord should trigger which
   action. Not yet wired into the daemon's startup — see
   [CLAUDE.md](CLAUDE.md) for the current status.

A key combo that matches a bound action is always intercepted (never
passed through to the OS/app directly) and dispatched; everything else
stays usable exactly as before — that's the concrete meaning of
"intercept every keycode as deep as possible."

## Fallback behavior

Every action carries a fallback tier:

| Tier             | Behavior                                              | Example                  |
|------------------|--------------------------------------------------------|---------------------------|
| `silent`         | Keycode fallback, no notice                            | `save`                    |
| `notify_attempt` | Keycode fallback + a brief notice that it was guessed   | `duplicate.line`          |
| `notify_only`    | No fallback, just an "unsupported" notice               | `go_to.definition`        |

Actions that don't belong to any particular app (`shutdown`, `move.left`,
…) instead go through the **OS-wide system listener** — its own target,
identified by `os = "linux"`/`os = "windows"` in `config/targets.toml`,
which the router tries whenever the focused app doesn't support the
action, before falling back to the keycode path.

## Spotlight search

`keylex --spotlight` opens a fuzzy-searchable list of every known action
in the terminal (Enter dispatches the selected action exactly like a real
keystroke) — cross-platform, since both the fuzzy matching
(`nucleo-matcher`) and the terminal UI (`crossterm`) are pure Rust
libraries with no OS-specific code. The "valid options" never come from a
static list — they're discovered live, via handshake, from each connected
target (e.g. the VS Code extension, which brings its own
`keylex.spotlight` command for this) — see
[docs/protocol.md](docs/protocol.md#action-catalog-handshake-list_actions)
and [CLAUDE.md](CLAUDE.md) for details, including optional zoxide-style
"last used" tracking and the (untested in this dev environment) GNOME
Shell search-provider integration under `extensions/linux-extension/`.

## Status

Early prototype. The Rust dispatch pipeline (registry, router, capture)
is in place and tested on Linux. The Windows capture backend
(`src/capture/windows.rs`) is a careful port: it compiles
(`cargo check --target x86_64-pc-windows-msvc`), but is untested outside
a real Windows machine. First target-architecture building block: the VS
Code adapter (the official extension API, clearly documented commands).
Architecture details: [CLAUDE.md](CLAUDE.md); adapter wire format:
[docs/protocol.md](docs/protocol.md); Rust coding conventions:
[docs/rust-coding-guidelines.md](docs/rust-coding-guidelines.md).

## Privacy & Security

Keylex sits deep in the keyboard input path by design, so it's worth being
explicit about what that does and doesn't mean for your data.

**What is never logged, stored, or transmitted:** raw keystrokes,
reconstructed text, or window titles. The capture backends
(`src/capture/linux.rs`, `src/capture/windows.rs`) and focus resolution
(`src/focus/`) only ever produce abstract action IDs (e.g. `close.tab`) and
the focused process's executable name (e.g. `Code.exe`) — never the
content you typed or the title of the window you typed it into.

**What is logged:** abstract action IDs and dispatch results (e.g.
`close.tab -> Native`), printed to stdout only — never to a file, and
never off-device. There is no telemetry, crash reporting, or analytics
anywhere in the codebase.

**Network activity:** the only network code is two local, loopback-bound
(`127.0.0.1`) IPC channels used to reach the VS Code and Chrome
integrations (see [docs/protocol.md](docs/protocol.md)). Neither is
authenticated right now — the shared-secret token both used to require has
been deliberately dropped for now, since this is currently a single-user
local tool; the WebSocket transport still supports an Origin allowlist as
defense-in-depth. See
[docs/protocol.md#trust-model--authentication](docs/protocol.md#trust-model--authentication)
for the full current threat model and the keypair-based scheme planned to
replace the token. Nothing Keylex does ever reaches a server outside your
machine.

**GDPR framing:** since all processing happens locally, on your own
device, for your own configured use, Keylex isn't acting as a data
controller or processor on anyone's behalf — there's no third party's data
involved, and nothing leaves the device. This is a statement about the
*current* codebase, not a permanent guarantee: it holds only as long as
there's no telemetry, crash-reporting, or cloud-sync feature added (see
[CLAUDE.md](CLAUDE.md)'s "Known gaps" section, which flags this
explicitly as a standing constraint on future changes).

## Setup

```bash
cargo build

cargo run -- --demo   # smoke test: two example dispatches, no hardware needed
cargo run              # real, blocking keyboard interception
cargo run -- --spotlight   # interactive fuzzy search over all actions
```

For an end-to-end test without the real VS Code extension:

```bash
node scripts/fake-vscode-listener.js   # simulates the extension's socket side
cargo run                               # in a second terminal
```

## Testing the VS Code adapter

`./run <vscode-command-id>` (from the repo root) is a minimal test client,
independent of the Rust registry: it sends exactly that one command over
the `keylex/v0` socket to the extension — e.g.
`./run workbench.action.closeActiveEditor`. See
[docs/protocol.md](docs/protocol.md) for the wire format.

For `./run` to reach anything at all,
`extensions/vscode-extension/extension.js` first needs to be running
somewhere. Two ways, in increasing order of permanence:

1. **Extension Development Host (for testing, a throwaway window)** —
   open `extensions/vscode-extension/` as a folder in VS Code, then launch
   it from the "Run and Debug" panel ("Run Keylex Extension"). This opens
   a second window titled `[Extension Development Host]` in which the
   extension is active — only in that one window, not in your normal VS
   Code windows.
2. **Installed permanently (runs in every normal window)** — the
   extension isn't packaged as a real `.vsix` yet; the fastest option for
   local development is a symlink into
   `~/.vscode/extensions/keylex-vscode-adapter` (pointing at
   `extensions/vscode-extension/` in this repo), then restart VS Code.
   From then on, **every** VS Code window runs a socket server on port
   7777 in the background — not just per-project.

`extension.js` currently has **no** command allowlist (deliberately
removed for local testing) and the socket is currently
**unauthenticated** (the earlier shared-secret token was deliberately
removed, see
[docs/protocol.md](docs/protocol.md#trust-model--authentication)) — any
local process that can reach the connection can run any registered
command, not just ones declared anywhere. This should be locked back down
before any use outside your own machine (see the comment at the top of
the file).
