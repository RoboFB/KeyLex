//! The one error type the three config layers report.

use std::fmt;

/// Anything that stops the config from loading: a file that can't be read
/// or parsed, a word missing from `vocabulary.toml`, a malformed chord, a
/// target claiming an action nothing declares.
///
/// The message is the whole payload. Every caller does the same thing with
/// every kind — report it and refuse to start — so there is nothing here
/// worth matching on, and a variant per rule would only be ceremony.
#[derive(Debug)]
pub struct ConfigError(String);

impl ConfigError {
    pub(crate) fn new(message: impl Into<String>) -> ConfigError {
        ConfigError(message.into())
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ConfigError {}
