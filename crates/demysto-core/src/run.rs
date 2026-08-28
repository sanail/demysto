//! One execution of an Action against one Selection, and what it produced.
//!
//! What the Run says to a Model belongs to the Action it runs; see `action`.

use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// What one Turn produced, failure included: the Run that opened the
/// Conversation, or a follow-up asked in it.
///
/// A failure is an entry the Conversation window shows rather than an error
/// that stops it, for the same reason a failed Capture is one: the user asked a
/// question and is owed an answer to it, even when the answer is what went
/// wrong. Ticket 11 gives it the retry and the route into Settings.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "status", content = "detail", rename_all = "snake_case")]
pub enum RunOutcome {
    Answered(String),
    /// The user stopped it, keeping whatever had already arrived — which may be
    /// nothing, when they stopped it before the Model said anything.
    Stopped(String),
    Failed(RunError),
}

/// A Turn's stop signal.
///
/// Shared between the thread waiting on the Provider and whoever asks the Turn
/// to stop, which is necessarily another thread: the first is inside the
/// request for as long as the request lasts.
#[derive(Debug, Clone, Default)]
pub(crate) struct Stopping(Arc<AtomicBool>);

impl Stopping {
    /// Asks the Turn holding this to stop at the next fragment it is handed.
    pub(crate) fn stop(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    /// Whether it has been asked to.
    pub(crate) fn stopped(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

/// Why a Run produced no answer.
///
/// Every variant carries the whole sentence the user is shown. The variants are
/// there so that ticket 11 can offer a different affordance per kind — a retry,
/// a route into Settings — not so that the interface can compose the wording.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", content = "message", rename_all = "snake_case")]
pub enum RunError {
    /// Demysto has no Provider it can use, so nothing was sent.
    Configuration(String),
    /// There was nothing for a Turn to be put to: no Selection to run an Action
    /// against, or no Conversation to ask a follow-up in.
    NothingToRun(String),
    /// The interface asked for an Action Demysto does not have.
    NoSuchAction(String),
    /// The Provider could not be reached at all.
    Unreachable(String),
    /// The Provider was reached and answered with an error of its own.
    Provider(String),
    /// The Provider answered with something that is not the contract's shape.
    Malformed(String),
}

impl RunError {
    /// The sentence the user is shown.
    pub fn message(&self) -> &str {
        match self {
            Self::Configuration(message)
            | Self::NothingToRun(message)
            | Self::NoSuchAction(message)
            | Self::Unreachable(message)
            | Self::Provider(message)
            | Self::Malformed(message) => message,
        }
    }
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for RunError {}

/// What the user is told when the Hotkey found nothing to act on.
pub(crate) fn nothing_to_run() -> RunError {
    RunError::NothingToRun(
        "There is nothing to run an Action on: select some text, or copy it, and press the \
         Hotkey again."
            .to_owned(),
    )
}

/// What the user is told when a follow-up arrived with no Conversation to add
/// it to.
///
/// Not reachable from a window showing one, which is the only place a follow-up
/// can be typed; reachable the moment the Conversation it was typed into has
/// fallen off the end of the store.
pub(crate) fn no_conversation() -> RunError {
    RunError::NothingToRun(
        "There is no Conversation to ask this in. Press the Hotkey to start one.".to_owned(),
    )
}

/// What the user is told when the Action they chose is not one Demysto has.
///
/// Not reachable from a Palette showing this build's catalogue, and reachable
/// the moment ticket 09 lets an Action be deleted while a Palette listing it is
/// still on screen.
pub(crate) fn no_such_action(id: &str) -> RunError {
    RunError::NoSuchAction(format!(
        "There is no Action called \"{id}\". It may have been removed since the Palette \
         opened; press the Hotkey again."
    ))
}
