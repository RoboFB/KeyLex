"""Bestimmt den Prozessnamen des aktuell fokussierten Fensters.

Wird bei jedem Dispatch gebraucht, um gegen targets.toml zu matchen. Ein
leerer/unbekannter Rückgabewert ist kein Fehlerfall -- Router.dispatch
fällt dann automatisch auf den Keycode-Fallback zurück.
"""
from __future__ import annotations

import ctypes
import logging
import sys

log = logging.getLogger("keylex.input.active_window")

_warned_no_xdotool = False

if sys.platform == "win32":
    from ctypes import wintypes

    _user32 = ctypes.windll.user32
    _kernel32 = ctypes.windll.kernel32
    _PROCESS_QUERY_LIMITED_INFORMATION = 0x1000

    # Ohne explizite restype/argtypes nimmt ctypes für jeden Rückgabewert
    # standardmäßig ein 32-bit c_int an -- auf 64-bit-Windows werden Handles
    # dadurch stillschweigend abgeschnitten (derselbe Bug, der windows.py's
    # SetWindowsHookExW mit ERROR_MOD_NOT_FOUND fehlschlagen ließ).
    _user32.GetForegroundWindow.restype = ctypes.c_void_p
    _user32.GetWindowThreadProcessId.argtypes = [ctypes.c_void_p, ctypes.POINTER(wintypes.DWORD)]
    _kernel32.OpenProcess.restype = ctypes.c_void_p
    _kernel32.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
    _kernel32.QueryFullProcessImageNameW.argtypes = [
        ctypes.c_void_p, wintypes.DWORD, wintypes.LPWSTR, ctypes.POINTER(wintypes.DWORD),
    ]
    _kernel32.CloseHandle.argtypes = [ctypes.c_void_p]


def focused_process_name() -> str:
    if sys.platform == "win32":
        return _focused_process_windows()
    return _focused_process_linux()


def _focused_process_windows() -> str:
    from ctypes import wintypes

    hwnd = _user32.GetForegroundWindow()
    if not hwnd:
        return ""
    pid = wintypes.DWORD()
    _user32.GetWindowThreadProcessId(hwnd, ctypes.byref(pid))

    handle = _kernel32.OpenProcess(_PROCESS_QUERY_LIMITED_INFORMATION, False, pid.value)
    if not handle:
        return ""
    try:
        buf = ctypes.create_unicode_buffer(260)
        size = wintypes.DWORD(len(buf))
        if not _kernel32.QueryFullProcessImageNameW(handle, 0, buf, ctypes.byref(size)):
            return ""
        return buf.value.rsplit("\\", 1)[-1]
    finally:
        _kernel32.CloseHandle(handle)


def _focused_process_linux() -> str:
    import shutil
    import subprocess

    global _warned_no_xdotool
    if shutil.which("xdotool") is None:
        if not _warned_no_xdotool:
            log.warning(
                "xdotool nicht gefunden -- fokussierter Prozess kann unter X11 "
                "nicht ermittelt werden, Aktionen laufen über den Keycode-"
                "Fallback. (Wayland wird von diesem Prototyp noch nicht "
                "unterstützt -- offener Punkt, siehe Plan.)"
            )
            _warned_no_xdotool = True
        return ""
    try:
        pid = subprocess.run(
            ["xdotool", "getactivewindow", "getwindowpid"],
            capture_output=True, text=True, timeout=0.5, check=True,
        ).stdout.strip()
        return subprocess.run(
            ["ps", "-p", pid, "-o", "comm="],
            capture_output=True, text=True, timeout=0.5, check=True,
        ).stdout.strip()
    except (subprocess.SubprocessError, OSError):
        return ""
