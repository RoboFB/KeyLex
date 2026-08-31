//! Windows global hotkey via `RegisterHotKey`/`WM_HOTKEY` -- Win32's own
//! API for exactly this job, unlike `capture::windows`'s low-level hook,
//! which sees and must judge every key. Claiming the combo this way is
//! also what keeps the shell from acting on it too: Win+T, unclaimed,
//! cycles the taskbar, and a registered hotkey pre-empts that.
//!
//! Never compiled or run on an actual Windows machine -- this dev
//! environment is Linux-only and has no way to install the
//! `x86_64-pc-windows-msvc` target either -- so treat this as a careful
//! port, unverified even by `cargo check`, same caveat `capture::windows`
//! already carries for its own file.

use std::io;

use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT,
    MOD_SHIFT, MOD_WIN,
};
use windows::Win32::UI::WindowsAndMessaging::{GetMessageW, MSG, WM_HOTKEY};

use crate::capture::vk_for_token;
use crate::config::KeyCombo;

/// Arbitrary but fixed: `listen` only ever registers one hotkey at a time.
const HOTKEY_ID: i32 = 1;

/// `MOD_NOREPEAT` is what stops Win32 from re-firing `WM_HOTKEY` for as
/// long as the combo stays held, matching the Linux backend's own
/// down-edge-only trigger.
fn modifiers_for(combo: &KeyCombo) -> Option<HOT_KEY_MODIFIERS> {
    let mut bits = MOD_NOREPEAT.0;
    for token in &combo.modifiers {
        bits |= match token.as_str() {
            "ctrl" => MOD_CONTROL.0,
            "shift" => MOD_SHIFT.0,
            "alt" => MOD_ALT.0,
            "win" => MOD_WIN.0,
            other => {
                eprintln!("keylex: hotkey {combo}: unknown modifier {other:?}");
                return None;
            }
        };
    }
    Some(HOT_KEY_MODIFIERS(bits))
}

/// Blocks forever, calling `on_trigger` once per press of `combo` -- Win32
/// itself judges "held down together" via `RegisterHotKey`, so unlike the
/// Linux backend there is no held-key state to track here.
pub fn listen(combo: &KeyCombo, mut on_trigger: impl FnMut()) -> io::Result<()> {
    let modifiers = modifiers_for(combo)
        .ok_or_else(|| io::Error::other(format!("hotkey {combo}: unsupported modifier")))?;
    let vk = vk_for_token(&combo.key)
        .ok_or_else(|| io::Error::other(format!("hotkey {combo}: unknown key {:?}", combo.key)))?;

    // SAFETY: a null hwnd registers the hotkey against this thread's own
    // message queue, which is what the message loop below then pumps.
    unsafe { RegisterHotKey(None, HOTKEY_ID, modifiers, vk.0 as u32) }
        .map_err(|e| io::Error::other(format!("RegisterHotKey failed: {e}")))?;
    println!("keylex: spotlight hotkey listener active (waiting for {combo})");

    let mut message = MSG::default();
    // SAFETY: `message` is a valid, writable MSG for the whole loop, and
    // each call only reads the message the previous one just filled in --
    // the same pattern `capture::windows::pump_messages` uses.
    while unsafe { GetMessageW(&mut message, None, 0, 0) }.as_bool() {
        if message.message == WM_HOTKEY && message.wParam.0 as i32 == HOTKEY_ID {
            on_trigger();
        }
    }

    // SAFETY: `HOTKEY_ID` is the id `RegisterHotKey` just registered above.
    unsafe {
        let _ = UnregisterHotKey(None, HOTKEY_ID);
    }
    Ok(())
}
