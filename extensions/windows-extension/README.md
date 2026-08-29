# Keylex Windows system listener

Reference client for the `keylex/v0` socket transport (see
[../../docs/protocol.md](../../docs/protocol.md)), backing the
`system-windows` target in [../../config/targets.toml](../../config/targets.toml).
Windows counterpart of `../linux-extension`: handles the same OS-wide actions
(`shutdown`, `show.desktop`, `move.left`,
`move.right`) regardless of which app has focus, by shelling out to
PowerShell -- Win32 `SetWindowPos`/`ShowWindow` (via inline C#) for the window
moves, `Shell.Application.ToggleDesktop()` for show-desktop, and
`shutdown /s /t 0`.

**Untested outside a real Windows machine** -- this repo's dev environment is
Linux-only, the same caveat [../../src/capture/windows.rs](../../src/capture/windows.rs)
and [../../src/focus/windows.rs](../../src/focus/windows.rs) already carry.

## Requirements

- Node.js for Windows
- PowerShell on `PATH` (default on any supported Windows version)

## Run

```powershell
node listener.js
```

Run this alongside the daemon on the same Windows machine. It listens on
`127.0.0.1:7780`, matching the `system-windows` target's `address` in
`config/targets.toml`. **No authentication yet** -- see
[../../docs/protocol.md](../../docs/protocol.md#trust-model--authentication)
and [../../CLAUDE.md](../../CLAUDE.md)'s "Known gaps": any local process
that can reach that port can trigger `shutdown` and the window commands
below.

## Command mapping

| `command` (wire)       | Action          | Implementation                                                      |
|--------------------------|-----------------|-----------------------------------------------------------------------|
| `os.system.shutdown`     | `shutdown`      | `shutdown /s /t 0`                                                    |
| `os.desktop.show`        | `show.desktop`  | `(New-Object -ComObject Shell.Application).ToggleDesktop()`           |
| `os.window.move_left`    | `move.left`     | restore + `SetWindowPos` foreground window to left half of screen     |
| `os.window.move_right`   | `move.right`    | restore + `SetWindowPos` foreground window to right half of screen    |
