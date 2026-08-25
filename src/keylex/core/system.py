"""Platzhalter für plattformspezifische Bausteine:
- Notifier: kleines Popup (ähnlich Caps-Lock-Anzeige unter Windows)
- FallbackSender: simuliert einen Keycode systemweit

Für den Prototyp nur Logging; echte Implementierung folgt pro OS.
"""
from __future__ import annotations

import logging

log = logging.getLogger("keylex.system")


class Notifier:
    def show(self, message: str) -> None:
        log.info("[notify] %s", message)


class FallbackSender:
    def send(self, keycode: str) -> None:
        log.info("[fallback keycode] %s", keycode)
