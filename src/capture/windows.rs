//! Windows capture: WH_KEYBOARD_LL hook, ported from the previous ctypes
//! implementation. Untestable on this (Linux) dev machine -- same caveat
//! the project already carried for the Python listener.
//!
//! WH_KEYBOARD_LL's callback must be a plain `extern "system" fn`, not a
//! closure, so the registry/router live in a thread-local for the
//! duration of `run()` (single-threaded: the hook and the message loop
//! that drives it run on the same thread that calls `run()`).
//!
//! Chords need a bounded debounce window (see `src/capture/linux.rs` for
//! the shared rationale), but a low-level hook callback must return
//! immediately -- it can't sleep or block waiting to see whether more keys
//! join a chord. Unlike Linux, no second thread is needed for the timer:
//! `run()` already pumps a `GetMessageW`/`DispatchMessageW` loop on the
//! hook's own thread, and a `SetTimer`/`TIMERPROC` timer posts into that
//! same queue, so `DispatchMessageW` invokes `timer_proc` directly without
//! needing a real window. Because a suppressed key's original event can
//! never be "let through" after the fact once the hook has returned,
//! replaying a timed-out or broken chord candidate means synthesizing a
//! fresh keystroke via `SendInput` rather than re-emitting the original
//! event the way `src/capture/linux.rs`'s virtual uinput device can.

use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};

use windows::core::PCWSTR;
use windows::Win32::Foundation::{HMODULE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, GetAsyncKeyState, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
    VIRTUAL_KEY, VK_LWIN, VK_MENU, VK_SHIFT, VK_CONTROL,
    VK_LCONTROL, VK_RCONTROL, VK_LSHIFT, VK_RSHIFT, VK_LMENU, VK_RMENU, VK_RWIN,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, KBDLLHOOKSTRUCT, LLKHF_INJECTED, MSG,
    PostQuitMessage, SetTimer, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, HHOOK,
    HC_ACTION, KillTimer, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

use crate::config::{KeyCombo, Registry};
use crate::dispatch::{Adapter, FallbackSender, Notifier, Router};

/// How long a chord stays "pending" after its most recent member key went
/// down before giving up and replaying it as normal keystrokes. Matches
/// `src/capture/linux.rs`'s `CHORD_DEBOUNCE_WINDOW`; not yet configurable.
const CHORD_DEBOUNCE_MS: u32 = 35;

fn vk_for_letter_digit(vk: u32) -> Option<char> {
    match vk {
        0x30..=0x39 => Some((b'0' + (vk - 0x30) as u8) as char), // '0'-'9'
        0x41..=0x5A => Some((b'a' + (vk - 0x41) as u8) as char), // 'A'-'Z' (VK codes == ASCII)
        _ => None,
    }
}

const VK_SNAPSHOT: u32 = 0x2C;

fn key_token(vk: u32) -> Option<String> {
    if vk == VK_SNAPSHOT {
        return Some("prtsc".to_string());
    }
    vk_for_letter_digit(vk).map(|c| c.to_string())
}

/// Which modifier name (if any) a *specific* left/right virtual-key code
/// names. Distinct from `pressed_modifiers()` below, which queries the
/// generic (side-independent) VK codes live via `GetAsyncKeyState` -- this
/// one is for recognizing an individual key-event's `vkCode` so a modifier
/// can be tracked through the chord state machine the same way a plain key
/// is, when (and only when) it's actually part of a configured chord.
fn modifier_token(vk: u32) -> Option<&'static str> {
    match vk {
        v if v == VK_LCONTROL.0 as u32 || v == VK_RCONTROL.0 as u32 => Some("ctrl"),
        v if v == VK_LSHIFT.0 as u32 || v == VK_RSHIFT.0 as u32 => Some("shift"),
        v if v == VK_LMENU.0 as u32 || v == VK_RMENU.0 as u32 => Some("alt"),
        v if v == VK_LWIN.0 as u32 || v == VK_RWIN.0 as u32 => Some("win"),
        _ => None,
    }
}

/// Live physical modifier state via `GetAsyncKeyState`, used by the
/// existing single-key(+modifier) trigger path. Unlike
/// `src/capture/linux.rs`'s self-tracked `pressed_modifiers` set, this
/// always reflects true hardware state regardless of whether Keylex has
/// suppressed that modifier's own key event -- there is no equivalent on
/// Windows to Linux's "don't fold a still-pending chord modifier into the
/// tracked set yet" behavior, since nothing here is tracked incrementally
/// in the first place. A modifier that's mid-chord-decision (pending, or
/// already consumed by a matched chord) will still read as "held" here.
/// Documented as a deliberate, unavoidable platform difference rather than
/// a bug: fixing it would mean replacing this OS query with Keylex's own
/// tracked state, a much larger change to an already-untestable backend.
fn pressed_modifiers() -> std::collections::BTreeSet<String> {
    fn down(vk: VIRTUAL_KEY) -> bool {
        unsafe { (GetAsyncKeyState(vk.0 as i32) as u16 & 0x8000) != 0 }
    }
    let mut mods = std::collections::BTreeSet::new();
    if down(VK_CONTROL) {
        mods.insert("ctrl".to_string());
    }
    if down(VK_SHIFT) {
        mods.insert("shift".to_string());
    }
    if down(VK_MENU) {
        mods.insert("alt".to_string());
    }
    if down(VK_LWIN) {
        mods.insert("win".to_string());
    }
    mods
}

fn vk_for_token(token: &str) -> Option<VIRTUAL_KEY> {
    match token {
        "ctrl" | "control" => Some(VK_CONTROL),
        "shift" => Some(VK_SHIFT),
        "alt" | "menu" => Some(VK_MENU),
        "win" | "lwin" => Some(VK_LWIN),
        "prtsc" | "printscreen" => Some(VIRTUAL_KEY(VK_SNAPSHOT as u16)),
        _ => {
            let mut chars = token.chars();
            let c = chars.next()?;
            if chars.next().is_some() {
                return None;
            }
            if c.is_ascii_alphanumeric() {
                Some(VIRTUAL_KEY(c.to_ascii_uppercase() as u16))
            } else {
                None
            }
        }
    }
}

fn key_input(vk: VIRTUAL_KEY, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: if up { KEYEVENTF_KEYUP } else { Default::default() },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn send_keycode(keycode: &str) -> windows::core::Result<()> {
    let combo = KeyCombo::parse(keycode);
    let mut vks = Vec::new();
    for modifier in &combo.modifiers {
        match vk_for_token(modifier) {
            Some(vk) => vks.push(vk),
            None => {
                eprintln!("keylex: unknown modifier {modifier:?} in fallback keycode {keycode:?}");
                return Ok(());
            }
        }
    }
    match vk_for_token(&combo.key) {
        Some(vk) => vks.push(vk),
        None => {
            eprintln!("keylex: unknown key {:?} in fallback keycode {keycode:?}", combo.key);
            return Ok(());
        }
    }

    let down: Vec<INPUT> = vks.iter().map(|vk| key_input(*vk, false)).collect();
    let up: Vec<INPUT> = vks.iter().rev().map(|vk| key_input(*vk, true)).collect();

    unsafe {
        let sent_down = SendInput(&down, std::mem::size_of::<INPUT>() as i32);
        let sent_up = SendInput(&up, std::mem::size_of::<INPUT>() as i32);
        if sent_down as usize != down.len() || sent_up as usize != up.len() {
            eprintln!(
                "keylex: SendInput only processed {sent_down}/{} down, {sent_up}/{} up",
                down.len(),
                up.len()
            );
        }
    }
    Ok(())
}

/// Synthesize a single down or up event for a chord-member token via
/// `SendInput`, used for chord replay (timeout/break) instead of a literal
/// re-emit, since there's no virtual re-emit device on Windows the way
/// `src/capture/linux.rs`'s uinput device provides. These synthetic events
/// re-enter `hook_proc` marked `LLKHF_INJECTED`, which is why `hook_proc`
/// unconditionally passes injected events straight through without
/// re-running chord matching on them.
fn synthesize_key(token: &str, up: bool) {
    let Some(vk) = vk_for_token(token) else {
        eprintln!("keylex: chord replay: unknown key token {token:?}");
        return;
    };
    let input = [key_input(vk, up)];
    unsafe {
        let sent = SendInput(&input, std::mem::size_of::<INPUT>() as i32);
        if sent as usize != input.len() {
            eprintln!(
                "keylex: chord replay: SendInput only processed {sent}/1 for {token:?} (up={up})"
            );
        }
    }
}

/// What a chord-member key that's no longer "pending" (undecided) is
/// currently doing, for the rest of its physical hold: still being
/// swallowed as part of a matched chord, or passing through normally
/// (via synthesized `SendInput` events) because the chord attempt broke or
/// timed out. Mirrors `src/capture/linux.rs`'s `ChordKeyState`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ChordKeyState {
    ConsumedByChord,
    ReplayedPassthrough,
}

/// Chord-member keys currently held down and not yet resolved.
struct PendingChord {
    /// Insertion order, so a timeout/break replay reproduces the keys in
    /// the order they were actually pressed.
    order: Vec<String>,
}

/// All chord-related state, extending `HookState` alongside the existing
/// `Router`. `current_timer_id` is the live Win32 timer (if any) backing
/// the current `pending` window -- Win32's `SetTimer` is a *repeating*
/// timer by default, so every firing of `timer_proc` must `KillTimer` it
/// explicitly, whether or not it's still the one we care about.
struct ChordState {
    pending: Option<PendingChord>,
    held: HashMap<String, ChordKeyState>,
    current_timer_id: Option<usize>,
}

impl ChordState {
    fn new() -> ChordState {
        ChordState {
            pending: None,
            held: HashMap::new(),
            current_timer_id: None,
        }
    }
}

fn kill_current_timer(state: &mut ChordState) {
    if let Some(id) = state.current_timer_id.take() {
        unsafe {
            let _ = KillTimer(None, id);
        }
    }
}

fn arm_timer(state: &mut ChordState) {
    kill_current_timer(state);
    // hwnd = NULL: the timer isn't associated with any window, so the
    // nIDEvent we pass is ignored by Win32 and a fresh id is minted and
    // returned instead -- that returned id is what `timer_proc` later
    // receives, letting us recognize which pending window fired.
    let id = unsafe { SetTimer(None, 0, CHORD_DEBOUNCE_MS, Some(timer_proc)) };
    if id == 0 {
        eprintln!("keylex: chord debounce: SetTimer failed");
        return;
    }
    state.current_timer_id = Some(id);
}

fn replay_pending(state: &mut ChordState) {
    let Some(pending) = state.pending.take() else {
        return;
    };
    for token in &pending.order {
        synthesize_key(token, false);
        state.held.insert(token.clone(), ChordKeyState::ReplayedPassthrough);
    }
}

fn on_chord_key_down(state: &mut ChordState, token: String, registry: &Registry, router: &Router) {
    let Some(mut pending) = state.pending.take() else {
        state.pending = Some(PendingChord { order: vec![token] });
        arm_timer(state);
        return;
    };

    let mut candidate: BTreeSet<String> = pending.order.iter().cloned().collect();
    candidate.insert(token.clone());

    if let Some(action_id) = registry.action_for_chord(&candidate) {
        let action_id = action_id.to_string();
        for member in &pending.order {
            state.held.insert(member.clone(), ChordKeyState::ConsumedByChord);
        }
        state.held.insert(token, ChordKeyState::ConsumedByChord);
        kill_current_timer(state);

        let focused = crate::focus::focused_process_name();
        let result = router.dispatch(&action_id, &focused);
        println!("{action_id} -> {result:?} (chord)");
        return;
    }

    if registry.is_chord_prefix(&candidate) {
        pending.order.push(token);
        state.pending = Some(pending);
        arm_timer(state); // restart the window from this latest key
        return;
    }

    // This key breaks every chord the old pending set could have become:
    // replay the old keys as normal keystrokes, then let this key start a
    // fresh pending sequence of its own.
    state.pending = Some(pending);
    replay_pending(state);
    kill_current_timer(state);
    on_chord_key_down(state, token, registry, router);
}

fn on_chord_key_up(state: &mut ChordState, token: String) {
    if let Some(pending) = &mut state.pending {
        if let Some(pos) = pending.order.iter().position(|t| t == &token) {
            pending.order.remove(pos);
            let now_empty = pending.order.is_empty();
            // Early resolution: released before the chord completed or
            // timed out. Replay just this key's down+up immediately --
            // faster feedback for plain typing than waiting out the rest
            // of the window.
            synthesize_key(&token, false);
            synthesize_key(&token, true);
            if now_empty {
                state.pending = None;
                kill_current_timer(state);
            }
            return;
        }
    }

    match state.held.remove(&token) {
        Some(ChordKeyState::ConsumedByChord) => {} // matched trigger: fully consumed
        Some(ChordKeyState::ReplayedPassthrough) => synthesize_key(&token, true),
        None => {
            // No tracked down for this token (shouldn't normally happen)
            // -- fail safe by letting the up through so nothing is left
            // stuck "down" at the OS level.
            synthesize_key(&token, true);
        }
    }
}

fn on_chord_key_repeat(state: &mut ChordState, token: String) {
    if let Some(pending) = &state.pending {
        if pending.order.contains(&token) {
            return; // undecided: drop, don't extend the window on autorepeat
        }
    }
    if state.held.get(&token) == Some(&ChordKeyState::ReplayedPassthrough) {
        synthesize_key(&token, false); // another synthetic down: mirrors real autorepeat
    }
    // ConsumedByChord (or untracked, which shouldn't happen for a repeat):
    // swallow.
}

struct WindowsFallbackSender;

impl FallbackSender for WindowsFallbackSender {
    fn send(&self, keycode: &str) {
        if let Err(e) = send_keycode(keycode) {
            eprintln!("keylex: failed to send fallback keycode {keycode:?}: {e}");
        }
    }
}

thread_local! {
    static HOOK_STATE: RefCell<Option<*const HookState>> = const { RefCell::new(None) };
}

struct HookState<'a> {
    router: Router<'a>,
    chord_state: RefCell<ChordState>,
}

unsafe extern "system" fn timer_proc(_hwnd: HWND, _msg: u32, id_event: usize, _time: u32) {
    // SetTimer with hwnd=NULL is a *repeating* timer by default -- always
    // kill this specific id so it doesn't keep firing every
    // CHORD_DEBOUNCE_MS forever; a chord debounce is meant to be one-shot.
    let _ = KillTimer(None, id_event);

    HOOK_STATE.with(|state| {
        let Some(ptr) = *state.borrow() else { return };
        let hook_state = &*ptr;
        let mut chord_state = hook_state.chord_state.borrow_mut();
        if chord_state.current_timer_id == Some(id_event) {
            chord_state.current_timer_id = None;
            replay_pending(&mut chord_state);
        }
        // Otherwise: a stale timer for a pending window that was already
        // resolved (matched, broken, or released early) some other way --
        // already killed above, nothing left to do.
    });
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code as u32 != HC_ACTION {
        return CallNextHookEx(None, code, wparam, lparam);
    }

    let info = &*(lparam.0 as *const KBDLLHOOKSTRUCT);

    // Never process our own SendInput-synthesized events (chord replay,
    // fallback keycodes): re-running chord matching on an event we just
    // injected ourselves would corrupt state or loop forever.
    if (info.flags.0 & LLKHF_INJECTED.0) != 0 {
        return CallNextHookEx(None, code, wparam, lparam);
    }

    let is_down = wparam.0 as u32 == WM_KEYDOWN || wparam.0 as u32 == WM_SYSKEYDOWN;
    let is_up = wparam.0 as u32 == WM_KEYUP || wparam.0 as u32 == WM_SYSKEYUP;

    let is_modifier_vk = modifier_token(info.vkCode).is_some();
    let token = modifier_token(info.vkCode)
        .map(str::to_string)
        .or_else(|| key_token(info.vkCode));

    let Some(token) = token else {
        return CallNextHookEx(None, code, wparam, lparam);
    };

    let hook_state_ptr = HOOK_STATE.with(|state| *state.borrow());
    let Some(ptr) = hook_state_ptr else {
        return CallNextHookEx(None, code, wparam, lparam);
    };
    let hook_state = &*ptr;

    if hook_state.router.registry.is_chord_member(&token) {
        let mut chord_state = hook_state.chord_state.borrow_mut();
        // WH_KEYBOARD_LL carries no repeat-count: whether a WM_KEYDOWN is
        // a fresh press or autorepeat is inferred from our own tracking --
        // a genuine fresh key can't send WM_KEYDOWN again without a
        // WM_KEYUP in between.
        let already_tracked = chord_state
            .pending
            .as_ref()
            .is_some_and(|p| p.order.contains(&token))
            || chord_state.held.contains_key(&token);

        if is_down {
            if already_tracked {
                on_chord_key_repeat(&mut chord_state, token);
            } else {
                on_chord_key_down(&mut chord_state, token, &hook_state.router.registry, &hook_state.router);
            }
        } else if is_up {
            on_chord_key_up(&mut chord_state, token);
        }
        return LRESULT(1); // chord-member keys are always suppressed at the source
    }

    if is_modifier_vk {
        // Not part of any configured chord: preserve the original,
        // untouched-passthrough behavior. Modifier state for the
        // single-key(+modifier) trigger path below is read live via
        // GetAsyncKeyState in `pressed_modifiers()`, never tracked from
        // individual key events.
        return CallNextHookEx(None, code, wparam, lparam);
    }

    let combo = KeyCombo {
        key: token,
        modifiers: pressed_modifiers(),
    };

    let matched = hook_state
        .router
        .registry
        .action_for_trigger(&combo)
        .map(str::to_string);

    if let Some(action_id) = matched {
        if is_down {
            let focused = crate::focus::focused_process_name();
            let result = hook_state.router.dispatch(&action_id, &focused);
            println!("{action_id} -> {result:?}");
        }
        if is_down || is_up {
            return LRESULT(1); // suppress: OS/app never sees this key
        }
    }

    CallNextHookEx(None, code, wparam, lparam)
}

pub fn run(
    registry: &Registry,
    adapters: HashMap<String, Box<dyn Adapter>>,
    notifier: Box<dyn Notifier>,
) -> std::io::Result<()> {
    let hook_state = HookState {
        router: Router {
            registry,
            adapters,
            notifier,
            fallback_sender: Box::new(WindowsFallbackSender),
        },
        chord_state: RefCell::new(ChordState::new()),
    };

    HOOK_STATE.with(|state| {
        *state.borrow_mut() = Some(&hook_state as *const HookState);
    });

    let result = unsafe {
        let hmodule: HMODULE = GetModuleHandleW(PCWSTR::null()).unwrap_or_default();
        let hook: windows::core::Result<HHOOK> =
            SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), hmodule, 0);
        match hook {
            Ok(hook) => {
                println!("keylex: windows keyboard hook active");
                let mut msg = MSG::default();
                while GetMessageW(&mut msg, None, 0, 0).into() {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
                let _ = UnhookWindowsHookEx(hook);
                Ok(())
            }
            Err(e) => Err(std::io::Error::other(format!("SetWindowsHookExW failed: {e}"))),
        }
    };

    HOOK_STATE.with(|state| {
        *state.borrow_mut() = None;
    });

    result
}

#[allow(dead_code)]
fn request_stop() {
    unsafe { PostQuitMessage(0) };
}
