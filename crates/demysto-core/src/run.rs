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
/// wrong. The window is where the retry and the route into Settings hang off it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "status", content = "detail", rename_all = "snake_case")]
pub enum RunOutcome {
    Answered(String),
    /// The user stopped it, keeping whatever had already arrived — which may be
    /// nothing, when they stopped it before the Model said anything.
    Stopped(String),
    /// The answer began and then broke off: a stream cut, a Provider that
    /// stopped speaking mid-sentence, an event that was not the contract's
    /// shape after several that were.
    ///
    /// Held apart from [`Self::Failed`] because what the user is owed differs:
    /// a failure has nothing to show and is offered a retry, while this has
    /// most of an answer and is offered the rest of it (user story 46). The
    /// text is never empty — a break before the first word is a failure.
    Interrupted {
        text: String,
        error: RunError,
    },
    Failed(RunError),
}

impl RunOutcome {
    /// What a Turn that will produce no more comes to: the failure alone, or
    /// what it had already delivered with the failure under it.
    ///
    /// The one place that distinction is decided, so that every path decides it
    /// the same way. It matters most on the paths that never reach a Provider:
    /// a continuation refused because the settings changed under it would
    /// otherwise throw away the paragraphs the first attempt did deliver.
    pub(crate) fn stopped_short(text: String, error: RunError) -> Self {
        match text.is_empty() {
            true => Self::Failed(error),
            false => Self::Interrupted { text, error },
        }
    }

    /// What the Model managed to say, where it said anything at all.
    pub fn text(&self) -> Option<&str> {
        match self {
            Self::Answered(text) | Self::Stopped(text) | Self::Interrupted { text, .. } => {
                Some(text)
            }
            Self::Failed(_) => None,
        }
    }

    /// Why this Turn has no more to show, where something went wrong.
    pub fn error(&self) -> Option<&RunError> {
        match self {
            Self::Interrupted { error, .. } | Self::Failed(error) => Some(error),
            Self::Answered(_) | Self::Stopped(_) => None,
        }
    }
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

/// Why a Run produced no answer, or stopped producing one.
///
/// Every variant carries the whole sentence the user is shown. The variants are
/// there so that the interface can offer a different affordance per kind — a
/// retry, a route into Settings — not so that it can compose the wording.
///
/// Internally tagged rather than tagged-and-contented, so that the one variant
/// carrying more than a sentence can carry it as a field of its own beside the
/// sentence rather than inside it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RunError {
    /// Demysto has no Provider it can use, so nothing was sent.
    Configuration { message: String },
    /// There was nothing for a Turn to be put to: no Selection to run an Action
    /// against, or no Conversation to ask a follow-up in.
    NothingToRun { message: String },
    /// The interface asked for an Action Demysto does not have.
    NoSuchAction { message: String },
    /// The Provider could not be reached at all.
    Unreachable { message: String },
    /// The Provider was reached and then said nothing for longer than Demysto
    /// waits. Held apart from [`Self::Unreachable`] because it is the one
    /// failure where trying again with nothing changed is the reasonable thing
    /// to do.
    TimedOut { message: String },
    /// The Provider was reached and answered with an error of its own.
    Provider { message: String },
    /// The Provider refused the credentials rather than the request. Names the
    /// Provider, because the fix is in that Provider's own settings and the
    /// interface is expected to offer the way there (user story 45).
    Authentication { message: String, provider: String },
    /// The Provider answered with something that is not the contract's shape.
    ///
    /// `reason` is what was wrong with it, without the quotation of what
    /// arrived that `message` carries — see [`Self::logged`].
    Malformed { message: String, reason: String },
}

impl RunError {
    /// The sentence the user is shown.
    pub fn message(&self) -> &str {
        match self {
            Self::Configuration { message }
            | Self::NothingToRun { message }
            | Self::NoSuchAction { message }
            | Self::Unreachable { message }
            | Self::TimedOut { message }
            | Self::Provider { message }
            | Self::Authentication { message, .. }
            | Self::Malformed { message, .. } => message,
        }
    }

    /// What this failure says in a log, which is not always what it says on
    /// screen.
    ///
    /// An answer that was not the contract's shape is quoted back to the user
    /// so that they can see what arrived instead — and what arrived is, as
    /// often as not, the Model's own words. A log outlives the session that
    /// held them, so it gets the reason without the quotation (ADR-0010).
    pub(crate) fn logged(&self) -> &str {
        match self {
            Self::Malformed { reason, .. } => reason,
            other => other.message(),
        }
    }

    /// The Provider whose settings fix this, where one is to blame for it.
    pub fn provider(&self) -> Option<&str> {
        match self {
            Self::Authentication { provider, .. } => Some(provider),
            _ => None,
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
    RunError::NothingToRun {
        message: "There is nothing to run an Action on: select some text, or copy it, and press \
                  the Hotkey again."
            .to_owned(),
    }
}

/// What the user is told when a follow-up arrived with no Conversation to add
/// it to.
///
/// Not reachable from a window showing one, which is the only place a follow-up
/// can be typed; reachable the moment the Conversation it was typed into has
/// fallen off the end of the store.
pub(crate) fn no_conversation() -> RunError {
    RunError::NothingToRun {
        message: "There is no Conversation to ask this in. Press the Hotkey to start one."
            .to_owned(),
    }
}

/// What the user is told when the Action they chose is not one Demysto has.
///
/// Not reachable from a Palette showing this build's catalogue, and reachable
/// the moment an Action is deleted while a Palette listing it is still on
/// screen.
pub(crate) fn no_such_action(id: &str) -> RunError {
    RunError::NoSuchAction {
        message: format!(
            "There is no Action called \"{id}\". It may have been removed since the Palette \
             opened; press the Hotkey again."
        ),
    }
}

/// What the user is told when a retry or a continuation arrived with no Turn to
/// act on — a Conversation whose last Turn is still being answered, or one the
/// store no longer holds.
pub(crate) fn nothing_to_retry() -> RunError {
    RunError::NothingToRun {
        message: "There is no Turn to try again. Ask the question again to start a new one."
            .to_owned(),
    }
}
