//! The Provider adapter: one implementation of the OpenAI Chat Completions
//! contract, parameterised by base URL and credentials.
//!
//! Every service Demysto supports is that contract at a different address, so
//! there is one adapter and no vendor branching (the spec's *Core modules*).
//! This is the only module in the crate that performs network I/O, and the only
//! one the test suite stands a server in front of rather than a fake: what is
//! asserted is the request that actually went out. Ticket 04 adds the streaming
//! half of the same contract.

use std::sync::OnceLock;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::config::Provider;
use crate::run::RunError;

/// The contract's one endpoint, joined onto whatever base URL the user gave.
const ENDPOINT: &str = "chat/completions";

/// How long a Provider has to answer at all before Demysto stops waiting. Long,
/// because a slow answer is still an answer; ticket 06 gives the user a way to
/// stop waiting sooner.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);

/// How long the endpoint has to accept a connection. Short, because a base URL
/// that is wrong is wrong immediately, and the user should hear so.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Asks a Provider one question and waits for the whole answer.
pub(crate) fn answer(provider: &Provider, prompt: &str) -> Result<String, RunError> {
    let response = client()?
        .post(endpoint(&provider.base_url))
        .bearer_auth(&provider.api_key)
        .json(&Request {
            model: &provider.model,
            messages: vec![Message {
                role: "user",
                content: prompt,
            }],
            stream: false,
        })
        .send()
        .map_err(|error| RunError::Unreachable(unreachable(provider, &error)))?;

    // Read as text before it is parsed: an error carries a body worth showing
    // the user, and so does a body that turns out not to be the contract's.
    let status = response.status();
    let body = response
        .text()
        .map_err(|error| RunError::Unreachable(unreachable(provider, &error)))?;

    if !status.is_success() {
        return Err(RunError::Provider(refused(status.as_u16(), &body)));
    }

    let answer: Answer = serde_json::from_str(&body)
        .map_err(|error| RunError::Malformed(malformed(&error.to_string(), &body)))?;

    answer
        .choices
        .into_iter()
        .next()
        .and_then(|choice| choice.message.content)
        .filter(|answer| !answer.trim().is_empty())
        .ok_or_else(|| RunError::Malformed(malformed("it holds no answer", &body)))
}

/// The base URL and the endpoint, with exactly one slash between them: a base
/// URL is copied out of documentation as often with a trailing slash as without.
fn endpoint(base_url: &str) -> String {
    format!("{}/{ENDPOINT}", base_url.trim_end_matches('/'))
}

/// The one client the process uses.
///
/// Built once and kept: it carries the connection pool and the TLS setup, and
/// building one per Run would put a handshake in front of every answer.
fn client() -> Result<&'static reqwest::blocking::Client, RunError> {
    static CLIENT: OnceLock<Result<reqwest::blocking::Client, String>> = OnceLock::new();

    CLIENT
        .get_or_init(|| {
            reqwest::blocking::Client::builder()
                .connect_timeout(CONNECT_TIMEOUT)
                .timeout(REQUEST_TIMEOUT)
                .user_agent(concat!("demysto/", env!("CARGO_PKG_VERSION")))
                .build()
                .map_err(|error| error.to_string())
        })
        .as_ref()
        .map_err(|error| {
            RunError::Unreachable(format!("Demysto could not open a connection: {error}"))
        })
}

fn unreachable(provider: &Provider, error: &reqwest::Error) -> String {
    format!("{} could not be reached: {error}", provider.base_url)
}

/// What the Provider said when it refused, in its own words where it gave any.
///
/// Its message is the one worth showing: it is the only party that knows
/// whether the key is wrong, the Model does not exist, or the account is out of
/// credit, and paraphrasing it would lose exactly that.
fn refused(status: u16, body: &str) -> String {
    let message = serde_json::from_str::<Refusal>(body)
        .map(|refusal| refusal.error.message)
        .ok()
        .filter(|message| !message.trim().is_empty())
        .unwrap_or_else(|| snippet(body));

    match message.is_empty() {
        true => format!("The Provider refused the request (HTTP {status})."),
        false => format!("The Provider refused the request (HTTP {status}): {message}"),
    }
}

fn malformed(reason: &str, body: &str) -> String {
    format!("The Provider's answer was not one Demysto could read ({reason}): {}", snippet(body))
}

/// Enough of a body to recognise it by, and no more: the whole of one is a page
/// of HTML as often as it is anything worth reading.
fn snippet(body: &str) -> String {
    const LIMIT: usize = 300;

    let body = body.trim();
    match body.char_indices().nth(LIMIT) {
        Some((end, _)) => format!("{}…", &body[..end]),
        None => body.to_owned(),
    }
}

#[derive(Serialize)]
struct Request<'a> {
    model: &'a str,
    messages: Vec<Message<'a>>,
    /// Stated rather than left out, so that a Provider defaulting the other way
    /// cannot hand back a stream this ticket has no parser for.
    stream: bool,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct Answer {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Reply,
}

#[derive(Deserialize)]
struct Reply {
    /// Absent when the Model produced no text at all, which the contract allows
    /// and a Run cannot show.
    content: Option<String>,
}

#[derive(Deserialize)]
struct Refusal {
    error: RefusalBody,
}

#[derive(Deserialize)]
struct RefusalBody {
    message: String,
}
