//! Demysto's product logic.
//!
//! This crate deliberately depends on no user interface toolkit: it is the
//! single seam the test suite attaches to (see `docs/spec/0001-v1-text-actions.md`).
//! The Tauri layer in `src-tauri` is a set of thin adapters over the [`Demysto`]
//! facade defined here, and nothing in this crate may reference Tauri types.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, RwLock};
use std::time::Duration;

mod action;
mod capture;
mod catalogue;
mod config;
mod conversation;
mod desktop;
mod files;
mod hotkey;
mod i18n;
mod language;
mod log;
mod model;
mod paths;
mod provider;
mod run;
mod selection;
mod settings;
mod sse;
mod stream;

pub use action::{Action, Parameter};
pub use capture::{Capture, CaptureError, CaptureOutcome, Captured, Capturing};
pub use catalogue::{ActionEdit, ActionError, ActionStanding, Catalogue, DefinedAction};
pub use config::{ConfigError, DEFAULT_LARGE_SELECTION};
pub use conversation::{Conversation, Summary, Turn};
pub use fluent::FluentArgs as Args;
pub use i18n::{Interface, Words, LANGUAGE_ENV};
pub use paths::{config_dir, ConfigDirError, CONFIG_DIR_ENV};
pub use run::{RunError, RunOutcome};
pub use selection::{Kind, Selection};
pub use settings::{
    ConfiguredModel, ConfiguredProvider, Edit, KeyEdit, KeyStanding, Preset, ProviderEdit, Settings,
};

use config::{Config, Environment};
use conversation::Store;
use run::Stopping;
use stream::Assembly;

/// The facade every user interface talks to.
pub struct Demysto {
    config_dir: PathBuf,
    version: String,
    capture: Box<dyn Capture>,
    /// What a Capture on this desktop can read, so that the interface can say
    /// so where the answer is not the whole Selection (user story 56). Held
    /// from startup for the reason the environment is: it is the session
    /// Demysto was launched into, and a session does not change under it.
    capturing: Capturing,
    /// The environment as it was when Demysto started, which is where a key
    /// may come from. Held rather than looked at again, so that the settings
    /// can be read a second time — which every save does — and arrive at the
    /// same keys the first read did.
    env: Environment,
    /// The settings Demysto is running on, however reading them went. A file
    /// that cannot be used is no reason to refuse to start: the Palette still
    /// opens, and the Run is where the user is told what to fix.
    ///
    /// Behind a lock because the settings window writes them: a Run reads them
    /// through it and takes out everything it needs before letting go, so that
    /// saving never waits on a Provider and a Run never asks half of one set of
    /// settings and half of another.
    config: RwLock<Result<Config, ConfigError>>,
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
    /// How long a Provider has to answer before Demysto stops waiting. A field
    /// for the reason the throttle is one: a timeout nobody can reach in a test
    /// is a timeout nobody has tested.
    timeout: Duration,
    /// Where this session says what it did, for a bug report to carry. What it
    /// records and what it deliberately does not is `log`'s.
    log: log::Log,
    /// Demysto's own words, in the language this session speaks them.
    ///
    /// Behind a lock for the reason the settings are: the settings window
    /// changes the language, and everything that says anything — a Run
    /// composing a failure, the tray menu being rebuilt, the window asking what
    /// to draw itself in — reads it through this.
    words: RwLock<Words>,
}

/// What one Turn asks, before anything has been put to a Provider: which Model
/// it resolves through, and what it says.
///
/// Held together because they travel together — the Turn that opens a
/// Conversation takes them from the Action, and every follow-up takes them from
/// the Conversation it is asked in.
struct Asking {
    /// The Model the Action bound, `None` when it bound none.
    binding: Option<String>,
    /// What the Turn is about, which is what decides whether it needs a Model
    /// that can see.
    kind: Kind,
    prompt: String,
    /// What already arrived for this Turn, when it is being continued after a
    /// stream broke; empty for every Turn asked for the first time.
    delivered: String,
}

impl From<conversation::Asked> for Asking {
    fn from(asked: conversation::Asked) -> Self {
        Self {
            binding: asked.binding,
            kind: asked.kind,
            prompt: asked.prompt,
            delivered: asked.delivered,
        }
    }
}

/// The keys a Hotkey may be on its own, because they type nothing.
///
/// A free function as well as a method on the facade: the shell claims Hotkeys
/// before it has one to ask, and its own suite checks this list against the
/// parser that has to read it.
pub fn keys_that_need_no_modifier() -> Vec<&'static str> {
    hotkey::keys_that_need_no_modifier()
}

/// Whether Demysto is running in a Wayland session.
///
/// A free function because the shell asks it before there is a facade to ask,
/// and because what it decides there is not a Capture: on Wayland a Hotkey
/// cannot be claimed from the display server either, and has to be asked of the
/// GlobalShortcuts portal instead (ADR-0003). One question, because it is one
/// fact about the session — what a Capture makes of it is
/// [`Status::capturing`].
pub fn wayland_session() -> bool {
    desktop::wayland()
}

/// What the application can report about itself before anything is configured.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Status {
    /// The running version of the application.
    pub version: String,
    /// Where this instance reads and writes its configuration.
    pub config_dir: PathBuf,
    /// How long a Selection may be before Demysto says so, where the settings
    /// state nothing. Reported so that the window can say what leaving the
    /// field blank means rather than repeat a number of its own.
    pub large_selection_default: u64,
    /// What a Capture on this desktop can read. Everywhere but Wayland this is
    /// the Selection, and the windows that show it say nothing; on Wayland it
    /// is the fact the window says a sentence about.
    pub capturing: Capturing,
    /// The interface language the environment fixes, `None` where nothing is
    /// exported.
    ///
    /// Reported for the reason a key found in a variable is: the field in
    /// Settings would otherwise invite somebody to choose a language that goes
    /// on being ignored, and never say why.
    pub language_env: Option<Exported>,
}

/// A setting the environment has already decided, and the variable that decided
/// it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Exported {
    pub variable: &'static str,
    pub value: String,
}

/// A count with its thousands marked off, because the number in a warning about
/// size is the whole of what the warning says and "43120" is not a size anybody
/// reads at a glance.
///
/// Grouped here rather than by Fluent, which formats a number without grouping
/// it: the catalogue is handed the digits already marked off, and separately the
/// count itself, which is what chooses between "символ", "символа" and
/// "символов". The separator is the one the language writes — a comma in
/// English, a non-breaking space in Russian.
fn grouped(count: u64, interface: Interface) -> String {
    let separator = match interface {
        Interface::English => ",",
        Interface::Russian => "\u{a0}",
    };

    count
        .to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(|group| String::from_utf8_lossy(group).into_owned())
        .collect::<Vec<_>>()
        .join(separator)
}

impl Demysto {
    /// Builds a facade rooted at an explicit configuration directory.
    ///
    /// The version is supplied by the caller rather than read from this crate's
    /// own `CARGO_PKG_VERSION`: what the user is running is the application, and
    /// the library's version is nobody's business but the build's.
    pub fn new(config_dir: impl Into<PathBuf>, version: impl Into<String>) -> Self {
        let (capture, capturing) = desktop::for_platform();

        Self::with_capture(config_dir, version, capture, capturing)
    }

    /// Builds a facade over a Capture chosen by the caller, which is how the
    /// test suite keeps the desktop out of it.
    ///
    /// The Capture and what it can read arrive together, because the second is
    /// what the first was chosen by: a suite substituting the Capture a Wayland
    /// session gets and leaving the answer to `capturing` behind would be a
    /// facade whose interface says the opposite of what its Capture does.
    pub fn with_capture(
        config_dir: impl Into<PathBuf>,
        version: impl Into<String>,
        capture: Box<dyn Capture>,
        capturing: Capturing,
    ) -> Self {
        let config_dir = config_dir.into();

        // Taken once, here, and nowhere else in the crate: the environment
        // holds the key, and a key that can change under a running Demysto is a
        // key nobody can reason about (the spec's *Core modules*). It holds the
        // language too, which is why it is read before anything else happens.
        let env = Environment::snapshot();

        // Settled before the settings are read, because reading them is already
        // something that has to be said in a language: every sentence about a
        // file that could not be parsed is one. The language the file states is
        // taken out of it first, on its own, and a file that states none — or
        // one nobody can parse — leaves the operating system deciding.
        let words = Words::spoken(i18n::chosen(
            env.get(i18n::LANGUAGE_ENV),
            config::stated_language(&config_dir).as_deref(),
            i18n::system_language(),
        ));

        Self::speaking(config_dir, version, capture, capturing, env, words)
    }

    /// The same facade with the language settled rather than looked up.
    ///
    /// Which is what the suite builds: every assertion about a sentence would
    /// otherwise be an assertion about the language the machine running the
    /// tests happens to be set to.
    fn speaking(
        config_dir: PathBuf,
        version: impl Into<String>,
        capture: Box<dyn Capture>,
        capturing: Capturing,
        env: Environment,
        words: Words,
    ) -> Self {
        let version = version.into();
        let log = log::Log::new(&config_dir);
        log.started(&version, &config_dir);

        Self {
            config: RwLock::new(config::load(&config_dir, &env, &words)),
            words: RwLock::new(words),
            env,
            config_dir,
            version,
            capture,
            capturing,
            last_capture: Mutex::new(None),
            store: Mutex::new(Store::new()),
            stopping: Mutex::new(None),
            throttle: stream::THROTTLE,
            timeout: provider::REQUEST_TIMEOUT,
            log,
        }
    }

    /// The same facade with nothing held back, so that a test can see every
    /// state a Run passed through rather than the few a clock let out.
    #[cfg(test)]
    fn unthrottled(mut self) -> Self {
        self.throttle = Duration::ZERO;
        self
    }

    /// The same facade that gives up on a Provider almost at once, so that the
    /// suite can reach the timeout without waiting two minutes for it.
    #[cfg(test)]
    fn impatient(mut self) -> Self {
        self.timeout = Duration::from_millis(200);
        self
    }

    /// Re-reads the settings the way saving them does, so that the suite can
    /// change them under a Conversation as the settings window does.
    #[cfg(test)]
    fn settings_changed(&self) {
        self.reload();
    }

    /// Re-reads the settings, and with them the language the interface speaks.
    ///
    /// The two together because the second is stated in the first: a save that
    /// changed the language leaves everything Demysto has yet to say owed in
    /// the new one, and the tray menu and the windows ask for it again.
    fn reload(&self) {
        let words = Words::spoken(i18n::chosen(
            self.env.get(i18n::LANGUAGE_ENV),
            config::stated_language(&self.config_dir).as_deref(),
            i18n::system_language(),
        ));

        *self.config.write().unwrap() = config::load(&self.config_dir, &self.env, &words);
        *self.words.write().unwrap() = words;
    }

    /// Demysto's own words, for the parts of the interface that are native and
    /// have to say something.
    pub fn words(&self) -> std::sync::RwLockReadGuard<'_, Words> {
        self.words.read().unwrap()
    }

    /// The language the interface speaks, so that the windows can pick the same
    /// catalogue this side of the channel is reading.
    pub fn language(&self) -> Interface {
        self.words().interface()
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

    /// The whole of what the Conversation on screen is about, for the window
    /// that quotes the opening of it and has been asked to show the rest.
    ///
    /// Asked for rather than carried on the Conversation, because a Selection
    /// has no size limit and the Conversation crosses to the window every time
    /// a Turn begins or ends; this crosses once, when somebody asks.
    pub fn selection(&self) -> Option<String> {
        self.store
            .lock()
            .unwrap()
            .showing()
            .and_then(Conversation::selection_text)
            .map(ToOwned::to_owned)
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

        catalogue::runnable(&self.config_dir, &self.words())
            .into_iter()
            .filter(|action| action.accepts(selection.kind()))
            .collect()
    }

    /// Every Action there is, with everything about it, for the window that
    /// writes them.
    ///
    /// Unlike [`Self::actions`] this is not filtered by anything: what the
    /// Palette lists depends on what was captured, and what can be edited does
    /// not.
    pub fn catalogue(&self) -> Catalogue {
        catalogue::read(&self.config_dir, &self.words())
    }

    /// Writes one Action, and answers with the catalogue as the directory then
    /// holds it.
    ///
    /// An edit naming no Action creates one, under an identifier found from its
    /// name; one naming a built-in writes the Override of it, stating only what
    /// the user changed. Either way what comes back is what was read off the
    /// disk afterwards, for the reason a saved settings file is read back: a
    /// save is only finished when it reads back.
    pub fn save_action(&self, edit: &ActionEdit) -> Result<Catalogue, ActionError> {
        catalogue::write(
            &self.config_dir,
            edit,
            &self.models_configured(),
            &self.words(),
        )
    }

    /// Takes an Action off the user: deletes one of their own, or removes an
    /// Override and leaves the built-in it was over.
    pub fn delete_action(&self, id: &str) -> Result<Catalogue, ActionError> {
        catalogue::delete(&self.config_dir, id, &self.words())
    }

    /// Every Model configured, by the name one is bound by, so that an Action
    /// cannot be saved bound to a Model nothing offers. Empty where the
    /// settings could not be read at all — which is a state in which nothing
    /// offers any Model, and the window has none to choose from either.
    ///
    /// Read from the file rather than answered from what Demysto is running on,
    /// for the reason [`Self::settings`] is: the window builds the list it
    /// offers out of the file, and a Model added to that file by hand while
    /// Demysto ran would otherwise be offered by the window and refused by the
    /// save.
    fn models_configured(&self) -> Vec<String> {
        match config::load(&self.config_dir, &self.env, &self.words()) {
            Ok(config) => config
                .models()
                .map(|(provider, model)| config::qualified(provider, model))
                .collect(),
            Err(_) => Vec::new(),
        }
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
        // A follow-up goes to the Model the Conversation resolves to, on the
        // Selection it is about — which is the Action's binding until a retry
        // switched it, and that Model from then on.
        let (id, asking) = {
            let mut store = self.store.lock().unwrap();
            let Some(conversation) = store.follow_up(question) else {
                return RunOutcome::Failed(run::no_conversation(&self.words()));
            };

            (
                conversation.id,
                Asking {
                    binding: conversation.binding().map(ToOwned::to_owned),
                    kind: conversation.kind(),
                    prompt: question.to_owned(),
                    delivered: String::new(),
                },
            )
        };

        let outcome = self.ask(id, asking, showing);
        self.store.lock().unwrap().answered(id, outcome.clone());

        outcome
    }

    /// Asks the last Turn of the Conversation on screen again, optionally
    /// somewhere else.
    ///
    /// `model` is the Model to switch this Conversation to, `None` for trying
    /// again with nothing changed — the two affordances a failed Turn is
    /// offered (user stories 20 and 44), which are one operation because
    /// switching a Model without asking anything would leave the user looking
    /// at the same failure.
    ///
    /// The Turn is asked again rather than added to: a retry is the same
    /// question, and a Conversation that accumulated one copy of it per attempt
    /// would send every failed attempt as context for the next.
    pub fn retry(&self, model: Option<&str>, showing: impl FnMut(&str)) -> RunOutcome {
        // Bound rather than passed straight in, so that the store is let go of
        // before the Turn is asked: a temporary guard would live to the end of
        // the whole statement, and the asking takes the same lock.
        let asked = self.store.lock().unwrap().retrying(model);

        self.again(asked, showing)
    }

    /// Asks the Model for the rest of an answer that broke off part-way (user
    /// story 46).
    ///
    /// What already arrived is put back to the Model as its own words, and what
    /// comes back is added to it, so that the Turn ends up holding one answer.
    pub fn continue_answer(&self, showing: impl FnMut(&str)) -> RunOutcome {
        let asked = self.store.lock().unwrap().continuing();

        self.again(asked, showing)
    }

    /// Whether the Conversation on screen has a Turn to try again or carry on,
    /// so that the window can show that it is under way before it is.
    ///
    /// Answers with whether there was one, the way [`Self::about_to_follow_up`]
    /// does.
    pub fn about_to_retry(&self) -> bool {
        self.store
            .lock()
            .unwrap()
            .showing()
            .and_then(|showing| showing.turns.last())
            .is_some_and(|turn| turn.outcome.is_some())
    }

    /// Asks a Turn the store has put back to being asked, however it came to be
    /// one: a retry, or a continuation.
    fn again(&self, asked: Option<conversation::Asked>, showing: impl FnMut(&str)) -> RunOutcome {
        let Some(asked) = asked else {
            return RunOutcome::Failed(run::nothing_to_retry(&self.words()));
        };

        let id = asked.id;
        let outcome = self.ask(id, asked.into(), showing);

        self.store.lock().unwrap().answered(id, outcome.clone());

        outcome
    }

    /// Every Model configured, by the name one is switched to, so that the
    /// Conversation window can offer somewhere else to ask (user story 20).
    pub fn models(&self) -> Vec<String> {
        self.models_configured()
    }

    /// Where this session's log files are, for the button in Settings that
    /// opens the folder (user story 63).
    pub fn logs_dir(&self) -> PathBuf {
        self.log.dir().to_owned()
    }

    /// Records something the interface would otherwise only have said on a
    /// window that may never be opened — a Hotkey nobody could claim, an
    /// Action file nobody could read.
    pub fn note(&self, line: &str) {
        self.log.said(line);
    }

    /// Opens the Conversation a Run of `action` fills, on what was captured,
    /// and answers with what it is asked for by.
    fn conversation_for(&self, action: &str) -> u64 {
        let captured = self.last_capture();
        let selection = captured
            .as_ref()
            .and_then(CaptureOutcome::selection)
            .cloned();

        // Composed here rather than where the request is made, because here is
        // before: the interface declares the Run, shows the window, and only
        // then runs it, so a warning put on the Conversation now is on screen
        // while the Model is still being asked — which is the whole of what
        // "before you spend tokens on it" can mean for something that is not
        // going to block (user story 48).
        let warning = selection
            .as_ref()
            .and_then(|selection| self.warning(selection));

        self.store.lock().unwrap().open(
            catalogue::named(&self.config_dir, action, &self.words()),
            selection,
            warning,
        )
    }

    /// What the user is told about a Selection large enough to be worth knowing
    /// about before it is paid for. `None` for one that is not, and for a user
    /// who has turned the warning off.
    ///
    /// It warns and does not refuse. Nothing is cut and nothing is split — the
    /// spec puts both out of scope for v1 — so the only honest thing to do with
    /// a large Selection is send it and say that it was large.
    fn warning(&self, selection: &Selection) -> Option<String> {
        let held = self.config.read().unwrap();
        let config = held.as_ref().ok()?;

        let at = config.large_selection()?;
        let characters = selection.as_text().chars().count() as u64;

        (characters > at).then(|| {
            let words = self.words();
            let interface = words.interface();

            say!(
                &words,
                "run-large-selection",
                // The count as a number as well as as digits: the first is what
                // chooses the word after it, and the second is what is read.
                "characters" = characters,
                "shown" = grouped(characters, interface),
                "limit" = grouped(at, interface),
                "setting" = config::LARGE_SELECTION_SETTING,
                "path" = config.path.display().to_string(),
            )
        })
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
            return RunOutcome::Failed(run::nothing_captured(captured.as_ref(), &self.words()));
        };

        // The Palette offers only the Actions that accept what was captured, and
        // is the only gate on that today: with one Selection kind there is no
        // Action a Run could reach that would refuse it. The kind images bring
        // is where this needs a check of its own.
        // Read into a binding rather than matched on directly, so that the
        // words the catalogue was read with are let go of before the words the
        // failure is said in are asked for: two live read guards on one lock
        // deadlock against a write that queued between them.
        let named = catalogue::named(&self.config_dir, action, &self.words());

        let Some(action) = named else {
            return RunOutcome::Failed(run::no_such_action(action, &self.words()));
        };

        self.ask(
            id,
            Asking {
                binding: action.model.clone(),
                kind: selection.kind(),
                prompt: action.prompt(selection, parameters, self.language()),
                delivered: String::new(),
            },
            showing,
        )
    }

    /// Puts the Conversation to the Model `asking` resolves to, with its prompt
    /// as what the Turn now being asked sends.
    fn ask(&self, id: u64, asking: Asking, mut showing: impl FnMut(&str)) -> RunOutcome {
        // Recorded before anything is resolved, so that a Turn which never
        // reached a Provider is still a Turn — one the user can try again once
        // they have fixed what stopped it (user story 44). Nothing is sent by
        // recording it; what a Turn holds and what a Turn cost are separate
        // facts, and only the second needs the settings to be right.
        let Asking {
            binding,
            kind,
            prompt,
            delivered,
        } = asking;

        let Some(said) = self.store.lock().unwrap().asking(id, prompt) else {
            return RunOutcome::stopped_short(delivered, run::no_conversation(&self.words()));
        };

        // Everything the request needs comes out of the settings here, and the
        // lock goes with it: a Provider has two minutes to answer, and saving
        // the settings must not be two minutes of a frozen window.
        let resolved = match self.resolving(binding.as_deref(), kind) {
            Ok(resolved) => resolved,
            Err(error) => {
                self.log.failed(&error);

                // What a continuation was already carrying survives a Provider
                // it could not reach: the settings changing under an answer
                // that broke off is no reason to lose the half of it that did
                // arrive.
                return RunOutcome::stopped_short(delivered, error);
            }
        };

        self.log.asking(&resolved, said.len());

        // Installed before the request and taken down after it, so that Stop
        // between two Runs stops neither — and released before the waiting
        // starts, because Stop arrives on another thread and would otherwise
        // wait for the Run it is trying to end.
        let stopping = Stopping::default();
        *self.stopping.lock().unwrap() = Some(stopping.clone());

        // Taken out of the lock before the request rather than read through it,
        // for the reason the settings are taken out before one: a Provider has
        // two minutes to answer, and a save must not wait on it. What the words
        // cost is one parse of a file measured in kilobytes, once per Turn.
        let words = Words::spoken(self.language());

        let mut assembly = Assembly::continuing(self.throttle, delivered);
        let asked = provider::answer(
            &resolved,
            &said,
            self.timeout,
            &stopping,
            &words,
            |fragment| {
                if let Some(answer) = assembly.push(fragment) {
                    showing(&answer);
                }
            },
        );

        *self.stopping.lock().unwrap() = None;

        let outcome = match asked {
            // What arrived before it broke is the user's, and is offered the
            // rest of itself rather than thrown away (user story 46). Nothing
            // having arrived leaves nothing to keep, and that is a plain
            // failure with a retry on it.
            Err(error) => RunOutcome::stopped_short(assembly.text(), error),
            Ok(()) if stopping.stopped() => RunOutcome::Stopped(assembly.text()),
            Ok(()) => RunOutcome::Answered(assembly.text()),
        };

        self.log.answered(&outcome);

        outcome
    }

    /// Which Model this Turn goes to, and how to reach it — taken out of the
    /// settings so that the request can be made without holding them.
    fn resolving(&self, binding: Option<&str>, kind: Kind) -> Result<model::Resolved, RunError> {
        match self.config.read().unwrap().as_ref() {
            Err(error) => Err(RunError::Configuration {
                message: error.to_string(),
            }),
            Ok(config) => model::resolve(config, binding, kind, &self.words()),
        }
    }

    /// The Hotkey the settings state for the Palette, `None` where they state
    /// none or could not be read at all.
    ///
    /// Every way of failing answers `None`, which is the whole point of asking
    /// here rather than through [`Self::settings`]: a settings file nobody can
    /// parse must not cost somebody the Hotkey the tool opens with. What it
    /// costs them is the Hotkey they chose, and the shell says so.
    ///
    /// Read from the file rather than taken from what Demysto is running on,
    /// for the reason the Actions are read from disk every time — and because
    /// what Demysto is running on is an error until a Provider is configured,
    /// which would lose a Hotkey stated by somebody who has not got that far.
    pub fn palette_hotkey(&self) -> Option<String> {
        // One guard rather than two: a second `read` taken while the first is
        // still held deadlocks against a `write` that queued between them, and
        // saving the settings is exactly that write.
        let words = self.words();
        let (path, text) = config::read(&self.config_dir, &words).ok()?;

        config::stated(config::parse(&path, &text, &words).ok()?.palette_hotkey)
    }

    /// The keys a Hotkey may be on its own, for the window that records one.
    pub fn keys_that_need_no_modifier(&self) -> Vec<&'static str> {
        keys_that_need_no_modifier()
    }

    /// Whether a Hotkey stating this key and nothing else is one to claim.
    pub fn needs_no_modifier(&self, key: &str) -> bool {
        hotkey::needs_no_modifier(key)
    }

    /// The settings as the file now holds them, for the window that edits it.
    ///
    /// Read from the file rather than answered from what startup made of it:
    /// the window edits the file, and the file may have been edited by hand
    /// since Demysto started. Keys are not in what comes back — see `settings`.
    pub fn settings(&self) -> Result<Settings, ConfigError> {
        settings::read(&self.config_dir, &self.env, &self.words())
    }

    /// Writes what the window edited, and runs on it from here on.
    ///
    /// Answers with the settings as the file then holds them, which is what the
    /// window shows next: a save is only finished when it can be read back.
    pub fn save_settings(&self, edit: &Edit) -> Result<Settings, ConfigError> {
        self.verifying(edit)?;

        let saved = settings::write(&self.config_dir, &self.env, edit, &self.words())?;

        // Read again from the file just written rather than composed from the
        // edit, so that what Demysto runs on is exactly what its next start
        // would read — including the failure a saved file can still be, which
        // the Run is where the user is told about. The language comes with it:
        // a save that changed it is a save after which everything Demysto has
        // yet to say is owed in the new one (user story 59).
        self.reload();

        Ok(saved)
    }

    /// Puts every key typed into the window to its Provider before any of it
    /// is written — ticket 08's "A key entered here is verified against the
    /// Provider before it is saved", and user story 42's "immediately rather
    /// than at the first Run".
    ///
    /// Only a key typed now: one the file already holds was put to its Provider
    /// when it was typed, and asking again would put a request behind every
    /// save, per Provider. Only against a Model the Provider is configured
    /// with, because with none there is nothing to ask.
    ///
    /// A refusal stops the save: that is the Provider saying the key is wrong,
    /// and writing it would be storing something already known not to work.
    /// Nothing else does — an endpoint that is down, or a laptop off the
    /// network, is no evidence about a key, and refusing there would leave
    /// somebody unable to configure Demysto until their server came back.
    fn verifying(&self, edit: &Edit) -> Result<(), ConfigError> {
        for provider in &edit.providers {
            let (KeyEdit::Set { .. }, Some(model)) = (&provider.api_key, provider.models.first())
            else {
                continue;
            };

            // Both ways a Provider can refuse a key: one that says the key is
            // wrong, and one that says the request is. Only the first is
            // reliably an authentication failure, and both are the Provider
            // declining what was typed rather than the network declining to
            // carry it.
            let refused = match self.verify(provider, &model.id) {
                Err(RunError::Provider { message } | RunError::Authentication { message, .. }) => {
                    Some(message)
                }
                _ => None,
            };

            if let Some(message) = refused {
                return Err(ConfigError::Refused(format!(
                    "The Provider \"{}\" did not accept this key, so nothing was saved. \
                     {message}",
                    provider.name
                )));
            }
        }

        Ok(())
    }

    /// Every preset there is, so that the window can offer them.
    pub fn presets(&self) -> Vec<Preset> {
        settings::presets()
    }

    /// The Model identifiers a Provider says it offers, so that the user picks
    /// one rather than typing it from memory (user story 34).
    ///
    /// Asked of the Provider as the window has it now — a key just typed
    /// included — rather than as the file holds it, because the commonest
    /// moment to want the list is while configuring a Provider that has not
    /// been saved yet.
    pub fn models_offered_by(&self, provider: &ProviderEdit) -> Result<Vec<String>, RunError> {
        // Out of the lock before the request, for the reason a Run's are.
        let words = Words::spoken(self.language());

        provider::models(
            &settings::endpoint(&self.config_dir, &self.env, provider, &words)?,
            self.timeout,
            &words,
        )
    }

    /// Whether a Provider accepts a key, asked of the Provider itself (user
    /// story 42).
    ///
    /// Against a Model, because that is the request a Run makes and the only
    /// one that proves the key rather than the endpoint — ADR-0008.
    pub fn verify(&self, provider: &ProviderEdit, model: &str) -> Result<(), RunError> {
        let words = Words::spoken(self.language());

        provider::verify(
            &settings::endpoint(&self.config_dir, &self.env, provider, &words)?,
            model,
            self.timeout,
            &words,
        )
    }

    pub fn status(&self) -> Status {
        Status {
            version: self.version.clone(),
            config_dir: self.config_dir.clone(),
            large_selection_default: DEFAULT_LARGE_SELECTION,
            capturing: self.capturing.clone(),
            // Only where it names a language Demysto speaks. `i18n::chosen`
            // passes over one that does not and lets the settings decide, so
            // reporting it would have the window say the field it is beside
            // changes nothing — at the moment it is the thing deciding.
            language_env: self
                .env
                .get(LANGUAGE_ENV)
                .filter(|value| Interface::matching(value).is_some())
                .map(|value| Exported {
                    variable: LANGUAGE_ENV,
                    value,
                }),
        }
    }
}

#[cfg(test)]
mod tests {
    //! Every test calls the facade the Tauri commands call, with the desktop
    //! substituted at its edge — the one seam of the spec's *Testing Decisions*.

    use std::sync::Arc;

    use mockito::{Matcher, Mock, Server, ServerGuard};
    use serde_json::json;
    use tempfile::TempDir;

    use super::*;
    use crate::capture::fake::{self, FakeDesktop};
    // For the one test that puts two different texts on a fake clipboard in
    // turn, so that two Conversations are about two different Selections.
    use crate::capture::Desktop;

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

        /// The same Demysto that gives up on a Provider almost at once, for the
        /// tests that are about what happens when one never answers.
        fn impatient(mut self) -> Self {
            self.demysto = self.demysto.impatient();
            self
        }
    }

    /// A Demysto rooted at a directory the test already holds, for the one test
    /// that starts a second session over the first one's configuration.
    fn as_a_session(config_dir: &Path, desktop: fake::Reading) -> Demysto {
        let (capture, capturing) = desktop;

        Demysto::speaking(
            config_dir.to_owned(),
            "1.2.3",
            capture,
            capturing,
            Environment::snapshot(),
            i18n::english(),
        )
    }

    /// A Demysto with nothing configured to talk to.
    fn demysto(desktop: fake::Reading) -> Rooted {
        rooted(desktop, None)
    }

    /// A Demysto configured with one Provider, at `base_url`.
    fn demysto_asking(desktop: fake::Reading, base_url: &str) -> Rooted {
        rooted(desktop, Some(&one_provider(base_url)))
    }

    fn rooted(desktop: fake::Reading, settings: Option<&str>) -> Rooted {
        let dir = TempDir::new().unwrap();

        if let Some(settings) = settings {
            std::fs::write(dir.path().join(config::FILE_NAME), settings).unwrap();
        }

        let (capture, capturing) = desktop;

        Rooted {
            demysto: Demysto::speaking(
                dir.path().to_owned(),
                "1.2.3",
                capture,
                capturing,
                Environment::snapshot(),
                i18n::english(),
            )
            .unthrottled(),
            _dir: dir,
        }
    }

    /// A settings file naming one Provider, at `base_url`, offering one Model
    /// and nominating it.
    ///
    /// The Provider names no preset and no variable of its own, so that the
    /// environment of whoever is running the suite cannot reach into it.
    fn one_provider(base_url: &str) -> String {
        format!(
            "default_model = \"a provider/a-model\"\n\n{}",
            provider("a provider", base_url, "a-key", "a-model")
        )
    }

    /// One Provider block, offering one Model.
    fn provider(name: &str, base_url: &str, api_key: &str, model: &str) -> String {
        format!(
            "[[providers]]\nname = \"{name}\"\nbase_url = \"{base_url}\"\n\
             api_key = \"{api_key}\"\nmodels = [{{ id = \"{model}\" }}]\n"
        )
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
        ready_with(&one_provider(&format!("{}/v1", server.url())), selection)
    }

    /// The same, speaking whatever language its settings file states.
    ///
    /// The one facade the suite does not hand English to. Neither the variable
    /// nor the operating system is asked — an assertion about a sentence must
    /// not turn on the machine the suite is running on — so what is left is the
    /// leg this exists to exercise: the file (user story 59).
    fn ready_speaking(settings: &str, selection: &str) -> Rooted {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join(config::FILE_NAME),
            format!("version = 1\n\n{settings}"),
        )
        .unwrap();

        let desktop = Arc::new(FakeDesktop::new(None, Some(selection)));
        let (capture, capturing) = fake::over(&desktop);

        let demysto = Rooted {
            demysto: Demysto::speaking(
                dir.path().to_owned(),
                "1.2.3",
                capture,
                capturing,
                Environment::snapshot(),
                Words::spoken(i18n::chosen(
                    None,
                    config::stated_language(dir.path()).as_deref(),
                    None,
                )),
            )
            .unthrottled(),
            _dir: dir,
        };

        demysto.capture();

        demysto
    }

    /// A Demysto whose settings file holds exactly `settings`, having captured
    /// `selection`.
    fn ready_with(settings: &str, selection: &str) -> Rooted {
        let desktop = Arc::new(FakeDesktop::new(None, Some(selection)));
        let demysto = rooted(
            fake::over(&desktop),
            Some(&format!("version = 1\n\n{settings}")),
        );
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
            CaptureOutcome::Failed(error) => {
                panic!("the Capture failed: {}", error.message(&i18n::english()))
            }
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
            demysto((Box::new(Broken), Capturing::Selection)).capture(),
            CaptureOutcome::Failed(CaptureError::Clipboard("no owner".to_owned()))
        );
    }

    #[test]
    fn a_desktop_withholding_the_permission_says_so_rather_than_finding_nothing() {
        let desktop = Arc::new(
            FakeDesktop::new(Some("copied a moment ago"), Some("a paragraph"))
                .refusing_permission(),
        );

        // Not the clipboard it happens to be holding, and not "nothing was
        // selected": there is a Selection, and the reason Demysto cannot have
        // it is one the user can do something about.
        assert_eq!(
            demysto(fake::over(&desktop)).capture(),
            CaptureOutcome::Failed(CaptureError::Permission)
        );
    }

    #[test]
    fn a_capture_the_permission_refuses_disturbs_nothing() {
        let desktop = Arc::new(
            FakeDesktop::new(Some("copied a moment ago"), Some("a paragraph"))
                .refusing_permission(),
        );

        demysto(fake::over(&desktop)).capture();

        assert_eq!(
            desktop.clipboard_now(),
            Some("copied a moment ago".to_owned())
        );
    }

    #[test]
    fn the_permission_is_asked_about_at_every_capture() {
        // macOS withdraws it whenever the binary's signature changes, so an
        // answer held from startup is one that goes stale under a running
        // Demysto (the spec's *Shell and platform*).
        let desktop = Arc::new(FakeDesktop::new(None, Some("a paragraph")));
        let demysto = demysto(fake::over(&desktop));

        demysto.capture();
        demysto.capture();

        assert_eq!(desktop.permission_checks(), 2);
    }

    #[test]
    fn a_run_after_a_refused_permission_reports_the_permission() {
        let desktop = Arc::new(FakeDesktop::new(None, Some("a paragraph")).refusing_permission());
        let demysto = demysto(fake::over(&desktop));

        demysto.capture();

        // The path an Action's own Hotkey takes: nothing on screen said the
        // Capture was refused, so the Conversation is where it has to be said —
        // and "select some text and press the Hotkey again" would be advice
        // that cannot work (user story 55).
        assert_eq!(
            demysto.run("explain", &BTreeMap::new(), |_| {}).error(),
            Some(&RunError::Permission {
                message: CaptureError::Permission.message(&i18n::english())
            })
        );
    }

    #[test]
    fn a_run_after_a_capture_that_simply_found_nothing_still_says_so() {
        let desktop = Arc::new(FakeDesktop::new(None, None));
        let demysto = demysto(fake::over(&desktop));

        demysto.capture();

        assert!(matches!(
            demysto.run("explain", &BTreeMap::new(), |_| {}).error(),
            Some(RunError::NothingToRun { .. })
        ));
    }

    #[test]
    fn a_clipboard_only_session_needs_no_permission() {
        // It types into nothing, so there is nothing for a desktop to withhold
        // — which is the whole of Wayland's position (ADR-0003).
        let desktop =
            Arc::new(FakeDesktop::new(Some("copied by hand"), None).refusing_permission());

        assert_eq!(
            captured(&demysto(fake::clipboard_only_over(&desktop))),
            Captured::Clipboard(Selection::text("copied by hand"))
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

    /// What the windows show is what the Capture actually does, at the seam
    /// where the Capture is substituted (user story 56).
    #[test]
    fn a_session_that_reads_only_the_clipboard_says_so_for_the_interface() {
        let desktop = Arc::new(FakeDesktop::new(None, None));

        assert_eq!(
            demysto(fake::clipboard_only_over(&desktop))
                .status()
                .capturing,
            Capturing::ClipboardOnly
        );
    }

    #[test]
    fn a_session_that_reads_the_selection_has_nothing_to_say_about_it() {
        let desktop = Arc::new(FakeDesktop::new(None, Some("a paragraph")));

        assert_eq!(
            demysto(fake::over(&desktop)).status().capturing,
            Capturing::Selection
        );
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
    fn a_provider_that_refuses_the_key_says_so_in_its_own_words_and_names_itself() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_status(401)
            .with_body(json!({ "error": { "message": "Incorrect API key provided" } }).to_string())
            .create();

        let RunOutcome::Failed(RunError::Authentication { message, provider }) =
            run(&ready_to_run(&server, "a paragraph"))
        else {
            panic!("a refused key should be reported as one");
        };

        assert!(message.contains("Incorrect API key provided"), "{message}");
        assert!(message.contains("401"), "{message}");

        // Which Provider, so that the window can offer the settings that fix it
        // rather than the settings in general (user story 45).
        assert_eq!(provider, "a provider");
    }

    #[test]
    fn a_key_the_provider_is_not_allowed_to_use_here_is_a_refused_key_too() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_status(403)
            .with_body(
                json!({ "error": { "message": "Your account is not permitted" } }).to_string(),
            )
            .create();

        let RunOutcome::Failed(RunError::Authentication { message, .. }) =
            run(&ready_to_run(&server, "a paragraph"))
        else {
            panic!("a forbidden request should be reported as a refused key");
        };

        assert!(
            message.contains("Your account is not permitted"),
            "{message}"
        );
    }

    #[test]
    fn a_refusal_with_nothing_to_say_is_still_reported() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_status(500)
            .with_body("<html>Bad Gateway</html>")
            .create();

        // A Provider's own fault rather than the key's, and reported as the
        // kind it is: nothing in Settings would fix a server that is unwell,
        // so this is the failure that is offered a retry and not a route there.
        let RunOutcome::Failed(RunError::Provider { message }) =
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
            RunOutcome::Failed(RunError::Malformed { .. })
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
            RunOutcome::Failed(RunError::Malformed { .. })
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
            RunOutcome::Failed(RunError::Malformed { .. })
        ));
    }

    #[test]
    fn a_provider_that_cannot_be_reached_names_the_address_that_did_not_answer() {
        let desktop = Arc::new(FakeDesktop::new(None, Some("a paragraph")));
        let demysto = demysto_asking(fake::over(&desktop), "http://127.0.0.1:1/v1");
        demysto.capture();

        let RunOutcome::Failed(RunError::Unreachable { message }) = run(&demysto) else {
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
            RunOutcome::Failed(RunError::NothingToRun { message: _ })
        ));
    }

    #[test]
    fn a_run_with_no_provider_configured_names_the_file_that_would_configure_one() {
        let desktop = Arc::new(FakeDesktop::new(None, Some("a paragraph")));
        let demysto = demysto(fake::over(&desktop));
        demysto.capture();

        let RunOutcome::Failed(RunError::Configuration { message }) = run(&demysto) else {
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

    /// The other half of user story 30: the language the answer is asked for
    /// in is the language the interface speaks, which is now something the user
    /// can change rather than a constant.
    ///
    /// Asked for by its English name whatever the interface is drawn in — see
    /// `Interface::prompt_name`, and `language` for why a Model is told
    /// "Russian" rather than "Русский".
    #[test]
    fn explaining_asks_for_the_answer_in_whatever_language_the_interface_speaks() {
        let mut server = Server::new();
        let endpoint = asked_for(
            &mut server,
            vec![Matcher::Regex("answer in Russian".to_owned())],
        );

        let settings = format!(
            "language = \"ru\"\n{}",
            one_provider(&format!("{}/v1", server.url()))
        );

        run(&ready_speaking(
            &settings,
            "Der Mensch ist frei geschaffen, ist frei",
        ));

        endpoint.assert();
    }

    /// The Palette lists the built-ins in the language it is drawn in, and asks
    /// for their Parameters in it — a Palette that came up in English under a
    /// Russian interface would be the half of the tool the user reaches most.
    #[test]
    fn the_built_in_actions_are_named_in_the_language_the_interface_speaks() {
        let demysto = ready_speaking("language = \"ru\"\n", "a paragraph");

        let named: Vec<String> = demysto
            .actions()
            .into_iter()
            .map(|action| action.name)
            .collect();

        assert_eq!(named, ["Объяснить", "Перевести", "Пересказать"]);
    }

    /// And the language the translation offers to translate into is the one the
    /// user reads, by the name a Model is told it in.
    #[test]
    fn translating_offers_the_interface_language_as_what_to_translate_into() {
        let demysto = ready_speaking("language = \"ru\"\n", "a paragraph");

        let offered = demysto
            .actions()
            .into_iter()
            .find(|action| action.id == "translate")
            .expect("Translate is a built-in");

        assert_eq!(offered.parameters[0].default, "Russian");
        assert_eq!(offered.parameters[0].label, "На какой язык?");
    }

    /// User story 59: the language is changed in Settings, and everything
    /// Demysto has yet to say is owed in it — without a restart, and without
    /// the window that changed it being the only one that notices.
    #[test]
    fn a_language_saved_in_settings_is_the_one_demysto_speaks() {
        let demysto = unconfigured("a paragraph");

        assert_eq!(demysto.language(), Interface::English);

        let written = saved(
            &demysto,
            &Edit {
                language: Some("ru".to_owned()),
                ..edited(Vec::new(), None)
            },
        );

        assert_eq!(written.language.as_deref(), Some("ru"));
        assert_eq!(demysto.language(), Interface::Russian);

        // Which is not a fact about the facade alone: the tray menu is rebuilt
        // from the catalogue, and the Palette from the Actions.
        assert!(
            demysto.catalogue().actions[0].name == "Объяснить",
            "{:?}",
            demysto.catalogue().actions[0].name
        );
    }

    /// And it is stated in the file, which is where the next start reads it.
    #[test]
    fn the_language_saved_is_written_where_the_next_start_reads_it() {
        let demysto = unconfigured("a paragraph");

        saved(
            &demysto,
            &Edit {
                language: Some("ru".to_owned()),
                ..edited(Vec::new(), None)
            },
        );

        assert!(
            stated_in(&settings_file(&demysto)).contains("language = \"ru\""),
            "{}",
            settings_file(&demysto)
        );
    }

    /// Following the operating system is the setting taken out of the file, not
    /// a language written into it: somebody who chose one and changed their
    /// mind is back where a fresh installation starts (user story 58).
    #[test]
    fn choosing_to_follow_the_system_takes_the_language_out_of_the_file() {
        let demysto = unconfigured("a paragraph");

        saved(
            &demysto,
            &Edit {
                language: Some("ru".to_owned()),
                ..edited(Vec::new(), None)
            },
        );

        let written = saved(&demysto, &edited(Vec::new(), None));

        assert_eq!(written.language, None);
        assert!(
            !stated_in(&settings_file(&demysto)).contains("language"),
            "{}",
            settings_file(&demysto)
        );
    }

    /// A tag no catalogue answers to is not written and not reported: the
    /// window offers a list of two, so anything else came from somewhere that
    /// is not the window, and writing it would put a setting in the file that
    /// nothing acts on.
    #[test]
    fn a_language_demysto_does_not_speak_is_not_written() {
        let demysto = unconfigured("a paragraph");

        let written = saved(
            &demysto,
            &Edit {
                language: Some("xh".to_owned()),
                ..edited(Vec::new(), None)
            },
        );

        assert_eq!(written.language, None);
    }

    /// The window is told about the variable only where it is deciding
    /// something.
    ///
    /// One naming a language Demysto does not speak decides nothing —
    /// `i18n::chosen` passes it over and lets the settings answer — and saying
    /// so beside a field that does work would have the window report the
    /// opposite of what it is doing.
    #[test]
    fn a_language_variable_demysto_cannot_honour_is_not_reported_as_deciding() {
        let exported = |value: &str| {
            let dir = TempDir::new().unwrap();
            let desktop = Arc::new(FakeDesktop::new(None, None));
            let (capture, capturing) = fake::over(&desktop);

            Demysto::speaking(
                dir.path().to_owned(),
                "1.2.3",
                capture,
                capturing,
                config::Environment::holding(&[(LANGUAGE_ENV, value)]),
                i18n::english(),
            )
            .status()
            .language_env
        };

        assert_eq!(exported("xh"), None);
        assert_eq!(
            exported("ru"),
            Some(Exported {
                variable: LANGUAGE_ENV,
                value: "ru".to_owned()
            })
        );
    }

    /// A file that states one, by hand, in a language Demysto does not speak is
    /// read the same way: English, and nothing said about it. Refusing to start
    /// over a two-letter typo would be worse than speaking English.
    #[test]
    fn a_file_naming_a_language_demysto_does_not_speak_is_english() {
        let demysto = ready_speaking("language = \"xh\"\n", "a paragraph");

        assert_eq!(demysto.language(), Interface::English);
        assert_eq!(demysto.settings().unwrap().language, None);
    }

    /// The whole of what a Russian interface is for, in one sentence: the
    /// warning about a large Selection counts in it, and counts grammatically
    /// (user story 60).
    #[test]
    fn the_warning_about_a_large_selection_counts_in_the_interface_language() {
        let settings = format!(
            "language = \"ru\"\nlarge_selection = 3\n{}",
            // Nothing is asked of it: the warning is composed before the Model
            // is, which is the whole of what "before you spend tokens on it"
            // means for something that does not block.
            one_provider("http://127.0.0.1:1/v1")
        );

        let demysto = ready_speaking(&settings, "четыре символа");

        demysto.run("explain", &BTreeMap::new(), |_| {});

        let warning = showing(&demysto)
            .warning
            .expect("a Selection over the stated length is warned about");

        assert!(warning.contains("14 символов"), "{warning}");
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

        let RunOutcome::Failed(RunError::NoSuchAction { message }) =
            running(&demysto, "explan", &[])
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
    fn the_window_is_given_the_selection_its_answer_is_about() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body(answering("an answer"))
            .create();

        let demysto = ready_to_run(&server, "a paragraph");
        run(&demysto);

        // The Action's name says what was done; this says what it was done to.
        assert_eq!(showing(&demysto).preview.as_deref(), Some("a paragraph"));
        assert_eq!(demysto.selection().as_deref(), Some("a paragraph"));
    }

    #[test]
    fn a_selection_longer_than_the_window_quotes_is_cut_and_says_so() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body(answering("an answer"))
            .create();

        // Multibyte, because a Selection is as likely to be Russian as English
        // and the cut is counted in characters rather than in bytes.
        let long = "письмо ".repeat(100);
        let demysto = ready_to_run(&server, &long);
        run(&demysto);

        let opening: String = long.chars().take(conversation::PREVIEW).collect();

        assert_eq!(showing(&demysto).preview, Some(format!("{opening}…")));

        // And the rest of it is there for the asking, which is what expanding
        // the quotation does.
        assert_eq!(demysto.selection(), Some(long));
    }

    #[test]
    fn a_conversation_gone_back_to_is_quoted_by_its_own_selection() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body(answering("an answer"))
            .create();

        // Captured from the clipboard rather than from a selection, because
        // that is the desktop a test can put two different texts on in turn.
        let desktop = Arc::new(FakeDesktop::new(Some("the first paragraph"), None));
        let demysto = rooted(
            fake::over(&desktop),
            Some(&format!(
                "version = 1\n\n{}",
                one_provider(&format!("{}/v1", server.url()))
            )),
        );

        demysto.capture();
        run(&demysto);
        let earlier = showing(&demysto).id;

        desktop
            .set_clipboard_text(Some("the second paragraph"))
            .expect("the fake clipboard should take it");
        demysto.capture();
        run(&demysto);

        assert_eq!(
            showing(&demysto).preview.as_deref(),
            Some("the second paragraph")
        );

        // The Selection belongs to the Conversation, not to the session: going
        // back to an earlier one quotes what that one was asked about.
        assert_eq!(
            demysto
                .show_conversation(earlier)
                .and_then(|shown| shown.preview)
                .as_deref(),
            Some("the first paragraph")
        );
        assert_eq!(demysto.selection().as_deref(), Some("the first paragraph"));
    }

    #[test]
    fn a_conversation_with_no_selection_has_nothing_to_quote() {
        let desktop = Arc::new(FakeDesktop::new(None, None));
        let demysto = rooted(fake::over(&desktop), None);

        demysto.capture();

        // The empty Conversation a declared Run leaves behind when there was
        // nothing to run it on: the window heads it with the Action and quotes
        // nothing, rather than quoting an empty box.
        demysto.about_to_run("explain");

        assert_eq!(showing(&demysto).preview, None);
        assert_eq!(demysto.selection(), None);
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
        // replaces it, per the spec's *Errors*, and the retry hangs off it.
        let turns = turns(&demysto);

        assert_eq!(turns.len(), 2);
        assert!(
            turns.iter().all(|(_, outcome)| matches!(
                outcome,
                Some(RunOutcome::Failed(RunError::Provider { message: _ }))
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
            RunOutcome::Failed(RunError::NothingToRun { message: _ })
        ));
    }

    #[test]
    fn a_run_goes_to_the_provider_that_offers_the_model_the_default_names() {
        // Two Providers at two addresses with two keys, which is the whole
        // point of configuring more than one: the Default Model says which.
        let mut cheap = Server::new();
        let mut dear = Server::new();

        let asked = dear
            .mock("POST", "/v1/chat/completions")
            .match_header("authorization", "Bearer dear-key")
            .match_body(Matcher::PartialJson(json!({ "model": "sharp" })))
            .with_body(answering("an answer"))
            .create();
        let untouched = cheap
            .mock("POST", "/v1/chat/completions")
            .with_body(answering("the wrong Model answered"))
            .expect(0)
            .create();

        let settings = format!(
            "default_model = \"dear/sharp\"\n\n{}\n{}",
            provider(
                "cheap",
                &format!("{}/v1", cheap.url()),
                "cheap-key",
                "everyday"
            ),
            provider("dear", &format!("{}/v1", dear.url()), "dear-key", "sharp"),
        );

        assert_eq!(
            run(&ready_with(&settings, "a paragraph")),
            RunOutcome::Answered("an answer".to_owned())
        );

        asked.assert();
        untouched.assert();
    }

    #[test]
    fn a_provider_missing_its_key_costs_only_the_models_it_offers() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body(answering("an answer"))
            .create();

        // The first Provider names no key anywhere; the Default Model is the
        // second's, and the Run should never learn of the first.
        let settings = format!(
            "default_model = \"working/a-model\"\n\n\
             [[providers]]\nname = \"broken\"\nbase_url = \"http://127.0.0.1:1/v1\"\n\
             models = [{{ id = \"a-model\" }}]\n\n{}",
            provider(
                "working",
                &format!("{}/v1", server.url()),
                "a-key",
                "a-model"
            ),
        );

        assert_eq!(
            run(&ready_with(&settings, "a paragraph")),
            RunOutcome::Answered("an answer".to_owned())
        );
    }

    #[test]
    fn a_default_model_naming_nothing_sends_nothing_anywhere() {
        let mut server = Server::new();
        let untouched = server
            .mock("POST", "/v1/chat/completions")
            .with_body(answering("an answer"))
            .expect(0)
            .create();

        let settings = format!(
            "default_model = \"a provider/imagined\"\n\n{}",
            provider(
                "a provider",
                &format!("{}/v1", server.url()),
                "a-key",
                "a-model"
            )
        );

        let RunOutcome::Failed(RunError::Configuration { message }) =
            run(&ready_with(&settings, "a paragraph"))
        else {
            panic!("a Default Model naming nothing should fail for want of a setting");
        };

        assert!(message.contains("default_model"), "{message}");
        assert!(message.contains("a provider/a-model"), "{message}");
        untouched.assert();
    }

    #[test]
    fn a_provider_with_no_key_names_the_key_rather_than_being_asked() {
        let settings = "default_model = \"mine/a-model\"\n\n\
                        [[providers]]\nname = \"mine\"\nbase_url = \"http://127.0.0.1:1/v1\"\n\
                        models = [{ id = \"a-model\" }]\n";

        let RunOutcome::Failed(RunError::Configuration { message }) =
            run(&ready_with(settings, "a paragraph"))
        else {
            panic!("a Provider with no key should fail for want of one");
        };

        assert!(message.contains("api_key"), "{message}");
    }

    /// A settings file naming one local Provider, whose service has no keys.
    ///
    /// The preset is what says so; the stated base URL is what points it at the
    /// suite's server instead of the port LM Studio really listens on, which is
    /// the same override a user needs for a server on a port of their own.
    fn keyless_provider(base_url: &str) -> String {
        format!(
            "default_model = \"local/a-model\"\n\n\
             [[providers]]\nname = \"local\"\npreset = \"lmstudio\"\n\
             base_url = \"{base_url}\"\nmodels = [{{ id = \"a-model\" }}]\n"
        )
    }

    #[test]
    fn a_run_against_a_service_with_no_keys_sends_no_key() {
        // Not a placeholder, and not an empty header: nothing at all.
        let mut server = Server::new();
        let asked = server
            .mock("POST", "/v1/chat/completions")
            .match_header("authorization", Matcher::Missing)
            .with_body(answering("an answer"))
            .create();

        let settings = keyless_provider(&format!("{}/v1", server.url()));

        assert_eq!(
            run(&ready_with(&settings, "a paragraph")),
            RunOutcome::Answered("an answer".to_owned())
        );

        asked.assert();
    }

    #[test]
    fn the_model_list_of_a_service_with_no_keys_is_fetched_without_one() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("GET", "/v1/models")
            .match_header("authorization", Matcher::Missing)
            .with_body(json!({ "data": [{ "id": "a-model" }] }).to_string())
            .create();

        let settings = keyless_provider(&format!("{}/v1", server.url()));
        let demysto = ready_with(&settings, "a paragraph");

        let local = ProviderEdit {
            was: Some("local".to_owned()),
            preset: Some("lmstudio".to_owned()),
            base_url: Some(format!("{}/v1", server.url())),
            ..drafted("local")
        };

        assert_eq!(demysto.models_offered_by(&local).unwrap(), ["a-model"]);
    }

    #[test]
    fn the_model_list_comes_from_the_provider_that_offers_them() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("GET", "/v1/models")
            .match_header("authorization", "Bearer a-key")
            .with_body(
                json!({ "data": [{ "id": "a-model" }, { "id": "another-model" }] }).to_string(),
            )
            .create();

        let demysto = ready_to_run(&server, "a paragraph");

        // The key is the one the file already holds for this Provider: the
        // window is never shown it, so asking for the list cannot resend it.
        let saved = ProviderEdit {
            was: Some("a provider".to_owned()),
            base_url: Some(format!("{}/v1", server.url())),
            ..drafted("a provider")
        };

        assert_eq!(
            demysto.models_offered_by(&saved).unwrap(),
            ["a-model", "another-model"]
        );
    }

    #[test]
    fn the_model_list_of_a_provider_that_answers_nowhere_says_which_setting_is_missing() {
        let server = Server::new();
        let demysto = ready_to_run(&server, "a paragraph");

        // Neither a base URL nor a preset to take one from: nothing to ask.
        let Err(RunError::Configuration { message }) =
            demysto.models_offered_by(&drafted("imagined"))
        else {
            panic!("a Provider that answers nowhere should fail for want of a setting");
        };

        assert!(message.contains("imagined"), "{message}");
    }

    /// A Provider as the settings window hands one back: named, and otherwise
    /// stating nothing. Every test below says only what it is about.
    fn drafted(name: &str) -> ProviderEdit {
        ProviderEdit {
            was: None,
            name: name.to_owned(),
            base_url: None,
            preset: None,
            api_key_env: None,
            api_key: KeyEdit::Keep,
            models: Vec::new(),
        }
    }

    /// One Model of a Provider, as the window states it.
    fn offering(id: &str, vision: bool) -> ConfiguredModel {
        ConfiguredModel {
            id: id.to_owned(),
            vision,
        }
    }

    /// What the window saves: these Providers, and this Default Model.
    fn edited(providers: Vec<ProviderEdit>, default: Option<&str>) -> Edit {
        Edit {
            providers,
            default_model: default.map(ToOwned::to_owned),
            default_vision_model: None,
            palette_hotkey: None,
            large_selection: None,
            language: None,
        }
    }

    /// One Provider, at `base_url`, with a key typed into the window and one
    /// Model — the whole of what somebody configures on a first run.
    fn configuring(base_url: &str) -> Edit {
        edited(
            vec![ProviderEdit {
                base_url: Some(base_url.to_owned()),
                api_key: KeyEdit::Set {
                    key: "a-key".to_owned(),
                },
                models: vec![offering("a-model", false)],
                ..drafted("a provider")
            }],
            Some("a provider/a-model"),
        )
    }

    /// Somewhere a key typed into the window can be put to its Provider.
    ///
    /// Every save of a typed key does that before it writes anything, so a test
    /// that saves one needs a Provider to accept it — even when what the test
    /// is about is what ends up in the file. The Mock comes back with the
    /// address because dropping it would take it off the server again.
    fn accepting(server: &mut ServerGuard) -> (Mock, String) {
        let accepted = server
            .mock("POST", "/v1/chat/completions")
            .with_body(answering("hello"))
            .create();

        (accepted, format!("{}/v1", server.url()))
    }

    /// A Demysto with nothing configured, having captured `selection` — the
    /// state a fresh installation is in when Settings is first opened.
    fn unconfigured(selection: &str) -> Rooted {
        let desktop = Arc::new(FakeDesktop::new(None, Some(selection)));
        let demysto = rooted(fake::over(&desktop), None);
        demysto.capture();

        demysto
    }

    /// What the settings file holds, as text.
    fn settings_file(demysto: &Demysto) -> String {
        std::fs::read_to_string(demysto.config_dir().join(config::FILE_NAME)).unwrap()
    }

    /// The lines of a settings file that configure something, as against the
    /// ones that explain what configuring something looks like.
    fn stated_in(file: &str) -> String {
        file.lines()
            .filter(|line| !line.trim_start().starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn saved(demysto: &Demysto, edit: &Edit) -> Settings {
        demysto
            .save_settings(edit)
            .expect("the settings should have saved")
    }

    #[test]
    fn a_provider_configured_in_the_interface_is_what_the_next_run_asks() {
        // The whole of ticket 08 in one Run: a Provider, a key and a Model
        // entered in the window, and an Action running against them with
        // nothing restarted and nothing edited by hand.
        let mut server = Server::new();
        let asked = server
            .mock("POST", "/v1/chat/completions")
            .match_header("authorization", "Bearer a-key")
            .match_body(Matcher::PartialJson(json!({ "model": "a-model" })))
            .with_body(answering("an answer"))
            // Twice, and the same request both times: the save puts the key
            // typed into the window to this Provider before writing it, and
            // that is the request a Run makes — which is the point of it.
            .expect(2)
            .create();

        let demysto = unconfigured("a paragraph");
        saved(&demysto, &configuring(&format!("{}/v1", server.url())));

        assert_eq!(run(&demysto), RunOutcome::Answered("an answer".to_owned()));

        asked.assert();
    }

    #[test]
    fn the_settings_the_interface_writes_are_the_settings_it_reads_back() {
        let demysto = unconfigured("a paragraph");

        let edit = Edit {
            providers: vec![
                ProviderEdit {
                    preset: Some("openai".to_owned()),
                    api_key_env: Some("MY_OWN_KEY".to_owned()),
                    models: vec![offering("gpt-4o-mini", false), offering("gpt-4o", true)],
                    ..drafted("openai")
                },
                ProviderEdit {
                    base_url: Some("http://localhost:9999/v1".to_owned()),
                    preset: Some("ollama".to_owned()),
                    models: vec![offering("qwen3", false)],
                    ..drafted("local")
                },
            ],
            default_model: Some("openai/gpt-4o-mini".to_owned()),
            default_vision_model: Some("openai/gpt-4o".to_owned()),
            palette_hotkey: Some("Ctrl+Alt+Space".to_owned()),
            large_selection: Some(1_000),
            language: Some("ru".to_owned()),
        };

        let written = saved(&demysto, &edit);

        // What the save answered with came from reading the file back, and
        // asking again reads it again: two round trips through the same file.
        assert_eq!(demysto.settings().unwrap(), written);

        assert_eq!(
            written
                .providers
                .iter()
                .map(|it| it.name.as_str())
                .collect::<Vec<_>>(),
            ["openai", "local"]
        );
        assert_eq!(written.providers[0].preset.as_deref(), Some("openai"));
        assert_eq!(written.providers[0].base_url, None);
        assert_eq!(
            written.providers[0].api_key_env.as_deref(),
            Some("MY_OWN_KEY")
        );
        assert_eq!(written.providers[0].models, edit.providers[0].models);
        assert_eq!(written.large_selection, Some(1_000));
        assert_eq!(
            written.providers[1].base_url.as_deref(),
            Some("http://localhost:9999/v1")
        );
        assert_eq!(written.default_model.as_deref(), Some("openai/gpt-4o-mini"));
        assert_eq!(
            written.default_vision_model.as_deref(),
            Some("openai/gpt-4o")
        );
    }

    #[test]
    fn a_model_marked_vision_capable_is_still_marked_when_it_is_read_back() {
        // Stated by the user and nowhere inferred, so the one place it can be
        // lost is between the window and the file.
        let demysto = unconfigured("a paragraph");

        let edit = edited(
            vec![ProviderEdit {
                preset: Some("openai".to_owned()),
                models: vec![offering("gpt-4o-mini", false), offering("gpt-4o", true)],
                ..drafted("openai")
            }],
            None,
        );

        let written = saved(&demysto, &edit);

        assert_eq!(
            written.providers[0]
                .models
                .iter()
                .map(|model| model.vision)
                .collect::<Vec<_>>(),
            [false, true]
        );
    }

    #[test]
    fn a_provider_removed_in_the_interface_is_gone_from_the_settings() {
        let demysto = unconfigured("a paragraph");

        let both = edited(
            vec![
                ProviderEdit {
                    preset: Some("openai".to_owned()),
                    ..drafted("openai")
                },
                ProviderEdit {
                    preset: Some("ollama".to_owned()),
                    ..drafted("local")
                },
            ],
            None,
        );
        saved(&demysto, &both);

        let kept = edited(
            vec![ProviderEdit {
                was: Some("local".to_owned()),
                preset: Some("ollama".to_owned()),
                ..drafted("local")
            }],
            None,
        );
        let written = saved(&demysto, &kept);

        assert_eq!(
            written
                .providers
                .iter()
                .map(|it| it.name.as_str())
                .collect::<Vec<_>>(),
            ["local"]
        );
        // The preamble names every preset, "openai" among them, so what has to
        // have gone is the Provider rather than the word.
        // What the file states, rather than what it says: the preamble's own
        // commented example names an "openai" Provider and always will.
        assert!(!stated_in(&settings_file(&demysto)).contains("name = \"openai\""));
    }

    #[test]
    fn a_provider_renamed_in_the_interface_keeps_the_key_it_was_configured_with() {
        // The window is never shown the key, so it cannot hand it back with the
        // new name: what the file holds has to follow the rename by itself.
        let mut server = Server::new();
        let asked = server
            .mock("POST", "/v1/chat/completions")
            .match_header("authorization", "Bearer a-key")
            .with_body(answering("an answer"))
            .create();

        let demysto = ready_to_run(&server, "a paragraph");

        let renamed = edited(
            vec![ProviderEdit {
                was: Some("a provider".to_owned()),
                base_url: Some(format!("{}/v1", server.url())),
                models: vec![offering("a-model", false)],
                ..drafted("the same provider")
            }],
            Some("the same provider/a-model"),
        );
        let written = saved(&demysto, &renamed);

        assert_eq!(written.providers[0].key, KeyStanding::InFile);
        assert_eq!(run(&demysto), RunOutcome::Answered("an answer".to_owned()));

        asked.assert();
    }

    #[test]
    fn the_key_is_not_in_what_the_interface_is_shown() {
        let mut server = Server::new();
        let (_accepted, at) = accepting(&mut server);
        // ADR-0002 pays for the key being on disk with exactly one promise, and
        // this is it: the shape the window is handed has nowhere to put a key,
        // and this is the assertion that says so of the values as well.
        let demysto = unconfigured("a paragraph");
        saved(&demysto, &configuring(&at));

        let shown = serde_json::to_string(&demysto.settings().unwrap()).unwrap();

        assert!(settings_file(&demysto).contains("a-key"));
        assert!(!shown.contains("a-key"), "{shown}");
    }

    #[test]
    fn a_key_taken_out_in_the_interface_is_gone_from_the_file() {
        let mut server = Server::new();
        let (_accepted, at) = accepting(&mut server);
        let demysto = unconfigured("a paragraph");
        saved(&demysto, &configuring(&at));

        let forgotten = edited(
            vec![ProviderEdit {
                was: Some("a provider".to_owned()),
                base_url: Some(at.clone()),
                api_key: KeyEdit::Forget,
                ..drafted("a provider")
            }],
            None,
        );
        let written = saved(&demysto, &forgotten);

        assert_eq!(written.providers[0].key, KeyStanding::Missing);
        assert!(!settings_file(&demysto).contains("a-key"));
    }

    #[test]
    fn the_interface_says_where_a_key_is_without_saying_what_it_is() {
        let demysto = unconfigured("a paragraph");

        let stated = edited(
            vec![
                ProviderEdit {
                    preset: Some("openai".to_owned()),
                    api_key: KeyEdit::Set {
                        key: "in-the-file".to_owned(),
                    },
                    ..drafted("openai")
                },
                ProviderEdit {
                    preset: Some("ollama".to_owned()),
                    ..drafted("local")
                },
                ProviderEdit {
                    base_url: Some("https://elsewhere.example/v1".to_owned()),
                    ..drafted("elsewhere")
                },
            ],
            None,
        );
        let written = saved(&demysto, &stated);

        assert_eq!(written.providers[0].key, KeyStanding::InFile);
        // A local server has no keys at all, so there is none to go looking for.
        assert_eq!(written.providers[1].key, KeyStanding::NotNeeded);
        assert_eq!(written.providers[2].key, KeyStanding::Missing);
    }

    #[test]
    fn what_the_settings_file_says_about_itself_survives_the_interface_writing_it() {
        let mut server = Server::new();
        let (_accepted, at) = accepting(&mut server);
        // The file is the user's, and it is also where Demysto explains itself
        // to somebody who opened it. A save is a guest in it.
        let demysto = unconfigured("a paragraph");
        let before = settings_file(&demysto);

        saved(&demysto, &configuring(&at));
        let after = settings_file(&demysto);

        let prose: Vec<&str> = before
            .lines()
            .filter(|line| line.starts_with('#'))
            .collect();

        assert!(!prose.is_empty(), "the template should explain itself");
        for line in prose {
            assert!(after.contains(line), "{line}");
        }
    }

    #[test]
    fn the_palette_hotkey_the_interface_writes_is_the_one_it_reads_back() {
        let demysto = unconfigured("a paragraph");

        let written = saved(
            &demysto,
            &Edit {
                palette_hotkey: Some("Ctrl+Alt+P".to_owned()),
                ..edited(Vec::new(), None)
            },
        );

        assert_eq!(written.palette_hotkey.as_deref(), Some("Ctrl+Alt+P"));
        // What the save answered with came from reading the file back, and
        // asking again reads it again.
        assert_eq!(demysto.settings().unwrap(), written);
    }

    #[test]
    fn a_palette_hotkey_nobody_could_claim_is_still_written() {
        // The core does not judge a Hotkey, and must not start: whether a
        // combination parses is the desktop's answer, and a check here would be
        // a second opinion that could disagree with the one that matters. What
        // it costs somebody is a sentence in the window, not a refused save.
        let demysto = unconfigured("a paragraph");

        let written = saved(
            &demysto,
            &Edit {
                palette_hotkey: Some("not a hotkey at all".to_owned()),
                ..edited(Vec::new(), None)
            },
        );

        assert_eq!(
            written.palette_hotkey.as_deref(),
            Some("not a hotkey at all")
        );
    }

    #[test]
    fn a_blank_palette_hotkey_is_one_nobody_stated() {
        // Absence rather than emptiness: `palette_hotkey = ""` is a combination
        // that could never be claimed, so it would report itself on every start
        // — where the user only meant to go back to the built-in one.
        let demysto = unconfigured("a paragraph");

        let written = saved(
            &demysto,
            &Edit {
                palette_hotkey: Some("   ".to_owned()),
                ..edited(Vec::new(), None)
            },
        );

        assert_eq!(written.palette_hotkey, None);
        assert!(
            !stated_in(&settings_file(&demysto)).contains("palette_hotkey"),
            "{}",
            settings_file(&demysto)
        );
    }

    #[test]
    fn a_palette_hotkey_stated_by_hand_is_read_as_it_was_written() {
        // Trimmed, because the Hotkey parser will not trim it: " F13" would
        // otherwise read back looking right and refuse to be claimed, for a
        // reason nothing on screen could show.
        let demysto = unconfigured("a paragraph");
        let path = demysto.config_dir().join(config::FILE_NAME);

        std::fs::write(&path, "palette_hotkey = \"  F13  \"\n").unwrap();

        assert_eq!(
            demysto.settings().unwrap().palette_hotkey.as_deref(),
            Some("F13")
        );
        assert_eq!(demysto.palette_hotkey().as_deref(), Some("F13"));
    }

    #[test]
    fn a_palette_hotkey_can_be_set_without_disturbing_what_is_written_around_it() {
        // Adding a root key is the one edit that could move somebody's own
        // comment away from the line it was written about, and the test that
        // guards the preamble never adds one.
        let demysto = unconfigured("a paragraph");
        let path = demysto.config_dir().join(config::FILE_NAME);
        let mine = "# the cheap one, for everything that is not hard";

        std::fs::write(&path, format!("{mine}\ndefault_model = \"local/qwen3\"\n")).unwrap();

        // A keyless preset, so that saving asks no Provider anything.
        let local = ProviderEdit {
            preset: Some("ollama".to_owned()),
            models: vec![offering("qwen3", false)],
            ..drafted("local")
        };

        saved(
            &demysto,
            &Edit {
                palette_hotkey: Some("F13".to_owned()),
                ..edited(vec![local], Some("local/qwen3"))
            },
        );

        let after = settings_file(&demysto);
        let lines: Vec<&str> = after.lines().collect();
        let at = lines.iter().position(|line| *line == mine).expect(&after);

        assert!(
            lines[at + 1].starts_with("default_model"),
            "the comment should still sit above what it was written about: {after}"
        );
        assert!(after.contains("palette_hotkey = \"F13\""), "{after}");
    }

    #[test]
    fn a_key_that_types_nothing_needs_no_modifier() {
        let demysto = unconfigured("a paragraph");

        for key in [
            "Pause",
            "PrintScreen",
            "ScrollLock",
            "F13",
            "F24",
            "AudioVolumeMute",
            "MediaPlayPause",
            "MediaTrackPrevious",
        ] {
            assert!(demysto.needs_no_modifier(key), "{key}");
        }
    }

    #[test]
    fn a_key_that_types_something_still_needs_a_modifier() {
        // F12 beside F13 is the boundary somebody will widen by accident: the
        // ordinary function row is bound by applications, and claiming one
        // takes it from every one of them.
        let demysto = unconfigured("a paragraph");

        for key in [
            "KeyE",
            "Digit1",
            "F1",
            "F12",
            "Escape",
            "Space",
            "Enter",
            "Tab",
            "Backspace",
            "Delete",
            "Insert",
            "ArrowUp",
            "Home",
            "PageDown",
            "Comma",
        ] {
            assert!(!demysto.needs_no_modifier(key), "{key}");
        }
    }

    #[test]
    fn every_key_that_needs_no_modifier_is_one_a_hotkey_could_be_written_with() {
        let demysto = unconfigured("a paragraph");
        let keys = demysto.keys_that_need_no_modifier();

        for key in &keys {
            assert!(!key.is_empty());
            // A `+` is what separates a Hotkey's parts, on both sides of the
            // bridge: one inside a key would be read as two.
            assert!(!key.contains('+'), "{key}");
            assert!(!key.contains(char::is_whitespace), "{key}");
            assert_eq!(keys.iter().filter(|held| *held == key).count(), 1, "{key}");
        }
    }

    #[test]
    fn settings_the_interface_could_not_read_back_are_not_written() {
        let mut server = Server::new();
        let (_accepted, at) = accepting(&mut server);
        let demysto = unconfigured("a paragraph");
        saved(&demysto, &configuring(&at));
        let before = settings_file(&demysto);

        // Two Providers of one name: a Model of either could not be named, so
        // this is a file Demysto could not act on.
        let clashing = edited(
            vec![
                ProviderEdit {
                    preset: Some("openai".to_owned()),
                    ..drafted("twice")
                },
                ProviderEdit {
                    preset: Some("deepseek".to_owned()),
                    ..drafted("twice")
                },
            ],
            None,
        );

        let Err(ConfigError::Malformed(message)) = demysto.save_settings(&clashing) else {
            panic!("two Providers of one name should not have saved");
        };

        assert!(message.contains("twice"), "{message}");
        assert_eq!(settings_file(&demysto), before);
    }

    #[test]
    fn the_last_provider_can_be_removed_in_the_interface() {
        let mut server = Server::new();
        let (_accepted, at) = accepting(&mut server);
        // Where somebody starting over passes through. Demysto has nothing to
        // run against afterwards, and says so at the Run rather than by
        // refusing to save.
        let demysto = unconfigured("a paragraph");
        saved(&demysto, &configuring(&at));

        let emptied = saved(&demysto, &edited(Vec::new(), None));

        assert!(emptied.providers.is_empty());
        assert!(matches!(run(&demysto), RunOutcome::Failed(_)));
    }

    #[cfg(unix)]
    #[test]
    fn the_settings_file_is_still_owner_only_after_the_interface_writes_it() {
        use std::os::unix::fs::PermissionsExt;

        let mut server = Server::new();
        let (_accepted, at) = accepting(&mut server);
        let demysto = unconfigured("a paragraph");
        saved(&demysto, &configuring(&at));

        let file = demysto.config_dir().join(config::FILE_NAME);
        let mode = std::fs::metadata(&file).unwrap().permissions().mode();

        assert_eq!(mode & 0o777, 0o600, "{mode:o}");
    }

    #[test]
    fn a_key_is_verified_against_the_provider_before_it_is_saved() {
        let mut server = Server::new();
        let tried = server
            .mock("POST", "/v1/chat/completions")
            .match_header("authorization", "Bearer a-key")
            .match_body(Matcher::PartialJson(json!({ "model": "a-model" })))
            .with_body(answering("hello"))
            .create();

        let demysto = unconfigured("a paragraph");

        // Nothing saved: the Provider being verified is the one on screen.
        let typed = ProviderEdit {
            base_url: Some(format!("{}/v1", server.url())),
            api_key: KeyEdit::Set {
                key: "a-key".to_owned(),
            },
            ..drafted("a provider")
        };

        assert_eq!(demysto.verify(&typed, "a-model"), Ok(()));
        assert!(demysto.settings().unwrap().providers.is_empty());

        tried.assert();
    }

    #[test]
    fn a_key_the_provider_refuses_is_not_saved() {
        // The ticket asks for a key to be "verified against the Provider before
        // it is saved", which is a stronger thing than a button somewhere that
        // would have said so.
        let mut server = Server::new();
        let _refusing = server
            .mock("POST", "/v1/chat/completions")
            .with_status(401)
            .with_body(json!({ "error": { "message": "Incorrect API key provided" } }).to_string())
            .create();

        let demysto = unconfigured("a paragraph");
        let before = settings_file(&demysto);

        let Err(ConfigError::Refused(message)) =
            demysto.save_settings(&configuring(&format!("{}/v1", server.url())))
        else {
            panic!("a key the Provider refused should not have been saved");
        };

        assert!(message.contains("Incorrect API key provided"), "{message}");
        assert!(message.contains("a provider"), "{message}");
        assert_eq!(settings_file(&demysto), before);
    }

    #[test]
    fn a_provider_that_cannot_be_reached_does_not_stop_a_key_being_saved() {
        // A server that is not running yet, or a laptop off the network, is no
        // evidence about a key — and somebody who could not save until their
        // Provider came back would be somebody who could not configure Demysto.
        let demysto = unconfigured("a paragraph");

        // Nothing listens on port 1, and nothing is meant to: the connection is
        // refused at once rather than waited on.
        let written = saved(&demysto, &configuring("http://127.0.0.1:1/v1"));

        assert_eq!(written.providers[0].key, KeyStanding::InFile);
    }

    #[test]
    fn a_key_saved_without_a_model_to_try_it_against_is_saved_unverified() {
        // There is no request to make: verification is a request to a Model,
        // and this Provider has none yet. Ticket 08's own order — fetch the
        // Models, then verify — is what puts one there.
        let demysto = unconfigured("a paragraph");

        let alone = edited(
            vec![ProviderEdit {
                base_url: Some("http://127.0.0.1:1/v1".to_owned()),
                api_key: KeyEdit::Set {
                    key: "a-key".to_owned(),
                },
                ..drafted("a provider")
            }],
            None,
        );
        let written = saved(&demysto, &alone);

        assert_eq!(written.providers[0].key, KeyStanding::InFile);
        assert!(written.providers[0].models.is_empty());
    }

    #[test]
    fn a_default_naming_a_model_no_provider_offers_is_not_saved() {
        // What renaming a Provider does to the Default Model that named it. The
        // key follows a rename and a nomination cannot, so the window is told
        // to pick again rather than left to write settings whose next Run fails.
        let mut server = Server::new();
        let (_accepted, at) = accepting(&mut server);
        let demysto = unconfigured("a paragraph");
        saved(&demysto, &configuring(&at));

        let renamed = edited(
            vec![ProviderEdit {
                was: Some("a provider".to_owned()),
                base_url: Some(at.clone()),
                models: vec![offering("a-model", false)],
                ..drafted("renamed")
            }],
            // Still naming the Provider by the name it no longer has.
            Some("a provider/a-model"),
        );

        let Err(ConfigError::Malformed(message)) = demysto.save_settings(&renamed) else {
            panic!("a Default Model naming nothing should not have saved");
        };

        assert!(message.contains("default_model"), "{message}");
        assert!(message.contains("renamed/a-model"), "{message}");
    }

    #[test]
    fn a_key_the_provider_refuses_is_reported_in_the_providers_own_words() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_status(401)
            .with_body(json!({ "error": { "message": "Incorrect API key provided" } }).to_string())
            .create();

        let demysto = unconfigured("a paragraph");

        let typed = ProviderEdit {
            base_url: Some(format!("{}/v1", server.url())),
            api_key: KeyEdit::Set {
                key: "the-wrong-key".to_owned(),
            },
            ..drafted("a provider")
        };

        let Err(RunError::Authentication { message, .. }) = demysto.verify(&typed, "a-model")
        else {
            panic!("a key the Provider refused should be reported as its refusal");
        };

        assert!(message.contains("Incorrect API key provided"), "{message}");
    }

    #[test]
    fn every_preset_the_settings_file_understands_is_offered_by_the_interface() {
        // The window offers what the file accepts, from the same one place: a
        // preset it offered and the file refused would be a dead end.
        let demysto = unconfigured("a paragraph");

        for preset in demysto.presets() {
            let using = edited(
                vec![ProviderEdit {
                    preset: Some(preset.name.clone()),
                    models: vec![offering("a-model", false)],
                    ..drafted("a provider")
                }],
                None,
            );
            let written = saved(&demysto, &using);

            assert_eq!(
                written.providers[0].preset.as_deref(),
                Some(preset.name.as_str())
            );
            assert_eq!(
                written.providers[0].key == KeyStanding::NotNeeded,
                !preset.needs_key,
                "{}",
                preset.name
            );
        }
    }

    // The Action catalogue on disk: what the user writes, what they change
    // about a built-in, and the effective set the two make together (ticket 09).

    /// An edit of a new Action, stating the two things one has to state.
    fn writing(name: &str, template: &str) -> ActionEdit {
        ActionEdit {
            id: None,
            name: name.to_owned(),
            template: template.to_owned(),
            parameters: Vec::new(),
            model: None,
            hotkey: None,
            accepts: vec![Kind::Text],
        }
    }

    /// An edit of the Action already filed under `id`, which is what the window
    /// hands back for a built-in it is overriding.
    fn changing(id: &str, name: &str, template: &str) -> ActionEdit {
        ActionEdit {
            id: Some(id.to_owned()),
            ..writing(name, template)
        }
    }

    fn authored(demysto: &Demysto, edit: &ActionEdit) -> Catalogue {
        demysto
            .save_action(edit)
            .expect("the Action should have saved")
    }

    /// Why a save was refused. Panics when it was not.
    fn refused(demysto: &Demysto, edit: &ActionEdit) -> String {
        match demysto.save_action(edit) {
            Err(error) => error.message().to_owned(),
            Ok(_) => panic!("the Action should not have saved"),
        }
    }

    /// The Action the catalogue holds under `id`. Panics when it holds none.
    fn defined(demysto: &Demysto, id: &str) -> DefinedAction {
        demysto
            .catalogue()
            .actions
            .into_iter()
            .find(|action| action.id == id)
            .unwrap_or_else(|| panic!("the catalogue should hold an Action called {id:?}"))
    }

    /// Where the Actions live, whether or not anything has put one there yet.
    fn actions_dir(demysto: &Demysto) -> PathBuf {
        demysto.config_dir().join(catalogue::DIR_NAME)
    }

    /// Writes an Action file by hand, the way a file somebody was sent arrives.
    fn dropped_in(demysto: &Demysto, id: &str, text: &str) {
        let dir = actions_dir(demysto);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{id}.toml")), text).unwrap();
    }

    /// What the catalogue is filed under, in its order.
    fn catalogued(demysto: &Demysto) -> Vec<String> {
        demysto
            .catalogue()
            .actions
            .into_iter()
            .map(|action| action.id)
            .collect()
    }

    #[test]
    fn a_fresh_installation_has_the_built_in_actions_and_nothing_written_anywhere() {
        let demysto = unconfigured("a paragraph");

        assert_eq!(catalogued(&demysto), ["explain", "translate", "summarize"]);
        assert!(demysto.catalogue().unreadable.is_empty());

        // ADR-0005: the configuration directory belongs to the user, and a
        // built-in seeded into it could never be improved by a later version.
        // Reading the catalogue is not an occasion to write into it.
        assert!(
            !actions_dir(&demysto).exists(),
            "reading the catalogue should not have created a directory"
        );
    }

    #[test]
    fn an_action_the_user_writes_is_offered_beside_the_built_in_ones() {
        let demysto = unconfigured("a paragraph");

        authored(
            &demysto,
            &writing("Rewrite plainly", "Rewrite: {{selection}}"),
        );

        // The built-ins keep their order, which is how often they are reached
        // for; the user's own follow.
        assert_eq!(
            offered(&demysto),
            ["Explain", "Translate", "Summarize", "Rewrite plainly"]
        );
    }

    #[test]
    fn an_action_the_user_writes_runs_its_own_prompt() {
        let mut server = Server::new();
        let endpoint = asked_for(
            &mut server,
            vec![
                Matcher::Regex("Rewrite this so a child could read it".to_owned()),
                Matcher::Regex("Ceci n'est pas une pipe".to_owned()),
            ],
        );

        let demysto = ready_to_run(&server, "Ceci n'est pas une pipe");
        authored(
            &demysto,
            &writing(
                "Rewrite plainly",
                "Rewrite this so a child could read it:\n\n{{selection}}",
            ),
        );

        running(&demysto, "rewrite-plainly", &[]);

        endpoint.assert();
    }

    #[test]
    fn each_action_the_user_writes_is_a_file_of_its_own() {
        // User story 29: one file each, so that one can be backed up or sent to
        // a colleague.
        let demysto = unconfigured("a paragraph");

        authored(
            &demysto,
            &writing("Rewrite plainly", "Rewrite: {{selection}}"),
        );
        authored(
            &demysto,
            &writing("Find the flaw", "Fault it: {{selection}}"),
        );

        let mut files: Vec<String> = std::fs::read_dir(actions_dir(&demysto))
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        files.sort();

        assert_eq!(files, ["find-the-flaw.toml", "rewrite-plainly.toml"]);
    }

    #[test]
    fn an_action_carried_to_another_installation_as_a_file_runs_there() {
        // The other half of user story 29: a file arriving in that directory is
        // an Action, with nothing to import and nothing restarted.
        let mut server = Server::new();
        let endpoint = asked_for(&mut server, vec![Matcher::Regex("Fault it".to_owned())]);

        let demysto = ready_to_run(&server, "a paragraph");
        dropped_in(
            &demysto,
            "find-the-flaw",
            "name = \"Find the flaw\"\ntemplate = \"Fault it: {{selection}}\"\n",
        );

        assert!(offered(&demysto).contains(&"Find the flaw".to_owned()));

        running(&demysto, "find-the-flaw", &[]);

        endpoint.assert();
    }

    #[test]
    fn an_action_collects_the_parameters_it_declares() {
        let mut server = Server::new();
        let endpoint = asked_for(
            &mut server,
            vec![Matcher::Regex("Rewrite this for a lawyer".to_owned())],
        );

        let demysto = ready_to_run(&server, "a paragraph");
        authored(
            &demysto,
            &ActionEdit {
                parameters: vec![Parameter {
                    id: "reader".to_owned(),
                    label: "For whom?".to_owned(),
                    default: "a child".to_owned(),
                }],
                ..writing("Rewrite for", "Rewrite this for {{reader}}: {{selection}}")
            },
        );

        assert_eq!(
            defined(&demysto, "rewrite-for").parameters[0].label,
            "For whom?"
        );

        running(&demysto, "rewrite-for", &[("reader", "a lawyer")]);

        endpoint.assert();
    }

    #[test]
    fn an_action_can_be_deleted() {
        let demysto = unconfigured("a paragraph");
        let written = authored(
            &demysto,
            &writing("Rewrite plainly", "Rewrite: {{selection}}"),
        );
        let path = written.actions.last().unwrap().path.clone().unwrap();

        demysto
            .delete_action("rewrite-plainly")
            .expect("the Action should have been deleted");

        assert_eq!(catalogued(&demysto), ["explain", "translate", "summarize"]);
        assert!(!path.exists(), "the file should have gone with it");
    }

    #[test]
    fn deleting_an_action_that_is_already_gone_says_so_rather_than_failing_silently() {
        let demysto = unconfigured("a paragraph");

        let Err(ActionError::NoSuchAction(message)) = demysto.delete_action("rewrite-plainly")
        else {
            panic!("deleting an Action nothing holds should be reported as one");
        };

        assert!(message.contains("rewrite-plainly"), "{message}");
    }

    #[test]
    fn an_override_replaces_the_prompt_of_the_built_in_it_is_filed_under() {
        // User story 26: adjust the wording without recreating the Action.
        let mut server = Server::new();
        let endpoint = asked_for(
            &mut server,
            vec![Matcher::Regex("Explain it in one sentence".to_owned())],
        );

        let demysto = ready_to_run(&server, "a paragraph");
        authored(
            &demysto,
            &changing(
                "explain",
                "Explain",
                "Explain it in one sentence: {{selection}}",
            ),
        );

        // Still the first Action in the Palette, and still called what it was:
        // an Override changes an Action, it does not add one.
        assert_eq!(catalogued(&demysto), ["explain", "translate", "summarize"]);
        assert_eq!(
            defined(&demysto, "explain").standing,
            ActionStanding::Overridden
        );

        run(&demysto);

        endpoint.assert();
    }

    #[test]
    fn an_override_states_only_what_the_user_changed() {
        // So that a built-in whose wording a later version improves still
        // improves for somebody who only ever bound a Model to it (ADR-0005).
        let demysto = unconfigured("a paragraph");
        let built_in = defined(&demysto, "summarize");

        authored(
            &demysto,
            &ActionEdit {
                hotkey: Some("Ctrl+Alt+S".to_owned()),
                ..changing("summarize", &built_in.name, &built_in.template)
            },
        );

        let written =
            std::fs::read_to_string(defined(&demysto, "summarize").path.unwrap()).unwrap();

        assert!(written.contains("hotkey = \"Ctrl+Alt+S\""), "{written}");
        assert!(!written.contains("name ="), "{written}");
        assert!(!written.contains("template ="), "{written}");
    }

    #[test]
    fn an_override_that_states_nothing_is_no_override_at_all() {
        let demysto = unconfigured("a paragraph");
        let built_in = defined(&demysto, "summarize");

        // Whitespace around what was typed is not a change: a textarea that
        // gained a trailing newline on its way through a window must not leave
        // an Override behind that says nothing.
        authored(
            &demysto,
            &changing(
                "summarize",
                &format!("  {}  ", built_in.name),
                &format!("{}\n", built_in.template),
            ),
        );

        assert_eq!(defined(&demysto, "summarize"), built_in);
        assert!(
            !actions_dir(&demysto).join("summarize.toml").exists(),
            "an Override of nothing should leave no file"
        );
    }

    #[test]
    fn an_override_can_bind_a_model_the_built_in_did_not() {
        let mut server = Server::new();
        let asked = server
            .mock("POST", "/v1/chat/completions")
            .match_body(Matcher::PartialJson(json!({ "model": "the-careful-one" })))
            .with_body(answering("an answer"))
            .create();

        let demysto = ready_with(
            &format!(
                "default_model = \"a provider/a-model\"\n\n\
                 [[providers]]\nname = \"a provider\"\nbase_url = \"{}/v1\"\n\
                 api_key = \"a-key\"\n\
                 models = [{{ id = \"a-model\" }}, {{ id = \"the-careful-one\" }}]\n",
                server.url()
            ),
            "a paragraph",
        );

        authored(
            &demysto,
            &ActionEdit {
                model: Some("a provider/the-careful-one".to_owned()),
                ..changing("explain", "Explain", "Explain: {{selection}}")
            },
        );

        run(&demysto);

        asked.assert();
    }

    #[test]
    fn removing_an_override_restores_the_built_in() {
        // User story 27: experimenting with the prompt is not a one-way door.
        let mut server = Server::new();
        let endpoint = asked_for(
            &mut server,
            vec![Matcher::Regex("Explain the text below".to_owned())],
        );

        let demysto = ready_to_run(&server, "a paragraph");
        let built_in = defined(&demysto, "explain");

        authored(
            &demysto,
            &changing("explain", "Explain briefly", "One sentence: {{selection}}"),
        );
        demysto
            .delete_action("explain")
            .expect("the Override should have been removed");

        assert_eq!(defined(&demysto, "explain"), built_in);

        run(&demysto);

        endpoint.assert();
    }

    #[test]
    fn an_action_named_after_a_built_in_is_a_second_action_and_not_an_override() {
        let demysto = unconfigured("a paragraph");

        authored(
            &demysto,
            &writing("Explain", "Explain it my way: {{selection}}"),
        );

        // Both are listed, under the one name the user chose for both, and each
        // is reachable: the identifier is what an Override is keyed on, and
        // creating an Action never takes one that is spoken for.
        assert_eq!(
            offered(&demysto),
            ["Explain", "Translate", "Summarize", "Explain"]
        );
        assert_eq!(
            catalogued(&demysto),
            ["explain", "translate", "summarize", "explain-2"]
        );
        assert_eq!(
            defined(&demysto, "explain").standing,
            ActionStanding::BuiltIn
        );
        assert_eq!(
            defined(&demysto, "explain-2").template,
            "Explain it my way: {{selection}}"
        );
    }

    #[test]
    fn every_built_in_action_reaches_a_user_who_already_has_a_configuration_directory() {
        // ADR-0005's whole point: built-ins are compiled in, so the set a build
        // ships is the set every user gets, however long they have had that
        // directory and whatever is in it.
        let demysto = unconfigured("a paragraph");

        authored(
            &demysto,
            &writing("Rewrite plainly", "Rewrite: {{selection}}"),
        );
        authored(
            &demysto,
            &changing("explain", "Explain", "One sentence: {{selection}}"),
        );

        let held = catalogued(&demysto);

        for built_in in action::built_in(&i18n::english()) {
            assert!(held.contains(&built_in.id), "{} is missing", built_in.id);
        }
    }

    #[test]
    fn renaming_an_action_leaves_it_in_the_file_it_was_already_in() {
        // The identifier is the identity: an Override is keyed on it, and so —
        // once ticket 10 lands — is a Hotkey. Renaming is not re-filing.
        let demysto = unconfigured("a paragraph");

        authored(
            &demysto,
            &writing("Rewrite plainly", "Rewrite: {{selection}}"),
        );
        authored(
            &demysto,
            &changing(
                "rewrite-plainly",
                "Put it plainly",
                "Rewrite: {{selection}}",
            ),
        );

        assert_eq!(defined(&demysto, "rewrite-plainly").name, "Put it plainly");
        assert_eq!(
            catalogued(&demysto),
            ["explain", "translate", "summarize", "rewrite-plainly"]
        );
    }

    #[test]
    fn what_the_window_writes_is_what_it_reads_back() {
        let demysto = unconfigured("a paragraph");

        let written = authored(
            &demysto,
            &ActionEdit {
                hotkey: Some("Ctrl+Alt+R".to_owned()),
                parameters: vec![Parameter {
                    id: "reader".to_owned(),
                    label: "For whom?".to_owned(),
                    default: "a child".to_owned(),
                }],
                ..writing("Rewrite for", "Rewrite this for {{reader}}: {{selection}}")
            },
        );

        assert_eq!(
            written.actions.last().cloned(),
            Some(defined(&demysto, "rewrite-for"))
        );
        assert_eq!(
            defined(&demysto, "rewrite-for"),
            DefinedAction {
                id: "rewrite-for".to_owned(),
                name: "Rewrite for".to_owned(),
                template: "Rewrite this for {{reader}}: {{selection}}".to_owned(),
                parameters: vec![Parameter {
                    id: "reader".to_owned(),
                    label: "For whom?".to_owned(),
                    default: "a child".to_owned(),
                }],
                model: None,
                // Written as it was stated: what a combination means, and
                // whether anything else has it, is the desktop's answer to give
                // and the shell's to ask for.
                hotkey: Some("Ctrl+Alt+R".to_owned()),
                accepts: vec![Kind::Text],
                standing: ActionStanding::Authored,
                path: Some(actions_dir(&demysto).join("rewrite-for.toml")),
            }
        );
    }

    #[test]
    fn an_action_file_nobody_can_read_is_reported_without_taking_the_others_down() {
        let demysto = unconfigured("a paragraph");

        authored(
            &demysto,
            &writing("Rewrite plainly", "Rewrite: {{selection}}"),
        );
        dropped_in(
            &demysto,
            "half-a-thought",
            "name = \"Half a thought\"\ntemp",
        );
        dropped_in(
            &demysto,
            "nameless",
            "template = \"Do something: {{selection}}\"\n",
        );

        let catalogue = demysto.catalogue();

        assert_eq!(
            catalogue
                .actions
                .into_iter()
                .map(|action| action.id)
                .collect::<Vec<_>>(),
            ["explain", "translate", "summarize", "rewrite-plainly"]
        );
        assert_eq!(catalogue.unreadable.len(), 2);
        assert!(
            catalogue
                .unreadable
                .iter()
                .any(|said| said.contains("half-a-thought")),
            "{:?}",
            catalogue.unreadable
        );
        assert!(
            catalogue
                .unreadable
                .iter()
                .any(|said| said.contains("nameless") && said.contains("name")),
            "{:?}",
            catalogue.unreadable
        );
    }

    #[test]
    fn an_action_file_from_a_newer_demysto_is_left_alone_rather_than_guessed_at() {
        let demysto = unconfigured("a paragraph");

        dropped_in(
            &demysto,
            "explain",
            "version = 9\nname = \"Explain\"\ntemplate = \"{{selection}}\"\n",
        );

        assert_eq!(
            defined(&demysto, "explain").standing,
            ActionStanding::BuiltIn
        );
        assert!(
            demysto.catalogue().unreadable[0].contains("version 9"),
            "{:?}",
            demysto.catalogue().unreadable
        );
    }

    #[test]
    fn an_action_with_nothing_to_say_is_refused() {
        let demysto = unconfigured("a paragraph");

        assert!(refused(&demysto, &writing("  ", "Rewrite: {{selection}}")).contains("name"));
        assert!(refused(&demysto, &writing("Rewrite plainly", "  ")).contains("prompt"));
    }

    #[test]
    fn a_parameter_that_could_never_be_collected_is_refused() {
        let demysto = unconfigured("a paragraph");

        let declaring = |id: &str, label: &str| ActionEdit {
            parameters: vec![
                Parameter {
                    id: id.to_owned(),
                    label: label.to_owned(),
                    default: String::new(),
                },
                Parameter {
                    id: "reader".to_owned(),
                    label: "For whom?".to_owned(),
                    default: String::new(),
                },
            ],
            ..writing("Rewrite for", "Rewrite: {{selection}}")
        };

        // A Parameter named after something Demysto fills in would never be
        // asked for: the template's own variables are answered first.
        assert!(refused(&demysto, &declaring("selection", "Which?")).contains("selection"));
        assert!(refused(&demysto, &declaring("reader", "Which?")).contains("Two Parameters"));
        assert!(refused(&demysto, &declaring("tone", "  ")).contains("label"));
        assert!(refused(&demysto, &declaring(" ", "Which?")).contains("Parameter"));
    }

    #[test]
    fn an_action_bound_to_a_model_no_provider_offers_is_refused() {
        // The window had the whole list of Models on screen as this was
        // written, so a binding that resolves to nothing is caught here rather
        // than met at the next Run — as `settings::nominating` catches the same
        // mistake in the two defaults.
        let demysto = ready_with(&one_provider("http://127.0.0.1:1/v1"), "a paragraph");

        let message = refused(
            &demysto,
            &ActionEdit {
                model: Some("a provider/a-model-nobody-has".to_owned()),
                ..writing("Rewrite plainly", "Rewrite: {{selection}}")
            },
        );

        assert!(
            message.contains("a provider/a-model-nobody-has"),
            "{message}"
        );
        assert!(message.contains("a provider/a-model"), "{message}");
        assert!(
            !actions_dir(&demysto).exists(),
            "nothing should have been written"
        );
    }

    #[test]
    fn an_identifier_that_could_not_be_a_file_is_refused() {
        let demysto = unconfigured("a paragraph");

        for id in ["../elsewhere", "with/a/path", ".hidden", "aux"] {
            let message = refused(
                &demysto,
                &changing(id, "Rewrite plainly", "Rewrite: {{selection}}"),
            );

            assert!(message.contains(id), "{message}");
        }
    }

    #[test]
    fn an_action_named_in_an_alphabet_of_the_users_own_is_filed_under_that_name() {
        // A user writing their Actions in Russian should not find them all
        // called `action-2`.
        let demysto = unconfigured("a paragraph");

        authored(
            &demysto,
            &writing("Объяснить проще", "Проще: {{selection}}"),
        );

        assert_eq!(
            catalogued(&demysto),
            ["explain", "translate", "summarize", "объяснить-проще"]
        );
    }

    #[test]
    fn an_action_deleted_while_a_palette_is_still_listing_it_sends_nothing_anywhere() {
        // Reachable now that an Action can be deleted: the Palette holds the
        // list it opened with, and Enter on one that has since gone is not a
        // request to send anything.
        let desktop = Arc::new(FakeDesktop::new(None, Some("a paragraph")));
        let demysto = demysto_asking(fake::over(&desktop), "http://127.0.0.1:1/v1");
        demysto.capture();

        authored(
            &demysto,
            &writing("Rewrite plainly", "Rewrite: {{selection}}"),
        );
        demysto.delete_action("rewrite-plainly").unwrap();

        let RunOutcome::Failed(RunError::NoSuchAction { message }) =
            running(&demysto, "rewrite-plainly", &[])
        else {
            panic!("an Action that has been deleted should be reported as one Demysto lacks");
        };

        assert!(message.contains("rewrite-plainly"), "{message}");
    }

    #[test]
    fn settings_that_nominate_no_default_model_can_be_saved() {
        // The window sends "None" as an empty string, and a fresh installation
        // is exactly that: a Provider configured and no default nominated yet.
        // Refusing it would leave somebody unable to save at all.
        let demysto = unconfigured("a paragraph");

        let written = saved(
            &demysto,
            &Edit {
                providers: vec![ProviderEdit {
                    base_url: Some("https://api.example.com/v1".to_owned()),
                    models: vec![offering("a-model", false)],
                    ..drafted("a provider")
                }],
                default_model: Some(String::new()),
                default_vision_model: Some("   ".to_owned()),
                palette_hotkey: Some("   ".to_owned()),
                large_selection: None,
                language: Some(String::new()),
            },
        );

        assert_eq!(written.default_model, None);
        assert_eq!(written.default_vision_model, None);

        // Not stated at all rather than stated as nothing: `default_model = ""`
        // is a file that names a Model called "", which is what the save was
        // about to be refused for.
        assert!(
            !settings_file(&demysto)
                .lines()
                .any(|line| line.starts_with("default_")),
            "{}",
            settings_file(&demysto)
        );
    }

    #[test]
    fn a_provider_the_window_sends_a_blank_variable_for_is_one_that_named_none() {
        // Every field the window has no value for arrives as an empty string.
        // Read as a named variable, `api_key_env = ""` is the user saying this
        // Provider is authenticated — which for a keyless preset turns a Model
        // list into a demand for a key the service does not have.
        let mut server = Server::new();
        let _endpoint = server
            .mock("GET", "/v1/models")
            .match_header("authorization", Matcher::Missing)
            .with_body(json!({ "data": [{ "id": "a-model" }] }).to_string())
            .create();

        let demysto = ready_with(
            &keyless_provider(&format!("{}/v1", server.url())),
            "a paragraph",
        );

        let local = ProviderEdit {
            was: Some("local".to_owned()),
            preset: Some("lmstudio".to_owned()),
            base_url: Some(format!("{}/v1", server.url())),
            api_key_env: Some(String::new()),
            ..drafted("local")
        };

        assert_eq!(demysto.models_offered_by(&local).unwrap(), ["a-model"]);
    }

    #[test]
    fn a_model_with_no_name_is_refused_rather_than_written() {
        // "Add a Model" adds an empty row, so this is what saving without
        // typing into it produces — and it would otherwise reach a Provider as
        // a request naming no Model at all.
        let demysto = unconfigured("a paragraph");

        let Err(ConfigError::Malformed(message)) = demysto.save_settings(&edited(
            vec![ProviderEdit {
                base_url: Some("https://api.example.com/v1".to_owned()),
                models: vec![offering("a-model", false), offering("  ", false)],
                ..drafted("a provider")
            }],
            None,
        )) else {
            panic!("a Model with no name should be refused");
        };

        assert!(message.contains("a provider"), "{message}");
        assert!(!settings_file(&demysto).contains("a-model"));
    }

    #[test]
    fn an_action_can_bind_a_model_the_file_gained_since_demysto_started() {
        // The window builds the list it offers out of the file, which it reads
        // afresh; the check that an Action's binding names something real has
        // to read the same file, or the window offers what the save refuses.
        let demysto = ready_with(&one_provider("http://127.0.0.1:1/v1"), "a paragraph");

        std::fs::write(
            demysto.config_dir().join(config::FILE_NAME),
            settings_file(&demysto).replace(
                "models = [{ id = \"a-model\" }]",
                "models = [{ id = \"a-model\" }, { id = \"added-by-hand\" }]",
            ),
        )
        .unwrap();

        authored(
            &demysto,
            &ActionEdit {
                model: Some("a provider/added-by-hand".to_owned()),
                ..writing("Rewrite plainly", "Rewrite: {{selection}}")
            },
        );

        assert_eq!(
            defined(&demysto, "rewrite-plainly").model.as_deref(),
            Some("a provider/added-by-hand")
        );
    }

    #[test]
    fn conversations_do_not_outlive_the_session_that_held_them() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body(answering("an answer"))
            .create();

        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join(config::FILE_NAME),
            format!(
                "version = 1\n\n{}",
                one_provider(&format!("{}/v1", server.url()))
            ),
        )
        .unwrap();

        let desktop = Arc::new(FakeDesktop::new(None, Some("a paragraph")));
        let session = as_a_session(dir.path(), fake::over(&desktop));
        session.capture();
        run(&session);

        assert_eq!(session.conversations().len(), 1);

        // The same configuration directory, and nothing of the Conversation in
        // it: history is held in memory and nowhere else (user story 62).
        let next = as_a_session(dir.path(), fake::over(&desktop));

        assert!(next.conversations().is_empty());
    }

    // Errors, interruption and logs.

    /// What a raw server answers one request with.
    enum Says {
        /// These bytes, and then the connection closed. Written without a
        /// `Content-Length` and without chunking, so the body ends where the
        /// connection does — which is how a stream that was cut short reaches
        /// the reader as one.
        This(String),
        /// Nothing at all, holding the connection open past anything a test
        /// waits for.
        Nothing,
        /// These bytes, and then silence: a stream that started and stopped
        /// without ever closing, which is what a Provider that has gone away
        /// mid-answer looks like from here.
        ThenNothing(String),
    }

    /// How long a raw server holds a connection it is meant never to finish.
    ///
    /// It only has to outlast the client's own timeout, which the suite sets to
    /// 200 ms, and a wide margin costs nothing: the client gives up first on
    /// every path this is used on, and the thread holding the socket is
    /// detached, so nothing waits for it.
    ///
    /// Two seconds was enough in practice and is not what made this suite
    /// flake — that was `provider::broke_off`, reading the shape of the error
    /// rather than the clock. The margin is wide here so that a hold expiring
    /// first is never among the explanations to consider again.
    const HELD_OPEN: Duration = Duration::from_secs(30);

    /// A server that answers each connection with exactly what it was told to,
    /// and remembers what was asked of it.
    ///
    /// Here rather than through the mock server because the two failures this
    /// is for are ones a well-behaved HTTP library will not produce on request:
    /// a stream that stops in the middle of an answer, and an endpoint that
    /// accepts a connection and then says nothing.
    struct Raw {
        base_url: String,
        asked: Arc<Mutex<Vec<String>>>,
    }

    fn raw(script: Vec<Says>) -> Raw {
        use std::io::{Read, Write};
        use std::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base_url = format!("http://{}/v1", listener.local_addr().unwrap());
        let asked = Arc::new(Mutex::new(Vec::new()));
        let recording = Arc::clone(&asked);

        std::thread::spawn(move || {
            for (connection, says) in listener.incoming().zip(script) {
                let Ok(mut connection) = connection else {
                    return;
                };

                // Read what came in, so that the client's own write finishes,
                // and keep the body: what a continuation sends is exactly what
                // this is here to assert.
                let mut request = [0; 16 * 1024];
                let read = connection.read(&mut request).unwrap_or(0);
                recording
                    .lock()
                    .unwrap()
                    .push(String::from_utf8_lossy(&request[..read]).into_owned());

                match says {
                    Says::This(body) => {
                        let _ = connection.write_all(
                            format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\n\r\n{body}"
                            )
                            .as_bytes(),
                        );
                    }
                    // Held rather than closed: a connection that closes is an
                    // answer of a kind, and this is the endpoint that gives none.
                    Says::Nothing => std::thread::sleep(HELD_OPEN),
                    Says::ThenNothing(body) => {
                        let _ = connection.write_all(
                            format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\n\r\n{body}"
                            )
                            .as_bytes(),
                        );
                        let _ = connection.flush();
                        std::thread::sleep(HELD_OPEN);
                    }
                }
            }
        });

        Raw { base_url, asked }
    }

    impl Raw {
        /// What each request carried, in the order they arrived.
        fn asked(&self) -> Vec<String> {
            self.asked.lock().unwrap().clone()
        }
    }

    /// Two fragments of an answer, and then the connection closed without the
    /// contract's full stop: a stream that broke mid-sentence.
    fn cut_off(fragments: &[&str]) -> Says {
        Says::This(
            fragments
                .iter()
                .map(|fragment| json!({ "choices": [{ "delta": { "content": fragment } }] }))
                .map(|event| format!("data: {event}\n\n"))
                .collect(),
        )
    }

    #[test]
    fn a_stream_that_breaks_part_way_keeps_what_it_delivered() {
        let server = raw(vec![cut_off(&["The first half", " of an answer"])]);
        let demysto = ready_with(&one_provider(&server.base_url), "a paragraph");

        let RunOutcome::Interrupted { text, error } = run(&demysto) else {
            panic!("a stream that broke should keep what arrived");
        };

        assert_eq!(text, "The first half of an answer");
        assert!(
            error.message().contains("before the answer was finished"),
            "{error}"
        );
    }

    #[test]
    fn what_a_broken_stream_delivered_is_on_screen_while_it_is_still_arriving() {
        let server = raw(vec![cut_off(&["The first half", " of an answer"])]);
        let demysto = ready_with(&one_provider(&server.base_url), "a paragraph");

        let (shown, _) = watching(&demysto);

        assert_eq!(
            shown.last().map(String::as_str),
            Some("The first half of an answer")
        );
    }

    #[test]
    fn a_stream_that_ends_tidily_and_says_nothing_is_still_no_answer() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body(format!(
                "data: {}\n\ndata: [DONE]\n\n",
                json!({ "choices": [{ "delta": { "role": "assistant" } }] })
            ))
            .create();

        // The contract's full stop is not the same as an answer: a stream of
        // the event that opens one and nothing else has told the user nothing.
        assert!(matches!(
            run(&ready_to_run(&server, "a paragraph")),
            RunOutcome::Failed(RunError::Malformed { .. })
        ));
    }

    #[test]
    fn a_stream_that_breaks_before_the_first_word_is_a_failure_with_nothing_to_keep() {
        let server = raw(vec![Says::This(String::new())]);
        let demysto = ready_with(&one_provider(&server.base_url), "a paragraph");

        // Nothing to keep and nothing to carry on from, so it is the failure
        // with a retry on it rather than the answer with the rest to come.
        assert!(
            matches!(
                run(&demysto),
                RunOutcome::Failed(RunError::Malformed { .. })
            ),
            "an empty stream should be reported as one"
        );
    }

    #[test]
    fn a_broken_answer_can_be_carried_on_and_ends_up_one_answer() {
        let server = raw(vec![
            cut_off(&["The first half"]),
            Says::This(answering(" and the rest of it.")),
        ]);
        let demysto = ready_with(&one_provider(&server.base_url), "a paragraph");

        run(&demysto);
        let outcome = demysto.continue_answer(|_| {});

        assert_eq!(
            outcome,
            RunOutcome::Answered("The first half and the rest of it.".to_owned())
        );

        // One Turn, not two: the user asked one question and is reading one
        // answer to it.
        assert_eq!(turns(&demysto).len(), 1);
    }

    #[test]
    fn carrying_on_puts_what_already_arrived_back_to_the_model() {
        let server = raw(vec![
            cut_off(&["The first half"]),
            Says::This(answering(" and the rest.")),
        ]);
        let demysto = ready_with(&one_provider(&server.base_url), "a paragraph");

        run(&demysto);
        demysto.continue_answer(|_| {});

        let carrying_on = server.asked().pop().expect("a second request went out");

        assert!(carrying_on.contains("The first half"), "{carrying_on}");
        assert!(
            carrying_on.contains("Carry on from exactly where"),
            "{carrying_on}"
        );
    }

    #[test]
    fn there_is_nothing_to_carry_on_from_an_answer_that_finished() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body(answering("a whole answer"))
            .create();

        let demysto = ready_to_run(&server, "a paragraph");
        run(&demysto);

        assert!(matches!(
            demysto.continue_answer(|_| {}),
            RunOutcome::Failed(RunError::NothingToRun { .. })
        ));
    }

    #[test]
    fn a_model_that_says_it_has_finished_has_finished_however_the_socket_ends() {
        // Several OpenAI-compatible servers close the connection instead of
        // sending the sentinel. Without the Model's own full stop being read as
        // one, every answer from those would be reported as having broken off,
        // with an offer to carry on from a reply that was already complete.
        let server = raw(vec![Says::This(format!(
            "data: {}\n\ndata: {}\n\n",
            json!({ "choices": [{ "delta": { "content": "A whole answer." } }] }),
            json!({ "choices": [{ "delta": {}, "finish_reason": "stop" }] })
        ))]);
        let demysto = ready_with(&one_provider(&server.base_url), "a paragraph");

        assert_eq!(
            run(&demysto),
            RunOutcome::Answered("A whole answer.".to_owned())
        );
    }

    #[test]
    fn a_continuation_that_never_reaches_a_provider_keeps_what_already_arrived() {
        let server = raw(vec![cut_off(&["The first half"])]);
        let demysto = ready_with(&one_provider(&server.base_url), "a paragraph");

        run(&demysto);

        // The settings change under the broken answer, the way they do when
        // somebody opens Settings to fix the Provider that broke it and takes
        // the wrong Model out. The continuation now fails before anything is
        // sent.
        std::fs::write(
            demysto.config_dir().join(config::FILE_NAME),
            format!(
                "version = 1\n\ndefault_model = \"a provider/absent\"\n\n{}",
                provider("a provider", &server.base_url, "a-key", "a-model")
            ),
        )
        .unwrap();
        demysto.settings_changed();

        let RunOutcome::Interrupted { text, error } = demysto.continue_answer(|_| {}) else {
            panic!("a continuation that never left the machine should keep what arrived");
        };

        assert_eq!(text, "The first half");
        assert!(matches!(error, RunError::Configuration { .. }), "{error:?}");

        // And the Conversation keeps it too, so that the offer to carry on is
        // still there once the settings are right.
        assert!(matches!(
            showing(&demysto).turns[0].outcome,
            Some(RunOutcome::Interrupted { .. })
        ));
    }

    #[test]
    fn nothing_a_malformed_answer_quoted_back_reaches_the_log() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body("data: {\"choices\": [{\"delta\": {\"content\": \"a secret\"}}] oops\n\n")
            .create();

        let demysto = ready_to_run(&server, "a paragraph");
        let outcome = run(&demysto);

        // The user is shown what arrived, because seeing it is how they place
        // the fault; the log is not, because what arrived is the Model's own
        // words and a log outlives the session (ADR-0010).
        assert!(
            outcome
                .error()
                .is_some_and(|error| error.message().contains("a secret")),
            "{outcome:?}"
        );
        assert!(
            !logged(&demysto).contains("a secret"),
            "{}",
            logged(&demysto)
        );
        assert!(
            logged(&demysto).contains("failed: "),
            "{}",
            logged(&demysto)
        );
    }

    #[test]
    fn a_provider_that_answers_nothing_at_all_is_a_timeout_and_not_a_dead_address() {
        let server = raw(vec![Says::Nothing]);
        let demysto = ready_with(&one_provider(&server.base_url), "a paragraph").impatient();

        let RunOutcome::Failed(RunError::TimedOut { message }) = run(&demysto) else {
            panic!("a Provider that never answers should time out");
        };

        assert!(message.contains("stopped waiting"), "{message}");
    }

    #[test]
    fn a_provider_that_goes_quiet_mid_answer_times_out_and_keeps_what_it_said() {
        let Says::This(started) = cut_off(&["The first half"]) else {
            unreachable!("cut_off answers with bytes to send")
        };

        let server = raw(vec![Says::ThenNothing(started)]);
        let demysto = ready_with(&one_provider(&server.base_url), "a paragraph").impatient();

        let RunOutcome::Interrupted { text, error } = run(&demysto) else {
            panic!("a stream that went quiet should keep what arrived");
        };

        assert_eq!(text, "The first half");
        assert!(
            matches!(error, RunError::TimedOut { .. }),
            "a Provider that stopped speaking is a timeout, not a dead address: {error:?}"
        );
    }

    #[test]
    fn a_failed_turn_is_asked_again_rather_than_asked_beside() {
        let mut server = Server::new();
        let refusing = server
            .mock("POST", "/v1/chat/completions")
            .with_status(500)
            .with_body("<html>Bad Gateway</html>")
            .expect(1)
            .create();

        let demysto = ready_to_run(&server, "a paragraph");
        assert!(matches!(run(&demysto), RunOutcome::Failed(_)));
        refusing.assert();

        let _answering = server
            .mock("POST", "/v1/chat/completions")
            .with_body(answering("an answer at last"))
            .create();

        assert_eq!(
            demysto.retry(None, |_| {}),
            RunOutcome::Answered("an answer at last".to_owned())
        );

        // The retry is the Turn, not a Turn beside it: a Conversation that
        // accumulated an entry per attempt would send every failure as context
        // for the next one.
        assert_eq!(turns(&demysto), [(None, answered("an answer at last"))]);
    }

    #[test]
    fn a_turn_that_never_reached_a_provider_can_still_be_asked_again() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body(answering("an answer"))
            .create();

        // A settings file naming a Default Model no Provider offers: the Run
        // fails before anything is sent, which is the failure most worth being
        // able to try again after fixing it.
        let demysto = ready_with(
            &format!(
                "default_model = \"a provider/absent\"\n\n{}",
                provider(
                    "a provider",
                    &format!("{}/v1", server.url()),
                    "a-key",
                    "a-model"
                )
            ),
            "a paragraph",
        );

        assert!(matches!(
            run(&demysto),
            RunOutcome::Failed(RunError::Configuration { .. })
        ));

        // What the Palette collected is not asked for again: the Turn kept what
        // it was going to send.
        let outcome = demysto.retry(Some("a provider/a-model"), |_| {});

        assert_eq!(outcome, RunOutcome::Answered("an answer".to_owned()));
    }

    #[test]
    fn a_turn_asked_again_can_be_asked_of_another_model() {
        let mut server = Server::new();
        let _refusing = server
            .mock("POST", "/v1/chat/completions")
            .match_body(Matcher::PartialJson(json!({ "model": "cheap" })))
            .with_status(500)
            .with_body("no")
            .create();

        let capable = server
            .mock("POST", "/v1/chat/completions")
            .match_body(Matcher::PartialJson(json!({ "model": "capable" })))
            .with_body(answering("a better answer"))
            .expect(1)
            .create();

        let demysto = ready_with(
            &format!(
                "default_model = \"a provider/cheap\"\n\n[[providers]]\nname = \"a provider\"\n\
                 base_url = \"{}/v1\"\napi_key = \"a-key\"\n\
                 models = [{{ id = \"cheap\" }}, {{ id = \"capable\" }}]\n",
                server.url()
            ),
            "a paragraph",
        );

        assert!(matches!(run(&demysto), RunOutcome::Failed(_)));

        assert_eq!(
            demysto.retry(Some("a provider/capable"), |_| {}),
            RunOutcome::Answered("a better answer".to_owned())
        );
        capable.assert();
    }

    #[test]
    fn a_model_switched_to_is_where_the_rest_of_the_conversation_goes() {
        let mut server = Server::new();
        let _refusing = server
            .mock("POST", "/v1/chat/completions")
            .match_body(Matcher::PartialJson(json!({ "model": "cheap" })))
            .with_status(500)
            .with_body("no")
            .create();

        let _capable = server
            .mock("POST", "/v1/chat/completions")
            .match_body(Matcher::PartialJson(json!({ "model": "capable" })))
            .with_body(answering("an answer"))
            .create();

        let demysto = ready_with(
            &format!(
                "default_model = \"a provider/cheap\"\n\n[[providers]]\nname = \"a provider\"\n\
                 base_url = \"{}/v1\"\napi_key = \"a-key\"\n\
                 models = [{{ id = \"cheap\" }}, {{ id = \"capable\" }}]\n",
                server.url()
            ),
            "a paragraph",
        );

        run(&demysto);
        demysto.retry(Some("a provider/capable"), |_| {});

        // The switch is a decision about the Conversation, so the follow-up
        // does not quietly go back to the Model that failed.
        assert_eq!(
            following_up(&demysto, "and this?"),
            RunOutcome::Answered("an answer".to_owned())
        );
    }

    #[test]
    fn there_is_nothing_to_try_again_before_anything_has_been_asked() {
        let demysto = demysto(fake::over(&Arc::new(FakeDesktop::new(None, None))));

        assert!(matches!(
            demysto.retry(None, |_| {}),
            RunOutcome::Failed(RunError::NothingToRun { .. })
        ));
    }

    #[test]
    fn the_models_a_conversation_can_be_switched_to_are_the_ones_configured() {
        let demysto = ready_with(
            "[[providers]]\nname = \"a provider\"\nbase_url = \"https://example.com/v1\"\n\
             api_key = \"a-key\"\nmodels = [{ id = \"cheap\" }, { id = \"capable\" }]\n",
            "a paragraph",
        );

        assert_eq!(demysto.models(), ["a provider/cheap", "a provider/capable"]);
    }

    #[test]
    fn a_selection_larger_than_the_settings_allow_is_warned_about_and_still_sent() {
        let mut server = Server::new();
        let selection = "word ".repeat(300);
        let endpoint = asked_for(&mut server, vec![Matcher::Regex("word word".to_owned())]);

        let demysto = ready_with(
            &format!(
                "large_selection = 100\n{}",
                one_provider(&format!("{}/v1", server.url()))
            ),
            &selection,
        );

        assert_eq!(run(&demysto), RunOutcome::Answered("an answer".to_owned()));

        // Sent whole: the warning is a warning, and nothing about it blocks or
        // truncates (user story 48).
        endpoint.assert();

        let warning = showing(&demysto)
            .warning
            .expect("a Selection over the size should be warned about");

        assert!(warning.contains("1,500 characters"), "{warning}");
        assert!(warning.contains("100"), "{warning}");
        assert!(warning.contains("large_selection"), "{warning}");
    }

    #[test]
    fn the_warning_is_on_the_conversation_before_the_run_that_fills_it() {
        let demysto = ready_with(
            &format!(
                "large_selection = 10\n{}",
                one_provider("https://example.com/v1")
            ),
            "a Selection well over ten characters",
        );

        // The interface declares the Run, shows the window, and only then runs
        // it: the warning has to be there for the window that loads in between.
        demysto.about_to_run("explain");

        assert!(showing(&demysto).warning.is_some());
    }

    #[test]
    fn a_selection_the_settings_are_happy_with_is_not_warned_about() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body(answering("an answer"))
            .create();

        let demysto = ready_to_run(&server, "a paragraph");
        run(&demysto);

        assert_eq!(showing(&demysto).warning, None);
    }

    #[test]
    fn a_size_of_zero_is_a_user_who_would_rather_not_be_told() {
        let demysto = ready_with(
            &format!(
                "large_selection = 0\n{}",
                one_provider("https://example.com/v1")
            ),
            &"word ".repeat(10_000),
        );

        demysto.about_to_run("explain");

        assert_eq!(showing(&demysto).warning, None);
    }

    #[test]
    fn a_follow_up_is_not_warned_about_the_selection_a_second_time() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body(answering("an answer"))
            .create();

        let demysto = ready_with(
            &format!(
                "large_selection = 10\n{}",
                one_provider(&format!("{}/v1", server.url()))
            ),
            "a Selection well over ten characters",
        );

        run(&demysto);
        let said_once = showing(&demysto).warning;
        following_up(&demysto, "and this?");

        // The Selection did not travel again and did not change, so neither did
        // what there is to say about it.
        assert_eq!(showing(&demysto).warning, said_once);
        assert_eq!(turns(&demysto).len(), 2);
    }

    /// Everything this session wrote to its log.
    fn logged(demysto: &Demysto) -> String {
        std::fs::read_to_string(demysto.logs_dir().join("demysto.log")).unwrap_or_default()
    }

    #[test]
    fn what_a_run_did_is_in_the_log_a_bug_report_can_carry() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body(answering("an answer"))
            .create();

        let demysto = ready_to_run(&server, "a paragraph");
        run(&demysto);

        let written = logged(&demysto);

        assert!(written.contains("Demysto 1.2.3 started"), "{written}");
        assert!(written.contains("asking a-model"), "{written}");
        assert!(written.contains("answered, 9 characters"), "{written}");
    }

    #[test]
    fn what_went_wrong_is_in_the_log_in_the_words_the_user_was_shown() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_status(401)
            .with_body(json!({ "error": { "message": "Incorrect API key provided" } }).to_string())
            .create();

        let demysto = ready_to_run(&server, "a paragraph");
        run(&demysto);

        assert!(
            logged(&demysto).contains("Incorrect API key provided"),
            "{}",
            logged(&demysto)
        );
    }

    #[test]
    fn nothing_the_user_was_looking_at_reaches_the_log() {
        let mut server = Server::new();
        let _endpoint = server
            .mock("POST", "/v1/chat/completions")
            .with_body(answering("the Higgs boson is a particle"))
            .create();

        let demysto = ready_with(
            &one_provider(&format!("{}/v1", server.url())),
            "a sentence nobody else should read",
        );
        run(&demysto);
        following_up(&demysto, "a question nobody else should read either");

        // The one promise a log written beside somebody's keys has to keep:
        // it records what happened, never what it was about (ADR-0010).
        let written = logged(&demysto);

        assert!(!written.contains("nobody else should read"), "{written}");
        assert!(!written.contains("Higgs"), "{written}");
        assert!(!written.contains("a-key"), "{written}");
    }
}
