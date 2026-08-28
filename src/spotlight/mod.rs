//! Fuzzy-searchable catalog of actions (`keylex --spotlight`, see
//! `docs/protocol.md#action-catalog-handshake-list_actions`).
//!
//! Two things are deliberately kept separate here:
//! - *Where entries come from*: every action id `Registry` knows, enriched
//!   -- never replaced wholesale -- by whatever a target reports live via
//!   the `list_actions` handshake. There is no hand-maintained list of
//!   "valid options" anywhere in this module; a target that stays silent
//!   just means its entries keep whatever the local config already said.
//! - *How entries are ranked*: fuzzy match score (`nucleo-matcher`, pure
//!   computation with no OS bindings, so ranking is identical on
//!   Linux/macOS/Windows) plus an optional frecency bonus.

mod frecency;
mod ui;

pub use frecency::Frecency;
pub use ui::run_interactive;

use std::path::Path;

use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher};
use serde::{Deserialize, Serialize};

use crate::adapters::SocketAdapter;
use crate::config::{AdapterKind, Registry};
use crate::dispatch::{Outcome, Router};

/// Marks an entry that no target has reported on yet, so it carries only
/// what the local config says about it.
const LOCAL: &str = "local";

/// One entry in the catalog. Serialized as-is by `--spotlight-query`, so
/// the field names are part of that CLI contract (see
/// `extensions/linux-extension/search-provider.js`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Entry {
    pub action_id: String,
    pub title: String,
    /// The key or chord bound to this action, for display only -- never
    /// used for matching or dispatch.
    pub key_hint: Option<String>,
    /// `"local"`, or the reporting target's program name once a
    /// `list_actions` handshake has enriched this entry.
    pub source: String,
    pub native_command: Option<String>,
}

impl Entry {
    /// Two kinds of entry, dispatched differently -- the same split
    /// `Index::merge_remote` made when it named them:
    /// - A real Keylex action id (`close.tab`) goes through the ordinary
    ///   focus-aware `Router::dispatch`, exactly as a key binding would.
    /// - Anything else is one target's own native command, with no
    ///   cross-app abstraction to route by focus, so it goes straight back
    ///   to the target that reported it whatever is focused right now.
    pub fn dispatch(&self, focused_process: Option<&str>, router: &Router) -> Outcome {
        if router.registry().has_action(&self.action_id) {
            return router.dispatch(&self.action_id, focused_process);
        }

        let Some(native_command) = &self.native_command else {
            return Outcome::Unsupported(self.action_id.clone());
        };
        let target = router
            .registry()
            .targets()
            .iter()
            .find(|t| t.program == self.source);
        match target {
            Some(target) => router.send_native(target, native_command),
            None => Outcome::Unsupported(format!("no target named {:?}", self.source)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Match {
    pub entry: Entry,
    pub score: u32,
}

/// One action a target reported via the `list_actions` handshake
/// (`docs/protocol.md#action-catalog-handshake-list_actions`).
#[derive(Debug, Clone, Deserialize)]
pub struct RemoteAction {
    pub id: String,
    pub native_command: String,
    pub title: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListActionsResponse {
    pub actions: Vec<RemoteAction>,
}

/// Turns a derived action id (`close.tab`) into a human title ("Close
/// Tab"), used until a target's handshake supplies a real one.
fn title_from_action_id(action_id: &str) -> String {
    action_id
        .split(['.', '_'])
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub struct Index<'a> {
    registry: &'a Registry,
    entries: Vec<Entry>,
    frecency: Frecency,
}

impl<'a> Index<'a> {
    /// The baseline catalog: every action id the config knows, which is
    /// enough to search with no target running at all.
    pub fn new(registry: &'a Registry, frecency: Frecency) -> Index<'a> {
        let entries = registry
            .action_ids()
            .map(|id| Entry {
                action_id: id.to_string(),
                title: title_from_action_id(id),
                key_hint: registry.trigger_for_action(id),
                source: LOCAL.to_string(),
                native_command: None,
            })
            .collect();
        Index {
            registry,
            entries,
            frecency,
        }
    }

    /// Merges a target's `list_actions` response into the catalog. An id
    /// that names a real Keylex action updates that entry in place (its
    /// `key_hint`, which the target has no way to know, survives). Anything
    /// else is one target's own command string with no cross-app
    /// abstraction, so it's namespaced `"<source>:<id>"` -- two targets can
    /// then never collide with each other or with a real action id, and
    /// `Entry::dispatch` can tell the two kinds apart by that same rule.
    pub fn merge_remote(&mut self, source: &str, remote: Vec<RemoteAction>) {
        for action in remote {
            let id = if self.registry.has_action(&action.id) {
                action.id
            } else {
                format!("{source}:{}", action.id)
            };
            match self.entries.iter_mut().find(|entry| entry.action_id == id) {
                Some(entry) => {
                    entry.title = action.title;
                    entry.source = source.to_string();
                    entry.native_command = Some(action.native_command);
                }
                None => self.entries.push(Entry {
                    action_id: id,
                    title: action.title,
                    key_hint: None,
                    source: source.to_string(),
                    native_command: Some(action.native_command),
                }),
            }
        }
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Records that `action_id` was just picked, so future searches rank it
    /// slightly higher.
    pub fn record_use(&mut self, action_id: &str) {
        self.frecency.touch(action_id);
    }

    /// Ranks the whole catalog against `query`. An empty query returns
    /// everything ordered by frecency alone (most used first), which is
    /// what a launcher should show before anything is typed.
    pub fn search(&self, query: &str) -> Vec<Match> {
        let now = frecency::now_secs();
        let boost = |entry: &Entry| self.frecency.boost(&entry.action_id, now);

        let mut matches: Vec<Match> = if query.trim().is_empty() {
            self.entries
                .iter()
                .map(|entry| Match {
                    score: boost(entry),
                    entry: entry.clone(),
                })
                .collect()
        } else {
            // Match against "<title> <action_id>" so the raw id
            // ("close.tab") works as well as the title ("close tab").
            let haystacks: Vec<String> = self
                .entries
                .iter()
                .map(|entry| format!("{} {}", entry.title, entry.action_id))
                .collect();
            let candidates = haystacks.iter().enumerate().map(|(i, h)| Candidate(i, h));

            Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart)
                .match_list(candidates, &mut Matcher::new(Config::DEFAULT))
                .into_iter()
                .map(|(candidate, score)| {
                    let entry = self.entries[candidate.0].clone();
                    Match {
                        score: score + boost(&entry),
                        entry,
                    }
                })
                .collect()
        };

        matches.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| a.entry.title.cmp(&b.entry.title))
        });
        matches
    }
}

/// Pairs a haystack with the `Index::entries` slot it came from, so
/// `Pattern::match_list` (which only needs `AsRef<str>`) can hand the slot
/// back alongside the score.
struct Candidate<'a>(usize, &'a str);

impl AsRef<str> for Candidate<'_> {
    fn as_ref(&self) -> &str {
        self.1
    }
}

/// Builds the catalog for one run: the baseline from `Registry`, plus a
/// best-effort `list_actions` handshake against every socket target. That
/// handshake is what makes an entry show up with the target's own live
/// title and command -- when, and only when, the target is running and
/// answers. A silent target is not an error; its entries just stay local.
pub fn bootstrap<'a>(
    registry: &'a Registry,
    config_dir: &Path,
    handshake_adapter: &SocketAdapter,
) -> Index<'a> {
    let mut index = Index::new(registry, Frecency::load(config_dir));
    for target in registry.targets() {
        if target.adapter != AdapterKind::Socket {
            continue;
        }
        if let Some(remote) = handshake_adapter.fetch_actions(target) {
            index.merge_remote(&target.program, remote);
        }
    }
    index
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A throwaway config dir per call -- these tests run in parallel, so
    /// they must not share one.
    fn registry() -> Registry {
        static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("keylex-spotlight-{}-{id}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("vocabulary.toml"),
            "modifiers = [\"close\", \"save\"]\nlocations = [\"tab\"]\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("actions.toml"),
            "[[action]]\nmodifier = \"close\"\nlocation = \"tab\"\nkey = \"ctrl+w\"\n[[action]]\nmodifier = \"save\"\n",
        )
        .unwrap();
        std::fs::write(dir.join("targets.toml"), "").unwrap();
        let registry = Registry::load(&dir).expect("fixture config should load");
        let _ = std::fs::remove_dir_all(&dir);
        registry
    }

    #[test]
    fn titles_are_humanized_from_the_action_id() {
        assert_eq!(title_from_action_id("close.tab"), "Close Tab");
        assert_eq!(title_from_action_id("go_to.definition"), "Go To Definition");
    }

    #[test]
    fn search_matches_by_title_and_by_raw_id() {
        let registry = registry();
        let index = Index::new(&registry, Frecency::default());

        assert_eq!(index.search("close tab")[0].entry.action_id, "close.tab");
        assert_eq!(
            index.search("close.tab")[0].entry.key_hint.as_deref(),
            Some("ctrl+w")
        );
    }

    #[test]
    fn empty_query_ranks_recently_used_first() {
        let registry = registry();
        let mut index = Index::new(&registry, Frecency::default());
        index.record_use("save");

        let results = index.search("");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].entry.action_id, "save");
    }

    #[test]
    fn merging_enriches_a_known_action_and_namespaces_an_unknown_one() {
        let registry = registry();
        let mut index = Index::new(&registry, Frecency::default());
        index.merge_remote(
            "vscode",
            vec![
                RemoteAction {
                    id: "close.tab".to_string(),
                    native_command: "workbench.action.closeActiveEditor".to_string(),
                    title: "Close Editor".to_string(),
                },
                RemoteAction {
                    id: "editor.action.formatDocument".to_string(),
                    native_command: "editor.action.formatDocument".to_string(),
                    title: "Format Document".to_string(),
                },
            ],
        );

        let known = index
            .entries()
            .iter()
            .find(|e| e.action_id == "close.tab")
            .unwrap();
        assert_eq!(known.title, "Close Editor");
        assert_eq!(
            known.key_hint.as_deref(),
            Some("ctrl+w"),
            "local binding should survive"
        );

        assert!(index
            .entries()
            .iter()
            .any(|e| e.action_id == "vscode:editor.action.formatDocument"));
    }
}
