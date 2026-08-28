//! Demysto's product logic.
//!
//! This crate deliberately depends on no user interface toolkit: it is the
//! single seam the test suite attaches to (see `docs/spec/0001-v1-text-actions.md`).
//! The Tauri layer in `src-tauri` is a set of thin adapters over the [`Demysto`]
//! facade defined here, and nothing in this crate may reference Tauri types.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

mod action;
mod capture;
mod config;
mod conversation;
mod desktop;
mod language;
mod paths;
mod provider;
mod run;
mod selection;
mod sse;
mod stream;

pub use action::{Action, Parameter};
pub use capture::{Capture, CaptureError, CaptureOutcome, Captured};
pub use conversation::{Conversation, Summary, Turn};
pub use paths::{config_dir, ConfigDirError, CONFIG_DIR_ENV};
pub use run::{RunError, RunOutcome};
pub use selection::Selection;

use config::{Config, ConfigError};
use conversation::Store;
use run::Stopping;
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
    /// This session's Conversations, and which of them the result window is
    /// showing. Held here for the reason the last Capture is: the window is
    /// shown while the request is still in flight, so it loads after the
    /// Conversation it is showing was opened.
    store: Mutex<Store>,
    /// The stop signal of the Turn under way, so that Stop — pressed on the
    /// thread the window is drawn on — reaches the thread inside the request.
    /// One rather than one per Turn, because the interface holds a flag that
    /// keeps two Turns from being under way at once.
    stopping: Mutex<Option<Stopping>>,
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
            store: Mutex::new(Store::new()),
            stopping: Mutex::new(None),
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

    /// Declares the Run that is about to begin, opening the Conversation it
    /// will fill and putting that on screen.
    ///
    /// The interface shows the result window for a Run that is about to begin,
    /// and a window shown before there is an answer asks the core what it is
    /// looking at as it loads. Told this first, it comes up saying it is asking,
    /// under the name of the Action it is asking for — rather than the answer to
    /// the question before this one, under that question's name.
    pub fn about_to_run(&self, action: &str) {
        self.conversation_for(action);
    }

    /// Declares the follow-up Turn that is about to be asked, so that the
    /// window shows the question while the Model is still answering it.
    ///
    /// Answers with whether there was a Conversation to ask it in.
    pub fn about_to_follow_up(&self, question: &str) -> bool {
        self.store.lock().unwrap().follow_up(question).is_some()
    }

    /// The Conversation the result window is showing, `None` before there has
    /// been one.
    pub fn conversation(&self) -> Option<Conversation> {
        self.store.lock().unwrap().showing().cloned()
    }

    /// This session's Conversations, newest first, as the list of them reads.
    pub fn conversations(&self) -> Vec<Summary> {
        self.store.lock().unwrap().summaries()
    }

    /// Puts an earlier Conversation on screen, and answers with it. `None` when
    /// the session no longer holds one by that name.
    pub fn show_conversation(&self, id: u64) -> Option<Conversation> {
        self.store.lock().unwrap().show(id).cloned()
    }

    /// Stops the Turn under way, keeping what has already arrived. Does nothing
    /// when there is no Turn to stop.
    pub fn stop(&self) {
        if let Some(stopping) = self.stopping.lock().unwrap().as_ref() {
            stopping.stop();
        }
    }

    /// The Actions that accept the last Capture, in the order the Palette
    /// lists them.
    ///
    /// Filtered against the Capture the core already holds rather than against
    /// a kind the interface names, for the reason [`Self::run`] takes its
    /// Selection from there: the Palette offers what can be run on what Demysto
    /// read. Nothing captured is no Actions — there is nothing for one to
    /// operate on, and the Palette says so instead.
    pub fn actions(&self) -> Vec<Action> {
        let captured = self.last_capture();
        let Some(selection) = captured.as_ref().and_then(CaptureOutcome::selection) else {
            return Vec::new();
        };

        action::built_in()
            .into_iter()
            .filter(|action| action.accepts(selection.kind()))
            .collect()
    }

    /// Runs the Action named by `action` against the last Capture in a
    /// Conversation of its own, showing the answer as it arrives.
    ///
    /// The Selection comes from the Capture the core already holds rather than
    /// from the interface: what gets run on is what Demysto read, not what a
    /// window says it read. `parameters` is what the Palette collected for the
    /// Parameters the Action declares; one it did not collect falls back to
    /// what that Parameter offered.
    ///
    /// `showing` is handed the whole answer so far, render-ready, every so
    /// often — see [`stream`] for what "render-ready" and "so often" mean and
    /// why they are decided here rather than in the window.
    pub fn run(
        &self,
        action: &str,
        parameters: &BTreeMap<String, String>,
        showing: impl FnMut(&str),
    ) -> RunOutcome {
        // Declared here as well as by the interface, so that the window has
        // nothing stale to find however it got here — the interface declares
        // the Run before it shows the window, and a caller that did not still
        // cannot leave the last one on screen.
        let id = self.conversation_for(action);
        let outcome = self.opening_turn(id, action, parameters, showing);

        self.store.lock().unwrap().answered(id, outcome.clone());

        outcome
    }

    /// Asks a follow-up in the Conversation on screen, showing the answer as it
    /// arrives, and adds the Turn to it.
    ///
    /// The question is the whole of what is sent: the Selection and everything
    /// said about it so far travel as the Turns before this one, which is what
    /// makes a follow-up cost nothing but typing.
    pub fn follow_up(&self, question: &str, showing: impl FnMut(&str)) -> RunOutcome {
        let Some(id) = self.store.lock().unwrap().follow_up(question) else {
            return RunOutcome::Failed(run::no_conversation());
        };

        let outcome = self.ask(id, question.to_owned(), showing);
        self.store.lock().unwrap().answered(id, outcome.clone());

        outcome
    }

    /// Opens the Conversation a Run of `action` fills, on what was captured,
    /// and answers with what it is asked for by.
    fn conversation_for(&self, action: &str) -> u64 {
        let captured = self.last_capture();
        let selection = captured
            .as_ref()
            .and_then(CaptureOutcome::selection)
            .cloned();

        self.store
            .lock()
            .unwrap()
            .open(action::named(action), selection)
    }

    /// Asks the Turn that opens a Conversation, which the Action asks on the
    /// user's behalf rather than the user in their own words.
    fn opening_turn(
        &self,
        id: u64,
        action: &str,
        parameters: &BTreeMap<String, String>,
        showing: impl FnMut(&str),
    ) -> RunOutcome {
        let captured = self.last_capture();
        let Some(selection) = captured.as_ref().and_then(CaptureOutcome::selection) else {
            return RunOutcome::Failed(run::nothing_to_run());
        };

        // The Palette offers only the Actions that accept what was captured, and
        // is the only gate on that today: with one Selection kind there is no
        // Action a Run could reach that would refuse it. The kind images bring
        // is where this needs a check of its own.
        let Some(action) = action::named(action) else {
            return RunOutcome::Failed(run::no_such_action(action));
        };

        self.ask(id, action.prompt(selection, parameters), showing)
    }

    /// Puts the Conversation to the Provider, with `prompt` as what the Turn
    /// now being asked sends.
    fn ask(&self, id: u64, prompt: String, mut showing: impl FnMut(&str)) -> RunOutcome {
        let config = match self.config.as_ref() {
            Ok(config) => config,
            Err(error) => return RunOutcome::Failed(RunError::Configuration(error.to_string())),
        };

        let Some(said) = self.store.lock().unwrap().asking(id, prompt) else {
            return RunOutcome::Failed(run::no_conversation());
        };

        // Installed before the request and taken down after it, so that Stop
        // between two Runs stops neither — and released before the waiting
        // starts, because Stop arrives on another thread and would otherwise
        // wait for the Run it is trying to end.
        let stopping = Stopping::default();
        *self.stopping.lock().unwrap() = Some(stopping.clone());

        let mut assembly = Assembly::new(self.throttle);
        let asked = provider::answer(&config.provider, &said, &stopping, |fragment| {
            if let Some(answer) = assembly.push(fragment) {
                showing(&answer);
            }
        });

        *self.stopping.lock().unwrap() = None;

        match asked {
            Err(error) => RunOutcome::Failed(error),
            Ok(()) if stopping.stopped() => RunOutcome::Stopped(assembly.text()),
            Ok(()) => RunOutcome::Answered(assembly.text()),
        }
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
            configured(dir.path(), base_url);
        }

        Rooted {
            demysto: Demysto::with_capture(dir.path(), "1.2.3", capture).unthrottled(),
            _dir: dir,
        }
    }

    /// Writes a settings file naming one Provider, at `base_url`.
    fn configured(dir: &Path, base_url: &str) {
        std::fs::write(
            dir.join(config::FILE_NAME),
            format!(
                "version = 1\n\n[[providers]]\nname = \"a provider\"\n\
                 base_url = \"{base_url}\"\nmodel = \"a-model\"\napi_key = \"a-key\"\n"
            ),
        )
        .unwrap();
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

    /// A Run of the Action every test that is not about the catalogue is about,
    /// and whose intermediate states nobody is watching.
    fn run(demysto: &Demysto) -> RunOutcome {
        running(demysto, "explain", &[])
    }

    /// A Run of one named Action, with what the Palette collected for the
    /// Parameters it declares.
    fn running(demysto: &Demysto, action: &str, parameters: &[(&str, &str)]) -> RunOutcome {
        demysto.run(action, &collected(parameters), |_| {})
    }

    fn collected(parameters: &[(&str, &str)]) -> BTreeMap<String, String> {
        parameters
            .iter()
            .map(|(id, value)| ((*id).to_owned(), (*value).to_owned()))
            .collect()
    }

    /// Every state a Run put on screen, and what it finally produced.
    fn watching(demysto: &Demysto) -> (Vec<String>, RunOutcome) {
        let mut shown = Vec::new();
        let outcome = demysto.run("explain", &BTreeMap::new(), |answer| {
            shown.push(answer.to_owned())
        });

        (shown, outcome)
    }

    /// The names of the Actions the Palette would list, in its order.
    fn offered(demysto: &Demysto) -> Vec<String> {
        demysto
            .actions()
            .into_iter()
            .map(|action| action.name)
            .collect()
    }

    /// A Demysto pointed at `server`, having captured `selection`, whose Runs
    /// are asserted on the request they send rather than the answer they get.
    fn asked_for(server: &mut ServerGuard, body: Vec<Matcher>) -> mockito::Mock {
        server
            .mock("POST", "/v1/chat/completions")
            .match_body(Matcher::AllOf(body))
            .with_body(answering("an answer"))
            .create()
    }

    /// A Demysto that has captured `selection` and is pointed at `server`.
    fn ready_to_run(server: &ServerGuard, selection: &str) -> Rooted {
        let desktop = Arc::new(FakeDesktop::new(None, Some(selection)));
        let demysto = demysto_asking(fake::over(&desktop), &format!("{}/v1", server.url()));
        demysto.capture();

        demysto
    }

    /// The Conversation the result window would be showing.
    fn showing(demysto: &Demysto) -> Conversation {
        demysto
            .conversation()
            .expect("there should be a Conversation on screen")
    }

    /// A follow-up Turn on the Conversation on screen, whose intermediate
    /// states nobody is watching.
    fn following_up(demysto: &Demysto, question: &str) -> RunOutcome {
        demysto.follow_up(question, |_| {})
    }

    /// What every Turn of the Conversation on screen asked, and what it was
    /// answered with.
    fn turns(demysto: &Demysto) -> Vec<(Option<String>, Option<RunOutcome>)> {
        showing(demysto)
            .turns
            .into_iter()
            .map(|turn| (turn.question, turn.outcome))
            .collect()
    }

    /// A Turn the Model answered, as the assertions below write one.
    fn answered(text: &str) -> Option<RunOutcome> {
        Some(RunOutcome::Answered(text.to_owned()))
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
    fn the_conversation_is_there_for_a_window_that_opens_after_the_run() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body(answering("an answer"))
            .create();

        let demysto = ready_to_run(&server, "a paragraph");
        run(&demysto);

        assert_eq!(turns(&demysto), [(None, answered("an answer"))]);
    }

    #[test]
    fn nothing_is_on_screen_before_the_first_run() {
        let desktop = Arc::new(FakeDesktop::new(None, None));
        let demysto = demysto(fake::over(&desktop));

        assert_eq!(demysto.conversation(), None);
        assert!(demysto.conversations().is_empty());
    }

    #[test]
    fn the_palette_is_offered_the_actions_that_accept_what_was_captured() {
        let desktop = Arc::new(FakeDesktop::new(None, Some("a paragraph")));
        let demysto = demysto(fake::over(&desktop));
        demysto.capture();

        // The order is the catalogue's, not the alphabet's: the first is the
        // one Enter runs without the user having read anything.
        assert_eq!(offered(&demysto), ["Explain", "Translate", "Summarize"]);
    }

    #[test]
    fn a_capture_that_found_nothing_leaves_no_action_to_offer() {
        let desktop = Arc::new(FakeDesktop::new(None, None));
        let demysto = demysto(fake::over(&desktop));
        demysto.capture();

        assert!(offered(&demysto).is_empty());
    }

    #[test]
    fn nothing_is_offered_before_the_first_capture() {
        let desktop = Arc::new(FakeDesktop::new(None, Some("a paragraph")));

        assert!(offered(&demysto(fake::over(&desktop))).is_empty());
    }

    #[test]
    fn a_window_shown_for_a_run_is_told_which_action_before_there_is_an_answer() {
        let desktop = Arc::new(FakeDesktop::new(None, None));
        let demysto = demysto(fake::over(&desktop));

        demysto.about_to_run("translate");

        assert_eq!(
            showing(&demysto).action.map(|action| action.name),
            Some("Translate".to_owned())
        );
        assert_eq!(turns(&demysto), [(None, None)]);
    }

    #[test]
    fn declaring_the_same_run_twice_opens_one_conversation_and_not_two() {
        // The interface declares a Run before it shows the window, and the Run
        // declares it again for a caller that did not.
        let desktop = Arc::new(FakeDesktop::new(None, None));
        let demysto = demysto(fake::over(&desktop));

        demysto.about_to_run("translate");
        demysto.about_to_run("translate");

        assert_eq!(demysto.conversations().len(), 1);
    }

    #[test]
    fn a_run_of_an_action_demysto_does_not_have_leaves_the_window_nothing_to_name() {
        let desktop = Arc::new(FakeDesktop::new(None, None));
        let demysto = demysto(fake::over(&desktop));

        demysto.about_to_run("translat");

        assert_eq!(showing(&demysto).action, None);
    }

    #[test]
    fn explaining_names_the_language_it_read_and_asks_for_the_one_the_user_reads() {
        let mut server = Server::new();
        let endpoint = asked_for(
            &mut server,
            vec![
                Matcher::Regex("The text is in German".to_owned()),
                Matcher::Regex("answer in English".to_owned()),
                Matcher::Regex("Der Mensch ist frei geschaffen".to_owned()),
            ],
        );

        run(&ready_to_run(
            &server,
            "Der Mensch ist frei geschaffen, ist frei",
        ));

        endpoint.assert();
    }

    #[test]
    fn a_selection_too_short_to_place_is_not_given_a_language_it_may_not_be_in() {
        // Two words are not enough to tell a language from — on the balance of
        // probabilities "borrow checker" is Shona — and a prompt is better told
        // nothing than told that.
        let mut server = Server::new();
        let endpoint = asked_for(
            &mut server,
            vec![Matcher::Regex(
                "The text is in an unknown language".to_owned(),
            )],
        );

        run(&ready_to_run(&server, "borrow checker"));

        endpoint.assert();
    }

    #[test]
    fn translating_asks_for_the_language_the_palette_collected() {
        let mut server = Server::new();
        let endpoint = asked_for(
            &mut server,
            vec![
                Matcher::Regex("Translate the text below into Georgian".to_owned()),
                Matcher::Regex("Ceci n'est pas une pipe".to_owned()),
            ],
        );

        running(
            &ready_to_run(&server, "Ceci n'est pas une pipe"),
            "translate",
            &[("target", "Georgian")],
        );

        endpoint.assert();
    }

    #[test]
    fn a_parameter_the_user_left_alone_falls_back_to_what_it_offered() {
        let mut server = Server::new();
        let endpoint = asked_for(
            &mut server,
            vec![Matcher::Regex(
                "Translate the text below into English".to_owned(),
            )],
        )
        .expect(2);

        let demysto = ready_to_run(&server, "Ceci n'est pas une pipe");

        // Nothing collected at all, and a field the user cleared and left:
        // both are the absence of an answer rather than an answer of nothing.
        running(&demysto, "translate", &[]);
        running(&demysto, "translate", &[("target", "   ")]);

        endpoint.assert();
    }

    #[test]
    fn each_action_sends_its_own_prompt_rather_than_one_wording_for_all_of_them() {
        let mut server = Server::new();
        let endpoint = asked_for(
            &mut server,
            vec![
                Matcher::Regex("Summarize the text below".to_owned()),
                Matcher::Regex("answer in English".to_owned()),
            ],
        );

        running(&ready_to_run(&server, "a paragraph"), "summarize", &[]);

        endpoint.assert();
    }

    #[test]
    fn an_action_demysto_does_not_have_sends_nothing_anywhere() {
        // The Provider is an address that refuses connections, so anything but
        // this outcome would mean a request went out.
        let desktop = Arc::new(FakeDesktop::new(None, Some("a paragraph")));
        let demysto = demysto_asking(fake::over(&desktop), "http://127.0.0.1:1/v1");
        demysto.capture();

        let RunOutcome::Failed(RunError::NoSuchAction(message)) = running(&demysto, "explan", &[])
        else {
            panic!("an Action Demysto does not have should be reported as one");
        };

        assert!(message.contains("explan"), "{message}");
    }

    #[test]
    fn a_run_opens_a_conversation_holding_the_turn_it_took() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body(answering("A painting of a pipe is not a pipe."))
            .create();

        let demysto = ready_to_run(&server, "Ceci n'est pas une pipe");
        run(&demysto);

        // The opening Turn asked nothing in the user's own words: the Action
        // asked it for them, and the window heads it with the Action's name.
        assert_eq!(
            turns(&demysto),
            [(None, answered("A painting of a pipe is not a pipe."))]
        );
        assert_eq!(
            showing(&demysto).action.map(|action| action.name),
            Some("Explain".to_owned())
        );
    }

    #[test]
    fn a_follow_up_is_asked_in_the_context_of_the_turns_before_it() {
        let mut server = Server::new();

        // Created first so that it is the one preferred where it matches: the
        // opening Run carries no follow-up and falls through to the mock below.
        let follow_up = server
            .mock("POST", "/v1/chat/completions")
            .match_body(Matcher::AllOf(vec![
                // The Selection is still there, in the prompt the Action
                // assembled for the Turn that opened the Conversation …
                Matcher::Regex("Ceci n'est pas une pipe".to_owned()),
                // … followed by what the Model replied, and then by the
                // question, in that order and each as whose words they are.
                Matcher::Regex(
                    [
                        r#""role":"user".*"#,
                        r#""role":"assistant","content":"A painting of a pipe is not a pipe\.".*"#,
                        r#""role":"user","content":"Why not\?""#,
                    ]
                    .concat(),
                ),
            ]))
            .with_body(answering("Because you cannot smoke it."))
            .create();

        let _opening = server
            .mock("POST", "/v1/chat/completions")
            .with_body(answering("A painting of a pipe is not a pipe."))
            .create();

        let demysto = ready_to_run(&server, "Ceci n'est pas une pipe");
        run(&demysto);

        assert_eq!(
            following_up(&demysto, "Why not?"),
            RunOutcome::Answered("Because you cannot smoke it.".to_owned())
        );
        follow_up.assert();
    }

    #[test]
    fn turns_accumulate_rather_than_replacing_one_another() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body(answering("an answer"))
            .create();

        let demysto = ready_to_run(&server, "a paragraph");
        run(&demysto);
        following_up(&demysto, "and then?");
        following_up(&demysto, "why?");

        assert_eq!(
            turns(&demysto),
            [
                (None, answered("an answer")),
                (Some("and then?".to_owned()), answered("an answer")),
                (Some("why?".to_owned()), answered("an answer")),
            ]
        );
    }

    #[test]
    fn a_second_run_opens_a_conversation_of_its_own() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body(answering("an answer"))
            .create();

        let demysto = ready_to_run(&server, "a paragraph");
        run(&demysto);
        run(&demysto);

        // A Run is a new question about a new Selection: it belongs beside the
        // Conversation before it rather than inside it.
        assert_eq!(demysto.conversations().len(), 2);
        assert_eq!(turns(&demysto).len(), 1);
    }

    #[test]
    fn the_session_holds_fifty_conversations_and_forgets_the_oldest_first() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body(answering("an answer"))
            .create();

        let demysto = ready_to_run(&server, "a paragraph");
        for _ in 0..conversation::CAP + 1 {
            run(&demysto);
        }

        let held = demysto.conversations();
        let ids: Vec<_> = held.iter().map(|held| held.id).collect();

        // Newest first, and the very first Run is the one that fell off.
        assert_eq!(ids.len(), conversation::CAP);
        assert_eq!(ids.first(), Some(&(conversation::CAP as u64 + 1)));
        assert_eq!(ids.last(), Some(&2));
    }

    #[test]
    fn a_conversation_the_user_goes_back_to_is_the_one_the_next_turn_joins() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body(answering("an answer"))
            .create();

        let demysto = ready_to_run(&server, "a paragraph");
        run(&demysto);
        let earlier = showing(&demysto).id;
        run(&demysto);

        assert_eq!(
            demysto.show_conversation(earlier).map(|shown| shown.id),
            Some(earlier)
        );

        following_up(&demysto, "and then?");

        assert_eq!(showing(&demysto).id, earlier);
        assert_eq!(turns(&demysto).len(), 2);
    }

    #[test]
    fn a_conversation_the_session_never_held_cannot_be_gone_back_to() {
        let desktop = Arc::new(FakeDesktop::new(None, None));

        assert_eq!(demysto(fake::over(&desktop)).show_conversation(7), None);
    }

    #[test]
    fn a_run_stopped_part_way_keeps_the_text_that_had_arrived() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body(streaming(&["A painting ", "of a pipe ", "is not a pipe."]))
            .create();

        let demysto = ready_to_run(&server, "Ceci n'est pas une pipe");

        // Stopped as the first words land, which is when a reader watching an
        // answer go nowhere reaches for it.
        let outcome = demysto.run("explain", &BTreeMap::new(), |_| demysto.stop());

        assert_eq!(outcome, RunOutcome::Stopped("A painting ".to_owned()));
        assert_eq!(turns(&demysto), [(None, Some(outcome))]);
    }

    #[test]
    fn what_a_stopped_turn_did_deliver_is_context_for_the_next_one() {
        let mut server = Server::new();

        let follow_up = server
            .mock("POST", "/v1/chat/completions")
            .match_body(Matcher::Regex(
                r#""role":"assistant","content":"A painting ""#.to_owned(),
            ))
            .with_body(answering("Because you cannot smoke it."))
            .create();

        let _opening = server
            .mock("POST", "/v1/chat/completions")
            .with_body(streaming(&["A painting ", "of a pipe ", "is not a pipe."]))
            .create();

        let demysto = ready_to_run(&server, "Ceci n'est pas une pipe");
        demysto.run("explain", &BTreeMap::new(), |_| demysto.stop());
        following_up(&demysto, "Why not?");

        follow_up.assert();
    }

    #[test]
    fn a_turn_that_failed_still_carries_its_selection_into_the_next_one() {
        let mut server = Server::new();

        // Created first so that it is the one preferred where it matches: the
        // opening Run carries no follow-up and falls through to the mock below.
        let follow_up = server
            .mock("POST", "/v1/chat/completions")
            .match_body(Matcher::AllOf(vec![
                // The Selection the refused Turn asked about is still there, so
                // that the Turn after it is about something.
                Matcher::Regex("Ceci n'est pas une pipe".to_owned()),
                Matcher::Regex(r#""role":"user","content":"try again\?""#.to_owned()),
            ]))
            .with_body(answering("A painting of a pipe is not a pipe."))
            .create();

        let _refusing = server
            .mock("POST", "/v1/chat/completions")
            .with_status(500)
            .with_body("<html>Bad Gateway</html>")
            .create();

        let demysto = ready_to_run(&server, "Ceci n'est pas une pipe");
        run(&demysto);

        // What the Provider never replied to is a question the user is still
        // owed an answer to. Dropping it because it failed once would leave
        // this Turn asking about nothing at all.
        assert_eq!(
            following_up(&demysto, "try again?"),
            RunOutcome::Answered("A painting of a pipe is not a pipe.".to_owned())
        );
        follow_up.assert();
    }

    #[test]
    fn a_turn_the_provider_refused_stays_in_the_conversation_as_the_turn_it_was() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_status(500)
            .with_body("<html>Bad Gateway</html>")
            .create();

        let demysto = ready_to_run(&server, "a paragraph");
        run(&demysto);
        following_up(&demysto, "and then?");

        // A failure is an entry in the Conversation rather than something that
        // replaces it, per the spec's *Errors*. Ticket 11 gives it a retry.
        let turns = turns(&demysto);

        assert_eq!(turns.len(), 2);
        assert!(
            turns.iter().all(|(_, outcome)| matches!(
                outcome,
                Some(RunOutcome::Failed(RunError::Provider(_)))
            )),
            "{turns:?}"
        );
    }

    #[test]
    fn a_follow_up_with_no_conversation_on_screen_sends_nothing_anywhere() {
        // The Provider is an address that refuses connections, so anything but
        // this outcome would mean a request went out.
        let desktop = Arc::new(FakeDesktop::new(None, Some("a paragraph")));
        let demysto = demysto_asking(fake::over(&desktop), "http://127.0.0.1:1/v1");

        assert!(matches!(
            following_up(&demysto, "and then?"),
            RunOutcome::Failed(RunError::NothingToRun(_))
        ));
    }

    #[test]
    fn conversations_do_not_outlive_the_session_that_held_them() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body(answering("an answer"))
            .create();

        let dir = TempDir::new().unwrap();
        configured(dir.path(), &format!("{}/v1", server.url()));

        let desktop = Arc::new(FakeDesktop::new(None, Some("a paragraph")));
        let session = Demysto::with_capture(dir.path(), "1.2.3", fake::over(&desktop));
        session.capture();
        run(&session);

        assert_eq!(session.conversations().len(), 1);

        // The same configuration directory, and nothing of the Conversation in
        // it: history is held in memory and nowhere else (user story 62).
        let next = Demysto::with_capture(dir.path(), "1.2.3", fake::over(&desktop));

        assert!(next.conversations().is_empty());
    }
}
