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

Zwei Config-Ebenen:
1. **Vokabular + Trigger** (`config/actions.toml`) — geräteunabhängige
   Aktionen (`close.tab`, `save`, `go_to.definition`, …), jeweils mit
   optionalem physischem Tastentrigger und Fallback-Verhalten.
2. **Output** (`config/targets.toml`) — pro Zielprogramm, welche
   Aktionen es unterstützt (Whitelist) und wie sie dorthin dispatcht
   werden.

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

Zusätzlich gibt es **System-Aktionen** (`[[system_action]]` in
`config/targets.toml`): global gültige, OS-spezifische Kombinationen
(z. B. Grafiktreiber-Reset unter Windows), die nicht von der
fokussierten App abhängen.

## Status

Früher Prototyp. Die Rust-Dispatch-Pipeline (Registry, Router, Capture)
steht und ist auf Linux getestet. Der Windows-Capture-Backend
(`src/capture/windows.rs`) ist ein sorgfältiger Port, aber außerhalb
einer echten Windows-Maschine ungetestet. Erster Zielarchitektur-
Baustein: VS-Code-Adapter (offizielle Extension-API, klar dokumentierte
Commands). Details zur Architektur: [CLAUDE.md](CLAUDE.md), zum
Adapter-Wire-Format: [docs/protocol.md](docs/protocol.md).

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
integrations (see [docs/protocol.md](docs/protocol.md)). Both require a
per-install shared secret on every message, and the WebSocket transport
additionally supports an Origin allowlist — see
[docs/protocol.md#trust-model--authentication](docs/protocol.md#trust-model--authentication)
for the full threat model. Nothing Keylex does ever reaches a server
outside your machine.

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
```

Für einen End-to-End-Test ohne die echte VS-Code-Extension:

```bash
node scripts/fake-vscode-listener.js   # simuliert die Extension-Socket-Seite
cargo run                               # in einem zweiten Terminal
```

## VS-Code-Adapter testen

`./run <vscode-command-id>` (im Repo-Root) ist ein minimaler, von der
Rust-Registry unabhängiger Test-Client: er liest `config/secret.token` und
schickt genau diesen einen Befehl über den `keylex/v0`-Socket an die
Extension — z. B. `./run workbench.action.closeActiveEditor`. Siehe
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
für lokales Testen) — jeder Befehl mit gültigem Token wird ausgeführt,
nicht nur die in `capabilities.toml` deklarierten. Vor jedem Einsatz
außerhalb der eigenen Maschine sollte das wieder eingeschränkt werden
(siehe Kommentar am Dateianfang).
