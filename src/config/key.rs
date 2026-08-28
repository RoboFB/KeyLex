//! The two shapes a keyboard trigger can take: an ordered combo with one
//! main key (`ctrl+w`), and an unordered chord of keys held together
//! (`ctrl+d+f`).

use std::collections::BTreeSet;
use std::fmt;

/// Token names recognized as modifiers, shared with the capture backends'
/// own modifier tables (`src/capture/linux.rs`, `src/capture/windows.rs`).
const MODIFIERS: [&str; 4] = ["ctrl", "shift", "alt", "win"];

pub fn is_modifier(token: &str) -> bool {
    MODIFIERS.contains(&token)
}

/// A `ctrl+shift+w`-style combo: one main key plus zero or more modifiers,
/// order-independent. The same syntax is used both for a key that triggers
/// an action and for the keycode sent by the fallback path.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyCombo {
    pub key: String,
    pub modifiers: BTreeSet<String>,
}

impl KeyCombo {
    /// `None` for a combo with no key in it at all (`""`, `"+"`), so that a
    /// binding can never silently match nothing.
    pub fn parse(raw: &str) -> Option<KeyCombo> {
        let mut tokens: Vec<String> = raw
            .split('+')
            .map(|token| token.trim().to_lowercase())
            .filter(|token| !token.is_empty())
            .collect();
        let key = tokens.pop()?;
        Some(KeyCombo {
            key,
            modifiers: tokens.into_iter().collect(),
        })
    }
}

impl fmt::Display for KeyCombo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for modifier in &self.modifiers {
            write!(f, "{modifier}+")?;
        }
        f.write_str(&self.key)
    }
}

/// A set of key tokens (modifiers and/or plain keys) that must all be held
/// down together, order-independent, to trigger an action. Distinct from
/// `KeyCombo`, whose `key`/`modifiers` split privileges one token as "the
/// one whose down-edge triggers the check" -- a real chord has no such
/// privileged member.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Chord(BTreeSet<String>);

impl Chord {
    /// Validates one raw `chord = [...]` binding. The rules exist so a
    /// chord always means "these keys, held together": fewer than two keys
    /// is a plain binding, a repeated key can never all be held at once,
    /// and modifiers alone would fire on any ordinary `ctrl+shift` hold.
    ///
    /// Unused until `config/keymap.toml` grows a loader that can produce a
    /// raw chord binding to validate (see CLAUDE.md's "Known gaps") -- kept
    /// because the capture-side chord machinery and its tests still need
    /// `Chord` values to construct.
    #[allow(dead_code)]
    pub(crate) fn parse(tokens: &[String]) -> Result<Chord, String> {
        if tokens.len() < 2 {
            return Err(format!("chord needs at least 2 keys, got {}", tokens.len()));
        }

        let mut chord = Chord::default();
        for token in tokens {
            let token = token.trim().to_lowercase();
            if !chord.0.insert(token.clone()) {
                return Err(format!("chord has duplicate key {token:?}"));
            }
        }

        if chord.tokens().all(is_modifier) {
            return Err(format!(
                "chord {chord} must include at least one non-modifier key"
            ));
        }
        Ok(chord)
    }

    pub fn insert(&mut self, token: impl Into<String>) {
        self.0.insert(token.into());
    }

    pub fn is_subset_of(&self, other: &Chord) -> bool {
        self.0.is_subset(&other.0)
    }

    pub fn tokens(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(String::as_str)
    }
}

impl<T: Into<String>> FromIterator<T> for Chord {
    fn from_iter<I: IntoIterator<Item = T>>(tokens: I) -> Chord {
        Chord(tokens.into_iter().map(Into::into).collect())
    }
}

impl fmt::Display for Chord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, token) in self.tokens().enumerate() {
            if i > 0 {
                f.write_str("+")?;
            }
            f.write_str(token)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combo_parses_modifiers_order_independently() {
        let a = KeyCombo::parse("ctrl+shift+w").unwrap();
        let b = KeyCombo::parse("shift+ctrl+w").unwrap();
        assert_eq!(a, b);
        assert_eq!(a.key, "w");
        assert_eq!(a.to_string(), "ctrl+shift+w");
    }

    #[test]
    fn combo_parses_a_bare_key() {
        let combo = KeyCombo::parse("prtsc").unwrap();
        assert_eq!(combo.key, "prtsc");
        assert!(combo.modifiers.is_empty());
    }

    #[test]
    fn empty_combo_is_rejected() {
        assert!(KeyCombo::parse("").is_none());
        assert!(KeyCombo::parse(" + ").is_none());
    }

    fn chord(tokens: &[&str]) -> Result<Chord, String> {
        Chord::parse(&tokens.iter().map(|t| t.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn chord_ignores_declared_order() {
        assert_eq!(chord(&["f", "ctrl", "d"]), chord(&["ctrl", "d", "f"]));
    }

    #[test]
    fn chord_rejects_short_duplicate_and_modifier_only_bindings() {
        assert!(chord(&["d"]).is_err());
        assert!(chord(&["d", "d"]).is_err());
        assert!(chord(&["ctrl", "shift"]).is_err());
    }
}
