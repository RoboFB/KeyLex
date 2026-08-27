# Pivot draft — a fuzzy-find command palette

Status: **exploratory, not decided.** This is a "maybe pivot" note, not a
spec — written down so the idea survives past one conversation and can be
argued with later. Nothing here is implemented and nothing here should be
treated as a commitment to build it. See [../CLAUDE.md](../CLAUDE.md) for
the architecture this would sit on top of, and
[protocol.md](protocol.md#native-command-strings) for the `supports`
whitelist format this idea leans on directly.

## The idea

Simplify the problem by *not* requiring every action to be reachable from
a physical key first. Instead: one hotkey opens a fuzzy-find bar
(Spotlight/`rofi`/`dmenu`-shaped), the user types a few characters, and it
ranks matches across **every native command every connected extension has
declared it supports** — not just the subset that happens to have a `key`
or `chord` bound in `actions.toml` today. Ranking uses frecency (frequency
+ recency decay), zoxide-style, instead of plain fuzzy edit-distance, so
the bar gets faster to use the more it's used and doesn't require the
"mnemonic-collision-free modifier" design goal that `hotkeys-reference.csv`
already conceded it can't fully enforce anymore (see `src/config.rs`'s
`Vocabulary` doc comment).

Framed against the existing pitch: Keylex today resolves *bound keys* to
native commands. This would add a second entry point that resolves
*typed text* to the same native commands, through the same dispatch path.

## Why this is a small step, not a rewrite

The data this needs mostly already exists:

- Every `extensions/<name>/capabilities.toml` already declares its
  `supports` map (action id → native command string) — see
  [protocol.md](protocol.md#native-command-strings). That map is the
  palette's candidate list, aggregated across every target in
  `targets.toml`, no new declaration format needed.
- Dispatch is already decoupled from capture: `Router::dispatch(action_id,
  focused_process)` in `src/dispatch.rs` doesn't care whether `action_id`
  came from a matched key/chord or was typed into a search box. The
  palette would just be a second caller of the same function.
- `actions.toml` entries with no `key` (e.g. `go_to.definition`, per
  CLAUDE.md's "Two config layers" section) are *already* dispatchable,
  just not reachable — this closes exactly that gap instead of opening a
  new one.

What's actually new:

- **A UI surface.** The daemon today is headless (capture/dispatch loop
  only). Neither `src/capture/linux.rs` nor `src/capture/windows.rs`
  render anything. This needs some kind of overlay: a minimal native
  popup, or shelling out to something already built for this (`rofi`,
  `wofi`, `fzf` in a terminal, a custom PowerToys-Run-style box on
  Windows). Cheapest first cut is almost certainly piping candidates into
  an existing launcher binary rather than writing a GUI from scratch.
- **A frecency store.** Needs persisted per-command usage stats (count +
  last-used timestamp, zoxide's actual algorithm) somewhere under the
  config dir, plus the scoring function itself.
- **A hotkey to summon it.** Presumably just another `actions.toml` entry
  with a `key`, dispatched to an internal "open palette" pseudo-action
  rather than to an adapter — the first action that *isn't* aimed at an
  external target.
- **Aggregation across targets.** Something has to walk every
  `[[target]]` in `targets.toml`, load each `capabilities.toml`, and
  merge their `supports` maps into one flat searchable list, including
  the `system_action` entries. Doesn't exist today — `Registry` currently
  only ever looks up a *single* target's supports map at dispatch time,
  keyed by focused process; it never needs the full union.

## Open questions (unresolved on purpose)

- **Scope of the union.** All native commands from all targets regardless
  of focus, or scoped to what's relevant to the currently focused app
  first (with a way to broaden)? Spotlight-everything is more powerful;
  scoped-to-focus is closer to what the fallback-tier/native-command
  design already optimizes for and avoids showing 200 VS Code commands
  while sitting in Chrome.
- **Where the popup renders.** Linux: X11-only again, same as focus
  detection (Wayland gap already noted in CLAUDE.md) — or an approach
  that sidesteps needing a window at all. Windows: no prior art in this
  codebase to build on.
- **Relationship to `fallback_tier`.** Does the palette bypass fallback
  entirely (native command or nothing), or fall back to keycode injection
  the same way key-triggered dispatch does?
- **Does this replace or sit beside key/chord bindings?** Nothing about
  this proposal requires removing `key`/`chord` — it's additive. Worth
  being explicit about that so it doesn't get read as a rewrite of the
  binding system.
- **Frecency store format and location.** Likely `<config-dir>/frecency.*`
  next to `secret.token`, but unspecified.

## Non-goals for a first cut

- No natural-language matching, no LLM involvement — plain fuzzy string
  match + frecency ranking, same class of algorithm zoxide/fzf use.
- No new adapter/transport work — reuses `adapters::SocketAdapter` /
  `adapters::WebSocketAdapter` exactly as-is via `Router::dispatch`.
- No attempt to solve Wayland or ship a real Windows implementation as
  part of landing this — inherits the same platform gaps the rest of the
  project already carries (see CLAUDE.md's "Known gaps" section).
