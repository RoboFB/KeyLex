# Keylex

Keylex generalisiert Tasteneingaben in **abstrakte Aktionen**
("close tab", "go_to definition", "save") und dispatcht diese Aktionen
je nach fokussierter Anwendung über deren **native API** — anstatt
einfach Keycodes zu simulieren. Nur wenn eine App keine eigene
Schnittstelle bietet, greift ein Keycode-Fallback.

Damit soll eine physische Taste auf jedem System und in jedem Programm
dasselbe Sinnvolle tun, egal ob VS Code, Chrome, Neovim oder ein
Terminal gerade fokussiert ist. Der Kern (Keyboard-Interception,
Fokus-Erkennung, Dispatch) ist in Rust geschrieben, um so tief wie
möglich in das OS eingreifen zu können; die App-seitigen Adapter (z. B.
die VS-Code-Extension) leben jeweils in der Sprache, die für diese App
am sinnvollsten ist — aktuell JavaScript.

## Architektur

```
Physische Tastatur                Rust-Daemon                    Ziele
                          (config/actions.toml Mapping)      (config/targets.toml)
──────────────────      ──────────────────────────────      ──────────────
Jeder Tastendruck  ─→  Capture (evdev/uinput | WH_KEYBOARD_LL)
                              │
                              ├─ kein Treffer → unverändert durchgereicht
                              │
                              └─ Treffer → interne Aktion       ┌─→  VS-Code-Adapter (Extension/Socket)
                                 z.B. "close.tab"          ─────┼─→  Chrome-Adapter (Native Messaging, geplant)
                                                                 ├─→  Neovim (RPC, geplant)
                                                                 ├─→  System-Aktion (fester OS-Keycode)
                                                                 └─→  Keycode-Fallback (generisch)
```

Drei Config-Ebenen:
1. **Wortliste** (`config/vocabulary.toml`) — die einzigen erlaubten
   Bausteine einer Aktions-ID: Verben (`close`, `save`, …) und Objekte
   (`tab`, `sidebar`, …).
2. **Aktionen + Trigger** (`config/actions.toml`) — geräteunabhängige
   Aktionen (`close.tab`, `save`, `go_to.definition`, …), jeweils mit
   optionalem physischem Tastentrigger und Fallback-Verhalten.
3. **Output** (`config/targets.toml`) — pro Zielprogramm, wie es
   erreicht wird und wo seine Capability-Liste (`capabilities.toml` in
   `extensions/<name>/`) steht.

Jeder Tastendruck, der zu einem konfigurierten Trigger passt, wird immer
abgefangen (nie ans OS/die App durchgereicht) und dispatcht; alles
andere bleibt unverändert nutzbar — das ist die konkrete Umsetzung von
"jeden Keycode so tief wie möglich abfangen".

## Aktionskategorien & Fallback-Verhalten

Jede Aktion trägt eine Fallback-Stufe:

| Stufe            | Verhalten                                             | Beispiel                 |
|------------------|--------------------------------------------------------|---------------------------|
| `silent`         | Keycode-Fallback ohne Hinweis                          | `save`                    |
| `notify_attempt` | Keycode-Fallback + kurzer Hinweis, dass geraten wurde   | `duplicate.line`          |
| `notify_only`    | Kein Fallback, nur "nicht unterstützt"-Meldung          | `go_to.definition`        |

Aktionen, die zu gar keiner App gehören (`shutdown`, `move.left`, …),
laufen stattdessen über den **OS-weiten Listener** — ein eigenes Target
mit `os = "linux"`/`os = "windows"` in `config/targets.toml`, das der
Router immer dann versucht, wenn die fokussierte App die Aktion nicht
unterstützt, und bevor er zum Keycode-Fallback greift.

## Spotlight-Suche

`keylex --spotlight` öffnet eine fuzzy-durchsuchbare Liste aller
konfigurierten Aktionen im Terminal (Enter dispatcht die gewählte Aktion
genau wie ein echter Tastendruck) — plattformunabhängig, da sowohl das
Fuzzy-Matching (`nucleo-matcher`) als auch die Terminal-UI (`crossterm`)
reine Rust-Bibliotheken ohne OS-spezifischen Code sind. Die "gültigen
Optionen" kommen dabei nie aus einer statischen Liste, sondern per
Handshake live vom jeweiligen Ziel (z. B. der VS-Code-Extension, die dafür
ihren eigenen `keylex.spotlight`-Befehl mitbringt) — siehe
[docs/protocol.md](docs/protocol.md#action-catalog-handshake-list_actions)
und [CLAUDE.md](CLAUDE.md) für Details, inklusive optionalem
Zoxide-artigem "zuletzt verwendet"-Tracking und der (in diesem
Entwicklungsumfeld ungetesteten) GNOME-Shell-Suchanbieter-Integration unter
`extensions/linux-extension/`.

## Status

Früher Prototyp. Die Rust-Dispatch-Pipeline (Registry, Router, Capture)
steht und ist auf Linux getestet. Das Windows-Capture-Backend
(`src/capture/windows.rs`) ist ein sorgfältiger Port: es kompiliert
(`cargo check --target x86_64-pc-windows-msvc`), ist aber außerhalb einer
echten Windows-Maschine ungetestet. Erster Zielarchitektur-
Baustein: VS-Code-Adapter (offizielle Extension-API, klar dokumentierte
Commands). Details zur Architektur: [CLAUDE.md](CLAUDE.md), zum
Adapter-Wire-Format: [docs/protocol.md](docs/protocol.md), zu
Rust-Code-Konventionen: [docs/rust-coding-guidelines.md](docs/rust-coding-guidelines.md).

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

cargo run -- --demo   # Smoke-Test: zwei Beispiel-Dispatches, keine Hardware nötig
cargo run              # echte, blockierende Keyboard-Interception
cargo run -- --spotlight   # interaktive Fuzzy-Suche über alle Aktionen
```

Für einen End-to-End-Test ohne die echte VS-Code-Extension:

```bash
node scripts/fake-vscode-listener.js   # simuliert die Extension-Socket-Seite
cargo run                               # in einem zweiten Terminal
```

## VS-Code-Adapter testen

`./run <vscode-command-id>` (im Repo-Root) ist ein minimaler, von der
Rust-Registry unabhängiger Test-Client: er schickt genau diesen einen
Befehl über den `keylex/v0`-Socket an die Extension — z. B.
`./run workbench.action.closeActiveEditor`. Siehe
[docs/protocol.md](docs/protocol.md) für das Wire-Format.

Damit `./run` überhaupt etwas erreicht, muss
`extensions/vscode-extension/extension.js` erst irgendwo laufen. Zwei
Wege, in aufsteigender Dauerhaftigkeit:

1. **Extension Development Host (zum Testen, Wegwerf-Fenster)** —
   `extensions/vscode-extension/` als Ordner in VS Code öffnen, dann im
   "Run and Debug"-Panel ("Run Keylex Extension") starten. Öffnet ein
   zweites Fenster mit Titel `[Extension Development Host]`, in dem die
   Extension aktiv ist — nur in diesem einen Fenster, nicht in den
   normalen VS-Code-Fenstern.
2. **Dauerhaft installiert (läuft in jedem normalen Fenster)** — die
   Extension ist noch nicht als echtes `.vsix` gepackt; am schnellsten
   für lokale Entwicklung ist ein Symlink nach
   `~/.vscode/extensions/keylex-vscode-adapter` (Ziel:
   `extensions/vscode-extension/` in diesem Repo), danach VS Code neu
   starten. Damit läuft ab dann in **jedem** VS-Code-Fenster ein
   Socket-Server auf Port 7777 im Hintergrund — nicht nur projektbezogen.

`extension.js` hat aktuell **keine** Befehls-Allowlist (bewusst entfernt
für lokales Testen) und der Socket ist derzeit **unauthentifiziert** (das
frühere Shared-Secret-Token wurde bewusst entfernt, siehe
[docs/protocol.md](docs/protocol.md#trust-model--authentication)) — jeder
lokale Prozess, der die Verbindung erreicht, kann jeden registrierten
Befehl ausführen, nicht nur die in `capabilities.toml` deklarierten. Vor
jedem Einsatz außerhalb der eigenen Maschine sollte das wieder
eingeschränkt werden (siehe Kommentar am Dateianfang).
