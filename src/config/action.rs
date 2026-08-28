//! `vocabulary.toml` and `actions.toml`: the checked word list, and the
//! actions built from it.

use std::collections::HashSet;

use serde::Deserialize;

use super::key::{Chord, KeyCombo};

/// The action-id vocabulary: every action's `modifier` and (if present)
/// `location` must appear in these sets, or the config is rejected
/// outright. See `docs/protocol.md#action-ids`.
#[derive(Debug, Deserialize)]
pub(crate) struct Vocabulary {
    #[serde(default)]
    modifiers: HashSet<String>,
    #[serde(default)]
    locations: HashSet<String>,
}

impl Vocabulary {
    /// The only place an action id is ever assembled: `modifier` alone
    /// (`save`) or `modifier.location` (`close.tab`), and only from words
    /// this vocabulary declares.
    fn action_id(&self, modifier: &str, location: Option<&str>) -> Result<String, String> {
        if !self.modifiers.contains(modifier) {
            return Err(format!("{modifier:?} is not a modifier in vocabulary.toml"));
        }
        match location {
            None => Ok(modifier.to_string()),
            Some(location) if self.locations.contains(location) => {
                Ok(format!("{modifier}.{location}"))
            }
            Some(location) => Err(format!("{location:?} is not a location in vocabulary.toml")),
        }
    }
}

/// What happens when no target can carry an action out natively.
/// Normalized at load time from `actions.toml`'s `fallback_tier` +
/// `fallback_keycode` pair, so dispatch never re-reads those strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fallback {
    /// Nothing safe to synthesize -- either the action is `notify_only`, or
    /// it named no `fallback_keycode` to send.
    Unsupported,
    Keycode {
        combo: KeyCombo,
        notify: bool,
    },
}

/// The physical binding that fires an action, if it has one. An action
/// without a trigger (`go_to.definition` today) is still dispatchable by
/// anything else that names it, e.g. the spotlight launcher.
#[derive(Debug)]
pub(crate) enum Trigger {
    Key(KeyCombo),
    Chord(Chord),
}

/// One validated `[[action]]` entry.
#[derive(Debug)]
pub(crate) struct Action {
    pub id: String,
    pub trigger: Option<Trigger>,
    pub fallback: Fallback,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
enum FallbackTier {
    Silent,
    #[default]
    NotifyAttempt,
    NotifyOnly,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawAction {
    modifier: String,
    location: Option<String>,
    key: Option<String>,
    chord: Option<Vec<String>>,
    #[serde(default)]
    fallback_tier: FallbackTier,
    fallback_keycode: Option<String>,
}

impl RawAction {
    pub(crate) fn resolve(self, vocabulary: &Vocabulary) -> Result<Action, String> {
        let id = vocabulary.action_id(&self.modifier, self.location.as_deref())?;
        let describe = |message: String| format!("action {id:?}: {message}");

        let trigger = match (self.key, self.chord) {
            (Some(_), Some(_)) => {
                return Err(describe(
                    "'key' and 'chord' are mutually exclusive".to_string(),
                ));
            }
            (Some(key), None) => {
                Some(Trigger::Key(KeyCombo::parse(&key).ok_or_else(|| {
                    describe(format!("{key:?} is not a key combo"))
                })?))
            }
            (None, Some(chord)) => Some(Trigger::Chord(Chord::parse(&chord).map_err(describe)?)),
            (None, None) => None,
        };

        let fallback = match (self.fallback_tier, self.fallback_keycode) {
            (FallbackTier::NotifyOnly, _) | (_, None) => Fallback::Unsupported,
            (tier, Some(keycode)) => Fallback::Keycode {
                combo: KeyCombo::parse(&keycode)
                    .ok_or_else(|| describe(format!("{keycode:?} is not a key combo")))?,
                notify: matches!(tier, FallbackTier::NotifyAttempt),
            },
        };

        Ok(Action {
            id,
            trigger,
            fallback,
        })
    }
}
