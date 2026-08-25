"""Keylex-Daemon: lädt Config, verbindet Router mit Adaptern und dem
plattformspezifischen Input-Listener.

`python -m keylex.daemon` startet den echten, blockierenden Input-Listener
(Windows: WH_KEYBOARD_LL-Hook, Linux: evdev/uinput). `--demo` überspringt
den Listener und dispatcht stattdessen zwei Beispiel-Aktionen einmalig --
nützlich für einen schnellen Smoke-Test ohne echte Hardware/Rechte.
"""
from __future__ import annotations

import argparse
import logging
import sys
from pathlib import Path

from keylex.adapters.vscode import SocketAdapter
from keylex.core.registry import Registry
from keylex.core.router import Router
from keylex.core.system import FallbackSender, Notifier
from keylex.input.active_window import focused_process_name
from keylex.input.base import InputListener
from keylex.input.events import InputEvent

logging.basicConfig(level=logging.INFO, format="%(name)s: %(message)s")
log = logging.getLogger("keylex.daemon")

MAIN_KEYBOARD_DEVICE_ID = "main_keyboard"  # einziges Keyboard-Device im aktuellen Prototyp


def _make_fallback_sender():
    if sys.platform == "win32":
        from keylex.input.windows import WindowsFallbackSender

        return WindowsFallbackSender()
    return FallbackSender()  # Linux: uinput-Injection noch nicht implementiert, nur Logging


def build_router() -> Router:
    config_dir = Path(__file__).parent / "config"
    registry = Registry(config_dir)
    registry.load()

    adapters = {
        "socket": SocketAdapter(),
        # "native_messaging": ChromeAdapter(),  # folgt
        # "rpc": NeovimAdapter(),                # folgt
    }
    return Router(registry, adapters, Notifier(), _make_fallback_sender())


def make_input_handler(router: Router):
    def handle(event: InputEvent) -> None:
        if event.phase != "down":
            return
        result = router.dispatch(event.action_id, focused_process=focused_process_name())
        log.info("%s -> %s", event.action_id, result)

    return handle


def start_input_listener(router: Router) -> InputListener:
    on_event = make_input_handler(router)

    if sys.platform == "win32":
        from keylex.input.windows import WindowsKeyboardListener

        listener: InputListener = WindowsKeyboardListener(
            router.registry, MAIN_KEYBOARD_DEVICE_ID, on_event,
        )
    else:
        from keylex.input.linux import LinuxKeyboardListener, discover_keyboard_path

        listener = LinuxKeyboardListener(
            router.registry, MAIN_KEYBOARD_DEVICE_ID, discover_keyboard_path(), on_event,
        )

    listener.start()  # blockiert, bis der Prozess beendet wird
    return listener


def run_demo(router: Router) -> None:
    result = router.dispatch("close.tab", focused_process="code")
    print(result)

    result = router.dispatch("go_to.definition", focused_process="chrome.exe")
    print(result)


def main() -> None:
    parser = argparse.ArgumentParser(description="Keylex router daemon")
    parser.add_argument(
        "--demo", action="store_true",
        help="Zwei Beispiel-Dispatches statt des echten Input-Listeners (kein Listener-Setup nötig)",
    )
    args = parser.parse_args()

    router = build_router()

    if args.demo:
        run_demo(router)
        return

    start_input_listener(router)


if __name__ == "__main__":
    main()
