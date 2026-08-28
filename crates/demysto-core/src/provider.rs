//! The Provider adapter: one implementation of the OpenAI Chat Completions
//! contract, parameterised by base URL and credentials.
//!
//! Every service Demysto supports is that contract at a different address, so
//! there is one adapter and no vendor branching (the spec's *Core modules*).
//! This is the only module in the crate that performs network I/O, and the only
//! one the test suite stands a server in front of rather than a fake: what is
//! asserted is the request that actually went out.

use std::io::Read;
use std::sync::OnceLock;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::config::Provider;
use crate::model::Resolved;
use crate::run::{RunError, Stopping};
use crate::sse;

/// The contract's endpoints, joined onto whatever base URL the user gave.
const ANSWERING: &str = "chat/completions";
const MODELS: &str = "models";

/// How long a Provider has to answer at all before Demysto stops waiting. Long,
/// because a slow answer is still an answer, and the user has Stop for the
/// answer that is going nowhere.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);

/// How long the endpoint has to accept a connection. Short, because a base URL
/// that is wrong is wrong immediately, and the user should hear so.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// What a Provider sends when it has no more of the answer to send.
const DONE: &str = "[DONE]";

/// How large a read off the socket may be. A fragment of an answer is a few
/// bytes; this is sized so that a fast Model's whole burst arrives in one read
/// rather than a dozen.
const READ_SIZE: usize = 8 * 1024;

/// How much of a body that turns out not to be the contract's is held on to, so
/// that the user can be shown what arrived instead. Generous in bytes because
/// [`snippet`] counts in characters.
const BODY_KEPT: usize = 4 * 300;

/// Puts a Conversation to a Provider and delivers the reply as it arrives.
///
/// `said` is the whole Conversation as the contract wants it — who said each
/// part, and what they said — ending on the Turn being asked now. The fragments
/// of the reply are handed over one at a time and not accumulated here: the
/// caller is assembling them anyway, and two copies of the same answer would be
/// two places for it to differ.
///
/// Answers as soon as `stopping` says the user has stopped waiting, which
/// leaves the caller holding what had arrived by then.
pub(crate) fn answer(
    resolved: &Resolved<'_>,
    said: &[(&str, String)],
    stopping: &Stopping,
    mut arriving: impl FnMut(&str),
) -> Result<(), RunError> {
    let provider = resolved.provider;
    let asking = authenticated(
        client()?.post(endpoint(&provider.base_url, ANSWERING)),
        resolved.api_key,
    );

    let mut response = asking
        .json(&Request {
            model: &resolved.model.id,
            messages: said
                .iter()
                .map(|(role, content)| Message { role, content })
                .collect(),
            stream: true,
        })
        .send()
        .map_err(|error| RunError::Unreachable(unreachable(provider, &error)))?;

    // A refusal is not a stream: it is one body, and one worth showing whole.
    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .map_err(|error| RunError::Unreachable(unreachable(provider, &error)))?;

        return Err(RunError::Provider(refused(status.as_u16(), &body)));
    }

    let mut events = sse::Events::new();
    let mut buffer = [0; READ_SIZE];
    let mut body_start = Vec::new();
    let mut delivered = false;

    'reading: loop {
        // Stop is looked for here as well as after each fragment, so that it is
        // seen between two reads rather than only between two fragments of one.
        // A stream that has gone silent is a different matter: this thread is
        // inside the read until the Provider says something or the request
        // times out, and Stop waits with it. Timeouts are ticket 11's.
        if stopping.stopped() {
            return Ok(());
        }

        // A stream that breaks part-way through loses the fragments already
        // delivered, because a failure is the whole of what a Run produced.
        // Ticket 11 owns keeping them and offering to continue.
        let read = response
            .read(&mut buffer)
            .map_err(|error| RunError::Unreachable(interrupted(provider, &error)))?;

        let chunk = &buffer[..read];
        keep(&mut body_start, chunk);

        // Nothing read is the socket closed, which leaves whatever is still
        // held as a last event the sender did not terminate.
        let payloads = match read {
            0 => events.finish(),
            _ => events.feed(chunk),
        };

        for payload in payloads {
            // The contract's full stop. What follows it, if anything does, is
            // not part of the answer.
            if payload.trim() == DONE {
                break 'reading;
            }

            if let Some(fragment) = fragment(&payload)? {
                delivered = true;
                arriving(&fragment);

                // Checked per fragment and not per read: a whole answer often
                // arrives in one read, and a Stop pressed over the second
                // sentence should not have to wait for the last.
                if stopping.stopped() {
                    return Ok(());
                }
            }
        }

        if read == 0 {
            break;
        }
    }

    match delivered {
        true => Ok(()),
        false => Err(RunError::Malformed(malformed(
            "it holds no answer",
            &String::from_utf8_lossy(&body_start),
        ))),
    }
}

/// Every Model identifier this Provider says it offers, in the order it lists
/// them.
///
/// What comes back is names and nothing else: whether a Model accepts images is
/// not something the contract reports, and guessing it here is exactly what the
/// `vision` flag exists to stop. The user picks from this list and says what
/// each one can do.
pub(crate) fn models(provider: &Provider, api_key: Option<&str>) -> Result<Vec<String>, RunError> {
    let response = authenticated(client()?.get(endpoint(&provider.base_url, MODELS)), api_key)
        .send()
        .map_err(|error| RunError::Unreachable(unreachable(provider, &error)))?;

    let status = response.status();
    let body = response
        .text()
        .map_err(|error| RunError::Unreachable(unreachable(provider, &error)))?;

    if !status.is_success() {
        return Err(RunError::Provider(refused(status.as_u16(), &body)));
    }

    let offered: Offered = serde_json::from_str(&body)
        .map_err(|error| RunError::Malformed(malformed(&error.to_string(), &body)))?;

    Ok(offered.data.into_iter().map(|model| model.id).collect())
}

/// The text one event carries, which is none when the Model sent a fragment
/// with nothing in it — the first of a stream, carrying only the role, is one.
fn fragment(payload: &str) -> Result<Option<String>, RunError> {
    let chunk: Chunk = serde_json::from_str(payload)
        .map_err(|error| RunError::Malformed(malformed(&error.to_string(), payload)))?;

    Ok(chunk
        .choices
        .into_iter()
        .next()
        .and_then(|choice| choice.delta.content)
        .filter(|fragment| !fragment.is_empty()))
}

/// Holds on to the start of the body, and only the start, so that a stream that
/// turns out not to be one can be quoted back to the user.
fn keep(body_start: &mut Vec<u8>, chunk: &[u8]) {
    let room = BODY_KEPT.saturating_sub(body_start.len());
    body_start.extend_from_slice(&chunk[..room.min(chunk.len())]);
}

/// The request, carrying the key where the Provider has one to carry.
///
/// A Provider with none is a local server that asks for nothing, and sending it
/// an invented key would be sending something nobody chose (ADR-0006). The
/// header is left off rather than filled with a placeholder.
fn authenticated(
    request: reqwest::blocking::RequestBuilder,
    api_key: Option<&str>,
) -> reqwest::blocking::RequestBuilder {
    match api_key {
        Some(key) => request.bearer_auth(key),
        None => request,
    }
}

/// The base URL and an endpoint, with exactly one slash between them: a base
/// URL is copied out of documentation as often with a trailing slash as without.
fn endpoint(base_url: &str, endpoint: &str) -> String {
    format!("{}/{endpoint}", base_url.trim_end_matches('/'))
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

/// A stream that started and then stopped: a different thing from an endpoint
/// that never answered, though there is nothing more to show for either.
fn interrupted(provider: &Provider, error: &std::io::Error) -> String {
    format!(
        "{} stopped answering part-way through: {error}",
        provider.base_url
    )
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
    format!(
        "The Provider's answer was not one Demysto could read ({reason}): {}",
        snippet(body)
    )
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
    /// cannot hand back one body where this is waiting for a stream.
    stream: bool,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

/// One event of a stream.
#[derive(Deserialize)]
struct Chunk {
    /// Required rather than defaulted: an event that carries no `choices` at
    /// all is not this contract, and a Provider reporting an error mid-stream
    /// sends exactly that. Better shown to the user than passed over.
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    delta: Delta,
}

#[derive(Deserialize)]
struct Delta {
    /// Absent on the event that opens a stream, which carries only the role.
    content: Option<String>,
}

/// What the Model list endpoint answers with.
#[derive(Deserialize)]
struct Offered {
    data: Vec<OfferedModel>,
}

#[derive(Deserialize)]
struct OfferedModel {
    id: String,
}

#[derive(Deserialize)]
struct Refusal {
    error: RefusalBody,
}

#[derive(Deserialize)]
struct RefusalBody {
    message: String,
}
