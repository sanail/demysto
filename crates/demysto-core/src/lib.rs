//! Demysto's product logic.
//!
//! This crate deliberately depends on no user interface toolkit: it is the
//! single seam the test suite attaches to (see `docs/spec/0001-v1-text-actions.md`).
//! The Tauri layer in `src-tauri` is a set of thin adapters over the [`Demysto`]
//! facade defined here, and nothing in this crate may reference Tauri types.

use std::path::{Path, PathBuf};

mod paths;

pub use paths::{config_dir, ConfigDirError, CONFIG_DIR_ENV};

/// The facade every user interface talks to.
pub struct Demysto {
    config_dir: PathBuf,
    version: String,
}

/// What the application can report about itself before anything is configured.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Status {
    /// The running version of the application.
    pub version: String,
    /// Where this instance reads and writes its configuration.
    pub config_dir: PathBuf,
}

impl Demysto {
    /// Builds a facade rooted at an explicit configuration directory.
    ///
    /// The version is supplied by the caller rather than read from this crate's
    /// own `CARGO_PKG_VERSION`: what the user is running is the application, and
    /// the library's version is nobody's business but the build's.
    pub fn new(config_dir: impl Into<PathBuf>, version: impl Into<String>) -> Self {
        Self {
            config_dir: config_dir.into(),
            version: version.into(),
        }
    }

    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    pub fn status(&self) -> Status {
        Status {
            version: self.version.clone(),
            config_dir: self.config_dir.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_reports_the_config_dir_it_was_built_with() {
        let demysto = Demysto::new("/somewhere/demysto", "1.2.3");

        assert_eq!(
            demysto.status().config_dir,
            PathBuf::from("/somewhere/demysto")
        );
    }

    #[test]
    fn status_reports_the_version_it_was_built_with() {
        let demysto = Demysto::new("/somewhere/demysto", "1.2.3");

        assert_eq!(demysto.status().version, "1.2.3");
    }
}
