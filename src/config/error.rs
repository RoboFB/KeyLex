//! The one error type the three config layers report.

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum ConfigError {
    Io {
        path: PathBuf,
        source: io::Error,
    },
    /// Boxed only because `toml`'s error is large enough to weigh down
    /// every `Result` in this module otherwise.
    Parse {
        path: PathBuf,
        source: Box<toml::de::Error>,
    },
    /// The files parsed, but their contents don't hold together: a word
    /// missing from `vocabulary.toml`, a malformed chord, a target
    /// supporting an action nothing declares. There is deliberately no
    /// variant per rule -- every caller does the same thing with all of
    /// them (report the message and refuse to start), so the message is
    /// the whole payload.
    Invalid(String),
}

impl ConfigError {
    pub(crate) fn io(path: &Path, source: io::Error) -> ConfigError {
        ConfigError::Io {
            path: path.to_path_buf(),
            source,
        }
    }

    pub(crate) fn parse(path: &Path, source: toml::de::Error) -> ConfigError {
        ConfigError::Parse {
            path: path.to_path_buf(),
            source: Box::new(source),
        }
    }

    pub(crate) fn invalid(message: impl Into<String>) -> ConfigError {
        ConfigError::Invalid(message.into())
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io { path, source } => {
                write!(f, "could not read {}: {source}", path.display())
            }
            ConfigError::Parse { path, source } => {
                write!(f, "could not parse {}: {source}", path.display())
            }
            ConfigError::Invalid(message) => write!(f, "invalid config: {message}"),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Io { source, .. } => Some(source),
            ConfigError::Parse { source, .. } => Some(source),
            ConfigError::Invalid(_) => None,
        }
    }
}
