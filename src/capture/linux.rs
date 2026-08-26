//! Linux capture: grab the physical keyboard device exclusively via
//! evdev, and re-emit everything that isn't a bound action's trigger
//! through a virtual uinput device -- matching the interception-tools/
//! evremap pattern (a raw evdev grab blinds the *whole* device, so
//! anything not meant to be intercepted has to be manually re-emitted).
//! A matched trigger is always consumed: it never reaches the OS/app
//! directly, only ever indirectly via the fallback path below.
//!
//! Single-key(+modifier) triggers match synchronously on the triggering
//! key's down-edge, same as always. Chords (`ChordEngine` below) need a
//! bounded debounce window instead, since a lone keystroke that happens to
//! be a chord member can't be told apart from "the start of a chord" until
//! either the rest of the chord arrives or the window times out. `evdev`'s
//! blocking `fetch_events()` has no timeout of its own, so capture is split
//! across two threads: a reader thread that only forwards raw events, and
//! this module's main thread, which owns all state and blocks on a channel
//! that also carries timer ticks -- letting a debounce "wake up" the same
//! loop that normally just waits on the next keystroke.

use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::io;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};
use evdev::{Device, EventType, InputEvent, InputEventKind, Key};

use crate::config::{KeyCombo, Registry};
use crate::dispatch::{Adapter, FallbackSender, Notifier, Router};

const LETTERS: &[(char, Key)] = &[
    ('a', Key::KEY_A), ('b', Key::KEY_B), ('c', Key::KEY_C), ('d', Key::KEY_D),
    ('e', Key::KEY_E), ('f', Key::KEY_F), ('g', Key::KEY_G), ('h', Key::KEY_H),
    ('i', Key::KEY_I), ('j', Key::KEY_J), ('k', Key::KEY_K), ('l', Key::KEY_L),
    ('m', Key::KEY_M), ('n', Key::KEY_N), ('o', Key::KEY_O), ('p', Key::KEY_P),
    ('q', Key::KEY_Q), ('r', Key::KEY_R), ('s', Key::KEY_S), ('t', Key::KEY_T),
    ('u', Key::KEY_U), ('v', Key::KEY_V), ('w', Key::KEY_W), ('x', Key::KEY_X),
    ('y', Key::KEY_Y), ('z', Key::KEY_Z),
];

const DIGITS: &[(char, Key)] = &[
    ('0', Key::KEY_0), ('1', Key::KEY_1), ('2', Key::KEY_2), ('3', Key::KEY_3),
    ('4', Key::KEY_4), ('5', Key::KEY_5), ('6', Key::KEY_6), ('7', Key::KEY_7),
    ('8', Key::KEY_8), ('9', Key::KEY_9),
];

const MODIFIERS: &[(&str, Key, Key)] = &[
    ("ctrl", Key::KEY_LEFTCTRL, Key::KEY_RIGHTCTRL),
    ("shift", Key::KEY_LEFTSHIFT, Key::KEY_RIGHTSHIFT),
    ("alt", Key::KEY_LEFTALT, Key::KEY_RIGHTALT),
    ("win", Key::KEY_LEFTMETA, Key::KEY_RIGHTMETA),
];

/// How long a chord stays "pending" after its most recent member key went
/// down before giving up and replaying it as normal keystrokes. Chosen to
/// match a typical "is this the start of a chord" window in comparable
/// tools (kmonad, karabiner); not yet configurable.
const CHORD_DEBOUNCE_WINDOW: Duration = Duration::from_millis(35);

/// Prototype scope: a-z / 0-9 / prtsc, same as the previous Python
/// listener -- full layout support is a later step.
fn token_to_key(token: &str) -> Option<Key> {
    if token == "prtsc" {
        return Some(Key::KEY_SYSRQ);
    }
    let mut chars = token.chars();
    let c = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    LETTERS
        .iter()
        .chain(DIGITS.iter())
        .find(|(ch, _)| *ch == c)
        .map(|(_, key)| *key)
}

fn key_to_token(key: Key) -> Option<String> {
    if key == Key::KEY_SYSRQ {
        return Some("prtsc".to_string());
    }
    LETTERS
        .iter()
        .chain(DIGITS.iter())
        .find(|(_, k)| *k == key)
        .map(|(c, _)| c.to_string())
}

fn modifier_name(key: Key) -> Option<&'static str> {
    MODIFIERS
        .iter()
        .find(|(_, left, right)| key == *left || key == *right)
        .map(|(name, _, _)| *name)
}

/// Only used by the fallback sender, which always presses the left
/// variant of a modifier -- same simplification the Windows fallback
/// sender already makes (one VK per modifier name, not handedness-aware).
fn modifier_key(name: &str) -> Option<Key> {
    MODIFIERS
        .iter()
        .find(|(n, _, _)| *n == name)
        .map(|(_, left, _)| *left)
}

/// The canonical `&'static str` for a modifier token name, if `token` names
/// one -- used to fold a chord member back into `pressed_modifiers` (which
/// borrows from `MODIFIERS` rather than owning strings) once it's resolved
/// as a normal keystroke via chord replay.
fn modifier_static_name(token: &str) -> Option<&'static str> {
    MODIFIERS
        .iter()
        .find(|(name, _, _)| *name == token)
        .map(|(name, _, _)| *name)
}

fn discover_keyboard() -> io::Result<Device> {
    for (_path, device) in evdev::enumerate() {
        if let Some(keys) = device.supported_keys() {
            if keys.contains(Key::KEY_A) && keys.contains(Key::KEY_SPACE) {
                return Ok(device);
            }
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "no keyboard device found via evdev",
    ))
}

struct LinuxFallbackSender {
    device: Rc<RefCell<VirtualDevice>>,
}

impl FallbackSender for LinuxFallbackSender {
    fn send(&self, keycode: &str) {
        let combo = KeyCombo::parse(keycode);
        let mut keys = Vec::new();
        for modifier in &combo.modifiers {
            match modifier_key(modifier) {
                Some(key) => keys.push(key),
                None => {
                    eprintln!("keylex: unknown modifier {modifier:?} in fallback keycode {keycode:?}");
                    return;
                }
            }
        }
        match token_to_key(&combo.key) {
            Some(key) => keys.push(key),
            None => {
                eprintln!("keylex: unknown key {:?} in fallback keycode {keycode:?}", combo.key);
                return;
            }
        }

        let mut device = self.device.borrow_mut();
        let down: Vec<InputEvent> = keys
            .iter()
            .map(|k| InputEvent::new(EventType::KEY, k.code(), 1))
            .collect();
        if let Err(e) = device.emit(&down) {
            eprintln!("keylex: failed to emit fallback keycode {keycode:?}: {e}");
            return;
        }
        let up: Vec<InputEvent> = keys
            .iter()
            .rev()
            .map(|k| InputEvent::new(EventType::KEY, k.code(), 0))
            .collect();
        if let Err(e) = device.emit(&up) {
            eprintln!("keylex: failed to release fallback keycode {keycode:?}: {e}");
        }
    }
}

/// Sent from the reader thread (raw evdev events) and from one-shot debounce
/// timer threads (chord timeouts) to the main thread, which owns all state.
enum Msg {
    Event(InputEvent),
    /// A chord debounce window elapsed. Carries the generation it was armed
    /// for, so a timer that fires after its chord was already resolved some
    /// other way (matched, broken, or released early) is a stale no-op.
    Timeout(u64),
}

/// What a chord-member key that's no longer "pending" (undecided) is
/// currently doing, for the rest of its physical hold: still being
/// swallowed as part of a matched chord, or passing through normally
/// because the chord attempt broke or timed out.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ChordKeyState {
    ConsumedByChord,
    ReplayedPassthrough,
}

/// Chord-member keys currently held down and not yet resolved: could still
/// complete a configured chord if the right keys follow within the window,
/// or could break/time out and get replayed as normal keystrokes.
struct PendingChord {
    /// Insertion order, so a timeout/break replay reproduces the keys in
    /// the order they were actually pressed.
    order: Vec<String>,
    down_events: HashMap<String, InputEvent>,
    generation: u64,
}

impl PendingChord {
    fn members(&self) -> BTreeSet<String> {
        self.order.iter().cloned().collect()
    }
}

/// All chord-related state for the capture loop. Kept separate from the
/// single-key+modifier trigger path (`pressed_modifiers`, matched via
/// `Registry::action_for_trigger`), which stays exactly as it always was
/// for any key that isn't a configured chord member.
struct ChordEngine {
    pending: Option<PendingChord>,
    held: HashMap<String, ChordKeyState>,
    generation: u64,
}

impl ChordEngine {
    fn new() -> ChordEngine {
        ChordEngine {
            pending: None,
            held: HashMap::new(),
            generation: 0,
        }
    }

    fn next_generation(&mut self) -> u64 {
        self.generation += 1;
        self.generation
    }

    /// Dispatch on a key event already known to be for a token that is a
    /// member of at least one configured chord (`Registry::is_chord_member`
    /// already checked by the caller).
    #[allow(clippy::too_many_arguments)]
    fn handle_chord_member_event(
        &mut self,
        token: String,
        is_modifier: bool,
        event: InputEvent,
        pressed_modifiers: &mut HashSet<&'static str>,
        registry: &Registry,
        router: &Router,
        virtual_device: &Rc<RefCell<VirtualDevice>>,
        tx: &mpsc::Sender<Msg>,
    ) -> io::Result<()> {
        match event.value() {
            1 => self.on_chord_key_down(
                token,
                is_modifier,
                event,
                pressed_modifiers,
                registry,
                router,
                virtual_device,
                tx,
            ),
            0 => self.on_chord_key_up(token, is_modifier, event, pressed_modifiers, virtual_device),
            _ => self.on_chord_key_repeat(token, event, virtual_device),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn on_chord_key_down(
        &mut self,
        token: String,
        is_modifier: bool,
        event: InputEvent,
        pressed_modifiers: &mut HashSet<&'static str>,
        registry: &Registry,
        router: &Router,
        virtual_device: &Rc<RefCell<VirtualDevice>>,
        tx: &mpsc::Sender<Msg>,
    ) -> io::Result<()> {
        let Some(mut pending) = self.pending.take() else {
            let mut order = Vec::new();
            let mut down_events = HashMap::new();
            order.push(token.clone());
            down_events.insert(token, event);
            let generation = self.next_generation();
            self.pending = Some(PendingChord { order, down_events, generation });
            arm_timer(tx.clone(), generation);
            return Ok(());
        };

        let mut candidate = pending.members();
        candidate.insert(token.clone());

        if let Some(action_id) = registry.action_for_chord(&candidate) {
            let action_id = action_id.to_string();
            for member in &pending.order {
                self.held.insert(member.clone(), ChordKeyState::ConsumedByChord);
            }
            self.held.insert(token, ChordKeyState::ConsumedByChord);
            self.next_generation(); // invalidate any in-flight timer for the old pending set

            let focused = crate::focus::focused_process_name();
            let result = router.dispatch(&action_id, &focused);
            println!("{action_id} -> {result} (chord)");
            return Ok(());
        }

        if registry.is_chord_prefix(&candidate) {
            pending.order.push(token.clone());
            pending.down_events.insert(token, event);
            let generation = self.next_generation();
            pending.generation = generation;
            self.pending = Some(pending);
            arm_timer(tx.clone(), generation);
            return Ok(());
        }

        // This key breaks every chord the old pending set could have
        // become: replay the old keys as normal keystrokes, then let this
        // key start a fresh pending sequence of its own (it might yet be
        // the start of a *different* chord).
        self.pending = Some(pending);
        self.replay_pending(pressed_modifiers, virtual_device)?;
        self.on_chord_key_down(token, is_modifier, event, pressed_modifiers, registry, router, virtual_device, tx)
    }

    fn on_chord_key_up(
        &mut self,
        token: String,
        is_modifier: bool,
        event: InputEvent,
        pressed_modifiers: &mut HashSet<&'static str>,
        virtual_device: &Rc<RefCell<VirtualDevice>>,
    ) -> io::Result<()> {
        if let Some(pending) = &mut self.pending {
            if let Some(down_event) = pending.down_events.remove(&token) {
                // Early resolution: released before the chord completed or
                // timed out. Replay just this key's down+up immediately --
                // faster feedback for plain typing than waiting out the
                // rest of the window.
                pending.order.retain(|t| t != &token);
                let now_empty = pending.order.is_empty();
                virtual_device.borrow_mut().emit(&[down_event])?;
                virtual_device.borrow_mut().emit(&[event])?;
                if now_empty {
                    self.pending = None;
                }
                self.next_generation();
                return Ok(());
            }
        }

        match self.held.remove(&token) {
            Some(ChordKeyState::ConsumedByChord) => Ok(()), // matched trigger: fully consumed
            Some(ChordKeyState::ReplayedPassthrough) => {
                if is_modifier {
                    if let Some(name) = modifier_static_name(&token) {
                        pressed_modifiers.remove(name);
                    }
                }
                virtual_device.borrow_mut().emit(&[event])
            }
            None => {
                // No tracked down for this token (shouldn't normally
                // happen) -- fail safe by passing it through.
                virtual_device.borrow_mut().emit(&[event])
            }
        }
    }

    fn on_chord_key_repeat(
        &mut self,
        token: String,
        event: InputEvent,
        virtual_device: &Rc<RefCell<VirtualDevice>>,
    ) -> io::Result<()> {
        if let Some(pending) = &self.pending {
            if pending.order.contains(&token) {
                return Ok(()); // undecided: drop, don't extend the window on autorepeat
            }
        }
        match self.held.get(&token) {
            Some(ChordKeyState::ConsumedByChord) => Ok(()),
            Some(ChordKeyState::ReplayedPassthrough) | None => {
                virtual_device.borrow_mut().emit(&[event])
            }
        }
    }

    /// Replay every currently pending key as an ordinary keystroke, in the
    /// order it was originally pressed, and clear the pending set. Used on
    /// both a debounce timeout and a same-key-down break.
    fn replay_pending(
        &mut self,
        pressed_modifiers: &mut HashSet<&'static str>,
        virtual_device: &Rc<RefCell<VirtualDevice>>,
    ) -> io::Result<()> {
        let Some(pending) = self.pending.take() else {
            return Ok(());
        };
        for token in &pending.order {
            let event = pending.down_events[token];
            virtual_device.borrow_mut().emit(&[event])?;
            self.held.insert(token.clone(), ChordKeyState::ReplayedPassthrough);
            if let Some(name) = modifier_static_name(token) {
                pressed_modifiers.insert(name);
            }
        }
        Ok(())
    }

    fn handle_timeout(
        &mut self,
        generation: u64,
        pressed_modifiers: &mut HashSet<&'static str>,
        virtual_device: &Rc<RefCell<VirtualDevice>>,
    ) -> io::Result<()> {
        let is_current = matches!(&self.pending, Some(p) if p.generation == generation);
        if is_current {
            self.replay_pending(pressed_modifiers, virtual_device)?;
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_key_event(
        &mut self,
        key: Key,
        event: InputEvent,
        pressed_modifiers: &mut HashSet<&'static str>,
        registry: &Registry,
        router: &Router,
        virtual_device: &Rc<RefCell<VirtualDevice>>,
        tx: &mpsc::Sender<Msg>,
    ) -> io::Result<()> {
        if let Some(name) = modifier_name(key) {
            if !registry.is_chord_member(name) {
                match event.value() {
                    1 => {
                        pressed_modifiers.insert(name);
                    }
                    0 => {
                        pressed_modifiers.remove(name);
                    }
                    _ => {}
                }
                return virtual_device.borrow_mut().emit(&[event]);
            }
            return self.handle_chord_member_event(
                name.to_string(),
                true,
                event,
                pressed_modifiers,
                registry,
                router,
                virtual_device,
                tx,
            );
        }

        let Some(token) = key_to_token(key) else {
            // Untracked key (e.g. arrow keys): always passthrough, same as
            // before chords existed.
            return virtual_device.borrow_mut().emit(&[event]);
        };

        if registry.is_chord_member(&token) {
            return self.handle_chord_member_event(
                token,
                false,
                event,
                pressed_modifiers,
                registry,
                router,
                virtual_device,
                tx,
            );
        }

        // Existing single-key(+modifier) trigger path, unchanged: matches
        // synchronously against the currently held *resolved* modifiers
        // (a modifier that's itself mid-chord-decision doesn't count yet).
        let combo = KeyCombo {
            key: token,
            modifiers: pressed_modifiers.iter().map(|s| s.to_string()).collect(),
        };
        if let Some(action_id) = registry.action_for_trigger(&combo).map(str::to_string) {
            if event.value() == 1 {
                let focused = crate::focus::focused_process_name();
                let result = router.dispatch(&action_id, &focused);
                println!("{action_id} -> {result}");
            }
            return Ok(()); // consumed: never re-emitted, down/up/repeat alike
        }

        virtual_device.borrow_mut().emit(&[event])
    }
}

fn arm_timer(tx: mpsc::Sender<Msg>, generation: u64) {
    thread::spawn(move || {
        thread::sleep(CHORD_DEBOUNCE_WINDOW);
        let _ = tx.send(Msg::Timeout(generation));
    });
}

pub fn run(
    registry: &Registry,
    adapters: HashMap<String, Box<dyn Adapter>>,
    notifier: Box<dyn Notifier>,
) -> io::Result<()> {
    let mut source = discover_keyboard()?;
    source.grab()?;
    println!(
        "keylex: linux keyboard capture active on {}",
        source.name().unwrap_or("<unnamed>")
    );

    let vdevice_name = "keylex-main-keyboard".to_string();
    let virtual_device = {
        let keys = source
            .supported_keys()
            .expect("a device that reached discover_keyboard() must report supported keys");
        VirtualDeviceBuilder::new()?
            .name(&vdevice_name)
            .with_keys(keys)?
            .build()?
    };
    let virtual_device = Rc::new(RefCell::new(virtual_device));

    let router = Router {
        registry,
        adapters,
        notifier,
        fallback_sender: Box::new(LinuxFallbackSender {
            device: Rc::clone(&virtual_device),
        }),
    };

    let mut pressed_modifiers: HashSet<&'static str> = HashSet::new();
    let mut chord_engine = ChordEngine::new();

    // Reader thread: owns the grabbed device, forwards raw events over a
    // channel. Keeping this separate from the main thread is what lets the
    // main thread also receive debounce-timer ticks on the same channel,
    // without ever needing a timeout on the blocking evdev read itself.
    let (tx, rx) = mpsc::channel::<Msg>();
    let reader_tx = tx.clone();
    thread::spawn(move || loop {
        match source.fetch_events() {
            Ok(events) => {
                for event in events {
                    if reader_tx.send(Msg::Event(event)).is_err() {
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
        let msg = rx
            .recv()
            .map_err(|_| io::Error::other("keylex: capture channel closed unexpectedly"))?;

        match msg {
            Msg::Timeout(generation) => {
                chord_engine.handle_timeout(generation, &mut pressed_modifiers, &virtual_device)?;
            }
            Msg::Event(event) => {
                let key = match event.kind() {
                    InputEventKind::Key(key) => key,
                    _ => {
                        virtual_device.borrow_mut().emit(&[event])?;
                        continue;
                    }
                };
                chord_engine.handle_key_event(
                    key,
                    event,
                    &mut pressed_modifiers,
                    registry,
                    &router,
                    &virtual_device,
                    &tx,
                )?;
            }
        }
    }
}
