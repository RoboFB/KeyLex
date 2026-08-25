"""Windows-Input-Listener: WH_KEYBOARD_LL-Hook über ctypes.

Bewusst ohne pywin32, um keine zusätzliche Abhängigkeit einzuführen --
die WinAPI-Aufrufe, die wir brauchen (SetWindowsHookExW, GetMessage-Loop),
sind über ctypes.windll vollständig erreichbar.

Nur Buchstaben/Ziffern werden aufgelöst (siehe _VK_TO_KEY); das deckt die
aktuellen devices.toml-Bindings ab. Vollständige Layout-Unterstützung
(MapVirtualKey, Sondertasten) ist ein späterer Ausbauschritt.
"""
from __future__ import annotations

import ctypes
import logging
from ctypes import wintypes

from keylex.core.registry import Registry
from keylex.input.base import InputListener
from keylex.input.events import InputEvent, InputHandler

log = logging.getLogger("keylex.input.windows")

WH_KEYBOARD_LL = 13
WM_KEYDOWN = 0x0100
WM_KEYUP = 0x0101
WM_SYSKEYDOWN = 0x0104
WM_SYSKEYUP = 0x0105

VK_SHIFT = (0xA0, 0xA1)   # L/R SHIFT
VK_CONTROL = (0xA2, 0xA3)  # L/R CONTROL
VK_MENU = (0xA4, 0xA5)     # L/R ALT
VK_WIN = (0x5B, 0x5C)      # L/R WIN

_VK_TO_KEY = {ord(c): c.lower() for c in "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"}
_VK_TO_KEY[0x2C] = "prtsc"  # VK_SNAPSHOT

user32 = ctypes.windll.user32
kernel32 = ctypes.windll.kernel32

_LRESULT = ctypes.c_long if ctypes.sizeof(ctypes.c_void_p) == 4 else ctypes.c_longlong
_HOOKPROC = ctypes.WINFUNCTYPE(_LRESULT, ctypes.c_int, wintypes.WPARAM, wintypes.LPARAM)

# ctypes defaults every undeclared return type to a 32-bit c_int, which
# silently truncates the 64-bit handles/pointers these functions actually
# return on 64-bit Windows -- must be declared explicitly or SetWindowsHookExW
# gets a garbage hMod and fails with ERROR_MOD_NOT_FOUND (126).
kernel32.GetModuleHandleW.restype = ctypes.c_void_p
kernel32.GetModuleHandleW.argtypes = [wintypes.LPCWSTR]

user32.SetWindowsHookExW.restype = ctypes.c_void_p
user32.SetWindowsHookExW.argtypes = [ctypes.c_int, _HOOKPROC, ctypes.c_void_p, wintypes.DWORD]

user32.CallNextHookEx.restype = _LRESULT
user32.CallNextHookEx.argtypes = [ctypes.c_void_p, ctypes.c_int, wintypes.WPARAM, wintypes.LPARAM]

user32.UnhookWindowsHookEx.restype = wintypes.BOOL
user32.UnhookWindowsHookEx.argtypes = [ctypes.c_void_p]

user32.GetMessageW.restype = ctypes.c_int
user32.GetMessageW.argtypes = [ctypes.POINTER(wintypes.MSG), ctypes.c_void_p, wintypes.UINT, wintypes.UINT]

user32.TranslateMessage.argtypes = [ctypes.POINTER(wintypes.MSG)]
user32.DispatchMessageW.argtypes = [ctypes.POINTER(wintypes.MSG)]
user32.PostQuitMessage.argtypes = [ctypes.c_int]

user32.GetAsyncKeyState.restype = ctypes.c_short
user32.GetAsyncKeyState.argtypes = [ctypes.c_int]

INPUT_KEYBOARD = 1
KEYEVENTF_KEYUP = 0x0002

_NAMED_VK = {
    "win": 0x5B, "lwin": 0x5B, "rwin": 0x5C,
    "ctrl": 0x11, "control": 0x11,
    "shift": 0x10,
    "alt": 0x12, "menu": 0x12,
    "prtsc": 0x2C, "printscreen": 0x2C,
}


class _KEYBDINPUT(ctypes.Structure):
    _fields_ = [
        ("wVk", wintypes.WORD),
        ("wScan", wintypes.WORD),
        ("dwFlags", wintypes.DWORD),
        ("time", wintypes.DWORD),
        ("dwExtraInfo", ctypes.POINTER(wintypes.ULONG)),
    ]


class _MOUSEINPUT(ctypes.Structure):
    _fields_ = [
        ("dx", wintypes.LONG),
        ("dy", wintypes.LONG),
        ("mouseData", wintypes.DWORD),
        ("dwFlags", wintypes.DWORD),
        ("time", wintypes.DWORD),
        ("dwExtraInfo", ctypes.POINTER(wintypes.ULONG)),
    ]


class _HARDWAREINPUT(ctypes.Structure):
    _fields_ = [
        ("uMsg", wintypes.DWORD),
        ("wParamL", wintypes.WORD),
        ("wParamH", wintypes.WORD),
    ]


class _INPUTUNION(ctypes.Union):
    # Must mirror the real Windows INPUT union (mi/ki/hi), not just "ki" --
    # SendInput rejects the call if cbSize doesn't match its actual struct
    # size, and a union with only one member is undersized vs. the real one.
    _fields_ = [("mi", _MOUSEINPUT), ("ki", _KEYBDINPUT), ("hi", _HARDWAREINPUT)]


class _INPUT(ctypes.Structure):
    _fields_ = [("type", wintypes.DWORD), ("union", _INPUTUNION)]


user32.SendInput.restype = wintypes.UINT
user32.SendInput.argtypes = [wintypes.UINT, ctypes.POINTER(_INPUT), ctypes.c_int]


def _vk_for_token(token: str) -> int:
    token = token.strip().lower()
    if token in _NAMED_VK:
        return _NAMED_VK[token]
    if len(token) == 1 and token.isalnum():
        return ord(token.upper())  # VK codes for 'A'-'Z'/'0'-'9' == ASCII
    raise ValueError(f"Unbekanntes Token im Keycode: {token!r}")


def _key_input(vk: int, *, up: bool) -> _INPUT:
    flags = KEYEVENTF_KEYUP if up else 0
    ki = _KEYBDINPUT(wVk=vk, wScan=0, dwFlags=flags, time=0, dwExtraInfo=None)
    return _INPUT(type=INPUT_KEYBOARD, union=_INPUTUNION(ki=ki))


def send_keycode(keycode: str) -> None:
    """Simuliert einen Keycode wie "win+d" systemweit via SendInput --
    Modifier zuerst gedrückt halten, dann die Haupttaste, dann alles in
    umgekehrter Reihenfolge wieder loslassen.
    """
    vks = [_vk_for_token(token) for token in keycode.split("+")]

    down = (_INPUT * len(vks))(*(_key_input(vk, up=False) for vk in vks))
    sent_down = user32.SendInput(len(down), down, ctypes.sizeof(_INPUT))

    up = (_INPUT * len(vks))(*(_key_input(vk, up=True) for vk in reversed(vks)))
    sent_up = user32.SendInput(len(up), up, ctypes.sizeof(_INPUT))

    if sent_down != len(down) or sent_up != len(up):
        raise OSError(
            f"SendInput: nur {sent_down}/{len(down)} down, {sent_up}/{len(up)} up "
            f"verarbeitet (GetLastError={ctypes.GetLastError()})"
        )


class WindowsFallbackSender:
    """Ersetzt die reine Logging-Variante aus core/system.py: sendet den
    Fallback-Keycode tatsächlich systemweit, statt ihn nur zu loggen."""

    def send(self, keycode: str) -> None:
        try:
            send_keycode(keycode)
            log.info("[fallback keycode] %s gesendet", keycode)
        except (ValueError, OSError) as exc:
            log.warning("Fallback-Keycode %r konnte nicht gesendet werden: %s", keycode, exc)


class _KBDLLHOOKSTRUCT(ctypes.Structure):
    _fields_ = [
        ("vkCode", wintypes.DWORD),
        ("scanCode", wintypes.DWORD),
        ("flags", wintypes.DWORD),
        ("time", wintypes.DWORD),
        ("dwExtraInfo", ctypes.POINTER(wintypes.ULONG)),
    ]


def _pressed_modifiers() -> frozenset[str]:
    def down(vk: int) -> bool:
        return bool(user32.GetAsyncKeyState(vk) & 0x8000)

    mods = set()
    if any(down(vk) for vk in VK_CONTROL):
        mods.add("ctrl")
    if any(down(vk) for vk in VK_SHIFT):
        mods.add("shift")
    if any(down(vk) for vk in VK_MENU):
        mods.add("alt")
    if any(down(vk) for vk in VK_WIN):
        mods.add("win")
    return frozenset(mods)


class WindowsKeyboardListener(InputListener):
    """Hook für ein einzelnes logisches Keyboard-Device (device_id aus
    devices.toml, z.B. "main_keyboard"). Windows unterscheidet physische
    Tastaturen an der WH_KEYBOARD_LL-Schnittstelle nicht voneinander, daher
    bildet eine Instanz aktuell "das Keyboard" als Ganzes ab.
    """

    def __init__(self, registry: Registry, device_id: str, on_event: InputHandler) -> None:
        super().__init__(on_event)
        self.registry = registry
        self.device_id = device_id
        self._hook_handle = None
        self._hook_proc = _HOOKPROC(self._hook)  # Referenz halten, sonst GC-Absturz

    def _hook(self, code: int, wparam: int, lparam: int) -> int:
        if code != 0:  # HC_ACTION
            return user32.CallNextHookEx(None, code, wparam, lparam)

        info = ctypes.cast(lparam, ctypes.POINTER(_KBDLLHOOKSTRUCT)).contents
        key = _VK_TO_KEY.get(info.vkCode)
        phase = "down" if wparam in (WM_KEYDOWN, WM_SYSKEYDOWN) else "up"

        binding = None
        if key is not None:
            modifiers = _pressed_modifiers()
            binding = self.registry.binding_for(self.device_id, key=key, modifiers=modifiers)
            if binding is not None and phase == "down":
                self.on_event(InputEvent(
                    device_id=self.device_id,
                    action_id=binding["event"],
                    phase=phase,
                    key=key,
                    modifiers=modifiers,
                ))

        if binding is not None and binding.get("mode") == "grab":
            return 1  # Taste unterdrücken (down und up), OS/App sieht sie nicht

        return user32.CallNextHookEx(None, code, wparam, lparam)

    def start(self) -> None:
        self._hook_handle = user32.SetWindowsHookExW(
            WH_KEYBOARD_LL, self._hook_proc, kernel32.GetModuleHandleW(None), 0,
        )
        if not self._hook_handle:
            raise OSError(f"SetWindowsHookExW fehlgeschlagen: {ctypes.GetLastError()}")
        log.info("Windows-Keyboard-Hook aktiv für Device %r", self.device_id)

        msg = wintypes.MSG()
        while user32.GetMessageW(ctypes.byref(msg), None, 0, 0) > 0:
            user32.TranslateMessage(ctypes.byref(msg))
            user32.DispatchMessageW(ctypes.byref(msg))

    def stop(self) -> None:
        if self._hook_handle:
            user32.UnhookWindowsHookEx(self._hook_handle)
            self._hook_handle = None
        user32.PostQuitMessage(0)
