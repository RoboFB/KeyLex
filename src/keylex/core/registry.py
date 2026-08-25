"""Lädt Actions-, Device- und Target-Configs und stellt Lookups bereit."""
from __future__ import annotations

import logging
import tomllib
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable

log = logging.getLogger("keylex.registry")

DEFAULT_BINDING_MODE = "observe"  # grab | observe


@dataclass
class ActionSpec:
    id: str
    fallback_tier: str = "notify_attempt"  # silent | notify_attempt | notify_only
    fallback_keycode: str | None = None


@dataclass
class Target:
    program: str
    match_process: list[str]
    adapter: str
    supports: dict[str, str] = field(default_factory=dict)
    extra: dict = field(default_factory=dict)


class GrammarError(ValueError):
    """Eine Action-ID passt zu keinem deklarierten Verb+Objekt-Paar."""


class Registry:
    def __init__(self, config_dir: Path):
        self.config_dir = config_dir
        self.verbs: dict[str, set[str]] = {}
        self.actions: dict[str, ActionSpec] = {}
        self.targets: list[Target] = []
        self.system_actions: dict[str, dict] = {}
        self.device_bindings: list[dict] = []

    def load(self) -> None:
        self._load_actions()
        self._load_targets()
        self._load_devices()
        self._validate_action_grammar()

    def _load_actions(self) -> None:
        data = tomllib.loads((self.config_dir / "actions.toml").read_text())
        for entry in data.get("verb", []):
            self.verbs[entry["name"]] = set(entry.get("objects", []))
        for entry in data.get("action", []):
            spec = ActionSpec(
                id=entry["id"],
                fallback_tier=entry.get("fallback_tier", "notify_attempt"),
                fallback_keycode=entry.get("fallback_keycode"),
            )
            self.actions[spec.id] = spec

    def _load_targets(self) -> None:
        data = tomllib.loads((self.config_dir / "targets.toml").read_text())
        for entry in data.get("target", []):
            self.targets.append(
                Target(
                    program=entry["program"],
                    match_process=entry.get("match_process", []),
                    adapter=entry["adapter"],
                    supports=entry.get("supports", {}),
                    extra={k: v for k, v in entry.items()
                           if k not in ("program", "match_process", "adapter", "supports")},
                )
            )
        for entry in data.get("system_action", []):
            self.system_actions[entry["id"]] = entry

    def _load_devices(self) -> None:
        data = tomllib.loads((self.config_dir / "devices.toml").read_text())
        for device in data.get("device", []):
            device_default_mode = device.get("default_mode", DEFAULT_BINDING_MODE)
            for binding in device.get("binding", []):
                self.device_bindings.append({
                    **binding,
                    "device_id": device["id"],
                    "mode": binding.get("mode", device_default_mode),
                })

    def _validate_action_grammar(self) -> None:
        """Prüft, dass jede referenzierte Action-ID entweder ein deklariertes
        System-Action ist, ein explizit registriertes einzelnes Wort (z.B.
        "save"), oder einem deklarierten Verb+Objekt-Paar aus actions.toml
        entspricht. Unbekannte Verb+Objekt-Kombinationen sind ein Config-Fehler
        und brechen den Start ab; ein nicht deklariertes einzelnes Wort wird
        nur geloggt, da es (wie "save") absichtlich ohne Objekt existieren kann.
        """
        known_bare = {action_id for action_id in self.actions if "." not in action_id}

        referenced: set[str] = set(self.actions)
        for target in self.targets:
            referenced.update(target.supports)
        for binding in self.device_bindings:
            event = binding.get("event")
            if event:
                referenced.add(event)

        for action_id in sorted(referenced):
            if action_id in self.system_actions or action_id.startswith("system."):
                continue
            if "." not in action_id:
                if action_id not in known_bare:
                    log.warning(
                        "Action %r hat kein Verb.Objekt-Format und ist nicht "
                        "in actions.toml deklariert", action_id,
                    )
                continue
            verb, _, obj = action_id.partition(".")
            if verb not in self.verbs:
                raise GrammarError(f"Unbekanntes Verb {verb!r} in Aktion {action_id!r}")
            if obj not in self.verbs[verb]:
                raise GrammarError(
                    f"Unbekanntes Objekt {obj!r} für Verb {verb!r} in Aktion {action_id!r}"
                )

    def action_spec(self, action_id: str) -> ActionSpec:
        return self.actions.get(action_id, ActionSpec(id=action_id))

    def target_for_process(self, process_name: str) -> Target | None:
        for target in self.targets:
            if process_name in target.match_process:
                return target
        return None

    def binding_for(
        self,
        device_id: str,
        *,
        key: str | None = None,
        button: str | int | None = None,
        modifiers: Iterable[str] = (),
    ) -> dict | None:
        """Findet das Binding, das zu einem InputEvent passt.

        Für Tastatur-Events wird nach (key, modifiers) gesucht, für
        Button-Geräte (Stream Deck, Space Mouse) nach `button`.
        """
        mod_set = frozenset(modifiers)
        for binding in self.device_bindings:
            if binding["device_id"] != device_id:
                continue
            if key is not None:
                if binding.get("key") != key:
                    continue
                if frozenset(binding.get("modifiers", [])) != mod_set:
                    continue
                return binding
            if button is not None and binding.get("button") == button:
                return binding
        return None
