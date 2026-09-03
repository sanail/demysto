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
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::i18n::{say, Words};
use crate::model::{Endpoint, Resolved};
use crate::run::{Arriving, RunError, Stopping};
use crate::sse;

/// The contract's endpoints, joined onto whatever base URL the user gave.
const ANSWERING: &str = "chat/completions";
const MODELS: &str = "models";

/// How long a Provider has to answer at all before Demysto stops waiting. Long,
/// because a slow answer is still an answer, and the user has Stop for the
/// answer that is going nowhere.
///
/// Carried into each request rather than built into the client, so that the
/// suite can take the waiting out of it the way it takes it out of the
/// throttle: a timeout nobody can reach is a timeout nobody has tested.
pub(crate) const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);

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
    resolved: &Resolved,
    said: &[(&str, String)],
    timeout: Duration,
    stopping: &Stopping,
    words: &Words,
    mut arriving: impl FnMut(Arriving),
) -> Result<(), RunError> {
    let provider = &resolved.endpoint;

    // Timed from the start of the request, because the elapsed time is what
    // says a read was cut short by Demysto's own timeout rather than by the
    // Provider. See `broke_off`.
    let started = Instant::now();

    let mut response = asking(provider, &resolved.model, said, timeout, words)?
        .send()
        .map_err(|error| sending(provider, timeout, &error, words))?;

    // A refusal is not a stream: it is one body, and one worth showing whole.
    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .map_err(|error| sending(provider, timeout, &error, words))?;

        return Err(refusal(provider, status.as_u16(), &body, words));
    }

    let mut events = sse::Events::new();
    let mut buffer = [0; READ_SIZE];
    let mut body_start = Vec::new();
    let mut delivered = false;
    let mut finished = false;
    // Handed over once and not per event: a Model reasons in hundreds of
    // fragments, and "the Model is reasoning" is one piece of news however many
    // of them carry it.
    let mut said_reasoning = false;

    'reading: loop {
        // Stop is looked for here as well as after each fragment, so that it is
        // seen between two reads rather than only between two fragments of one.
        // A stream that has gone silent is a different matter: this thread is
        // inside the read until the Provider says something or the request
        // times out, and Stop waits with it.
        if stopping.stopped() {
            return Ok(());
        }

        // A stream that breaks part-way through is reported as what it is, and
        // the caller keeps the fragments already handed over: they are the
        // user's answer as far as it got (user story 46).
        let read = response
            .read(&mut buffer)
            .map_err(|error| broke_off(provider, started.elapsed(), timeout, &error, words))?;

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
                finished = true;
                break 'reading;
            }

            let carried = carried(&payload, words)?;

            // The Model saying it has finished is the other way a stream ends
            // on purpose. Not every service sends the sentinel above — several
            // local servers and gateways close the connection instead — and
            // without this every one of their answers would be reported as
            // having broken off, with an offer to carry on from a reply that
            // was already complete.
            finished |= carried.finished;

            // Before the answer and never instead of it: an event may carry
            // both, and a Model that reasons first is a Model whose window has
            // been showing "asking" for as long as the reasoning has taken.
            if carried.reasoning && !said_reasoning {
                said_reasoning = true;
                arriving(Arriving::Reasoning);
            }

            if let Some(fragment) = carried.text {
                delivered = true;
                arriving(Arriving::Answer(&fragment));

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

    // A stream that said nothing is not an answer, however tidily it ended.
    if !delivered {
        return Err(malformed(
            &say!(words, "provider-no-answer-in-it"),
            &String::from_utf8_lossy(&body_start),
            words,
        ));
    }

    // The socket closed and the contract's full stop never came. A Provider
    // that had finished says so; one that stopped mid-sentence is a stream that
    // broke, whatever the network thought of it — and the difference to the
    // user is a paragraph that ends where the Model meant it to and one that
    // ends anywhere.
    match finished {
        true => Ok(()),
        false => Err(cut_short(provider, words)),
    }
}

/// Every Model identifier this Provider says it offers, in the order it lists
/// them.
///
/// What comes back is names and nothing else: whether a Model accepts images is
/// not something the contract reports, and guessing it here is exactly what the
/// `vision` flag exists to stop. The user picks from this list and says what
/// each one can do.
pub(crate) fn models(
    provider: &Endpoint,
    timeout: Duration,
    words: &Words,
) -> Result<Vec<String>, RunError> {
    let response = authenticated(
        client(words)?
            .get(joined(&provider.base_url, MODELS))
            .timeout(timeout),
        provider.api_key.as_deref(),
    )
    .send()
    .map_err(|error| sending(provider, timeout, &error, words))?;

    let status = response.status();
    let body = response
        .text()
        .map_err(|error| sending(provider, timeout, &error, words))?;

    if !status.is_success() {
        return Err(refusal(provider, status.as_u16(), &body, words));
    }

    let offered: Offered =
        serde_json::from_str(&body).map_err(|error| malformed(&error.to_string(), &body, words))?;

    Ok(offered.data.into_iter().map(|model| model.id).collect())
}

/// Whether this Provider accepts this key, asked of the Provider itself.
///
/// The request a Run makes, to the Model the user chose, with one word in it —
/// and the answer is thrown away unread. Nothing cheaper is worth making: the
/// Model list is public at some Providers, so fetching it would pass a key the
/// service would have refused, and user story 42 exists so that a wrong key is
/// learned about now rather than at the first Run. Asking the way a Run asks is
/// also what tells the user their Model identifier is one this key may use.
/// ADR-0008 records the decision and what it costs.
///
/// The stream is dropped at the headers rather than read to its end, so the
/// Model is cut off after the few tokens it takes a connection to close.
pub(crate) fn verify(
    provider: &Endpoint,
    model: &str,
    timeout: Duration,
    words: &Words,
) -> Result<(), RunError> {
    let said = [("user", "Hi".to_owned())];

    let response = asking(provider, model, &said, timeout, words)?
        .send()
        .map_err(|error| sending(provider, timeout, &error, words))?;

    let status = response.status();
    if status.is_success() {
        return Ok(());
    }

    let body = response
        .text()
        .map_err(|error| sending(provider, timeout, &error, words))?;

    Err(refusal(provider, status.as_u16(), &body, words))
}

/// The request both a Run and a verification make: the Conversation so far, put
/// to one Model at one Provider, as a stream.
fn asking(
    provider: &Endpoint,
    model: &str,
    said: &[(&str, String)],
    timeout: Duration,
    words: &Words,
) -> Result<reqwest::blocking::RequestBuilder, RunError> {
    let request = authenticated(
        client(words)?
            .post(joined(&provider.base_url, ANSWERING))
            .timeout(timeout),
        provider.api_key.as_deref(),
    );

    Ok(request.json(&Request {
        model,
        messages: said
            .iter()
            .map(|(role, content)| Message { role, content })
            .collect(),
        stream: true,
        thinking: provider
            .skip_reasoning
            .then_some(SkipReasoning { kind: "disabled" }),
    }))
}

/// What one event of a stream carries.
struct Carried {
    /// Its text, which is none when the Model sent a fragment with nothing in
    /// it — the first of a stream, carrying only the role, is one.
    text: Option<String>,
    /// Whether the Model was reasoning rather than answering here. The
    /// reasoning itself is not kept: what the window needs is that the Model is
    /// working, and holding a chain of thought nobody asked to read would be
    /// keeping the one part of the exchange the user never sees.
    reasoning: bool,
    /// Whether the Model said this is where the answer ends.
    finished: bool,
}

fn carried(payload: &str, words: &Words) -> Result<Carried, RunError> {
    let chunk: Chunk = serde_json::from_str(payload)
        .map_err(|error| malformed(&error.to_string(), payload, words))?;

    let choice = chunk.choices.into_iter().next();

    Ok(Carried {
        finished: choice
            .as_ref()
            .is_some_and(|choice| choice.finish_reason.is_some()),
        reasoning: choice.as_ref().is_some_and(|choice| {
            matches!(
                &choice.delta.reasoning_content,
                Some(Reasoned::Said(said)) if !said.is_empty()
            )
        }),
        text: choice
            .and_then(|choice| choice.delta.content)
            .filter(|fragment| !fragment.is_empty()),
    })
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
fn joined(base_url: &str, endpoint: &str) -> String {
    format!("{}/{endpoint}", base_url.trim_end_matches('/'))
}

/// The one client the process uses.
///
/// Built once and kept: it carries the connection pool and the TLS setup, and
/// building one per Run would put a handshake in front of every answer.
fn client(words: &Words) -> Result<&'static reqwest::blocking::Client, RunError> {
    static CLIENT: OnceLock<Result<reqwest::blocking::Client, String>> = OnceLock::new();

    CLIENT
        .get_or_init(|| {
            reqwest::blocking::Client::builder()
                .connect_timeout(CONNECT_TIMEOUT)
                .user_agent(concat!("demysto/", env!("CARGO_PKG_VERSION")))
                .build()
                .map_err(|error| error.to_string())
        })
        .as_ref()
        .map_err(|error| RunError::Unreachable {
            message: say!(words, "provider-no-connection", "detail" = error.clone()),
        })
}

/// What went wrong on the way out, or on the way back before there was a body:
/// an endpoint that never answered, or one that took longer than Demysto waits.
///
/// The two are held apart because only one of them is worth trying again
/// unchanged: an address nobody answers at is a setting to fix, and a Model
/// that thought for too long is a Model to ask a second time.
fn sending(
    provider: &Endpoint,
    waited: Duration,
    error: &reqwest::Error,
    words: &Words,
) -> RunError {
    match error.is_timeout() {
        true => RunError::TimedOut {
            message: say!(
                words,
                "provider-timed-out",
                "provider" = provider.base_url.clone(),
                "seconds" = waited.as_secs()
            ),
        },
        false => RunError::Unreachable {
            message: say!(
                words,
                "provider-unreachable",
                "provider" = provider.base_url.clone(),
                "detail" = error.to_string()
            ),
        },
    }
}

/// A stream that started and then stopped being read: a different thing from an
/// endpoint that never answered, because there is an answer so far to keep.
fn broke_off(
    provider: &Endpoint,
    waited: Duration,
    timeout: Duration,
    error: &std::io::Error,
    words: &Words,
) -> RunError {
    // The clock is the most reliable of the three answers, so it is asked
    // first.
    //
    // Which shape a timeout arrives in depends on where in the read it caught:
    // sometimes `ErrorKind::TimedOut`, sometimes a `reqwest::Error` down the
    // chain of causes, and sometimes an internal marker of reqwest's that has
    // no public type and can be recognised from outside only by its text. Three
    // answers to one question, and which of them turned up depended on how busy
    // the machine was: under load the same timeout arrived in the third shape
    // and was reported as a connection that broke. A clock knows no such
    // ambiguity.
    let timed_out = waited >= timeout
        || matches!(
            error.kind(),
            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
        )
        || timed_out_beneath(error);

    match timed_out {
        true => RunError::TimedOut {
            message: say!(
                words,
                "provider-went-quiet",
                "provider" = provider.base_url.clone()
            ),
        },
        false => RunError::Unreachable {
            message: say!(
                words,
                "provider-stopped-answering",
                "provider" = provider.base_url.clone(),
                "detail" = error.to_string()
            ),
        },
    }
}

/// Whether an `io` error is a timeout wearing another error's clothes.
///
/// A request that runs out of time while its body is still arriving does not
/// reach this as `ErrorKind::TimedOut`: the HTTP client wraps its own error and
/// leaves the kind generic, so the only place the fact survives is further down
/// the chain. Asked rather than assumed, because a Provider that stopped
/// speaking and one that was never there want different things said about them.
///
/// Not the whole answer, and no longer the first thing asked: the client also
/// has a timeout marker of its own with no public type, which nothing outside
/// that crate can recognise. `broke_off` looks at the clock before it looks
/// here.
fn timed_out_beneath(error: &std::io::Error) -> bool {
    use std::error::Error;

    let mut beneath: Option<&(dyn Error + 'static)> = error.source();

    while let Some(cause) = beneath {
        if cause
            .downcast_ref::<reqwest::Error>()
            .is_some_and(reqwest::Error::is_timeout)
        {
            return true;
        }

        beneath = cause.source();
    }

    false
}

/// A stream the Provider closed without the contract's full stop, having said
/// something first. Nothing went wrong at the socket, which is why this is not
/// [`broke_off`]: the connection ended cleanly in the middle of a sentence.
fn cut_short(provider: &Endpoint, words: &Words) -> RunError {
    RunError::Unreachable {
        message: say!(
            words,
            "provider-closed-early",
            "provider" = provider.base_url.clone()
        ),
    }
}

/// What a Provider refusing the request comes back as: its own words, and which
/// of the two kinds of refusal it was.
///
/// A refused key is held apart from every other refusal because it is the one
/// the user cannot fix from the Conversation: the fix is in that Provider's
/// settings, and the window is expected to offer the way there (user story 45).
/// 401 and 403 are the contract's two ways of saying it — one for a key that is
/// wrong, one for a key that is right and not allowed here.
fn refusal(provider: &Endpoint, status: u16, body: &str, words: &Words) -> RunError {
    let message = refused(status, body, words);

    match status {
        401 | 403 => RunError::Authentication {
            message,
            provider: provider.provider.clone(),
        },
        _ => RunError::Provider { message },
    }
}

/// What the Provider said when it refused, in its own words where it gave any.
///
/// Its message is the one worth showing: it is the only party that knows
/// whether the key is wrong, the Model does not exist, or the account is out of
/// credit, and paraphrasing it would lose exactly that.
fn refused(status: u16, body: &str, words: &Words) -> String {
    let message = serde_json::from_str::<Refusal>(body)
        .map(|refusal| refusal.error.message)
        .ok()
        .filter(|message| !message.trim().is_empty())
        .unwrap_or_else(|| snippet(body));

    match message.is_empty() {
        true => say!(words, "provider-refused", "status" = status),
        false => say!(
            words,
            "provider-refused-saying",
            "status" = status,
            "detail" = message
        ),
    }
}

/// What the user is shown, and what a log is given, for an answer that is not
/// the contract's shape.
///
/// Two halves rather than one because they go to different places: the user
/// gets the reason and what arrived, so they can see for themselves; the log
/// gets the reason alone, what arrived being the Model's own words (ADR-0010).
fn malformed(reason: &str, body: &str, words: &Words) -> RunError {
    RunError::Malformed {
        message: say!(
            words,
            "provider-malformed",
            "reason" = reason.to_owned(),
            "body" = snippet(body)
        ),
        reason: reason.to_owned(),
    }
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
    /// Left out entirely at a service not known to take it, which is what the
    /// `Option` is for: the field is DeepSeek's and not the contract's, and an
    /// endpoint that has never heard of it answers 400.
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking: Option<SkipReasoning>,
}

/// The one value the request's `thinking` field takes: the instruction not to
/// reason. See [`config::Reasoning`] for why Demysto sends it.
///
/// Named for the concept and not for the field, which is DeepSeek's word and
/// appears only where [`Request`] spells it.
#[derive(Serialize)]
struct SkipReasoning {
    #[serde(rename = "type")]
    kind: &'static str,
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
    /// Why the Model stopped, on the event where it did. Absent on every event
    /// before that one, and on the whole stream at a Provider that does not
    /// report it.
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct Delta {
    /// Absent on the event that opens a stream, which carries only the role.
    content: Option<String>,
    /// The Model's reasoning, which is not the answer and never joins it.
    ///
    /// Three spellings because the contract has none: `reasoning_content` is
    /// DeepSeek's and llama.cpp's, `reasoning` is OpenRouter's, and `thinking`
    /// is what several local servers send. Only the first is one Demysto has
    /// seen on the wire; the others are accepted so that a Model which reasons
    /// somewhere else is not silently taken for a Model that has gone quiet.
    #[serde(default, alias = "reasoning", alias = "thinking")]
    reasoning_content: Option<Reasoned>,
}

/// Reasoning as it arrives: a string at every service known to stream it, and
/// anything at all at one that structures it instead.
///
/// The second variant is why this is not a `String`: reasoning is not the
/// answer, so a service carrying something unexpected under one of these names
/// must not cost the user the answer that came with it. Refusing the event
/// would do exactly that.
#[derive(Deserialize)]
#[serde(untagged)]
enum Reasoned {
    Said(String),
    /// Anything else, accepted and dropped. `IgnoredAny` rather than a `Value`
    /// because nothing reads it: the variant exists so that deserialising the
    /// event succeeds, not so that the shape can be inspected later.
    Shaped(serde::de::IgnoredAny),
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
