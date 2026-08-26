# Keylex Chrome adapter

Reference client for the `keylex/v0` WebSocket transport (see
[../docs/protocol.md](../docs/protocol.md)). Not published to the Chrome Web
Store — load it as an unpacked extension for now.

## Install

1. Open `chrome://extensions/`.
2. Enable **Developer mode** (top right).
3. Click **Load unpacked** and select this `chrome-extension/` folder.
4. Open the extension's **Details → Extension options** (or click its icon
   on `chrome://extensions/`), paste in the contents of the daemon's
   `config/secret.token` (generated on first run -- see
   [../docs/protocol.md](../docs/protocol.md#trust-model--authentication)),
   and click **Save**. The daemon won't trust this connection at all until
   this token matches.
5. (Optional, recommended) Note the extension's ID from `chrome://extensions/`
   and set it as `allowed_origin = "chrome-extension://<id>"` under the
   `chrome` target in `config/targets.toml`, so the daemon also rejects any
   WebSocket connection whose `Origin` isn't this exact extension.
   **Caveat:** an unpacked extension's ID is derived from its install path
   and changes on every reload unless you pin a `"key"` in `manifest.json`.
   If you want a stable ID, generate your **own** RSA keypair locally
   (`openssl genrsa 2048 | openssl rsa -pubout -outform DER | openssl base64
   -A`, then put the resulting base64 string in `manifest.json`'s `"key"`
   field) -- don't reuse a key from anywhere public: since Chrome doesn't
   verify a signature for unpacked/dev extensions, a `"key"` value that's
   public knowledge lets anyone derive the same extension ID, which defeats
   the Origin check's purpose. Keep your generated `manifest.json` change
   local/untracked rather than committing it.

## Try it without the daemon

Run `node ../scripts/fake-chrome-listener.js` to start a fake Keylex daemon
that logs commands it receives, or run the real daemon (`cargo run` from the
repo root) with `config/targets.toml`'s `chrome` target enabled.

The extension connects to `ws://127.0.0.1:7778` on load (must match the
`chrome` target's `port` in `config/targets.toml`) and reconnects
automatically if the daemon restarts or the extension is reloaded.
