//! Optional zoxide-style "last used" tracking: a small on-disk score per
//! action id, combining how often and how recently it was picked from the
//! spotlight. Never required -- the default is "nothing has ever been
//! used", which simply contributes no ranking bonus.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

const FILE_NAME: &str = "spotlight_frecency.json";

pub(super) fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_secs())
        .unwrap_or_default()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Record {
    count: u32,
    last_used_secs: u64,
}

/// Usage stats for ranking. `Frecency::default()` is the in-memory-only
/// tracker that never touches disk, for callers that don't want a search
/// to leave a trace.
#[derive(Debug, Clone, Default)]
pub struct Frecency {
    records: HashMap<String, Record>,
    path: Option<PathBuf>,
}

impl Frecency {
    /// Loads persisted stats, starting empty if the file doesn't exist yet
    /// or no longer parses.
    pub fn load(config_dir: &Path) -> Frecency {
        let path = config_dir.join(FILE_NAME);
        let records = std::fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default();
        Frecency {
            records,
            path: Some(path),
        }
    }

    /// Records one use of `action_id` and persists immediately. A write
    /// failure is ignored on purpose: frecency is a ranking hint, not state
    /// worth failing a dispatch over.
    pub fn touch(&mut self, action_id: &str) {
        let record = self.records.entry(action_id.to_string()).or_default();
        record.count += 1;
        record.last_used_secs = now_secs();

        let Some(path) = &self.path else { return };
        if let Ok(json) = serde_json::to_string_pretty(&self.records) {
            let _ = std::fs::write(path, json);
        }
    }

    /// A small bonus added on top of the fuzzy match score -- never large
    /// enough for a well-used but poorly matching entry to outrank a strong
    /// match, only enough to break near-ties toward what's actually used.
    pub(super) fn boost(&self, action_id: &str, now: u64) -> u32 {
        let Some(record) = self.records.get(action_id) else {
            return 0;
        };
        let recency = match now.saturating_sub(record.last_used_secs) / 86_400 {
            0 => 1.0,
            1..=7 => 0.5,
            8..=30 => 0.2,
            _ => 0.05,
        };
        (f64::from(record.count) * recency * 20.0) as u32
    }
}
