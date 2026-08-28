//! Demysto's product logic.
//!
//! This crate deliberately depends on no user interface toolkit: it is the
//! single seam the test suite attaches to (see `docs/spec/0001-v1-text-actions.md`).
//! The Tauri layer in `src-tauri` is a set of thin adapters over the [`Demysto`]
//! facade defined here, and nothing in this crate may reference Tauri types.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

mod capture;
mod desktop;
mod paths;
mod selection;

pub use capture::{Capture, CaptureError, CaptureOutcome, Captured};
pub use paths::{config_dir, ConfigDirError, CONFIG_DIR_ENV};
pub use selection::Selection;

/// The facade every user interface talks to.
pub struct Demysto {
    config_dir: PathBuf,
    version: String,
    capture: Box<dyn Capture>,
    /// The last Capture, so that a Palette which loads after one still finds it.
    last_capture: Mutex<Option<CaptureOutcome>>,
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
        Self::with_capture(config_dir, version, desktop::for_platform())
    }

    /// Builds a facade over a Capture chosen by the caller, which is how the
    /// test suite keeps the desktop out of it.
    pub fn with_capture(
        config_dir: impl Into<PathBuf>,
        version: impl Into<String>,
        capture: Box<dyn Capture>,
    ) -> Self {
        Self {
            config_dir: config_dir.into(),
            version: version.into(),
            capture,
            last_capture: Mutex::new(None),
        }
    }

    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// Obtains a Selection from the foreground application or the clipboard,
    /// and remembers it.
    pub fn capture(&self) -> CaptureOutcome {
        let outcome = CaptureOutcome::from(self.capture.capture());
        *self.last_capture.lock().unwrap() = Some(outcome.clone());

        outcome
    }

    /// What the last Capture produced, or `None` before there has been one.
    pub fn last_capture(&self) -> Option<CaptureOutcome> {
        self.last_capture.lock().unwrap().clone()
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
    //! Every test calls the facade the Tauri commands call, with the desktop
    //! substituted at its edge — the one seam of the spec's *Testing Decisions*.

    use std::sync::Arc;

    use super::*;
    use crate::capture::fake::{self, FakeDesktop};

    fn demysto(capture: Box<dyn Capture>) -> Demysto {
        Demysto::with_capture("/somewhere/demysto", "1.2.3", capture)
    }

    fn captured(demysto: &Demysto) -> Captured {
        match demysto.capture() {
            CaptureOutcome::Captured(captured) => captured,
            CaptureOutcome::Failed(error) => panic!("the Capture failed: {error}"),
        }
    }

    #[test]
    fn captures_the_text_selected_in_the_foreground_application() {
        let desktop = Arc::new(FakeDesktop::new(None, Some("Ceci n'est pas une pipe")));

        assert_eq!(
            captured(&demysto(fake::over(&desktop))),
            Captured::Selection(Selection::text("Ceci n'est pas une pipe"))
        );
    }

    #[test]
    fn leaves_the_clipboard_holding_what_it_held_before() {
        let desktop = Arc::new(FakeDesktop::new(
            Some("a receipt"),
            Some("Ceci n'est pas une pipe"),
        ));

        demysto(fake::over(&desktop)).capture();

        assert_eq!(desktop.clipboard_now(), Some("a receipt".to_owned()));
    }

    #[test]
    fn a_copy_that_brings_nothing_worth_showing_still_puts_the_clipboard_back() {
        // A blank line, or an image where only text can be read: the copy
        // landed and overwrote the clipboard, and that it brought nothing this
        // can show is no reason to leave the user without what they had.
        let desktop = Arc::new(FakeDesktop::new(Some("a receipt"), Some("   ")));

        demysto(fake::over(&desktop)).capture();

        assert_eq!(desktop.clipboard_now(), Some("a receipt".to_owned()));
    }

    #[test]
    fn a_clipboard_that_cannot_be_written_back_still_yields_the_selection() {
        // The text has already been read. Failing to put the clipboard back is
        // a worse outcome for the user, not a reason to throw the Capture away
        // and make them press the Hotkey again.
        let desktop =
            Arc::new(FakeDesktop::new(None, Some("Ceci n'est pas une pipe")).refusing_to_restore());

        assert_eq!(
            captured(&demysto(fake::over(&desktop))),
            Captured::Selection(Selection::text("Ceci n'est pas une pipe"))
        );
    }

    #[test]
    fn empties_the_clipboard_again_when_it_was_empty_before() {
        let desktop = Arc::new(FakeDesktop::new(None, Some("Ceci n'est pas une pipe")));

        demysto(fake::over(&desktop)).capture();

        assert_eq!(desktop.clipboard_now(), None);
    }

    #[test]
    fn waits_for_a_copy_that_takes_a_moment_to_land() {
        let desktop = Arc::new(FakeDesktop::new(None, Some("late")).landing_after(3));

        assert_eq!(
            captured(&demysto(fake::over(&desktop))),
            Captured::Selection(Selection::text("late"))
        );
    }

    #[test]
    fn falls_back_to_the_clipboard_when_nothing_was_selected() {
        let desktop = Arc::new(FakeDesktop::new(Some("copied a moment ago"), None));

        assert_eq!(
            captured(&demysto(fake::over(&desktop))),
            Captured::Clipboard(Selection::text("copied a moment ago"))
        );
    }

    #[test]
    fn a_capture_that_finds_nothing_leaves_the_clipboard_alone() {
        let desktop = Arc::new(FakeDesktop::new(Some("copied a moment ago"), None));

        demysto(fake::over(&desktop)).capture();

        assert_eq!(
            desktop.clipboard_now(),
            Some("copied a moment ago".to_owned())
        );
    }

    #[test]
    fn nothing_selected_and_an_empty_clipboard_is_an_outcome_of_its_own() {
        let desktop = Arc::new(FakeDesktop::new(None, None));

        assert_eq!(captured(&demysto(fake::over(&desktop))), Captured::Nothing);
    }

    #[test]
    fn a_clipboard_holding_only_whitespace_counts_as_empty() {
        let desktop = Arc::new(FakeDesktop::new(Some("  \n "), None));

        assert_eq!(captured(&demysto(fake::over(&desktop))), Captured::Nothing);
    }

    #[test]
    fn a_selection_identical_to_the_clipboard_still_reaches_the_palette() {
        let desktop = Arc::new(FakeDesktop::new(
            Some("the same words"),
            Some("the same words"),
        ));

        // Indistinguishable from nothing having been selected, so it is reported
        // as the clipboard — the text is right either way, only the label differs.
        assert_eq!(
            captured(&demysto(fake::over(&desktop))),
            Captured::Clipboard(Selection::text("the same words"))
        );
    }

    #[test]
    fn a_clipboard_that_cannot_be_read_is_reported_rather_than_guessed_at() {
        struct Broken;

        impl Capture for Broken {
            fn capture(&self) -> Result<Captured, CaptureError> {
                Err(CaptureError::Clipboard("no owner".to_owned()))
            }
        }

        assert_eq!(
            demysto(Box::new(Broken)).capture(),
            CaptureOutcome::Failed(CaptureError::Clipboard("no owner".to_owned()))
        );
    }

    #[test]
    fn a_wayland_session_reads_what_the_user_copied_themselves() {
        let desktop = Arc::new(FakeDesktop::new(
            Some("copied by hand"),
            Some("selected but unreachable"),
        ));

        // The Selection is there and stays there: reaching for it would mean
        // typing into another application, which Wayland does not allow. See
        // ADR-0003.
        assert_eq!(
            captured(&demysto(fake::clipboard_only_over(&desktop))),
            Captured::Clipboard(Selection::text("copied by hand"))
        );
        assert_eq!(desktop.clipboard_now(), Some("copied by hand".to_owned()));
    }

    #[test]
    fn a_wayland_session_with_an_empty_clipboard_reports_nothing() {
        let desktop = Arc::new(FakeDesktop::new(None, Some("selected but unreachable")));

        assert_eq!(
            captured(&demysto(fake::clipboard_only_over(&desktop))),
            Captured::Nothing
        );
    }

    #[test]
    fn the_last_capture_is_remembered_for_a_palette_that_opens_after_it() {
        let desktop = Arc::new(FakeDesktop::new(None, Some("a paragraph")));
        let demysto = demysto(fake::over(&desktop));

        demysto.capture();

        assert_eq!(
            demysto.last_capture(),
            Some(CaptureOutcome::Captured(Captured::Selection(
                Selection::text("a paragraph")
            )))
        );
    }

    #[test]
    fn nothing_is_remembered_before_the_first_capture() {
        let desktop = Arc::new(FakeDesktop::new(None, None));

        assert_eq!(demysto(fake::over(&desktop)).last_capture(), None);
    }

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
