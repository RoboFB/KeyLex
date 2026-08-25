"""Plattformunabhängige Repräsentation eines Input-Events.

Listener (windows.py, linux.py) übersetzen Rohsignale ihrer Plattform in
InputEvent und melden es nur, wenn registry.binding_for() bereits ein
Binding gefunden hat -- action_id kommt daher direkt vom Listener und
muss vom Aufrufer nicht erneut nachgeschlagen werden.
"""
from __future__ import annotations

from dataclasses import dataclass, field
from typing import Callable

Phase = str  # "down" | "up"


@dataclass(frozen=True)
class InputEvent:
    device_id: str
    action_id: str
    phase: Phase
    key: str | None = None
    modifiers: frozenset[str] = field(default_factory=frozenset)
    button: str | int | None = None


InputHandler = Callable[[InputEvent], None]
