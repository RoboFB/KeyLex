//! The chord state machine, shared by both capture backends.
//!
//! A key that belongs to some configured chord can't be judged on its own
//! down-edge: it might be an ordinary keystroke, or the start of a chord.
//! So it is held "pending" instead of being passed on, until either the
//! rest of the chord arrives (consume everything and dispatch), a key
//! arrives that no configured chord could still contain, or the debounce
//! window elapses -- the last two replaying the pending keys as ordinary
//! keystrokes, as if the brief hold had never happened.
//!
//! Only the decisions live here. Actually emitting a key, and running the
//! debounce timer, are what each backend does differently, and are reached
//! through `Keyboard`.

use std::collections::HashMap;
use std::io;

use crate::config::{Chord, Registry};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Down,
    Up,
    Repeat,
}

/// The two effects the state machine needs from its backend.
pub trait Keyboard {
    /// Whatever the backend attached to a key when the event arrived: the
    /// original evdev event on Linux, where it can be re-emitted verbatim
    /// (so left/right handedness survives a replay), and nothing on
    /// Windows, where a suppressed event can never be let through after the
    /// fact and has to be synthesized from the token instead.
    type Event: Copy;

    fn press(&mut self, token: &str, event: Self::Event) -> io::Result<()>;
    fn release(&mut self, token: &str, event: Self::Event) -> io::Result<()>;

    /// (Re)starts the debounce window, which must later report `generation`
    /// back through `Chords::on_timeout`.
    fn arm_timer(&mut self, generation: u64);
}

/// What a chord-member key that is no longer undecided does for the rest of
/// its physical hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Held {
    /// Swallowed as part of a chord that matched.
    ByChord,
    /// Already replayed as an ordinary keystroke, so its release has to go
    /// through too.
    Replayed,
}

/// Chord-member keys held down but not yet resolved, in the order they were
/// pressed, so a replay reproduces them faithfully.
#[derive(Debug)]
struct Pending<E> {
    keys: Vec<(String, E)>,
    generation: u64,
}

impl<E> Pending<E> {
    fn contains(&self, token: &str) -> bool {
        self.keys.iter().any(|(held, _)| held == token)
    }

    fn chord(&self) -> Chord {
        self.keys.iter().map(|(token, _)| token.as_str()).collect()
    }
}

#[derive(Debug)]
pub struct Chords<E> {
    pending: Option<Pending<E>>,
    held: HashMap<String, Held>,
    /// Bumped whenever the pending set changes, so a debounce timer that
    /// fires after its chord was resolved some other way is recognizably
    /// stale and does nothing.
    generation: u64,
}

impl<E> Default for Chords<E> {
    fn default() -> Chords<E> {
        Chords {
            pending: None,
            held: HashMap::new(),
            generation: 0,
        }
    }
}

impl<E: Copy> Chords<E> {
    /// Whether this token is already down as far as the state machine is
    /// concerned. Only the Windows backend needs this: `WH_KEYBOARD_LL`
    /// reports no repeat count, so autorepeat has to be told from a fresh
    /// press by asking what is already tracked.
    #[cfg(windows)]
    pub fn is_tracked(&self, token: &str) -> bool {
        self.held.contains_key(token) || self.pending.as_ref().is_some_and(|p| p.contains(token))
    }

    /// Feeds one event for a token already known to be a chord member.
    /// Returns the action id when this event completed a configured chord;
    /// the caller dispatches it.
    pub fn on_key<K: Keyboard<Event = E>>(
        &mut self,
        token: &str,
        phase: Phase,
        event: E,
        registry: &Registry,
        keyboard: &mut K,
    ) -> io::Result<Option<String>> {
        match phase {
            Phase::Down => self.on_down(token, event, registry, keyboard),
            Phase::Up => self.on_up(token, event, keyboard).map(|()| None),
            Phase::Repeat => self.on_repeat(token, event, keyboard).map(|()| None),
        }
    }

    pub fn on_timeout<K: Keyboard<Event = E>>(
        &mut self,
        generation: u64,
        keyboard: &mut K,
    ) -> io::Result<()> {
        if self
            .pending
            .as_ref()
            .is_some_and(|p| p.generation == generation)
        {
            self.replay(keyboard)?;
        }
        Ok(())
    }

    fn on_down<K: Keyboard<Event = E>>(
        &mut self,
        token: &str,
        event: E,
        registry: &Registry,
        keyboard: &mut K,
    ) -> io::Result<Option<String>> {
        let Some(mut pending) = self.pending.take() else {
            let generation = self.next_generation();
            self.pending = Some(Pending {
                keys: vec![(token.to_string(), event)],
                generation,
            });
            keyboard.arm_timer(generation);
            return Ok(None);
        };

        let mut candidate = pending.chord();
        candidate.insert(token);

        if let Some(action_id) = registry.action_for_chord(&candidate) {
            for (member, _) in &pending.keys {
                self.held.insert(member.clone(), Held::ByChord);
            }
            self.held.insert(token.to_string(), Held::ByChord);
            self.next_generation(); // any in-flight timer is now stale
            return Ok(Some(action_id.to_string()));
        }

        if registry.is_chord_prefix(&candidate) {
            pending.keys.push((token.to_string(), event));
            pending.generation = self.next_generation();
            keyboard.arm_timer(pending.generation);
            self.pending = Some(pending);
            return Ok(None);
        }

        // Nothing the pending set could still have become survives this
        // key: replay those, then let this one start a pending set of its
        // own -- it may yet be the start of a different chord.
        self.pending = Some(pending);
        self.replay(keyboard)?;
        self.on_down(token, event, registry, keyboard)
    }

    fn on_up<K: Keyboard<Event = E>>(
        &mut self,
        token: &str,
        event: E,
        keyboard: &mut K,
    ) -> io::Result<()> {
        if let Some(mut pending) = self.pending.take() {
            if let Some(index) = pending.keys.iter().position(|(held, _)| held == token) {
                // Released while still undecided: replay just this key's
                // press and release now, rather than making plain typing
                // wait out the rest of the window.
                let (_, down) = pending.keys.remove(index);
                keyboard.press(token, down)?;
                keyboard.release(token, event)?;
                if !pending.keys.is_empty() {
                    pending.generation = self.next_generation();
                    keyboard.arm_timer(pending.generation);
                    self.pending = Some(pending);
                }
                return Ok(());
            }
            self.pending = Some(pending);
        }

        match self.held.remove(token) {
            Some(Held::ByChord) => Ok(()),
            // Replayed, or never tracked at all (which shouldn't happen):
            // let the release through, so nothing is left stuck down.
            _ => keyboard.release(token, event),
        }
    }

    fn on_repeat<K: Keyboard<Event = E>>(
        &mut self,
        token: &str,
        event: E,
        keyboard: &mut K,
    ) -> io::Result<()> {
        if self.pending.as_ref().is_some_and(|p| p.contains(token)) {
            return Ok(()); // still undecided: drop it, don't extend the window
        }
        match self.held.get(token) {
            Some(Held::ByChord) => Ok(()),
            _ => keyboard.press(token, event),
        }
    }

    /// Replays every pending key as an ordinary keystroke, in press order,
    /// and clears the pending set.
    fn replay<K: Keyboard<Event = E>>(&mut self, keyboard: &mut K) -> io::Result<()> {
        let Some(pending) = self.pending.take() else {
            return Ok(());
        };
        for (token, event) in pending.keys {
            keyboard.press(&token, event)?;
            self.held.insert(token, Held::Replayed);
        }
        Ok(())
    }

    fn next_generation(&mut self) -> u64 {
        self.generation += 1;
        self.generation
    }
}
