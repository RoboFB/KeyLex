"""Gemeinsame Schnittstelle für plattformspezifische Input-Listener."""
from __future__ import annotations

from abc import ABC, abstractmethod

from keylex.input.events import InputHandler


class InputListener(ABC):
    """Lauscht auf Rohsignale eines Geräts und meldet gematchte InputEvents.

    Bindings im "grab"-Modus werden vom Listener selbst unterdrückt (das
    Keycode-Signal erreicht OS/App nicht); "observe"-Bindings lösen den
    Handler aus, ohne das normale OS-Verhalten zu beeinflussen. Welcher
    Modus gilt, steht pro Binding in devices.toml (Registry.binding_for).
    """

    def __init__(self, on_event: InputHandler) -> None:
        self.on_event = on_event

    @abstractmethod
    def start(self) -> None:
        """Blockiert und lauscht, bis stop() aus einem anderen Thread/Signal-
        Handler aufgerufen wird."""

    @abstractmethod
    def stop(self) -> None:
        ...
