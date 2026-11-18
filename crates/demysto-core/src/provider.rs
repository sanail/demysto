//! All Provider network I/O lives here so callers can work in terms of Conversations.

use std::collections::HashMap;
use std::io::Read;
use std::sync::{LazyLock, Mutex, OnceLock};
use std::time::{Duration, Instant};

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::config::SkipReasoning;
use crate::conversation::Said;
use crate::i18n::{say, Words};
use crate::model::{Endpoint, Resolved};
use crate::run::{Arriving, RunError, Stopping};
use crate::sse;

const ANSWERING: &str = "chat/completions";
const MODELS: &str = "models";

/// Pass the timeout per request so callers can choose how long to wait.
pub(crate) const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

// Capability lookup has its own short deadline so a slow catalog cannot hold up a Turn.
const MODEL_LOOKUP_TIMEOUT: Duration = Duration::from_secs(2);
const MODEL_CAPABILITY_TTL: Duration = Duration::from_secs(3600);
const MODEL_LOOKUP_RETRY: Duration = Duration::from_secs(30);

const DONE: &str = "[DONE]";

/// Size the buffer to fit a fast Model's typical burst in one read.
const READ_SIZE: usize = 8 * 1024;

/// Reserve enough bytes to keep the diagnostic snippet even when characters use multiple bytes.
const BODY_KEPT: usize = 4 * 300;

/// The caller owns partial replies so it can keep the answer if the stream is interrupted.
pub(crate) fn answer(
    resolved: &Resolved,
    said: &[(&str, Said)],
    timeout: Duration,
    stopping: &Stopping,
    words: &Words,
    mut arriving: impl FnMut(Arriving),
) -> Result<(), RunError> {
    let provider = &resolved.endpoint;

    if stopping.stopped() {
        return Ok(());
    }
    let request = asking(provider, &resolved.model, said, timeout, words)?;
    if stopping.stopped() {
        return Ok(());
    }

    // A capability lookup should not count toward the answer's timeout.
    let started = Instant::now();

    let mut response = request
        .send()
        .map_err(|error| sending(provider, timeout, &error, words))?;

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
    let mut said_reasoning = false;

    'reading: loop {
        // A blocking socket read can delay Stop until the Provider sends data or the request times out.
        if stopping.stopped() {
            return Ok(());
        }

        // Propagate a broken read so the caller can preserve the partial reply and report the failure.
        let read = response
            .read(&mut buffer)
            .map_err(|error| broke_off(provider, started.elapsed(), timeout, &error, words))?;

        let chunk = &buffer[..read];
        keep(&mut body_start, chunk);

        // The server can close the socket without terminating its final event.
        let payloads = match read {
            0 => events.finish(),
            _ => events.feed(chunk),
        };

        for payload in payloads {
            if payload.trim() == DONE {
                finished = true;
                break 'reading;
            }

            let carried = carried(&payload, words)?;

            // Some Providers close the stream without a sentinel after sending a finish reason.
            finished |= carried.finished;

            // A single event can carry both reasoning and the beginning of the answer.
            if carried.reasoning && !said_reasoning {
                said_reasoning = true;
                arriving(Arriving::Reasoning);
            }

            if let Some(fragment) = carried.text {
                delivered = true;
                arriving(Arriving::Answer(&fragment));

                // One read can contain the entire reply, so check for Stop between fragments.
                if stopping.stopped() {
                    return Ok(());
                }
            }
        }

        if read == 0 {
            break;
        }
    }

    if !delivered {
        return Err(malformed(
            &say!(words, "provider-no-answer-in-it"),
            &String::from_utf8_lossy(&body_start),
            words,
        ));
    }

    match finished {
        true => Ok(()),
        false => Err(cut_short(provider, words)),
    }
}

/// The generic Models endpoint does not specify whether a Model accepts images.
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

/// Use a real completion because a public model list cannot verify access to the chosen Model.
/// Drop the stream after its headers to limit token use.
pub(crate) fn verify(
    provider: &Endpoint,
    model: &str,
    timeout: Duration,
    words: &Words,
) -> Result<(), RunError> {
    let said = [("user", Said::Words("Hi".to_owned()))];

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

fn asking(
    provider: &Endpoint,
    model: &str,
    said: &[(&str, Said)],
    timeout: Duration,
    words: &Words,
) -> Result<reqwest::blocking::RequestBuilder, RunError> {
    let client = client(words)?;
    let skip_openrouter_reasoning = provider.skip_reasoning == SkipReasoning::OpenRouter
        && openrouter_reasoning_is_optional(client, provider, model, timeout).unwrap_or(false);

    let request = authenticated(
        client
            .post(joined(&provider.base_url, ANSWERING))
            .timeout(timeout),
        provider.api_key.as_deref(),
    );

    Ok(request.json(&Request {
        model,
        messages: said
            .iter()
            .map(|(role, said)| Message {
                role,
                content: content(said),
            })
            .collect(),
        stream: true,
        thinking: (provider.skip_reasoning == SkipReasoning::DeepSeek)
            .then_some(DeepSeekThinking { kind: "disabled" }),
        reasoning: skip_openrouter_reasoning.then_some(OpenRouterReasoning { enabled: false }),
    }))
}

fn openrouter_reasoning_is_optional(
    client: &reqwest::blocking::Client,
    provider: &Endpoint,
    model: &str,
    timeout: Duration,
) -> Option<bool> {
    static CACHE: LazyLock<Mutex<HashMap<reqwest::Url, ModelCapability>>> =
        LazyLock::new(|| Mutex::new(HashMap::new()));

    let (author, slug) = model.split_once('/')?;
    let mut url = reqwest::Url::parse(&joined(&provider.base_url, "model")).ok()?;
    url.path_segments_mut().ok()?.extend([author, slug]);

    if let Some(cached) = CACHE
        .lock()
        .unwrap()
        .get(&url)
        .filter(|cached| cached.expires > Instant::now())
    {
        return Some(cached.optional_reasoning);
    }

    let response = authenticated(client.get(url.clone()), provider.api_key.as_deref())
        .timeout(timeout.min(MODEL_LOOKUP_TIMEOUT))
        .send();
    // Denial applies to this key and must not affect a request with corrected credentials.
    if response.as_ref().is_ok_and(|response| {
        matches!(
            response.status(),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
        )
    }) {
        return None;
    }

    let optional_reasoning = response
        .and_then(reqwest::blocking::Response::error_for_status)
        .and_then(|response| response.json::<ModelDetails>())
        .ok()
        .map(|model| {
            model
                .data
                .reasoning
                .and_then(|reasoning| reasoning.mandatory)
                == Some(false)
        });

    let now = Instant::now();
    let mut cache = CACHE.lock().unwrap();
    cache.retain(|_, capability| capability.expires > now);
    let capability = match optional_reasoning {
        Some(optional_reasoning) => {
            cache.insert(
                url,
                ModelCapability {
                    expires: now + MODEL_CAPABILITY_TTL,
                    optional_reasoning,
                },
            );
            optional_reasoning
        }
        None => {
            // Back off during a catalog outage and keep any result another request just fetched.
            cache
                .entry(url)
                .or_insert(ModelCapability {
                    expires: now + MODEL_LOOKUP_RETRY,
                    optional_reasoning: false,
                })
                .optional_reasoning
        }
    };
    Some(capability)
}

struct ModelCapability {
    expires: Instant,
    optional_reasoning: bool,
}

#[derive(Deserialize)]
struct ModelDetails {
    data: ModelMetadata,
}

#[derive(Deserialize)]
struct ModelMetadata {
    reasoning: Option<ReasoningMetadata>,
}

#[derive(Deserialize)]
struct ReasoningMetadata {
    mandatory: Option<bool>,
}

struct Carried {
    text: Option<String>,
    /// The window needs only a status, so reasoning text is discarded.
    reasoning: bool,
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

/// Keep a bounded prefix for diagnostics in case the response is not a stream.
fn keep(body_start: &mut Vec<u8>, chunk: &[u8]) {
    let room = BODY_KEPT.saturating_sub(body_start.len());
    body_start.extend_from_slice(&chunk[..room.min(chunk.len())]);
}

/// A keyless Provider may reject an invented placeholder.
fn authenticated(
    request: reqwest::blocking::RequestBuilder,
    api_key: Option<&str>,
) -> reqwest::blocking::RequestBuilder {
    match api_key {
        Some(key) => request.bearer_auth(key),
        None => request,
    }
}

/// Users can paste a base URL with or without a trailing slash.
fn joined(base_url: &str, endpoint: &str) -> String {
    format!("{}/{endpoint}", base_url.trim_end_matches('/'))
}

/// Reuse the connection pool and TLS setup across Turns.
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

/// An unreachable address points to Settings, while a timeout can be retried.
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

fn broke_off(
    provider: &Endpoint,
    waited: Duration,
    timeout: Duration,
    error: &std::io::Error,
    words: &Words,
) -> RunError {
    // Reqwest can report a body timeout through an internal error type, so also check the elapsed time.
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

/// Reqwest can wrap its timeout inside an io::Error with a generic kind.
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

fn cut_short(provider: &Endpoint, words: &Words) -> RunError {
    RunError::Unreachable {
        message: say!(
            words,
            "provider-closed-early",
            "provider" = provider.base_url.clone()
        ),
    }
}

/// Distinguish authentication errors so the window can point to the Provider's settings.
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

/// Only the Provider can explain whether the Model is unavailable, the key is invalid, or credit has run out.
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

/// Show the response snippet to the user and keep the Model's output out of logs.
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

/// The upstream body can be an entire HTML error page.
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
    /// Some Providers default to a single response body.
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking: Option<DeepSeekThinking>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning: Option<OpenRouterReasoning>,
}

#[derive(Serialize)]
struct DeepSeekThinking {
    #[serde(rename = "type")]
    kind: &'static str,
}

#[derive(Serialize)]
struct OpenRouterReasoning {
    enabled: bool,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: Content<'a>,
}

/// Keep the bare string for text-only messages because some Providers only accept parts for multimodal input.
#[derive(Serialize)]
#[serde(untagged)]
enum Content<'a> {
    Words(&'a str),
    Parts(Vec<Part<'a>>),
}

/// A data URL lets the Provider read the in-memory picture without another request.
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Part<'a> {
    Text { text: &'a str },
    ImageUrl { image_url: Url },
}

#[derive(Serialize)]
struct Url {
    url: String,
}

fn content(said: &Said) -> Content<'_> {
    match said {
        Said::Words(text) => Content::Words(text),
        Said::WordsAndPicture { text, picture } => Content::Parts(vec![
            Part::Text { text },
            Part::ImageUrl {
                image_url: Url { url: picture.url() },
            },
        ]),
    }
}

#[derive(Deserialize)]
struct Chunk {
    /// Require choices so a mid-stream Provider error cannot be mistaken for an empty event.
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    delta: Delta,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct Delta {
    /// The first event may carry only the role.
    content: Option<String>,
    /// Providers disagree on the name for reasoning: DeepSeek uses reasoning_content,
    /// OpenRouter uses reasoning, and some local servers use thinking.
    #[serde(default, alias = "reasoning", alias = "thinking")]
    reasoning_content: Option<Reasoned>,
}

/// Unknown reasoning formats must not discard answer text carried in the same event.
#[derive(Deserialize)]
#[serde(untagged)]
enum Reasoned {
    Said(String),
    /// IgnoredAny skips unfamiliar reasoning without allocating a value nobody reads.
    Shaped(serde::de::IgnoredAny),
}

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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use mockito::{Matcher, Server};
    use serde_json::json;

    use super::*;
    use crate::config::{self, Environment};
    use crate::selection::Kind;

    #[test]
    fn openrouter_reasoning_is_checked_again_after_correcting_the_key() {
        let mut server = Server::new();
        let words = crate::i18n::english();
        let model = "vendor/optional-reasoning";

        for status in [401, 403] {
            let prefix = format!("/{status}/v1");
            let model_path = format!("{prefix}/model/{model}");
            let completion_path = format!("{prefix}/chat/completions");
            let denied_key = format!("Bearer rejected-{status}");
            let allowed_key = format!("Bearer accepted-{status}");
            let denied_catalog = server
                .mock("GET", model_path.as_str())
                .match_header("authorization", denied_key.as_str())
                .with_status(status)
                .create();
            let denied_completion = server
                .mock("POST", completion_path.as_str())
                .match_header("authorization", denied_key.as_str())
                .with_status(status)
                .create();
            let catalog = server
                .mock("GET", model_path.as_str())
                .match_header("authorization", allowed_key.as_str())
                .with_body(json!({ "data": { "reasoning": { "mandatory": false } } }).to_string())
                .create();
            let completion = server
                .mock("POST", completion_path.as_str())
                .match_header("authorization", allowed_key.as_str())
                .match_body(Matcher::PartialJson(
                    json!({ "reasoning": { "enabled": false } }),
                ))
                .create();

            let mut endpoint = Endpoint {
                provider: "router".to_owned(),
                base_url: format!("{}{prefix}", server.url()),
                api_key: Some(format!("rejected-{status}")),
                skip_reasoning: SkipReasoning::OpenRouter,
            };
            assert!(matches!(
                verify(&endpoint, model, REQUEST_TIMEOUT, &words),
                Err(RunError::Authentication { .. })
            ));

            endpoint.api_key = Some(format!("accepted-{status}"));
            verify(&endpoint, model, REQUEST_TIMEOUT, &words).unwrap();

            denied_catalog.assert();
            denied_completion.assert();
            catalog.assert();
            completion.assert();
        }
    }

    #[test]
    fn openrouter_disables_only_optional_reasoning_and_caches_the_lookup() {
        let mut server = Server::new();
        let words = crate::i18n::english();

        for (model, status, metadata, disables_reasoning) in [
            (
                "deepseek/deepseek-v3.2:nitro",
                200,
                json!({ "data": { "reasoning": { "mandatory": false } } }),
                true,
            ),
            (
                "vendor/required",
                200,
                json!({ "data": { "reasoning": { "mandatory": true } } }),
                false,
            ),
            ("openrouter/auto", 200, json!({ "data": {} }), false),
            ("vendor/unknown", 503, json!({}), false),
        ] {
            let catalog = server
                .mock("GET", format!("/v1/model/{model}").as_str())
                .match_header("authorization", "Bearer test-key")
                .with_status(status)
                .with_body(metadata.to_string())
                .expect(1)
                .create();

            let mut body = json!({
                "model": model,
                "messages": [{ "role": "user", "content": "Hi" }],
                "stream": true,
            });
            if disables_reasoning {
                body["reasoning"] = json!({ "enabled": false });
            }
            let completion = server
                .mock("POST", "/v1/chat/completions")
                .match_body(Matcher::Json(body))
                .with_body(concat!(
                    "data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n\n",
                    "data: [DONE]\n\n",
                ))
                .expect(2)
                .create();

            let path = Path::new("settings.toml");
            let settings = format!(
                r#"
default_model = "router/{model}"

[[providers]]
name = "router"
preset = "openrouter"
base_url = "{}/v1"
api_key = "test-key"
models = [{{ id = "{model}" }}]
"#,
                server.url()
            );
            let file = config::parse(path, &settings, &words).unwrap();
            let config = config::resolve(path, file, &Environment::holding(&[]), &words).unwrap();
            let resolved = crate::model::resolve(&config, None, Kind::Text, &words).unwrap();

            verify(&resolved.endpoint, &resolved.model, REQUEST_TIMEOUT, &words).unwrap();
            let mut reply = String::new();
            answer(
                &resolved,
                &[("user", Said::Words("Hi".to_owned()))],
                REQUEST_TIMEOUT,
                &Stopping::default(),
                &words,
                |event| {
                    if let Arriving::Answer(text) = event {
                        reply.push_str(text);
                    }
                },
            )
            .unwrap();

            assert_eq!(reply, "Hello");
            catalog.assert();
            completion.assert();
        }
    }
}
