//! Fuzzy-searchable "spotlight" catalog of actions (`keylex --spotlight`,
//! see `docs/protocol.md#action-catalog-handshake-list_actions`).
//!
//! Two things are deliberately kept separate here:
//! - *Where entries come from*: every action id known to `Registry`
//!   (`actions.toml`/`targets.toml`), enriched -- never replaced wholesale --
//!   by whatever a target reports live via the `list_actions` handshake.
//!   There is no hand-maintained list of "valid options" anywhere in this
//!   module; a target that goes silent (not running, doesn't implement the
//!   handshake) just means its entries keep whatever the local config
//!   already said about them.
//! - *How entries are ranked*: fuzzy match score (via `nucleo-matcher`, pure
//!   computation with no OS bindings, so ranking is identical on
//!   Linux/macOS/Windows) plus an optional zoxide-style frecency bonus for
//!   actions dispatched from the spotlight itself before.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::style::Print;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{execute, queue};
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher};
use serde::{Deserialize, Serialize};

use crate::adapters::SocketAdapter;
use crate::config::Registry;
use crate::dispatch::{DispatchResult, DispatchStatus, Router};
use crate::focus;

const FRECENCY_FILE_NAME: &str = "spotlight_frecency.json";

/// One entry in the spotlight catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpotlightEntry {
    pub action_id: String,
    pub title: String,
    /// The physical key/chord bound to this action, rendered "ctrl+w"-style,
    /// purely for display -- not used for matching or dispatch.
    pub key_hint: Option<String>,
    /// "local" for an entry built only from `actions.toml`/`targets.toml`,
    /// or the reporting target's `program` name (e.g. "vscode") once a
    /// `list_actions` handshake has enriched it.
    pub source: String,
    pub native_command: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpotlightMatch {
    pub entry: SpotlightEntry,
    pub score: u32,
}

/// One action a target reported via the `list_actions` handshake response
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

/// Turns a derived action id (`close.tab`, `save`) into a human title
/// ("Close Tab", "Save") -- used until a target's handshake response
/// supplies a real one.
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

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct FrecencyRecord {
    count: u32,
    last_used_secs: u64,
}

/// Optional "last used" tracking, zoxide-style: a small on-disk score per
/// action id combining how often and how recently it was picked from the
/// spotlight, persisted as JSON at `<config-dir>/spotlight_frecency.json`.
/// Never required -- `Frecency::empty()` behaves like "nothing has ever
/// been used" and simply contributes no ranking boost.
#[derive(Debug, Clone, Default)]
pub struct Frecency {
    records: HashMap<String, FrecencyRecord>,
    path: Option<PathBuf>,
}

impl Frecency {
    /// Loads persisted usage stats from `<config_dir>/spotlight_frecency.json`,
    /// or starts empty if the file doesn't exist yet or fails to parse.
    pub fn load(config_dir: &Path) -> Frecency {
        let path = config_dir.join(FRECENCY_FILE_NAME);
        let records = fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default();
        Frecency {
            records,
            path: Some(path),
        }
    }

    /// An in-memory-only tracker that never persists -- for callers (tests,
    /// `--spotlight-query`) that don't want frecency to touch disk at all.
    pub fn empty() -> Frecency {
        Frecency {
            records: HashMap::new(),
            path: None,
        }
    }

    /// Records one use of `action_id` right now, and persists immediately if
    /// this tracker was loaded from a config dir. A write failure is
    /// non-fatal (frecency is a ranking hint, not durable state worth
    /// failing a dispatch over).
    pub fn touch(&mut self, action_id: &str) {
        let record = self.records.entry(action_id.to_string()).or_default();
        record.count += 1;
        record.last_used_secs = now_secs();
        self.save();
    }

    fn save(&self) {
        let Some(path) = &self.path else { return };
        if let Ok(json) = serde_json::to_string_pretty(&self.records) {
            let _ = fs::write(path, json);
        }
    }

    /// A small integer bonus added on top of the fuzzy match score -- never
    /// large enough that a well-used but poorly matching entry could outrank
    /// a strong fuzzy match, only enough to break ties/near-ties in favor of
    /// what's actually used often and recently (zoxide's "frecency" idea).
    fn boost(&self, action_id: &str, now: u64) -> u32 {
        let Some(record) = self.records.get(action_id) else {
            return 0;
        };
        let age_days = now.saturating_sub(record.last_used_secs) / 86_400;
        let recency_weight = match age_days {
            0 => 1.0,
            1..=7 => 0.5,
            8..=30 => 0.2,
            _ => 0.05,
        };
        ((record.count as f64) * recency_weight * 20.0) as u32
    }
}

/// The fuzzy-searchable action catalog itself.
pub struct Index {
    entries: Vec<SpotlightEntry>,
    frecency: Frecency,
    /// Action ids `Registry` already knew about at build time -- the
    /// dividing line `merge_remote` and `dispatch_entry` both use to tell a
    /// real, cross-app Keylex action id (`close.tab`) apart from a target's
    /// raw native command string that happens to have no such abstraction.
    local_action_ids: HashSet<String>,
}

/// Wraps a `(usize, &str)` pair so it can be passed straight into
/// `Pattern::match_list` (which needs `AsRef<str>` items) while keeping
/// track of which `Index::entries` slot each haystack came from.
struct Indexed<'a>(usize, &'a str);

impl AsRef<str> for Indexed<'_> {
    fn as_ref(&self) -> &str {
        self.1
    }
}

impl Index {
    /// Builds the baseline catalog from every action id `Registry` knows
    /// about -- this alone is enough to search by, with no target running
    /// at all. `refresh_from_targets` (via `SocketAdapter::fetch_actions`)
    /// enriches it further once a target answers the handshake.
    pub fn from_registry(registry: &Registry, frecency: Frecency) -> Index {
        let local_action_ids: HashSet<String> = registry.action_ids().map(String::from).collect();
        let entries = local_action_ids
            .iter()
            .map(|id| SpotlightEntry {
                action_id: id.clone(),
                title: title_from_action_id(id),
                key_hint: registry.trigger_for_action(id),
                source: "local".to_string(),
                native_command: None,
            })
            .collect();
        Index {
            entries,
            frecency,
            local_action_ids,
        }
    }

    /// Merges a target's `list_actions` handshake response into the
    /// catalog. Two cases:
    /// - `action.id` is already a real Keylex action id (`close.tab`) --
    ///   that entry's title/native command/source are updated in place (its
    ///   `key_hint`, which the target has no way to know, is left
    ///   untouched).
    /// - Anything else has no cross-app abstraction of its own (a target is
    ///   free to just report its native command string as `id`, see
    ///   docs/protocol.md#action-catalog-handshake-list_actions) -- it's
    ///   namespaced as `"<source>:<id>"` so two different targets can never
    ///   collide with each other or with a real action id, and so
    ///   `dispatch_entry` can later tell the two kinds apart by the same
    ///   rule (`local_action_ids`).
    pub fn merge_remote(&mut self, source: &str, remote: Vec<RemoteAction>) {
        for action in remote {
            let id = if self.local_action_ids.contains(&action.id) {
                action.id
            } else {
                format!("{source}:{}", action.id)
            };
            match self.entries.iter_mut().find(|e| e.action_id == id) {
                Some(existing) => {
                    existing.title = action.title;
                    existing.source = source.to_string();
                    existing.native_command = Some(action.native_command);
                }
                None => self.entries.push(SpotlightEntry {
                    action_id: id,
                    title: action.title,
                    key_hint: None,
                    source: source.to_string(),
                    native_command: Some(action.native_command),
                }),
            }
        }
    }

    pub fn entries(&self) -> &[SpotlightEntry] {
        &self.entries
    }

    /// Records that `action_id` was just picked from the spotlight, so
    /// future searches rank it slightly higher (see `Frecency::boost`).
    pub fn record_use(&mut self, action_id: &str) {
        self.frecency.touch(action_id);
    }

    /// Ranks the whole catalog against `query`. An empty query returns
    /// everything ordered by frecency alone (most used first), which is
    /// what a spotlight UI should show before the user has typed anything.
    pub fn search(&self, query: &str) -> Vec<SpotlightMatch> {
        let now = now_secs();
        if query.trim().is_empty() {
            let mut scored: Vec<SpotlightMatch> = self
                .entries
                .iter()
                .cloned()
                .map(|entry| {
                    let score = self.frecency.boost(&entry.action_id, now);
                    SpotlightMatch { entry, score }
                })
                .collect();
            scored.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.entry.title.cmp(&b.entry.title)));
            return scored;
        }

        let mut matcher = Matcher::new(Config::DEFAULT);
        let pattern = Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart);

        // Match against "<title> <action_id>" so a query like "close.tab"
        // (the raw id) works just as well as "close tab" (the title).
        let haystacks: Vec<String> = self
            .entries
            .iter()
            .map(|e| format!("{} {}", e.title, e.action_id))
            .collect();
        let items = haystacks.iter().enumerate().map(|(i, h)| Indexed(i, h.as_str()));

        let mut scored: Vec<SpotlightMatch> = pattern
            .match_list(items, &mut matcher)
            .into_iter()
            .map(|(indexed, fuzzy_score)| {
                let entry = self.entries[indexed.0].clone();
                let boost = self.frecency.boost(&entry.action_id, now);
                SpotlightMatch {
                    entry,
                    score: fuzzy_score + boost,
                }
            })
            .collect();
        scored.sort_by(|a, b| b.score.cmp(&a.score));
        scored
    }
}

/// Builds the spotlight catalog for one run: the baseline from `Registry`
/// plus a best-effort `list_actions` handshake
/// (docs/protocol.md#action-catalog-handshake-list_actions) against every
/// `adapter = "socket"` target -- this is what makes "close.tab in VS Code"
/// show up with VS Code's own live title/command instead of just the
/// locally-derived "Close Tab", when (and only when) the VS Code extension
/// is actually running and answers. A target that doesn't answer simply
/// keeps its local-only entries; nothing here treats that as an error.
pub fn bootstrap(registry: &Registry, config_dir: &Path, handshake_adapter: &SocketAdapter) -> Index {
    let frecency = Frecency::load(config_dir);
    let mut index = Index::from_registry(registry, frecency);
    for target in &registry.targets {
        if target.adapter != "socket" {
            continue;
        }
        if let Some(remote) = handshake_adapter.fetch_actions(target) {
            index.merge_remote(&target.program, remote);
        }
    }
    index
}

/// Dispatches one spotlight entry. Two cases, matching how `merge_remote`
/// named the entry:
/// - `entry.action_id` is a real Keylex action id (`close.tab`, whether it
///   came only from `actions.toml` or was additionally enriched by a
///   handshake) -- goes through the normal `Router::dispatch`, the exact
///   same focus-aware native-adapter-then-keycode-fallback path a real key
///   binding would take.
/// - Anything else is a raw native command with no cross-app abstraction
///   (see docs/protocol.md#action-catalog-handshake-list_actions) --
///   there's nothing to route by focus, so it's sent directly to whichever
///   target reported it (`entry.source`), regardless of what's currently
///   focused.
pub fn dispatch_entry(entry: &SpotlightEntry, focused_process: &str, router: &Router) -> DispatchResult {
    if router.registry.action_ids().any(|id| id == entry.action_id) {
        return router.dispatch(&entry.action_id, focused_process);
    }

    let Some(native_command) = &entry.native_command else {
        return DispatchResult {
            status: DispatchStatus::Unsupported,
            detail: entry.action_id.clone(),
        };
    };
    let Some(target) = router.registry.targets.iter().find(|t| t.program == entry.source) else {
        return DispatchResult {
            status: DispatchStatus::Unsupported,
            detail: format!("no target named {:?}", entry.source),
        };
    };
    match router.adapters.get(&target.adapter) {
        Some(adapter) => {
            adapter.send(target, native_command);
            DispatchResult {
                status: DispatchStatus::Native,
                detail: native_command.clone(),
            }
        }
        None => DispatchResult {
            status: DispatchStatus::Unsupported,
            detail: format!("adapter {} missing", target.adapter),
        },
    }
}

const MAX_VISIBLE_MATCHES: usize = 12;

/// Interactive, cross-platform terminal spotlight launcher (`keylex
/// --spotlight`): type to fuzzy-filter `index`, Up/Down to move the
/// selection, Enter to dispatch the highlighted action through `router` --
/// the exact same `Router::dispatch` the real capture loop uses (native
/// adapter first, keycode fallback otherwise) -- against whichever app is
/// currently focused, Esc/Ctrl-C to quit without dispatching anything.
/// Built on `crossterm`, which is pure terminal I/O with no OS-specific code
/// of its own, so this behaves identically in a Linux, macOS, or Windows
/// terminal.
pub fn run_interactive(index: &mut Index, router: &Router) -> io::Result<()> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, Hide)?;

    let mut query = String::new();
    let mut selected: usize = 0;
    let mut last_dispatch: Option<String> = None;

    let outcome = (|| -> io::Result<()> {
        loop {
            let matches = index.search(&query);
            if !matches.is_empty() && selected >= matches.len() {
                selected = matches.len() - 1;
            }
            render(&mut stdout, &query, &matches, selected, last_dispatch.as_deref())?;

            let Event::Key(key) = event::read()? else {
                continue;
            };
            if key.kind == KeyEventKind::Release {
                continue;
            }
            match key.code {
                KeyCode::Esc => break,
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                KeyCode::Enter => {
                    if let Some(m) = matches.get(selected) {
                        let entry = m.entry.clone();
                        let focused = focus::focused_process_name();
                        let result = dispatch_entry(&entry, &focused, router);
                        index.record_use(&entry.action_id);
                        last_dispatch = Some(format!("{} -> {result}", entry.action_id));
                        query.clear();
                        selected = 0;
                    }
                }
                KeyCode::Backspace => {
                    query.pop();
                    selected = 0;
                }
                KeyCode::Up => selected = selected.saturating_sub(1),
                KeyCode::Down => selected = selected.saturating_add(1),
                KeyCode::Char(c) => {
                    query.push(c);
                    selected = 0;
                }
                _ => {}
            }
        }
        Ok(())
    })();

    execute!(stdout, Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    outcome
}

fn render(
    stdout: &mut io::Stdout,
    query: &str,
    matches: &[SpotlightMatch],
    selected: usize,
    last_dispatch: Option<&str>,
) -> io::Result<()> {
    queue!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
    queue!(
        stdout,
        Print("Keylex spotlight -- type to search, Enter to run, Esc to quit\r\n"),
        Print(format!("> {query}\r\n\r\n"))
    )?;

    if matches.is_empty() {
        queue!(stdout, Print("  (no matches)\r\n"))?;
    }
    for (i, m) in matches.iter().take(MAX_VISIBLE_MATCHES).enumerate() {
        let marker = if i == selected { "> " } else { "  " };
        let key_hint = m.entry.key_hint.as_deref().map(|k| format!(" ({k})")).unwrap_or_default();
        queue!(
            stdout,
            Print(format!("{marker}{}{key_hint}  [{}]\r\n", m.entry.title, m.entry.source))
        )?;
    }

    if let Some(msg) = last_dispatch {
        queue!(stdout, Print(format!("\r\nlast: {msg}\r\n")))?;
    }

    stdout.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::path::PathBuf;
    use std::rc::Rc;

    use crate::dispatch::{Adapter, FallbackSender, Notifier};

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "keylex-spotlight-test-{name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn title_from_action_id_humanizes_modifier_and_location() {
        assert_eq!(title_from_action_id("close.tab"), "Close Tab");
        assert_eq!(title_from_action_id("save"), "Save");
        assert_eq!(title_from_action_id("go_to.definition"), "Go To Definition");
    }

    fn test_registry() -> Registry {
        let dir = temp_dir("registry");
        std::fs::write(
            dir.join("vocabulary.toml"),
            "modifiers = [\"close\", \"save\", \"go_to\"]\nlocations = [\"tab\", \"definition\"]\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("actions.toml"),
            r#"[[action]]
modifier = "close"
location = "tab"
key = "ctrl+w"

[[action]]
modifier = "save"
key = "ctrl+s"

[[action]]
modifier = "go_to"
location = "definition"
"#,
        )
        .unwrap();
        std::fs::write(dir.join("targets.toml"), "").unwrap();
        let registry = Registry::load(&dir).expect("fixture config should load");
        let _ = std::fs::remove_dir_all(&dir);
        registry
    }

    #[test]
    fn search_ranks_exact_title_match_first() {
        let registry = test_registry();
        let index = Index::from_registry(&registry, Frecency::empty());

        let results = index.search("close tab");
        assert_eq!(results[0].entry.action_id, "close.tab");
        assert_eq!(results[0].entry.key_hint.as_deref(), Some("ctrl+w"));
    }

    #[test]
    fn search_matches_by_raw_action_id_too() {
        let registry = test_registry();
        let index = Index::from_registry(&registry, Frecency::empty());

        let results = index.search("go_to.definition");
        assert_eq!(results[0].entry.action_id, "go_to.definition");
    }

    #[test]
    fn empty_query_returns_everything_ordered_by_frecency() {
        let registry = test_registry();
        let mut index = Index::from_registry(&registry, Frecency::empty());
        index.record_use("save");

        let results = index.search("");
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].entry.action_id, "save", "recently used entry should rank first");
    }

    #[test]
    fn merge_remote_enriches_without_dropping_local_key_hint() {
        let registry = test_registry();
        let mut index = Index::from_registry(&registry, Frecency::empty());

        index.merge_remote(
            "vscode",
            vec![RemoteAction {
                id: "close.tab".to_string(),
                native_command: "workbench.action.closeActiveEditor".to_string(),
                title: "Close Editor".to_string(),
            }],
        );

        let entry = index.entries().iter().find(|e| e.action_id == "close.tab").unwrap();
        assert_eq!(entry.title, "Close Editor");
        assert_eq!(entry.source, "vscode");
        assert_eq!(entry.native_command.as_deref(), Some("workbench.action.closeActiveEditor"));
        assert_eq!(entry.key_hint.as_deref(), Some("ctrl+w"), "local key binding should survive the merge");
    }

    #[test]
    fn merge_remote_namespaces_a_command_with_no_known_action_id() {
        let registry = test_registry();
        let mut index = Index::from_registry(&registry, Frecency::empty());

        index.merge_remote(
            "vscode",
            vec![RemoteAction {
                id: "editor.action.formatDocument".to_string(),
                native_command: "editor.action.formatDocument".to_string(),
                title: "Format Document".to_string(),
            }],
        );

        assert!(
            !index.entries().iter().any(|e| e.action_id == "editor.action.formatDocument"),
            "an id with no matching local Keylex action should never be left un-namespaced"
        );
        let entry = index
            .entries()
            .iter()
            .find(|e| e.action_id == "vscode:editor.action.formatDocument")
            .expect("an id with no local Keylex-action match should be namespaced by source");
        assert_eq!(entry.title, "Format Document");
        assert_eq!(entry.native_command.as_deref(), Some("editor.action.formatDocument"));
    }

    type RecordedCalls = Rc<RefCell<Vec<(String, String)>>>;

    struct RecordingAdapter(RecordedCalls);
    impl Adapter for RecordingAdapter {
        fn send(&self, target: &crate::config::Target, native_command: &str) {
            self.0.borrow_mut().push((target.program.clone(), native_command.to_string()));
        }
    }

    struct NoopNotifier;
    impl Notifier for NoopNotifier {
        fn show(&self, _message: &str) {}
    }

    struct NoopFallbackSender;
    impl FallbackSender for NoopFallbackSender {
        fn send(&self, _keycode: &str) {}
    }

    fn router_fixture(targets_toml: &str) -> (Registry, RecordedCalls) {
        let dir = temp_dir("dispatch-entry");
        std::fs::write(
            dir.join("vocabulary.toml"),
            "modifiers = [\"close\", \"save\", \"go_to\"]\nlocations = [\"tab\", \"definition\"]\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("actions.toml"),
            "[[action]]\nmodifier = \"close\"\nlocation = \"tab\"\nkey = \"ctrl+w\"\n",
        )
        .unwrap();
        std::fs::write(dir.join("targets.toml"), targets_toml).unwrap();
        let registry = Registry::load(&dir).expect("fixture config should load");
        let _ = std::fs::remove_dir_all(&dir);
        (registry, Rc::new(RefCell::new(Vec::new())))
    }

    #[test]
    fn dispatch_entry_routes_a_known_action_id_through_the_focus_aware_router() {
        let (registry, calls) = router_fixture(
            r#"[[target]]
program = "editor"
match_process = ["editor.exe"]
adapter = "socket"
exempt_command_grammar = true

  [target.supports]
  "close.tab" = "app.tab.close"
"#,
        );
        let mut adapters: HashMap<String, Box<dyn Adapter>> = HashMap::new();
        adapters.insert("socket".to_string(), Box::new(RecordingAdapter(Rc::clone(&calls))));
        let router = Router {
            registry: &registry,
            adapters,
            notifier: Box::new(NoopNotifier),
            fallback_sender: Box::new(NoopFallbackSender),
        };

        let entry = SpotlightEntry {
            action_id: "close.tab".to_string(),
            title: "Close Tab".to_string(),
            key_hint: Some("ctrl+w".to_string()),
            source: "local".to_string(),
            native_command: None,
        };
        let result = dispatch_entry(&entry, "editor.exe", &router);

        assert_eq!(result.status, DispatchStatus::Native);
        assert_eq!(calls.borrow().as_slice(), [("editor".to_string(), "app.tab.close".to_string())]);
    }

    #[test]
    fn dispatch_entry_sends_a_raw_command_straight_to_its_source_target_regardless_of_focus() {
        let (registry, calls) = router_fixture(
            r#"[[target]]
program = "vscode"
match_process = ["Code.exe"]
adapter = "socket"
"#,
        );
        let mut adapters: HashMap<String, Box<dyn Adapter>> = HashMap::new();
        adapters.insert("socket".to_string(), Box::new(RecordingAdapter(Rc::clone(&calls))));
        let router = Router {
            registry: &registry,
            adapters,
            notifier: Box::new(NoopNotifier),
            fallback_sender: Box::new(NoopFallbackSender),
        };

        let entry = SpotlightEntry {
            action_id: "vscode:editor.action.formatDocument".to_string(),
            title: "Format Document".to_string(),
            key_hint: None,
            source: "vscode".to_string(),
            native_command: Some("editor.action.formatDocument".to_string()),
        };
        // Deliberately not "Code.exe" -- a raw command has no cross-app
        // abstraction to route by focus, so this must dispatch regardless.
        let result = dispatch_entry(&entry, "totally-unrelated.exe", &router);

        assert_eq!(result.status, DispatchStatus::Native);
        assert_eq!(
            calls.borrow().as_slice(),
            [("vscode".to_string(), "editor.action.formatDocument".to_string())]
        );
    }

    #[test]
    fn frecency_persists_across_loads() {
        let dir = temp_dir("frecency-persist");
        let mut frecency = Frecency::load(&dir);
        frecency.touch("save");
        frecency.touch("save");

        let reloaded = Frecency::load(&dir);
        let now = now_secs();
        assert!(
            reloaded.boost("save", now) > 0,
            "usage recorded before reload should still boost after reload"
        );
        assert_eq!(reloaded.boost("never.used", now), 0);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
