//! One execution of an Action against one Selection, and what it produced.
//!
//! The prompt of the one Action this ticket has lives here too, as a string.
//! Ticket 05 brings the Action catalogue and the *Prompt assembly* the spec
//! describes, and the prompt moves there with them.

use std::fmt;

/// What one Run produced, failure included.
///
/// A failure is a state the result window shows rather than an error that stops
/// it, for the same reason a failed Capture is one: the user asked a question
/// and is owed an answer to it, even when the answer is what went wrong. Ticket
/// 11 gives it the retry and the route into Settings.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "status", content = "detail", rename_all = "snake_case")]
pub enum RunOutcome {
    Answered(String),
    Failed(RunError),
}

impl From<Result<String, RunError>> for RunOutcome {
    fn from(result: Result<String, RunError>) -> Self {
        match result {
            Ok(answer) => Self::Answered(answer),
            Err(error) => Self::Failed(error),
        }
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
    /// There was no Selection to run against.
    NothingToRun(String),
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

/// The prompt of the one Action this ticket has.
///
/// A fixed string, on purpose: ticket 05 brings the Action catalogue, prompt
/// templates, and the variables the spec's *Prompt assembly* lists, and there is
/// nothing to be gained from half-building that here.
const EXPLAIN: &str = "\
Explain the text below to somebody who has just run into it while reading. Say \
what it means and unpack anything in it that is not obvious. Be brief and \
concrete, and do not repeat the text back.

";

/// The prompt for explaining a Selection.
pub(crate) fn explain(selection: &str) -> String {
    format!("{EXPLAIN}{selection}")
}

/// What the user is told when the Hotkey found nothing to act on.
pub(crate) fn nothing_to_run() -> RunError {
    RunError::NothingToRun(
        "There is nothing to explain: select some text, or copy it, and press the Hotkey again."
            .to_owned(),
    )
}
