//! Windows capture: a `WH_KEYBOARD_LL` hook. Never compiled or run on an
//! actual Windows machine -- this dev environment is Linux-only -- so treat
//! it as a careful port, not a tested backend.
//!
//! Three constraints shape this file:
//!
//! - The hook callback must be a plain `extern "system" fn`, not a closure,
//!   so the router and chord state are reached through a thread-local for
//!   the duration of `run()`. Everything here is single-threaded: the hook
//!   and the message loop driving it share the thread that called `run()`.
//! - A low-level hook must return immediately, so a chord's debounce window
//!   can't be a sleep. Unlike Linux, no second thread is needed either:
//!   `run()` already pumps a message loop, and a `SetTimer` timer posts
//!   into that same queue for `DispatchMessageW` to deliver.
//! - A suppressed key can never be let through after the fact, so replaying
//!   a broken or timed-out chord means synthesizing fresh keystrokes with
//!   `SendInput` rather than re-emitting the originals the way the Linux
//!   uinput device can.

use std::cell::{Cell, RefCell};
use std::io;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{HMODULE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
    VIRTUAL_KEY, VK_CONTROL, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_MENU, VK_RCONTROL,
    VK_RMENU, VK_RSHIFT, VK_RWIN, VK_SHIFT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, KillTimer, SetTimer, SetWindowsHookExW,
    TranslateMessage, UnhookWindowsHookEx, HC_ACTION, HHOOK, KBDLLHOOKSTRUCT, LLKHF_INJECTED, MSG,
    WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

use super::chord::{Chords, Keyboard, Phase};
use crate::config::{is_modifier, KeyCombo, Registry};
use crate::dispatch::{Adapters, FallbackSender, Notifier, Router};
use crate::focus;

/// Matches `src/capture/linux.rs`'s debounce window; not yet configurable.
const DEBOUNCE_MS: u32 = 35;

const VK_SNAPSHOT: u32 = 0x2C;

/// The token a virtual-key code names, if Keylex knows one. Modifiers
/// resolve from their *specific* left/right code, so a modifier can travel
/// through the chord state machine like any other key when it is part of a
/// configured chord.
fn token(vk: u32) -> Option<String> {
    let modifier = |a: VIRTUAL_KEY, b: VIRTUAL_KEY| vk == a.0 as u32 || vk == b.0 as u32;
    let name = match vk {
        VK_SNAPSHOT => "prtsc",
        _ if modifier(VK_LCONTROL, VK_RCONTROL) => "ctrl",
        _ if modifier(VK_LSHIFT, VK_RSHIFT) => "shift",
        _ if modifier(VK_LMENU, VK_RMENU) => "alt",
        _ if modifier(VK_LWIN, VK_RWIN) => "win",
        // VK codes for letters and digits are their ASCII uppercase.
        0x30..=0x39 | 0x41..=0x5A => {
            return Some((vk as u8 as char).to_ascii_lowercase().to_string())
        }
        _ => return None,
    };
    Some(name.to_string())
}

/// Shared with `hotkey::windows`, which needs the same token->VK mapping
/// for `RegisterHotKey` that this hook uses for chord replay and fallback
/// keycodes.
pub(crate) fn vk_for_token(token: &str) -> Option<VIRTUAL_KEY> {
    match token {
        "ctrl" => Some(VK_CONTROL),
        "shift" => Some(VK_SHIFT),
        "alt" => Some(VK_MENU),
        "win" => Some(VK_LWIN),
        "prtsc" => Some(VIRTUAL_KEY(VK_SNAPSHOT as u16)),
        _ => {
            let mut chars = token.chars();
            let c = chars
                .next()
                .filter(|c| c.is_ascii_alphanumeric() && chars.next().is_none())?;
            Some(VIRTUAL_KEY(c.to_ascii_uppercase() as u16))
        }
    }
}

/// Live physical modifier state, for the single-combo trigger path. Unlike
/// Linux, which tracks its own set, this asks the OS -- so it stays right
/// even for a modifier whose own event Keylex suppressed. The flip side is
/// that a modifier mid-chord-decision still reads as held; fixing that
/// would mean replacing this query with tracked state, a much larger change
/// to a backend nothing here can test.
fn pressed_modifiers() -> std::collections::BTreeSet<String> {
    // SAFETY: GetAsyncKeyState only reads global keyboard state for a
    // virtual-key code and touches no memory of ours.
    let down = |vk: VIRTUAL_KEY| unsafe { GetAsyncKeyState(vk.0 as i32) as u16 & 0x8000 != 0 };
    [
        (VK_CONTROL, "ctrl"),
        (VK_SHIFT, "shift"),
        (VK_MENU, "alt"),
        (VK_LWIN, "win"),
    ]
    .into_iter()
    .filter(|(vk, _)| down(*vk))
    .map(|(_, name)| name.to_string())
    .collect()
}

fn key_input(vk: VIRTUAL_KEY, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: if up {
                    KEYEVENTF_KEYUP
                } else {
                    Default::default()
                },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

/// Injects keystrokes. They re-enter `hook_proc` flagged `LLKHF_INJECTED`,
/// which is exactly why that function passes injected events straight
/// through instead of matching on them again.
fn send_inputs(inputs: &[INPUT]) {
    // SAFETY: `inputs` is a live, fully initialized slice, and the size
    // argument is the size of the very type it holds, as SendInput requires.
    let sent = unsafe { SendInput(inputs, std::mem::size_of::<INPUT>() as i32) };
    if sent as usize != inputs.len() {
        eprintln!(
            "keylex: SendInput only processed {sent}/{} events",
            inputs.len()
        );
    }
}

/// Replays chord keys and runs the debounce timer. Holds the live timer's
/// id together with the generation it was armed for, so `timer_proc` can
/// tell its own window from one that was already resolved.
#[derive(Default)]
struct Emitter {
    timer: Option<(usize, u64)>,
}

impl Emitter {
    fn synthesize(&self, token: &str, up: bool) {
        match vk_for_token(token) {
            Some(vk) => send_inputs(&[key_input(vk, up)]),
            None => eprintln!("keylex: chord replay: unknown key token {token:?}"),
        }
    }

    /// The generation a fired timer was armed for, if it is still the live
    /// one; `None` for a stale timer whose chord was already resolved.
    fn fired(&mut self, id: usize) -> Option<u64> {
        match self.timer {
            Some((live, generation)) if live == id => {
                self.timer = None;
                Some(generation)
            }
            _ => None,
        }
    }
}

impl Keyboard for Emitter {
    /// Nothing: a suppressed event can't be re-emitted here, so replays are
    /// synthesized from the token alone.
    type Event = ();

    fn press(&mut self, token: &str, (): ()) -> io::Result<()> {
        self.synthesize(token, false);
        Ok(())
    }

    fn release(&mut self, token: &str, (): ()) -> io::Result<()> {
        self.synthesize(token, true);
        Ok(())
    }

    fn arm_timer(&mut self, generation: u64) {
        // A null hwnd means Win32 ignores the id we pass and mints its own,
        // returning it -- that is the id `timer_proc` will report back.
        // SAFETY: `timer_proc` is a real TIMERPROC, and this thread pumps
        // the message loop that delivers the callback.
        let id = unsafe { SetTimer(None, 0, DEBOUNCE_MS, Some(timer_proc)) };
        if id == 0 {
            eprintln!("keylex: chord debounce: SetTimer failed");
            return;
        }
        self.timer = Some((id, generation));
    }
}

struct Fallback;

impl FallbackSender for Fallback {
    fn send(&self, combo: &KeyCombo) {
        let mut keys = Vec::with_capacity(combo.modifiers.len() + 1);
        for token in combo.modifiers.iter().chain(std::iter::once(&combo.key)) {
            match vk_for_token(token) {
                Some(vk) => keys.push(vk),
                None => {
                    eprintln!("keylex: fallback keycode {combo}: unknown key {token:?}");
                    return;
                }
            }
        }

        let down: Vec<INPUT> = keys.iter().map(|vk| key_input(*vk, false)).collect();
        let up: Vec<INPUT> = keys.iter().rev().map(|vk| key_input(*vk, true)).collect();
        send_inputs(&down);
        send_inputs(&up);
    }
}

struct HookState<'a> {
    router: Router<'a>,
    chords: RefCell<Chords<()>>,
    emitter: RefCell<Emitter>,
}

thread_local! {
    /// Type-erased on purpose: `HookState` borrows the registry, but a
    /// thread-local must be `'static`. `run()` is the only writer, and the
    /// pointer is non-null only while the `HookState` it points at is alive
    /// on this same thread.
    static HOOK_STATE: Cell<*const ()> = const { Cell::new(std::ptr::null()) };
}

/// Runs `f` against the state `run()` installed, or reports `None` when no
/// hook is active (a callback arriving before setup or after teardown).
fn with_state<R>(f: impl FnOnce(&HookState) -> R) -> Option<R> {
    let state = HOOK_STATE.get();
    if state.is_null() {
        return None;
    }
    // SAFETY: only `run()` ever writes this cell, on this same thread: it
    // stores a pointer to a `HookState` local that outlives the message
    // loop, and nulls it again before that local is dropped. So a non-null
    // pointer here always refers to a live, immutably-borrowable value.
    Some(f(unsafe { &*state.cast::<HookState>() }))
}

fn dispatch(action_id: &str, router: &Router) {
    let focused = focus::focused_process_name();
    println!(
        "{action_id} -> {}",
        router.dispatch(action_id, focused.as_deref())
    );
}

/// Whether the key that produced this event still reaches the OS.
enum Handled {
    Suppress,
    PassOn,
}

fn on_key(state: &HookState, token: &str, phase: Phase) -> Handled {
    let registry = state.router.registry();

    if registry.is_chord_member(token) {
        let matched = {
            let mut chords = state.chords.borrow_mut();
            let mut emitter = state.emitter.borrow_mut();
            // A genuine fresh press can't arrive twice without a release in
            // between, so a down for an already-tracked key is autorepeat.
            let phase = match phase {
                Phase::Down if chords.is_tracked(token) => Phase::Repeat,
                phase => phase,
            };
            chords.on_key(token, phase, (), registry, &mut *emitter)
        };
        match matched {
            Ok(Some(action_id)) => dispatch(&action_id, &state.router),
            Ok(None) => {}
            Err(e) => eprintln!("keylex: chord handling failed: {e}"),
        }
        return Handled::Suppress; // chord members are always suppressed at the source
    }

    if is_modifier(token) {
        // Not part of any chord, so it passes through untouched; the combo
        // path below reads modifier state live from the OS anyway.
        return Handled::PassOn;
    }

    let combo = KeyCombo {
        key: token.to_string(),
        modifiers: pressed_modifiers(),
    };
    let Some(action_id) = registry.action_for_trigger(&combo).map(str::to_string) else {
        return Handled::PassOn;
    };
    if phase == Phase::Down {
        dispatch(&action_id, &state.router);
    }
    Handled::Suppress
}

unsafe extern "system" fn timer_proc(_hwnd: HWND, _msg: u32, id: usize, _time: u32) {
    // A `SetTimer` timer with a null hwnd repeats by default, so kill this
    // id whatever happens next: a chord debounce is meant to fire once.
    // SAFETY: `id` is the timer Win32 just reported as fired; killing an
    // already-dead timer is defined (it fails) rather than unsound.
    unsafe {
        let _ = KillTimer(None, id);
    }

    with_state(|state| {
        let mut emitter = state.emitter.borrow_mut();
        let Some(generation) = emitter.fired(id) else {
            return; // a window already resolved some other way
        };
        if let Err(e) = state
            .chords
            .borrow_mut()
            .on_timeout(generation, &mut *emitter)
        {
            eprintln!("keylex: chord replay failed: {e}");
        }
    });
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // SAFETY: passing the hook chain the very arguments it handed us is
    // what the WH_KEYBOARD_LL contract asks of a hook that doesn't handle
    // an event.
    let pass_on = || unsafe { CallNextHookEx(None, code, wparam, lparam) };

    if code != HC_ACTION as i32 {
        return pass_on();
    }
    // SAFETY: for HC_ACTION, Win32 documents lparam as a pointer to a
    // KBDLLHOOKSTRUCT that stays valid for this call.
    let event = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };

    // Never re-process our own injected events (chord replay, fallback
    // keycodes): matching on one would corrupt chord state or loop forever.
    if event.flags.0 & LLKHF_INJECTED.0 != 0 {
        return pass_on();
    }

    let phase = match wparam.0 as u32 {
        WM_KEYDOWN | WM_SYSKEYDOWN => Phase::Down,
        WM_KEYUP | WM_SYSKEYUP => Phase::Up,
        _ => return pass_on(),
    };
    let Some(token) = token(event.vkCode) else {
        return pass_on();
    };

    match with_state(|state| on_key(state, &token, phase)) {
        Some(Handled::Suppress) => LRESULT(1),
        Some(Handled::PassOn) | None => pass_on(),
    }
}

pub fn run(registry: &Registry, adapters: Adapters, notifier: Box<dyn Notifier>) -> io::Result<()> {
    let state = HookState {
        router: Router::new(registry, adapters, notifier, Box::new(Fallback)),
        chords: RefCell::new(Chords::default()),
        emitter: RefCell::new(Emitter::default()),
    };

    HOOK_STATE.set(std::ptr::from_ref(&state).cast());
    let result = pump_messages();
    HOOK_STATE.set(std::ptr::null());
    result
}

/// Installs the hook and runs the message loop that both delivers its
/// callbacks and drives the chord debounce timers, until the thread is
/// asked to quit.
fn pump_messages() -> io::Result<()> {
    // SAFETY: a null module name asks for the handle of the current process
    // image, which is what a global low-level hook needs.
    let module: HMODULE = unsafe { GetModuleHandleW(PCWSTR::null()) }.unwrap_or_default();
    // SAFETY: `hook_proc` is a real `extern "system"` hook procedure with
    // the signature WH_KEYBOARD_LL requires, and thread id 0 asks for the
    // global hook this needs to be.
    let hook: HHOOK = unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), module, 0) }
        .map_err(|e| io::Error::other(format!("SetWindowsHookExW failed: {e}")))?;

    println!("keylex: windows keyboard hook active");
    let mut message = MSG::default();
    // SAFETY: `message` is a valid, writable MSG for the whole loop, and
    // each call only reads the message the previous one just filled in.
    while unsafe { GetMessageW(&mut message, None, 0, 0) }.as_bool() {
        unsafe {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }

    // SAFETY: `hook` is the handle SetWindowsHookExW just returned and has
    // not been unhooked before now.
    unsafe {
        let _ = UnhookWindowsHookEx(hook);
    }
    Ok(())
}
