//! Linux global hotkey via X11's `XGrabKey` -- the same permission-free
//! mechanism desktop environments use for their own keyboard shortcuts
//! (GNOME's Settings > Keyboard Shortcuts included, almost certainly).
//! Unlike `capture::linux`'s evdev grab, which has to see and re-emit
//! *every* key and so genuinely needs raw device access, a global hotkey
//! only ever needs "tell me when this one combo fires no matter what's
//! focused" -- and the X server, which already owns real keyboard access,
//! hands exactly that out to any client for free over the ordinary X11
//! protocol. Wayland has no equivalent client-facing protocol for this by
//! design (compositor-specific extensions only), so this backend stays
//! X11-only, matching `focus::linux`'s existing stance.

use std::io;

use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConnectionExt as _, GrabMode, Keysym, ModMask, Setup};
use x11rb::protocol::Event;

use crate::config::KeyCombo;

/// The same `a-z`/`0-9`/`prtsc` vocabulary `capture::linux`'s own token
/// table understands, mapped to X11 keysyms instead of evdev keycodes --
/// ASCII-identical to the token itself for every key in that vocabulary,
/// which is what keeps this a one-liner rather than another lookup table.
fn keysym_for_token(token: &str) -> Option<Keysym> {
    if token == "prtsc" {
        return Some(0xff61); // XK_Print
    }
    let mut chars = token.chars();
    let c = chars
        .next()
        .filter(|c| c.is_ascii_alphanumeric() && chars.next().is_none())?;
    Some(Keysym::from(c.to_ascii_lowercase() as u32))
}

fn modmask_for(combo: &KeyCombo) -> Option<ModMask> {
    let mut mask = ModMask::default();
    for token in &combo.modifiers {
        mask |= match token.as_str() {
            "ctrl" => ModMask::CONTROL,
            "shift" => ModMask::SHIFT,
            "alt" => ModMask::M1,
            "win" => ModMask::M4,
            other => {
                eprintln!("keylex: hotkey {combo}: unknown modifier {other:?}");
                return None;
            }
        };
    }
    Some(mask)
}

/// Finds the keycode the X server currently has `keysym` mapped to.
/// `XGrabKey` grabs a keycode, not a keysym, so a combo like `win+t` has to
/// be translated through whatever layout is actually active, rather than
/// assumed from a fixed table the way `capture::linux`'s evdev keycodes
/// can be.
fn keycode_for(conn: &impl Connection, setup: &Setup, keysym: Keysym) -> io::Result<u8> {
    let count = setup.max_keycode - setup.min_keycode + 1;
    let mapping = conn
        .get_keyboard_mapping(setup.min_keycode, count)
        .map_err(io::Error::other)?
        .reply()
        .map_err(io::Error::other)?;
    let per_keycode = mapping.keysyms_per_keycode as usize;
    mapping
        .keysyms
        .chunks(per_keycode.max(1))
        .position(|group| group.contains(&keysym))
        .map(|i| setup.min_keycode + i as u8)
        .ok_or_else(|| {
            io::Error::other(format!("no key on this layout produces keysym {keysym:#x}"))
        })
}

/// `XGrabKey` matches a modifier state exactly, so without this a hotkey
/// held together with an *unrelated* lock modifier -- NumLock or CapsLock
/// being on, most commonly -- silently would never fire. Grabbing every
/// combination of those two alongside the real modifiers is the standard
/// X11 workaround (the same one window managers use for their own
/// bindings) rather than a special case worth skipping.
fn ignored_locks() -> [ModMask; 4] {
    let none = ModMask::default();
    [
        none,
        ModMask::LOCK,
        ModMask::M2,
        ModMask::LOCK | ModMask::M2,
    ]
}

/// Blocks forever, calling `on_trigger` once per press of `combo`. Grabbed
/// on every screen's root window -- that's what makes it fire regardless
/// of which window has focus, the entire point of a global hotkey.
pub fn listen(combo: &KeyCombo, mut on_trigger: impl FnMut()) -> io::Result<()> {
    let (conn, _screen_num) = x11rb::connect(None).map_err(io::Error::other)?;
    let setup = conn.setup();
    let modifiers = modmask_for(combo)
        .ok_or_else(|| io::Error::other(format!("hotkey {combo}: unsupported modifier")))?;
    let keysym = keysym_for_token(&combo.key)
        .ok_or_else(|| io::Error::other(format!("hotkey {combo}: unknown key {:?}", combo.key)))?;
    let keycode = keycode_for(&conn, setup, keysym)?;

    for screen in &setup.roots {
        for ignored in ignored_locks() {
            conn.grab_key(
                false,
                screen.root,
                modifiers | ignored,
                keycode,
                GrabMode::ASYNC,
                GrabMode::ASYNC,
            )
            .map_err(io::Error::other)?
            .check()
            .map_err(|e| {
                io::Error::other(format!(
                    "XGrabKey for {combo} failed (already bound by the desktop environment or another app?): {e}"
                ))
            })?;
        }
    }
    conn.flush().map_err(io::Error::other)?;
    println!("keylex: spotlight hotkey listener active on X11 (waiting for {combo})");

    loop {
        if let Event::KeyPress(event) = conn.wait_for_event().map_err(io::Error::other)? {
            if event.detail == keycode {
                on_trigger();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keysym_matches_the_key_itself_case_insensitively() {
        assert_eq!(keysym_for_token("t"), Some(0x74));
        assert_eq!(keysym_for_token("T"), Some(0x74));
        assert_eq!(keysym_for_token("5"), Some(0x35));
        assert_eq!(keysym_for_token("prtsc"), Some(0xff61));
    }

    #[test]
    fn keysym_rejects_anything_not_a_single_alphanumeric() {
        assert_eq!(keysym_for_token("ctrl"), None);
        assert_eq!(keysym_for_token(""), None);
        assert_eq!(keysym_for_token("ab"), None);
    }

    #[test]
    fn modmask_combines_every_modifier_in_the_combo() {
        let combo = KeyCombo::parse("ctrl+win+t").unwrap();
        assert_eq!(modmask_for(&combo), Some(ModMask::CONTROL | ModMask::M4));
    }

    #[test]
    fn modmask_is_empty_for_a_bare_key() {
        let combo = KeyCombo::parse("t").unwrap();
        assert_eq!(modmask_for(&combo), Some(ModMask::default()));
    }

    #[test]
    fn ignored_locks_covers_every_combination_of_capslock_and_numlock() {
        let locks = ignored_locks();
        assert_eq!(locks.len(), 4);
        assert!(locks.contains(&ModMask::default()));
        assert!(locks.contains(&(ModMask::LOCK | ModMask::M2)));
    }
}
