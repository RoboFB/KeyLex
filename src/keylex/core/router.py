"""Entscheidet, wie eine abstrakte Aktion für das aktuell fokussierte
Programm umgesetzt wird: nativer Adapter -> Keycode-Fallback -> Notify.
"""
from __future__ import annotations

import logging
from dataclasses import dataclass

from keylex.core.registry import Registry, Target

log = logging.getLogger("keylex.router")


@dataclass
class DispatchResult:
    status: str  # "native" | "fallback" | "unsupported"
    detail: str = ""


class Router:
    def __init__(self, registry: Registry, adapters: dict, notifier, fallback_sender):
        self.registry = registry
        self.adapters = adapters          # {"socket": SocketAdapter(), ...}
        self.notifier = notifier          # zeigt kurze Popups an
        self.fallback_sender = fallback_sender  # simuliert Keycodes

    def dispatch(self, action_id: str, focused_process: str) -> DispatchResult:
        target = self.registry.target_for_process(focused_process)
        spec = self.registry.action_spec(action_id)

        if target and action_id in target.supports:
            return self._dispatch_native(target, action_id)

        return self._dispatch_fallback(action_id, spec)

    def _dispatch_native(self, target: Target, action_id: str) -> DispatchResult:
        native_command = target.supports[action_id]
        adapter = self.adapters.get(target.adapter)
        if adapter is None:
            log.warning("Kein Adapter für %s registriert", target.adapter)
            return DispatchResult("unsupported", f"adapter {target.adapter} missing")
        adapter.send(target, native_command)
        return DispatchResult("native", native_command)

    def _dispatch_fallback(self, action_id: str, spec) -> DispatchResult:
        if spec.fallback_tier == "notify_only" or not spec.fallback_keycode:
            self.notifier.show(f"Aktion nicht unterstützt: {action_id}")
            return DispatchResult("unsupported", action_id)

        self.fallback_sender.send(spec.fallback_keycode)

        if spec.fallback_tier == "notify_attempt":
            self.notifier.show(f"Fallback versucht: {action_id}")

        return DispatchResult("fallback", spec.fallback_keycode)
