"""Stand-in for the (not yet built) VS Code extension's socket server.

There is no real VS Code-side listener yet, so `SocketAdapter` always times
out and every action falls through to the keycode fallback. Run this next
to `python -m keylex.daemon` to see a "native" dispatch actually arrive
somewhere, per the wire format in docs/protocol.md -- without needing the
real extension.
"""
from __future__ import annotations

import json
import socketserver

HOST, PORT = "127.0.0.1", 7777  # must match targets.toml's vscode target


class Handler(socketserver.StreamRequestHandler):
    def handle(self) -> None:
        for line in self.rfile:
            try:
                message = json.loads(line)
            except json.JSONDecodeError:
                print("bad message:", line)
                continue
            print("received command:", message.get("command"))


def main() -> None:
    with socketserver.TCPServer((HOST, PORT), Handler) as server:
        print(f"fake VS Code listener on {HOST}:{PORT} (Ctrl+C to stop)")
        server.serve_forever()


if __name__ == "__main__":
    main()
