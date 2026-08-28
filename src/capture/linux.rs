//! Linux capture: grab the physical keyboard exclusively via evdev and
//! re-emit everything that isn't a bound trigger through a virtual uinput
//! device -- the interception-tools/evremap pattern, since a raw evdev grab
//! blinds the *whole* device and anything not meant to be intercepted has
//! to be re-emitted by hand. A matched trigger is always consumed: it never
//! reaches the OS directly, only ever indirectly via the fallback path.
//!
//! `evdev`'s blocking `fetch_events()` has no timeout, so capture is split
//! across two threads: a reader that only forwards raw events, and the main
//! thread, which owns all state and blocks on a channel carrying both those
//! events and chord debounce ticks. That is what lets a timer wake the same
//! loop that otherwise just waits for the next keystroke.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::io;
use std::rc::Rc;
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Duration;

use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};
use evdev::{Device, EventType, InputEvent, InputEventKind, Key};

use super::chord::{self, Chords, Keyboard as _, Phase};
use crate::config::{is_modifier, KeyCombo, Registry};
use crate::dispatch::{Adapters, FallbackSender, Notifier, Router};
use crate::focus;

/// Prototype scope: a-z, 0-9 and prtsc. evdev's letter keycodes aren't in
/// alphabetical order, so this stays a table; full layout support is a
/// later step.
#[rustfmt::skip]
const KEYS: &[(char, Key)] = &[
    ('a', Key::KEY_A), ('b', Key::KEY_B), ('c', Key::KEY_C), ('d', Key::KEY_D),
    ('e', Key::KEY_E), ('f', Key::KEY_F), ('g', Key::KEY_G), ('h', Key::KEY_H),
    ('i', Key::KEY_I), ('j', Key::KEY_J), ('k', Key::KEY_K), ('l', Key::KEY_L),
    ('m', Key::KEY_M), ('n', Key::KEY_N), ('o', Key::KEY_O), ('p', Key::KEY_P),
    ('q', Key::KEY_Q), ('r', Key::KEY_R), ('s', Key::KEY_S), ('t', Key::KEY_T),
    ('u', Key::KEY_U), ('v', Key::KEY_V), ('w', Key::KEY_W), ('x', Key::KEY_X),
    ('y', Key::KEY_Y), ('z', Key::KEY_Z),
    ('0', Key::KEY_0), ('1', Key::KEY_1), ('2', Key::KEY_2), ('3', Key::KEY_3),
    ('4', Key::KEY_4), ('5', Key::KEY_5), ('6', Key::KEY_6), ('7', Key::KEY_7),
    ('8', Key::KEY_8), ('9', Key::KEY_9),
];

/// Modifier token, left key, right key. Both sides map to the one token;
/// only the left one is ever synthesized, the same simplification the
/// Windows fallback sender makes.
const MODIFIERS: &[(&str, Key, Key)] = &[
    ("ctrl", Key::KEY_LEFTCTRL, Key::KEY_RIGHTCTRL),
    ("shift", Key::KEY_LEFTSHIFT, Key::KEY_RIGHTSHIFT),
    ("alt", Key::KEY_LEFTALT, Key::KEY_RIGHTALT),
    ("win", Key::KEY_LEFTMETA, Key::KEY_RIGHTMETA),
];

/// How long a chord stays pending after its most recent member went down
/// before being replayed as ordinary keystrokes. Chosen to match the
/// comparable window in kmonad/karabiner; not yet configurable.
const DEBOUNCE_WINDOW: Duration = Duration::from_millis(35);

fn token_to_key(token: &str) -> Option<Key> {
    if token == "prtsc" {
        return Some(Key::KEY_SYSRQ);
    }
    if let Some((_, left, _)) = MODIFIERS.iter().find(|(name, _, _)| *name == token) {
        return Some(*left);
    }
    let mut chars = token.chars();
    let token = chars.next().filter(|_| chars.next().is_none())?;
    KEYS.iter().find(|(c, _)| *c == token).map(|(_, key)| *key)
}

fn key_to_token(key: Key) -> Option<String> {
    if key == Key::KEY_SYSRQ {
        return Some("prtsc".to_string());
    }
    if let Some((name, _, _)) = MODIFIERS.iter().find(|(_, l, r)| key == *l || key == *r) {
        return Some((*name).to_string());
    }
    KEYS.iter()
        .find(|(_, k)| *k == key)
        .map(|(c, _)| c.to_string())
}

/// Everything the capture loop can be woken by.
enum Message {
    Event(InputEvent),
    /// A chord debounce window elapsed, for the generation it was armed on.
    Timeout(u64),
}

/// The virtual device, shared between the capture loop's passthrough path
/// and the router's fallback sender, which both write to it.
type Uinput = Rc<RefCell<VirtualDevice>>;

/// Passthrough side of capture: re-emits events the daemon decided not to
/// consume, and tracks which modifiers are currently held (chord replay
/// included, which is why this and not the loop owns that set).
struct Emitter {
    device: Uinput,
    timers: Sender<Message>,
    modifiers: BTreeSet<String>,
}

impl Emitter {
    fn emit(&self, event: InputEvent) -> io::Result<()> {
        self.device.borrow_mut().emit(&[event])
    }
}

impl chord::Keyboard for Emitter {
    type Event = InputEvent;

    fn press(&mut self, token: &str, event: InputEvent) -> io::Result<()> {
        if is_modifier(token) {
            self.modifiers.insert(token.to_string());
        }
        self.emit(event)
    }

    fn release(&mut self, token: &str, event: InputEvent) -> io::Result<()> {
        if is_modifier(token) {
            self.modifiers.remove(token);
        }
        self.emit(event)
    }

    fn arm_timer(&mut self, generation: u64) {
        let timers = self.timers.clone();
        thread::spawn(move || {
            thread::sleep(DEBOUNCE_WINDOW);
            let _ = timers.send(Message::Timeout(generation));
        });
    }
}

struct Fallback {
    device: Uinput,
}

impl FallbackSender for Fallback {
    fn send(&self, combo: &KeyCombo) {
        let mut keys = Vec::with_capacity(combo.modifiers.len() + 1);
        for token in combo.modifiers.iter().chain(std::iter::once(&combo.key)) {
            match token_to_key(token) {
                Some(key) => keys.push(key),
                None => {
                    eprintln!("keylex: fallback keycode {combo}: unknown key {token:?}");
                    return;
                }
            }
        }

        let event =
            |key: &Key, down: bool| InputEvent::new(EventType::KEY, key.code(), down.into());
        let down: Vec<_> = keys.iter().map(|key| event(key, true)).collect();
        let up: Vec<_> = keys.iter().rev().map(|key| event(key, false)).collect();

        let mut device = self.device.borrow_mut();
        if let Err(e) = device.emit(&down).and_then(|()| device.emit(&up)) {
            eprintln!("keylex: failed to send fallback keycode {combo}: {e}");
        }
    }
}

fn discover_keyboard() -> io::Result<Device> {
    evdev::enumerate()
        .map(|(_path, device)| device)
        .find(|device| {
            device
                .supported_keys()
                .is_some_and(|keys| keys.contains(Key::KEY_A) && keys.contains(Key::KEY_SPACE))
        })
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "no keyboard device found via evdev",
            )
        })
}

/// One key event, once its token is known. The chord path takes precedence:
/// a key belonging to some configured chord can't be judged until the
/// debounce window resolves it (see `super::chord`), so it never reaches
/// the single-combo path below.
fn on_key(
    key: Key,
    event: InputEvent,
    registry: &Registry,
    router: &Router,
    chords: &mut Chords<InputEvent>,
    emitter: &mut Emitter,
) -> io::Result<()> {
    let phase = match event.value() {
        1 => Phase::Down,
        0 => Phase::Up,
        _ => Phase::Repeat,
    };

    // An unmapped key (arrows, function keys) can't be bound to anything
    // yet, so it always passes straight through.
    let Some(token) = key_to_token(key) else {
        return emitter.emit(event);
    };

    if registry.is_chord_member(&token) {
        if let Some(action_id) = chords.on_key(&token, phase, event, registry, emitter)? {
            dispatch(&action_id, router);
        }
        return Ok(());
    }

    if is_modifier(&token) {
        return match phase {
            Phase::Up => emitter.release(&token, event),
            _ => emitter.press(&token, event),
        };
    }

    let combo = KeyCombo {
        key: token,
        modifiers: emitter.modifiers.clone(),
    };
    let Some(action_id) = registry.action_for_trigger(&combo).map(str::to_string) else {
        return emitter.emit(event);
    };
    if phase == Phase::Down {
        dispatch(&action_id, router);
    }
    Ok(()) // consumed: never re-emitted, down/up/repeat alike
}

fn dispatch(action_id: &str, router: &Router) {
    let focused = focus::focused_process_name();
    println!(
        "{action_id} -> {}",
        router.dispatch(action_id, focused.as_deref())
    );
}

pub fn run(registry: &Registry, adapters: Adapters, notifier: Box<dyn Notifier>) -> io::Result<()> {
    let mut source = discover_keyboard()?;
    source.grab()?;
    println!(
        "keylex: linux keyboard capture active on {}",
        source.name().unwrap_or("<unnamed>")
    );

    let device: Uinput = {
        let keys = source
            .supported_keys()
            .expect("a device that reached discover_keyboard() must report supported keys");
        let device = VirtualDeviceBuilder::new()?
            .name("keylex-main-keyboard")
            .with_keys(keys)?
            .build()?;
        Rc::new(RefCell::new(device))
    };

    let router = Router::new(
        registry,
        adapters,
        notifier,
        Box::new(Fallback {
            device: Rc::clone(&device),
        }),
    );
    let (events, inbox) = mpsc::channel();
    let mut emitter = Emitter {
        device,
        timers: events.clone(),
        modifiers: BTreeSet::new(),
    };
    let mut chords = Chords::default();

    // Owns the grabbed device and does nothing but forward. Keeping it off
    // the main thread is what lets debounce ticks share the same channel.
    thread::spawn(move || loop {
        match source.fetch_events() {
            Ok(batch) => {
                for event in batch {
                    if events.send(Message::Event(event)).is_err() {
                        return; // main thread gone
                    }
                }
            }
            Err(e) => {
                eprintln!("keylex: evdev read error, capture thread stopping: {e}");
                return;
            }
        }
    });

    loop {
        let message = inbox
            .recv()
            .map_err(|_| io::Error::other("keylex: capture channel closed unexpectedly"))?;
        match message {
            Message::Timeout(generation) => chords.on_timeout(generation, &mut emitter)?,
            Message::Event(event) => match event.kind() {
                InputEventKind::Key(key) => {
                    on_key(key, event, registry, &router, &mut chords, &mut emitter)?;
                }
                _ => emitter.emit(event)?,
            },
        }
    }
}
