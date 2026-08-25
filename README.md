# Keylex

Keylex generalisiert Tasten-/Geräteeingaben in **abstrakte Aktionen**
("close tab", "go_to definition", "save") und dispatcht diese Aktionen
je nach fokussierter Anwendung über deren **native API** — anstatt
einfach Keycodes zu simulieren. Nur wenn eine App keine eigene
Schnittstelle bietet, greift ein Keycode-Fallback.

Damit soll eine physische Taste ("mein Custom-Keyboard-Knopf") auf
jedem System und in jedem Programm dasselbe Sinnvolle tun, egal ob
VS Code, Chrome, Neovim oder ein Terminal gerade fokussiert ist.

## Architektur

```
Input-Geräte          Router-Daemon                 Ziele
(devices.toml)     (actions.toml Mapping)      (targets.toml)
────────────      ──────────────────────      ──────────────
Tastatur      ─┐                          ┌─→  VS-Code-Adapter (Extension/Socket)
Space Mouse   ─┼─→  interne Aktion       ─┼─→  Chrome-Adapter (Native Messaging)
Stream Deck   ─┘    z.B. "close.tab"      ├─→  Neovim (RPC)
                                          ├─→  System-Aktion (fester OS-Keycode)
                                          └─→  Keycode-Fallback (generisch)
```

Drei getrennte Config-Ebenen:
1. **Input** (`config/devices.toml`) — welche Geräte werden abgehört,
   wie werden ihre Rohsignale in interne Events übersetzt.
2. **Mapping** (`config/actions.toml`) — geräteunabhängiges Vokabular
   aus Verben (`close`, `open`, `duplicate`, `go_to`, …) und Objekten
   (`tab`, `window`, `line`, `definition`, …).
3. **Output** (`config/targets.toml`) — pro Zielprogramm, welche
   Verb+Objekt-Kombinationen es unterstützt (Whitelist) und wie sie
   dorthin dispatcht werden.

## Aktionskategorien & Fallback-Verhalten

Jede Aktion trägt eine Fallback-Stufe:

| Stufe            | Verhalten                                             | Beispiel                 |
|------------------|--------------------------------------------------------|---------------------------|
| `silent`         | Keycode-Fallback ohne Hinweis                          | `save`                    |
| `notify_attempt` | Keycode-Fallback + kurzer Hinweis, dass geraten wurde   | `duplicate.line`          |
| `notify_only`    | Kein Fallback, nur "nicht unterstützt"-Meldung          | `go_to.definition`        |

Zusätzlich gibt es **System-Aktionen** (`config/system_actions.toml`):
global gültige, OS-spezifische Kombinationen (z. B. Grafiktreiber-Reset
unter Windows), die nicht von der fokussierten App abhängen.

## Status

Früher Prototyp. Config-getriebene Dispatch-Pipeline (Registry, Router,
Verb+Objekt-Grammatik-Validierung) steht und ist getestet. Echte Input-
Listener für Windows (WH_KEYBOARD_LL) und Linux (evdev/uinput) existieren,
sind aber außerhalb der jeweiligen Zielplattform ungetestet. Erster
Zielarchitektur-Baustein: VS-Code-Adapter (offizielle Extension-API, klar
dokumentierte Commands). Details zur Architektur: [CLAUDE.md](CLAUDE.md),
zum Adapter-Wire-Format: [docs/protocol.md](docs/protocol.md).

## Setup

```bash
python -m venv .venv
source .venv/bin/activate   # Windows: .venv\Scripts\activate
pip install -e ".[dev]"     # + ".[linux,dev]" auf Linux, für den evdev-Listener

python -m keylex.daemon --demo   # Smoke-Test: zwei Beispiel-Dispatches, keine Hardware nötig
python -m keylex.daemon          # echter, blockierender Input-Listener
```
