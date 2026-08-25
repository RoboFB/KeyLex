"""VS-Code-Adapter: spricht mit der Keylex-Extension über einen lokalen
Socket. Die Extension ruft intern vscode.commands.executeCommand(...) auf.

Erwartetes Extension-seitiges Protokoll (JSON, ein Objekt pro Zeile):
    {"command": "workbench.action.closeActiveEditor"}
"""
from __future__ import annotations

import json
import logging
import socket

log = logging.getLogger("keylex.adapter.vscode")


class SocketAdapter:
    def send(self, target, native_command: str) -> None:
        host, port = target.extra["address"].split(":")
        try:
            with socket.create_connection((host, int(port)), timeout=0.5) as sock:
                payload = json.dumps({"command": native_command}) + "\n"
                sock.sendall(payload.encode("utf-8"))
        except OSError as exc:
            log.warning("VS-Code-Adapter nicht erreichbar: %s", exc)
