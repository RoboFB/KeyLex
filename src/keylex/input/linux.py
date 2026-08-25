"""Linux-Input-Listener: evdev-Device exklusiv grabben, alles außer
"grab"-Bindings über ein virtuelles uinput-Device wieder ausgeben.

Folgt dem interception-tools/evremap-Muster: ein rohes evdev-Grab blendet
das *ganze* physische Gerät aus, daher muss der Listener jedes nicht
unterdrückte Event selbst re-emittieren, damit die Tastatur für alle
anderen Tasten normal nutzbar bleibt. Kein neuer Kernel-Code nötig --
evdev/uinput sind bereits Teil des Kernels.

Braucht das optionale "linux"-Extra (`pip install keylex[linux]`, python-
evdev) -- der einzige neue Laufzeit-Dependency dieses Schritts.
"""
from __future__ import annotations

import logging

from keylex.core.registry import Registry
from keylex.input.base import InputListener
from keylex.input.events import InputEvent, InputHandler

log = logging.getLogger("keylex.input.linux")

_MODIFIER_NAMES = {
    "KEY_LEFTCTRL": "ctrl", "KEY_RIGHTCTRL": "ctrl",
    "KEY_LEFTSHIFT": "shift", "KEY_RIGHTSHIFT": "shift",
    "KEY_LEFTALT": "alt", "KEY_RIGHTALT": "alt",
    "KEY_LEFTMETA": "win", "KEY_RIGHTMETA": "win",
}


def discover_keyboard_path() -> str:
    """Bester Versuch, die main_keyboard-Quelle für source = "auto" zu
    finden: erstes Gerät mit klassischen Buchstaben-Keycodes.
    """
    from evdev import InputDevice, ecodes, list_devices

    for path in list_devices():
        device = InputDevice(path)
        caps = device.capabilities().get(ecodes.EV_KEY, [])
        if ecodes.KEY_A in caps and ecodes.KEY_SPACE in caps:
            return path
    raise RuntimeError("Kein Keyboard-Device über evdev gefunden (devices.toml: source=\"auto\")")


def _key_name(ecodes, code: int) -> str | None:
    name = ecodes.KEY.get(code)
    if isinstance(name, list):
        name = name[0]
    if not name or not name.startswith("KEY_"):
        return None
    stripped = name[len("KEY_"):].lower()
    return stripped if len(stripped) == 1 else None  # Prototyp: nur a-z/0-9


class LinuxKeyboardListener(InputListener):
    def __init__(
        self, registry: Registry, device_id: str, device_path: str, on_event: InputHandler,
    ) -> None:
        super().__init__(on_event)
        self.registry = registry
        self.device_id = device_id
        self.device_path = device_path
        self._running = False
        self._pressed_modifiers: set[str] = set()

    def start(self) -> None:
        from evdev import InputDevice, UInput, ecodes

        self._ecodes = ecodes
        source = InputDevice(self.device_path)
        source.grab()
        virtual = UInput.from_device(source, name=f"keylex-{self.device_id}")
        self._running = True
        log.info("Linux-Keyboard-Listener aktiv für %s (%s)", self.device_id, self.device_path)

        try:
            for raw in source.read_loop():
                if not self._running:
                    break
                if raw.type != ecodes.EV_KEY:
                    virtual.write_event(raw)
                    virtual.syn()
                    continue
                self._handle(raw, virtual)
        finally:
            source.ungrab()
            virtual.close()

    def _handle(self, raw, virtual) -> None:
        ecodes = self._ecodes
        name = ecodes.KEY.get(raw.code)
        if isinstance(name, list):
            name = name[0]

        modifier = _MODIFIER_NAMES.get(name)
        if modifier is not None:
            if raw.value == 1:
                self._pressed_modifiers.add(modifier)
            elif raw.value == 0:
                self._pressed_modifiers.discard(modifier)
            virtual.write_event(raw)
            virtual.syn()
            return

        key = _key_name(ecodes, raw.code)
        is_repeat = raw.value == 2
        phase = {1: "down", 0: "up"}.get(raw.value)  # Autorepeat (2) löst nicht neu aus

        binding = None
        if key is not None:
            binding = self.registry.binding_for(
                self.device_id, key=key, modifiers=frozenset(self._pressed_modifiers),
            )

        if binding is not None and phase == "down" and not is_repeat:
            self.on_event(InputEvent(
                device_id=self.device_id,
                action_id=binding["event"],
                phase=phase,
                key=key,
                modifiers=frozenset(self._pressed_modifiers),
            ))

        if binding is not None and binding.get("mode") == "grab":
            return  # nicht re-emittieren -> Taste bleibt unterdrückt (down und up/repeat)

        virtual.write_event(raw)
        virtual.syn()

    def stop(self) -> None:
        self._running = False
