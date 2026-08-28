//! Demysto's product logic.
//!
//! This crate deliberately depends on no user interface toolkit: it is the
//! single seam the test suite attaches to (see `docs/spec/0001-v1-text-actions.md`).
//! The Tauri layer in `src-tauri` is a set of thin adapters over the [`Demysto`]
//! facade defined here, and nothing in this crate may reference Tauri types.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

mod capture;
mod config;
mod desktop;
mod paths;
mod provider;
mod run;
mod selection;
mod sse;
mod stream;

pub use capture::{Capture, CaptureError, CaptureOutcome, Captured};
pub use paths::{config_dir, ConfigDirError, CONFIG_DIR_ENV};
pub use run::{RunError, RunOutcome};
pub use selection::Selection;

use config::{Config, ConfigError};
use stream::Assembly;

/// The facade every user interface talks to.
pub struct Demysto {
    config_dir: PathBuf,
    version: String,
    capture: Box<dyn Capture>,
    /// The settings as they were at startup, however that went. A file that
    /// cannot be used is no reason to refuse to start: the Palette still opens,
    /// and the Run is where the user is told what to fix.
    config: Result<Config, ConfigError>,
    /// The last Capture, so that a Palette which loads after one still finds it.
    last_capture: Mutex<Option<CaptureOutcome>>,
    /// The last Run, for the same reason: the result window is shown while the
    /// request is still in flight, so it loads after the Run it is showing.
    last_run: Mutex<Option<RunOutcome>>,
    /// How often a Run under way hands over what has arrived so far. A field
    /// rather than a constant so that the suite can take the waiting out of it,
    /// the way it takes it out of a Capture.
    throttle: Duration,
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
        let config_dir = config_dir.into();

        Self {
            // Read once, here, and nowhere else in the crate: the environment
            // holds the key, and a key that can change under a running Demysto
            // is a key nobody can reason about (the spec's *Core modules*).
            config: config::load(&config_dir, &config::SystemEnv),
            config_dir,
            version: version.into(),
            capture,
            last_capture: Mutex::new(None),
            last_run: Mutex::new(None),
            throttle: stream::THROTTLE,
        }
    }

    /// The same facade with nothing held back, so that a test can see every
    /// state a Run passed through rather than the few a clock let out.
    #[cfg(test)]
    fn unthrottled(mut self) -> Self {
        self.throttle = Duration::ZERO;
        self
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

    /// Forgets the last Run.
    ///
    /// The interface shows the result window for a Run that is about to begin,
    /// and a window shown before there is an answer asks what the last one was
    /// as it loads. Told first to forget, it comes up saying it is asking —
    /// which is what the user should be looking at, rather than the answer to
    /// the question before this one.
    pub fn forget_last_run(&self) {
        *self.last_run.lock().unwrap() = None;
    }

    /// Runs the built-in explain Action against the last Capture, showing the
    /// answer as it arrives, and remembers what it produced.
    ///
    /// The Selection comes from the Capture the core already holds rather than
    /// from the interface: what gets explained is what Demysto read, not what a
    /// window says it read. Ticket 05 gives the caller an Action to choose.
    ///
    /// `showing` is handed the whole answer so far, render-ready, every so
    /// often — see [`stream`] for what "render-ready" and "so often" mean and
    /// why they are decided here rather than in the window.
    pub fn run(&self, showing: impl FnMut(&str)) -> RunOutcome {
        // Cleared before the request as well as written after it, so that the
        // window has nothing stale to find however it got here — the interface
        // forgets the last Run before it shows the window, and a caller that
        // did not still cannot leave one on screen.
        self.forget_last_run();

        let outcome = RunOutcome::from(self.answer(showing));
        *self.last_run.lock().unwrap() = Some(outcome.clone());

        outcome
    }

    /// What the last Run produced, `None` before there has been one and while
    /// one is under way.
    pub fn last_run(&self) -> Option<RunOutcome> {
        self.last_run.lock().unwrap().clone()
    }

    fn answer(&self, mut showing: impl FnMut(&str)) -> Result<String, RunError> {
        let captured = self.last_capture();
        let selection = captured
            .as_ref()
            .and_then(CaptureOutcome::selection)
            .ok_or_else(run::nothing_to_run)?;

        let config = self
            .config
            .as_ref()
            .map_err(|error| RunError::Configuration(error.to_string()))?;

        let mut assembly = Assembly::new(self.throttle);

        provider::answer(
            &config.provider,
            &run::explain(selection.as_text()),
            |fragment| {
                if let Some(answer) = assembly.push(fragment) {
                    showing(&answer);
                }
            },
        )?;

        Ok(assembly.text())
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

    use mockito::{Matcher, Server, ServerGuard};
    use serde_json::json;
    use tempfile::TempDir;

    use super::*;
    use crate::capture::fake::{self, FakeDesktop};

    /// A Demysto with a configuration directory of its own, kept for as long as
    /// the test holds it.
    ///
    /// The facade reads its settings as it is built, so every test gets a
    /// directory nobody else is writing to: the temporary directory standing in
    /// for the config location, per the spec's *Testing Decisions*.
    struct Rooted {
        demysto: Demysto,
        _dir: TempDir,
    }

    impl std::ops::Deref for Rooted {
        type Target = Demysto;

        fn deref(&self) -> &Demysto {
            &self.demysto
        }
    }

    impl Rooted {
        /// The same Demysto with the throttle a Run really has, for the tests
        /// that are about what the clock holds back rather than what arrives.
        fn throttled(mut self) -> Self {
            self.demysto.throttle = stream::THROTTLE;
            self
        }
    }

    /// A Demysto with nothing configured to talk to.
    fn demysto(capture: Box<dyn Capture>) -> Rooted {
        rooted(capture, None)
    }

    /// A Demysto configured with one Provider, at `base_url`.
    ///
    /// The Provider names no preset and no variable of its own, so that the
    /// environment of whoever is running the suite cannot reach into it.
    fn demysto_asking(capture: Box<dyn Capture>, base_url: &str) -> Rooted {
        rooted(capture, Some(base_url))
    }

    fn rooted(capture: Box<dyn Capture>, base_url: Option<&str>) -> Rooted {
        let dir = TempDir::new().unwrap();

        if let Some(base_url) = base_url {
            std::fs::write(
                dir.path().join(config::FILE_NAME),
                format!(
                    "version = 1\n\n[[providers]]\nname = \"a provider\"\n\
                     base_url = \"{base_url}\"\nmodel = \"a-model\"\napi_key = \"a-key\"\n"
                ),
            )
            .unwrap();
        }

        Rooted {
            demysto: Demysto::with_capture(dir.path(), "1.2.3", capture).unthrottled(),
            _dir: dir,
        }
    }

    /// The body an OpenAI-compatible Provider answers with, one event per
    /// fragment, ending as the contract says a stream ends.
    fn streaming(fragments: &[&str]) -> String {
        fragments
            .iter()
            .map(|fragment| json!({ "choices": [{ "delta": { "content": fragment } }] }))
            .map(|event| format!("data: {event}\n\n"))
            .chain(std::iter::once("data: [DONE]\n\n".to_owned()))
            .collect()
    }

    /// The same, for an answer that arrives in one piece.
    fn answering(answer: &str) -> String {
        streaming(&[answer])
    }

    /// A Run whose intermediate states nobody is watching.
    fn run(demysto: &Demysto) -> RunOutcome {
        demysto.run(|_| {})
    }

    /// Every state a Run put on screen, and what it finally produced.
    fn watching(demysto: &Demysto) -> (Vec<String>, RunOutcome) {
        let mut shown = Vec::new();
        let outcome = demysto.run(|answer| shown.push(answer.to_owned()));

        (shown, outcome)
    }

    /// A Demysto that has captured `selection` and is pointed at `server`.
    fn ready_to_run(server: &ServerGuard, selection: &str) -> Rooted {
        let desktop = Arc::new(FakeDesktop::new(None, Some(selection)));
        let demysto = demysto_asking(fake::over(&desktop), &format!("{}/v1", server.url()));
        demysto.capture();

        demysto
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
        let dir = TempDir::new().unwrap();
        let demysto = Demysto::new(dir.path(), "1.2.3");

        assert_eq!(demysto.status().config_dir, dir.path());
    }

    #[test]
    fn status_reports_the_version_it_was_built_with() {
        let dir = TempDir::new().unwrap();
        let demysto = Demysto::new(dir.path(), "1.2.3");

        assert_eq!(demysto.status().version, "1.2.3");
    }

    #[test]
    fn the_selection_reaches_the_provider_and_its_answer_comes_back() {
        let mut server = Server::new();
        let endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_header("content-type", "application/json")
            .with_body(answering("A painting of a pipe is not a pipe."))
            .create();

        let outcome = run(&ready_to_run(&server, "Ceci n'est pas une pipe"));

        endpoint.assert();
        assert_eq!(
            outcome,
            RunOutcome::Answered("A painting of a pipe is not a pipe.".to_owned())
        );
    }

    #[test]
    fn the_request_carries_the_key_the_model_and_the_selection() {
        let mut server = Server::new();
        let endpoint = server
            .mock("POST", "/v1/chat/completions")
            .match_header("authorization", "Bearer a-key")
            .match_header("content-type", "application/json")
            .match_body(Matcher::AllOf(vec![
                Matcher::PartialJson(json!({
                    "model": "a-model",
                    "stream": true,
                    "messages": [{ "role": "user" }],
                })),
                // The Selection itself, wherever the prompt around it puts it.
                Matcher::Regex("Ceci n'est pas une pipe".to_owned()),
            ]))
            .with_body(answering("A painting of a pipe is not a pipe."))
            .create();

        run(&ready_to_run(&server, "Ceci n'est pas une pipe"));

        endpoint.assert();
    }

    #[test]
    fn a_base_url_with_a_trailing_slash_still_reaches_the_endpoint() {
        let mut server = Server::new();
        let endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body(answering("an answer"))
            .create();

        let desktop = Arc::new(FakeDesktop::new(None, Some("a paragraph")));
        let demysto = demysto_asking(fake::over(&desktop), &format!("{}/v1/", server.url()));
        demysto.capture();
        run(&demysto);

        endpoint.assert();
    }

    #[test]
    fn a_provider_that_refuses_says_so_in_its_own_words() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_status(401)
            .with_body(json!({ "error": { "message": "Incorrect API key provided" } }).to_string())
            .create();

        let RunOutcome::Failed(RunError::Provider(message)) =
            run(&ready_to_run(&server, "a paragraph"))
        else {
            panic!("a refusal should be reported as one");
        };

        assert!(message.contains("Incorrect API key provided"), "{message}");
        assert!(message.contains("401"), "{message}");
    }

    #[test]
    fn a_refusal_with_nothing_to_say_is_still_reported() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_status(500)
            .with_body("<html>Bad Gateway</html>")
            .create();

        let RunOutcome::Failed(RunError::Provider(message)) =
            run(&ready_to_run(&server, "a paragraph"))
        else {
            panic!("a refusal should be reported as one");
        };

        assert!(message.contains("500"), "{message}");
    }

    #[test]
    fn the_answer_arrives_a_piece_at_a_time_rather_than_all_at_once() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body(streaming(&["A painting ", "of a pipe ", "is not a pipe."]))
            .create();

        let (shown, outcome) = watching(&ready_to_run(&server, "Ceci n'est pas une pipe"));

        // Each state carries the whole answer so far rather than the piece that
        // just landed: a window that missed one is corrected by the next.
        assert_eq!(
            shown,
            [
                "A painting ",
                "A painting of a pipe ",
                "A painting of a pipe is not a pipe.",
            ]
        );
        assert_eq!(
            outcome,
            RunOutcome::Answered("A painting of a pipe is not a pipe.".to_owned())
        );
    }

    #[test]
    fn a_code_fence_is_closed_in_every_state_the_user_is_shown() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body(streaming(&[
                "Like so:\n\n```",
                "rust\nfn main",
                "() {}\n```",
            ]))
            .create();

        let (shown, outcome) = watching(&ready_to_run(&server, "fn main"));

        // The block is a block from the moment it opens, so a renderer shown
        // these in turn never draws it as prose and then redraws it as code.
        assert_eq!(
            shown,
            [
                "Like so:\n\n```\n```",
                "Like so:\n\n```rust\nfn main\n```",
                "Like so:\n\n```rust\nfn main() {}\n```",
            ]
        );

        // The answer itself is what arrived, with nothing added: the fence it
        // ends on is the Model's own.
        assert_eq!(
            outcome,
            RunOutcome::Answered("Like so:\n\n```rust\nfn main() {}\n```".to_owned())
        );
    }

    #[test]
    fn the_answer_is_the_fragments_exactly_as_they_arrived() {
        let fragments = ["  A pipe", "\n\n- one\n", "- two", "  "];

        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body(streaming(&fragments))
            .create();

        assert_eq!(
            run(&ready_to_run(&server, "a paragraph")),
            RunOutcome::Answered(fragments.concat())
        );
    }

    #[test]
    fn the_answer_is_the_whole_of_what_arrived_however_little_the_throttle_showed() {
        let fragments = ["A painting ", "of a pipe ", "is not a pipe."];

        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body(streaming(&fragments))
            .create();

        let demysto = ready_to_run(&server, "Ceci n'est pas une pipe").throttled();
        let (shown, outcome) = watching(&demysto);

        // Every piece lands well inside one throttle window, so the user is
        // shown fewer states than there were pieces — and the answer is still
        // every piece, which is the whole point of throttling what is shown
        // rather than what is kept.
        assert!(shown.len() < fragments.len(), "{shown:?}");
        assert_eq!(outcome, RunOutcome::Answered(fragments.concat()));
    }

    #[test]
    fn the_event_that_opens_a_stream_is_not_a_state_worth_showing() {
        // The contract's first event carries the role and no text at all.
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body(
                "data: {\"choices\":[{\"delta\":{\"role\":\"assistant\"}}]}\n\n\
                 data: {\"choices\":[{\"delta\":{\"content\":\"an answer\"}}]}\n\n\
                 data: [DONE]\n\n",
            )
            .create();

        let (shown, outcome) = watching(&ready_to_run(&server, "a paragraph"));

        assert_eq!(shown, ["an answer"]);
        assert_eq!(outcome, RunOutcome::Answered("an answer".to_owned()));
    }

    #[test]
    fn a_keep_alive_between_the_fragments_is_not_one_of_them() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body(
                ": keep-alive\n\n\
                 data: {\"choices\":[{\"delta\":{\"content\":\"an answer\"}}]}\n\n\
                 data: [DONE]\n\n",
            )
            .create();

        assert_eq!(
            run(&ready_to_run(&server, "a paragraph")),
            RunOutcome::Answered("an answer".to_owned())
        );
    }

    #[test]
    fn an_event_that_is_not_the_contracts_shape_is_reported_as_such() {
        // A Provider reporting an error mid-stream sends exactly this, and
        // passing over what it says would leave the user with a blank window.
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body("data: {\"error\":{\"message\":\"rate limited\"}}\n\n")
            .create();

        assert!(matches!(
            run(&ready_to_run(&server, "a paragraph")),
            RunOutcome::Failed(RunError::Malformed(_))
        ));
    }

    #[test]
    fn a_stream_that_carries_no_answer_is_reported_as_such() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body(json!({ "choices": [] }).to_string())
            .create();

        assert!(matches!(
            run(&ready_to_run(&server, "a paragraph")),
            RunOutcome::Failed(RunError::Malformed(_))
        ));
    }

    #[test]
    fn an_answer_that_is_not_json_at_all_is_reported_rather_than_shown() {
        // What a proxy or a captive portal answers with, on the way to an
        // endpoint the user is sure they configured correctly.
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body("<html>Sign in to continue</html>")
            .create();

        assert!(matches!(
            run(&ready_to_run(&server, "a paragraph")),
            RunOutcome::Failed(RunError::Malformed(_))
        ));
    }

    #[test]
    fn a_provider_that_cannot_be_reached_names_the_address_that_did_not_answer() {
        let desktop = Arc::new(FakeDesktop::new(None, Some("a paragraph")));
        let demysto = demysto_asking(fake::over(&desktop), "http://127.0.0.1:1/v1");
        demysto.capture();

        let RunOutcome::Failed(RunError::Unreachable(message)) = run(&demysto) else {
            panic!("an endpoint that never answered should be reported as unreachable");
        };

        assert!(message.contains("127.0.0.1:1"), "{message}");
    }

    #[test]
    fn a_run_with_nothing_captured_sends_nothing_anywhere() {
        // The Provider is an address that refuses connections, so anything but
        // this outcome would mean a request went out.
        let desktop = Arc::new(FakeDesktop::new(None, None));
        let demysto = demysto_asking(fake::over(&desktop), "http://127.0.0.1:1/v1");
        demysto.capture();

        assert!(matches!(
            run(&demysto),
            RunOutcome::Failed(RunError::NothingToRun(_))
        ));
    }

    #[test]
    fn a_run_with_no_provider_configured_names_the_file_that_would_configure_one() {
        let desktop = Arc::new(FakeDesktop::new(None, Some("a paragraph")));
        let demysto = demysto(fake::over(&desktop));
        demysto.capture();

        let RunOutcome::Failed(RunError::Configuration(message)) = run(&demysto) else {
            panic!("a Run with nothing configured should say what to configure");
        };

        assert!(message.contains(config::FILE_NAME), "{message}");
    }

    #[test]
    fn the_last_run_is_remembered_for_a_window_that_opens_after_it() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body(answering("an answer"))
            .create();

        let demysto = ready_to_run(&server, "a paragraph");
        run(&demysto);

        assert_eq!(
            demysto.last_run(),
            Some(RunOutcome::Answered("an answer".to_owned()))
        );
    }

    #[test]
    fn nothing_is_remembered_before_the_first_run() {
        let desktop = Arc::new(FakeDesktop::new(None, None));

        assert_eq!(demysto(fake::over(&desktop)).last_run(), None);
    }
}
