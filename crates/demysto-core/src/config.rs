//! The settings file: what Demysto is configured with, and where the key for a
//! Provider comes from.
//!
//! Read at startup, and again whenever the settings window writes it. Nothing
//! else in the crate reads the environment, which is snapshotted here — per the
//! spec's *Core modules*. Which of the Models configured here a given Run uses
//! is `model`'s; how the window edits this file without flattening it is
//! `settings`'.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::files;

/// The file Demysto reads, inside the configuration directory.
pub(crate) const FILE_NAME: &str = "settings.toml";

/// The shape of the file this build understands. A file claiming a higher one
/// was written by a newer Demysto, and guessing at what it means would be a
/// good way to send somebody's key to the wrong place.
pub(crate) const VERSION: u32 = 1;

/// What separates the Provider from the Model in the name a Model is nominated
/// or bound by.
///
/// A Provider's own name may not hold one, so the first is always the divide —
/// which matters because a Model's identifier routinely holds several
/// (`anthropic/claude-sonnet-4.5` is one Model at one Provider).
const SEPARATOR: char = '/';

/// What the settings call the Model an Action resolves to when it binds none.
pub(crate) const MODEL_SETTING: &str = "default_model";
/// And the one an image Selection resolves to first.
pub(crate) const VISION_SETTING: &str = "default_vision_model";

/// Where the preamble names every preset there is, filled in from the presets
/// themselves so that adding one cannot leave the file describing the old set.
const PRESETS: &str = "{presets}";

/// The prose a fresh installation is met by: what the file is, and what each
/// field in the example under it means.
const PREAMBLE: &str = r#"# Demysto's settings.
#
# Read when Demysto starts, and again whenever Settings writes it — so restart
# Demysto after editing this file by hand.
#
# Uncomment the example below and fill it in.
#
# `preset` names a service Demysto knows the conventions of: it fills in
# `base_url`, and it says which environment variable that service's own
# documentation tells people to export. State `base_url` yourself for a service
# that has no preset, or to override what a preset fills in — a local server
# listening on a port of your own, say.
#
# The presets are:
#
{presets}
#
# A preset marked "no key" is a server running on this machine, which has no
# keys at all: a Provider using one needs none, and none is sent. Every other
# preset wants one.
#
# The key is looked for in the variable `api_key_env` names, then in the
# preset's own variable, then in `api_key` here. Leaving `api_key` out and
# exporting the variable instead keeps the secret out of this file.
#
# `models` lists the Models of a Provider you want to use. `vision` says
# whether one accepts images, and is stated rather than guessed at from the
# identifier, because a name is not a capability.
#
# A Model is named "<provider>/<model>" wherever one is nominated or bound.
# `default_model` is what an Action binding no Model of its own resolves to, and
# `default_vision_model` is what one resolves to for an image."#;

/// What the user is asked to uncomment.
///
/// Held apart from the prose rather than written into it so that the suite can
/// load the very text the instruction points at: a template whose example does
/// not parse would make a liar of the one instruction a new user is given.
const EXAMPLE: &str = r#"default_model = "deepseek/deepseek-chat"
default_vision_model = "openai/gpt-4o"

[[providers]]
name = "deepseek"
preset = "deepseek"
api_key = "sk-..."
models = [{ id = "deepseek-chat" }]

[[providers]]
name = "openai"
preset = "openai"
models = [{ id = "gpt-4o-mini" }, { id = "gpt-4o", vision = true }]

[[providers]]
name = "local"
preset = "lmstudio"
models = [{ id = "qwen/qwen3-8b" }]
"#;

/// What a fresh installation gets: a file that parses, says what goes in it,
/// and configures nothing until the user uncomments the example.
fn template() -> String {
    let example: String = EXAMPLE
        .lines()
        .map(|line| match line.is_empty() {
            true => "#\n".to_owned(),
            false => format!("# {line}\n"),
        })
        .collect();

    // Listed from the presets themselves rather than named in the prose, so
    // that a preset added later cannot leave the file describing the old set.
    // One to a line, so that neither can it push a line past the margin.
    let named = Preset::ALL.map(|preset| match preset.spec() {
        Spec {
            name,
            auth: Auth::Nothing,
            ..
        } => format!("#   {name} (no key)"),
        Spec { name, .. } => format!("#   {name}"),
    });

    let preamble = PREAMBLE.replace(PRESETS, &named.join("\n"));

    format!("{preamble}\n\nversion = {VERSION}\n\n{example}")
}

/// The configuration Demysto is running on, with the keys already resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Config {
    /// The file this was read from. Held because the errors composed later —
    /// when a Run resolves to a Model that is not there, or to a Provider with
    /// no key — still have to name the one place the fix can be made.
    pub(crate) path: PathBuf,
    pub(crate) providers: Vec<Provider>,
    /// The Model an Action that binds none of its own resolves to.
    pub(crate) default_model: Option<String>,
    /// The Model an Action that binds none of its own resolves to when the
    /// Selection is an image. Separate because the cheap everyday Model
    /// usually cannot see.
    pub(crate) default_vision_model: Option<String>,
}

/// A configured LLM endpoint, and the Models it offers.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Provider {
    /// What the user calls this Provider, and the first half of every name one
    /// of its Models is known by.
    pub(crate) name: String,
    pub(crate) base_url: String,
    /// What this Provider is authenticated with.
    ///
    /// Settled at load, while the file's path and the Provider's own fields are
    /// still at hand, and acted on only when a Run resolves to this Provider:
    /// one Provider missing its key is no reason for another Provider's Models
    /// to stop working.
    pub(crate) key: Key,
    pub(crate) models: Vec<Model>,
}

/// What Demysto will authenticate a Provider with, once the settings have been
/// read.
///
/// Deliberately without a derived `Debug`, for the reason [`Provider`] has a
/// hand-written one: a key that can be printed is a key that ends up in a panic
/// message or, once ticket 11 has them, a log.
#[derive(Clone, PartialEq, Eq)]
pub(crate) enum Key {
    /// The key to send, and which of the three sources it came out of.
    Found { key: String, from: Origin },
    /// The service has none to send — a server answering on this machine. The
    /// request goes out unauthenticated (ADR-0006).
    NotNeeded,
    /// The service wants one and none was found. The sentence says where to
    /// put one.
    Missing(String),
}

/// Which of ADR-0002's three sources a key was found in.
///
/// Carried so that the settings window can tell the user where their key is
/// without being handed the key: a field it must not overwrite with a blank,
/// and a variable it should name rather than invite them to paste over.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Origin {
    /// The `api_key` field of the settings file.
    File,
    /// An environment variable, which is named.
    Variable(String),
}

/// A specific Model offered by a Provider, with the capability Demysto needs to
/// know about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Model {
    /// What the Provider calls it, and what the request carries.
    pub(crate) id: String,
    /// Whether it accepts images. Stated by the user rather than inferred from
    /// the identifier: names are marketing, and a wrong guess here is either a
    /// refused request or a Model that could have seen and was never asked to.
    pub(crate) vision: bool,
}

impl fmt::Debug for ProviderEntry {
    /// Written out rather than derived, for the reason [`Provider`]'s own is.
    /// This one holds the key as the file states it — and, once the settings
    /// window builds one out of what somebody typed, as they typed it.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProviderEntry")
            .field("name", &self.name)
            .field("base_url", &self.base_url)
            .field("preset", &self.preset)
            .field("api_key_env", &self.api_key_env)
            .field("models", &self.models)
            .field("api_key", &self.api_key.as_ref().map(|_| "<not shown>"))
            .finish()
    }
}

impl fmt::Debug for Provider {
    /// Written out rather than derived, so that the key cannot arrive somewhere
    /// nobody meant to send it through a panic message or, once ticket 11 has
    /// them, a log. ADR-0002 leaves it readable on disk by its owner; that is
    /// the whole of what it leaves readable.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Provider")
            .field("name", &self.name)
            .field("base_url", &self.base_url)
            .field("models", &self.models)
            .field(
                "key",
                match &self.key {
                    Key::Found { .. } => &"<not shown>",
                    Key::NotNeeded => &"<not needed>",
                    Key::Missing(_) => &"<none>",
                },
            )
            .finish()
    }
}

impl Config {
    /// The Model named `name`, and the Provider that offers it. `None` when
    /// nothing configured here answers to that name.
    pub(crate) fn model(&self, name: &str) -> Option<(&Provider, &Model)> {
        let (provider, id) = name.split_once(SEPARATOR)?;
        let provider = self.providers.iter().find(|it| it.name == provider)?;

        provider
            .models
            .iter()
            .find(|model| model.id == id)
            .map(|model| (provider, model))
    }

    /// Every Model configured, across every Provider, in the order the file
    /// states them.
    pub(crate) fn models(&self) -> impl Iterator<Item = (&Provider, &Model)> {
        self.providers
            .iter()
            .flat_map(|provider| provider.models.iter().map(move |model| (provider, model)))
    }
}

/// What a Model is nominated or bound by: the Provider that offers it and its
/// own identifier, so that two Providers offering the same Model are still two
/// Models with two keys and two bills.
pub(crate) fn qualified(provider: &Provider, model: &Model) -> String {
    format!("{}{SEPARATOR}{}", provider.name, model.id)
}

/// What went wrong between the settings file and a Provider Demysto can use.
///
/// Every variant carries the whole sentence the user is shown: the file is the
/// only place the fix can be made, so an error that does not name it is an
/// error the user cannot act on.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", content = "message", rename_all = "snake_case")]
pub enum ConfigError {
    /// The file could not be read, or could not be created.
    Unreadable(String),
    /// A Provider refused a key entered in the settings window, so nothing was
    /// saved. Not a fault of the file — a fault the file was about to acquire.
    Refused(String),
    /// The file could not be written. Held apart from [`Self::Unreadable`]
    /// because they are opposite news: one is settings Demysto never had, the
    /// other is settings the user has just lost.
    Unwritable(String),
    /// The file was read but is not something Demysto can act on.
    Malformed(String),
    /// The file is valid and configures no Provider.
    NoProvider(String),
}

impl ConfigError {
    /// The sentence the user is shown.
    pub fn message(&self) -> &str {
        match self {
            Self::Unreadable(message)
            | Self::Refused(message)
            | Self::Unwritable(message)
            | Self::Malformed(message)
            | Self::NoProvider(message) => message,
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for ConfigError {}

/// The environment as it was when Demysto started.
///
/// A copy rather than a look at the live one, because the settings are no
/// longer read once: the window writes them and Demysto reads them again, and a
/// key that changed under it between those two reads is a key nobody can reason
/// about (the spec's *Core modules*). Taking it as a value also lets key
/// resolution be tested without mutating the environment of the whole test
/// binary — the same reason [`crate::paths::config_dir`] takes its inputs
/// rather than reading them.
#[derive(Clone, Default)]
pub(crate) struct Environment(BTreeMap<String, String>);

impl Environment {
    /// Everything exported to this process, as it stands now.
    pub(crate) fn snapshot() -> Self {
        Self(std::env::vars().collect())
    }

    fn get(&self, name: &str) -> Option<String> {
        self.0.get(name).cloned()
    }

    /// An environment holding exactly what a caller put in it.
    #[cfg(test)]
    pub(crate) fn holding(variables: &[(&str, &str)]) -> Self {
        Self(
            variables
                .iter()
                .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
                .collect(),
        )
    }
}

impl fmt::Debug for Environment {
    /// Written out rather than derived, for the reason [`Provider`]'s own is:
    /// this holds every variable the shell exported, which on the machine of
    /// anybody who uses more than one such tool is several other people's keys.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Environment").field(&"<not shown>").finish()
    }
}

/// Reads the settings file, creating it when it is not there yet, and resolves
/// the Providers it configures.
pub(crate) fn load(config_dir: &Path, env: &Environment) -> Result<Config, ConfigError> {
    let (path, text) = read(config_dir)?;
    let config = resolve(&path, parse(&path, &text)?, env)?;

    // Asked here rather than in [`resolve`], which the settings window also
    // goes through: a file configuring nothing is Demysto having nothing to run
    // against, and it is also what starting over passes through. Refusing to
    // save it would leave somebody unable to remove their last Provider.
    if config.providers.is_empty() {
        return Err(ConfigError::NoProvider(format!(
            "no Provider is configured; open {} and fill in the example it holds",
            path.display()
        )));
    }

    Ok(config)
}

/// Where the settings file is and what is written in it, the file being created
/// from the template when it is not there yet.
///
/// Held apart from [`load`] because the settings window edits the text rather
/// than the [`Config`] it loads to: the preamble, the user's own comments, and
/// anything a later Demysto wrote there all survive a round trip through the
/// text and none of them survives one through [`File`].
pub(crate) fn read(config_dir: &Path) -> Result<(PathBuf, String), ConfigError> {
    let path = config_dir.join(FILE_NAME);
    let text = read_or_create(&path)?;

    Ok((path, text))
}

/// The file as it is written, checked only for being a shape this build knows.
pub(crate) fn parse(path: &Path, text: &str) -> Result<File, ConfigError> {
    let file: File = toml::from_str(text).map_err(|error| unparseable(path, text, &error))?;

    if file.version > VERSION {
        return Err(ConfigError::Malformed(format!(
            "{} says it is version {}, and this Demysto understands version {VERSION}; \
             update Demysto, or point {} at another directory",
            path.display(),
            file.version,
            crate::paths::CONFIG_DIR_ENV,
        )));
    }

    Ok(file)
}

/// The Providers a parsed file configures, with their keys resolved. Every
/// error here is the file saying something nobody can act on.
pub(crate) fn resolve(path: &Path, file: File, env: &Environment) -> Result<Config, ConfigError> {
    let mut providers: Vec<Provider> = Vec::with_capacity(file.providers.len());

    for entry in &file.providers {
        nameable(entry, &providers, path)?;

        providers.push(Provider {
            name: entry.name.clone(),
            base_url: base_url(entry, path)?,
            key: resolve_key(entry, env, path),
            models: entry
                .models
                .iter()
                .map(|model| Model {
                    id: model.id.clone(),
                    vision: model.vision,
                })
                .collect(),
        });
    }

    Ok(Config {
        path: path.to_owned(),
        providers,
        default_model: file.default_model,
        default_vision_model: file.default_vision_model,
    })
}

/// The settings file as it is written, before anything is resolved.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct File {
    /// Absent in a file written by hand, which is the same as the first version.
    #[serde(default = "first_version")]
    version: u32,
    #[serde(default)]
    pub(crate) providers: Vec<ProviderEntry>,
    pub(crate) default_model: Option<String>,
    pub(crate) default_vision_model: Option<String>,
}

fn first_version() -> u32 {
    VERSION
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProviderEntry {
    /// What the user calls this Provider, and what the first half of a Model's
    /// name refers to.
    pub(crate) name: String,
    /// Absent when the preset supplies it.
    pub(crate) base_url: Option<String>,
    pub(crate) preset: Option<Preset>,
    pub(crate) api_key: Option<String>,
    pub(crate) api_key_env: Option<String>,
    /// The Models of this Provider the user wants to use — not everything it
    /// offers, which is what the Model list is fetched for.
    #[serde(default)]
    pub(crate) models: Vec<ModelEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ModelEntry {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) vision: bool,
}

/// A service Demysto knows the conventions of.
///
/// The three ADR-0002 fixes the key order for, and the two local servers
/// ADR-0006 adds. A preset is a decision about where somebody's key goes — or
/// that there is none — so one that no ADR records would be a decision nothing
/// recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Preset {
    Deepseek,
    Lmstudio,
    Ollama,
    Openai,
    Openrouter,
}

/// What Demysto knows about a service: the word the file names it by, where it
/// answers, and the environment variable its own documentation tells people to
/// export.
pub(crate) struct Spec {
    pub(crate) name: &'static str,
    pub(crate) base_url: &'static str,
    pub(crate) auth: Auth,
}

/// What a service wants by way of authentication.
///
/// Stated rather than inferred from the absence of a variable name: "nobody
/// documented a variable for this" and "this has no keys at all" are different
/// facts, and only the second may turn authentication off. A service that
/// wanted a key and named no variable for it would need a variant of its own.
pub(crate) enum Auth {
    /// A key, which the service's own documentation tells people to export as
    /// this.
    Variable(&'static str),
    /// Nothing. A server answering on this machine has no keys to want.
    Nothing,
}

impl Preset {
    /// Every preset there is, so that the template can name them: a preset
    /// nobody has heard of is a base URL somebody looks up anyway.
    pub(crate) const ALL: [Self; 5] = [
        Self::Deepseek,
        Self::Lmstudio,
        Self::Ollama,
        Self::Openai,
        Self::Openrouter,
    ];

    /// The preset the settings file — or the settings window, which writes the
    /// same word into it — calls `name`. `None` when nothing here is called that.
    pub(crate) fn named(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|preset| preset.spec().name == name)
    }

    /// Everything known about one service, in one place: a preset added here is
    /// a preset added once, and the match keeps the compiler asking.
    pub(crate) fn spec(self) -> Spec {
        match self {
            Self::Deepseek => Spec {
                name: "deepseek",
                base_url: "https://api.deepseek.com/v1",
                auth: Auth::Variable("DEEPSEEK_API_KEY"),
            },
            Self::Lmstudio => Spec {
                name: "lmstudio",
                base_url: "http://localhost:1234/v1",
                auth: Auth::Nothing,
            },
            Self::Ollama => Spec {
                name: "ollama",
                base_url: "http://localhost:11434/v1",
                auth: Auth::Nothing,
            },
            Self::Openai => Spec {
                name: "openai",
                base_url: "https://api.openai.com/v1",
                auth: Auth::Variable("OPENAI_API_KEY"),
            },
            Self::Openrouter => Spec {
                name: "openrouter",
                base_url: "https://openrouter.ai/api/v1",
                auth: Auth::Variable("OPENROUTER_API_KEY"),
            },
        }
    }
}

/// What a Provider is reached at: what it states, else what its preset knows.
///
/// A stated base URL wins over the preset's, so that a proxy or a regional
/// endpoint does not cost the user the preset's other half.
pub(crate) fn base_url(entry: &ProviderEntry, path: &Path) -> Result<String, ConfigError> {
    stated(entry.base_url.clone())
        .or_else(|| entry.preset.map(|preset| preset.spec().base_url.to_owned()))
        .ok_or_else(|| {
            ConfigError::Malformed(format!(
                "the Provider \"{}\" in {} states no base_url and no preset to take one from",
                entry.name,
                path.display()
            ))
        })
}

/// What has to hold before a Provider can be named in a Model's name at all.
///
/// A name that is empty, taken, or holds the separator would make some Model
/// unreachable or ambiguous, and a Model nobody can name is a Model nobody can
/// nominate — a failure that would otherwise surface as "no such Model" over a
/// file that plainly holds it.
fn nameable(entry: &ProviderEntry, taken: &[Provider], path: &Path) -> Result<(), ConfigError> {
    let malformed = |reason: String| {
        Err(ConfigError::Malformed(format!(
            "{reason} in {}",
            path.display()
        )))
    };

    if entry.name.trim().is_empty() {
        return malformed("a Provider is configured with no name".to_owned());
    }

    if entry.name.contains(SEPARATOR) {
        return malformed(format!(
            "the Provider \"{}\" has a \"{SEPARATOR}\" in its name, which is what separates a \
             Provider from a Model",
            entry.name
        ));
    }

    if taken.iter().any(|provider| provider.name == entry.name) {
        return malformed(format!(
            "two Providers are called \"{}\", so a Model of either cannot be named",
            entry.name
        ));
    }

    // A Model with no identifier is a Model nobody can name and a request with
    // no model in it. It arrives from a window where "Add a Model" adds an
    // empty row, so it is what saving without typing produces.
    if entry.models.iter().any(|model| model.id.trim().is_empty()) {
        return malformed(format!(
            "the Provider \"{}\" lists a Model with no name",
            entry.name
        ));
    }

    if let Some(duplicate) = duplicate_model(entry) {
        return malformed(format!(
            "the Provider \"{}\" lists the Model \"{duplicate}\" twice",
            entry.name
        ));
    }

    Ok(())
}

fn duplicate_model(entry: &ProviderEntry) -> Option<&str> {
    let mut seen = BTreeSet::new();

    entry
        .models
        .iter()
        .find(|model| !seen.insert(model.id.as_str()))
        .map(|model| model.id.as_str())
}

/// The key for a Provider, in the order ADR-0002 fixes: the variable the
/// Provider names, then the preset's conventional one, then the file itself.
///
/// A source that holds nothing but whitespace is not a source: an exported but
/// empty variable is a common state of a shell, and reading it as "the key is
/// the empty string" would turn a working configuration into a 401.
///
/// Only what happens when all three come up empty depends on the preset. A key
/// stated for a keyless service is still used, because the three sources are
/// asked first: somebody may have put a local server behind something that
/// wants one.
pub(crate) fn resolve_key(entry: &ProviderEntry, env: &Environment, path: &Path) -> Key {
    let from_env =
        |name: &str| stated(env.get(name)).map(|key| (key, Origin::Variable(name.to_owned())));

    let found = entry
        .api_key_env
        .as_deref()
        .and_then(from_env)
        .or_else(|| conventional(entry).and_then(from_env))
        .or_else(|| stated(entry.api_key.clone()).map(|key| (key, Origin::File)));

    match found {
        Some((key, from)) => Key::Found { key, from },
        // Naming a variable in `api_key_env` is the user saying this Provider
        // is authenticated, so a variable that turns out to hold nothing is a
        // fault to report rather than a service to stop authenticating. It
        // holds nothing routinely: an application launched from the Finder or a
        // desktop entry never sees what a shell profile exported.
        None if has_no_keys(entry) && entry.api_key_env.is_none() => Key::NotNeeded,
        None => Key::Missing(no_key(entry, path)),
    }
}

/// The variable this Provider's preset says the service documents, where it
/// documents one.
fn conventional(entry: &ProviderEntry) -> Option<&'static str> {
    match entry.preset?.spec().auth {
        Auth::Variable(name) => Some(name),
        Auth::Nothing => None,
    }
}

/// Whether this Provider's preset says the service has no keys at all.
///
/// Only a preset can say so — ADR-0006. A Provider written out by hand cannot
/// declare itself keyless, so no typo in this file quietly turns authentication
/// off for a service that wanted it.
fn has_no_keys(entry: &ProviderEntry) -> bool {
    entry
        .preset
        .is_some_and(|preset| matches!(preset.spec().auth, Auth::Nothing))
}

/// A value somebody actually stated: trimmed, and `None` when there was nothing
/// there but whitespace.
///
/// A key pasted out of a web page or read from a file arrives with a newline on
/// it often enough that trimming is the kinder default.
pub(crate) fn stated(value: Option<String>) -> Option<String> {
    let value = value?;
    let trimmed = value.trim();

    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

/// Everywhere the key was looked for, so that the user is told where to put one
/// rather than only that there isn't one.
fn no_key(entry: &ProviderEntry, path: &Path) -> String {
    // Sorted and deduplicated: a Provider that names its preset's own variable
    // in `api_key_env` would otherwise be told about it twice.
    let variables: BTreeSet<&str> = entry
        .api_key_env
        .as_deref()
        .into_iter()
        .chain(conventional(entry))
        .collect();

    match variables.is_empty() {
        true => format!(
            "The Provider \"{}\" has no API key: set api_key for it in {}, or name an \
             environment variable in api_key_env.",
            entry.name,
            path.display()
        ),
        false => format!(
            "The Provider \"{}\" has no API key: export {}, or set api_key for it in {}.",
            entry.name,
            variables.into_iter().collect::<Vec<_>>().join(" or "),
            path.display()
        ),
    }
}

fn read_or_create(path: &Path) -> Result<String, ConfigError> {
    match fs::read_to_string(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            create(path)?;

            // Read back rather than returning the template: the file on disk is
            // the source of truth, and it is the one the user will edit.
            fs::read_to_string(path).map_err(|error| unreadable(path, &error))
        }
        read => read.map_err(|error| unreadable(path, &error)),
    }
}

/// Writes the template, owner-only, without touching a file that is already
/// there.
fn create(path: &Path) -> Result<(), ConfigError> {
    use std::io::Write;

    if let Some(parent) = path.parent() {
        files::create_dir(parent).map_err(|error| unreadable(parent, &error))?;
    }

    let mut file = match files::options().create_new(true).write(true).open(path) {
        Ok(file) => file,
        // Somebody else got there between the read and this line. Their file is
        // as good as ours, and better than an error the user cannot act on.
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => return Ok(()),
        Err(error) => return Err(unreadable(path, &error)),
    };

    file.write_all(template().as_bytes())
        .map_err(|error| unreadable(path, &error))
}

/// Replaces the settings file with `text`, owner-only, without ever leaving a
/// half-written one behind — see [`crate::files::replace`], which `catalogue`
/// writes an Action through in the same way.
pub(crate) fn write(path: &Path, text: &str) -> Result<(), ConfigError> {
    files::replace(path, text).map_err(|error| unwritable(path, &error))
}

fn unreadable(path: &Path, error: &io::Error) -> ConfigError {
    ConfigError::Unreadable(format!("{} could not be read: {error}", path.display()))
}

fn unwritable(path: &Path, error: &io::Error) -> ConfigError {
    ConfigError::Unwritable(format!("{} could not be written: {error}", path.display()))
}

/// What a parse failure says, and where — but never the line it happened on.
///
/// `toml`'s own `Display` quotes the offending source line back, and in a file
/// whose purpose is to hold a key, that line is the key. This message ends up
/// in a window that also renders untrusted model output, and ADR-0002 asks for
/// exactly one thing in exchange for keeping the key out of the keychain: "The
/// key never enters the webview." So the reason and the line number cross, and
/// the file's own text does not.
fn unparseable(path: &Path, text: &str, error: &toml::de::Error) -> ConfigError {
    let line = error
        .span()
        .and_then(|span| text.get(..span.start))
        .map(|before| before.matches('\n').count() + 1);

    ConfigError::Malformed(match line {
        Some(line) => format!(
            "{} is not valid TOML at line {line}: {}",
            path.display(),
            error.message()
        ),
        None => format!("{} is not valid TOML: {}", path.display(), error.message()),
    })
}

#[cfg(test)]
mod tests {
    //! The temporary directory standing in for the config location, and a fake
    //! environment beside it — the two substitutions the spec's *Testing
    //! Decisions* names for this module.

    use tempfile::TempDir;

    use super::*;

    /// A settings file holding `body` under the version line, and the config it
    /// loads to.
    fn load_with(body: &str, env: &Environment) -> (TempDir, Result<Config, ConfigError>) {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join(FILE_NAME), format!("version = 1\n\n{body}")).unwrap();

        let loaded = load(dir.path(), env);
        (dir, loaded)
    }

    fn config(body: &str, env: &Environment) -> Config {
        let (_dir, loaded) = load_with(body, env);
        loaded.expect("the settings should have loaded")
    }

    fn error(body: &str, env: &Environment) -> ConfigError {
        let (_dir, loaded) = load_with(body, env);
        loaded.expect_err("the settings should not have loaded")
    }

    /// What the first Provider resolved to, for the tests about where a key
    /// comes from.
    fn key(body: &str, env: &Environment) -> Key {
        config(body, env).providers.remove(0).key
    }

    /// The key the first Provider found, and nothing else.
    fn found(body: &str, env: &Environment) -> String {
        match key(body, env) {
            Key::Found { key, .. } => key,
            other => panic!("the Provider should have found a key: {:?}", Named(&other)),
        }
    }

    /// The sentence a Provider that wanted a key and found none carries.
    fn no_key_message(body: &str, env: &Environment) -> String {
        match key(body, env) {
            Key::Missing(message) => message,
            other => panic!("the Provider should have wanted a key: {:?}", Named(&other)),
        }
    }

    /// A [`Key`] as a failing assertion can name it. `Key` has no `Debug` of
    /// its own on purpose, and a test is not a reason to give it one.
    struct Named<'a>(&'a Key);

    impl std::fmt::Debug for Named<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(match self.0 {
                Key::Found { .. } => "a key",
                Key::NotNeeded => "no key needed",
                Key::Missing(_) => "no key found",
            })
        }
    }

    /// A Provider naming its own variable, carrying a preset, and holding a key
    /// in the file — all three sources at once, so that a test can take one away.
    const EVERY_SOURCE: &str = "\
[[providers]]
name = \"deepseek\"
preset = \"deepseek\"
base_url = \"https://api.deepseek.com/v1\"
api_key = \"from-the-file\"
api_key_env = \"MY_OWN_KEY\"
models = [{ id = \"deepseek-chat\" }]
";

    /// The same Provider with `api_key_env` taken off it.
    fn without_its_own_variable() -> String {
        EVERY_SOURCE.replace("api_key_env = \"MY_OWN_KEY\"\n", "")
    }

    /// And with the preset taken off as well.
    fn file_only() -> String {
        without_its_own_variable().replace("preset = \"deepseek\"\n", "")
    }

    #[test]
    fn the_key_comes_from_the_variable_the_provider_names() {
        let env = Environment::holding(&[("MY_OWN_KEY", "from-my-own-variable")]);

        assert_eq!(found(EVERY_SOURCE, &env), "from-my-own-variable");
    }

    #[test]
    fn the_key_comes_from_the_presets_conventional_variable() {
        let env = Environment::holding(&[("DEEPSEEK_API_KEY", "from-the-preset")]);

        assert_eq!(found(&without_its_own_variable(), &env), "from-the-preset");
    }

    #[test]
    fn the_key_comes_from_the_file_when_the_environment_holds_none() {
        assert_eq!(
            found(&file_only(), &Environment::default()),
            "from-the-file"
        );
    }

    #[test]
    fn the_variable_the_provider_names_wins_over_the_presets() {
        let env = Environment::holding(&[
            ("MY_OWN_KEY", "from-my-own-variable"),
            ("DEEPSEEK_API_KEY", "from-the-preset"),
        ]);

        assert_eq!(found(EVERY_SOURCE, &env), "from-my-own-variable");
    }

    #[test]
    fn the_presets_variable_wins_over_the_file() {
        let env = Environment::holding(&[("DEEPSEEK_API_KEY", "from-the-preset")]);

        assert_eq!(found(EVERY_SOURCE, &env), "from-the-preset");
    }

    #[test]
    fn a_variable_that_is_set_but_empty_is_not_a_key() {
        // Exported and left empty is a common state of a shell profile, and
        // reading it as a key would turn a working configuration into a 401.
        let env = Environment::holding(&[("MY_OWN_KEY", ""), ("DEEPSEEK_API_KEY", "   ")]);

        assert_eq!(found(EVERY_SOURCE, &env), "from-the-file");
    }

    #[test]
    fn a_key_arrives_without_the_whitespace_around_it() {
        let env = Environment::holding(&[("MY_OWN_KEY", "  from-my-own-variable\n")]);

        assert_eq!(found(EVERY_SOURCE, &env), "from-my-own-variable");
    }

    #[test]
    fn no_key_anywhere_names_every_variable_that_was_looked_at() {
        let missing = EVERY_SOURCE.replace("api_key = \"from-the-file\"\n", "");
        let message = no_key_message(&missing, &Environment::default());

        assert!(message.contains("MY_OWN_KEY"), "{message}");
        assert!(message.contains("DEEPSEEK_API_KEY"), "{message}");
        assert!(message.contains("api_key"), "{message}");
    }

    #[test]
    fn a_provider_with_no_variables_to_name_still_says_where_a_key_goes() {
        let missing = file_only().replace("api_key = \"from-the-file\"\n", "");
        let message = no_key_message(&missing, &Environment::default());

        assert!(message.contains(FILE_NAME), "{message}");
        assert!(message.contains("api_key_env"), "{message}");
    }

    #[test]
    fn a_provider_with_no_key_says_which_provider_it_is() {
        // Several Providers may be configured, and only one of them is broken.
        let missing = file_only().replace("api_key = \"from-the-file\"\n", "");
        let message = no_key_message(&missing, &Environment::default());

        assert!(message.contains("deepseek"), "{message}");
    }

    #[test]
    fn a_provider_missing_its_key_leaves_the_others_configured() {
        let both = format!(
            "{}\n[[providers]]\nname = \"openai\"\npreset = \"openai\"\n\
             models = [{{ id = \"gpt-4o-mini\" }}]\n",
            file_only().replace("api_key = \"from-the-file\"\n", "")
        );

        let config = config(&both, &Environment::holding(&[("OPENAI_API_KEY", "a-key")]));

        assert!(matches!(config.providers[0].key, Key::Missing(_)));
        assert!(matches!(&config.providers[1].key, Key::Found { key, .. } if key == "a-key"));
    }

    #[test]
    fn a_service_with_no_keys_needs_none_stated() {
        let local = "[[providers]]\nname = \"local\"\npreset = \"lmstudio\"\n\
                     models = [{ id = \"qwen/qwen3-8b\" }]\n";

        assert!(matches!(
            key(local, &Environment::default()),
            Key::NotNeeded
        ));
    }

    #[test]
    fn a_key_stated_for_a_service_with_none_is_still_used() {
        // A local server put behind something that does want one.
        let local = "[[providers]]\nname = \"local\"\npreset = \"lmstudio\"\n\
                     api_key = \"from-the-file\"\nmodels = [{ id = \"a-model\" }]\n";

        assert_eq!(found(local, &Environment::default()), "from-the-file");
    }

    #[test]
    fn a_keyless_preset_still_fills_in_the_address() {
        let local = "[[providers]]\nname = \"local\"\npreset = \"ollama\"\n";

        assert_eq!(
            config(local, &Environment::default()).providers[0].base_url,
            "http://localhost:11434/v1"
        );
    }

    #[test]
    fn a_variable_named_for_a_keyless_service_is_still_a_key_that_must_be_found() {
        // The dangerous case: naming api_key_env is the user saying this
        // Provider is authenticated, and an application launched from the
        // Finder never sees what a shell profile exported. Falling through to
        // "this service needs no key" would send the request unauthenticated —
        // and with base_url overridden, send it to a remote host.
        let named = "[[providers]]\nname = \"local\"\npreset = \"lmstudio\"\n\
                     api_key_env = \"MY_LOCAL_KEY\"\nmodels = [{ id = \"a-model\" }]\n";

        let Key::Missing(message) = key(named, &Environment::default()) else {
            panic!("a variable that holds nothing should be reported, not passed over");
        };

        assert!(message.contains("MY_LOCAL_KEY"), "{message}");
    }

    #[test]
    fn a_variable_named_for_a_keyless_service_is_used_when_it_holds_one() {
        let named = "[[providers]]\nname = \"local\"\npreset = \"lmstudio\"\n\
                     api_key_env = \"MY_LOCAL_KEY\"\nmodels = [{ id = \"a-model\" }]\n";
        let env = Environment::holding(&[("MY_LOCAL_KEY", "from-my-own-variable")]);

        assert_eq!(found(named, &env), "from-my-own-variable");
    }

    #[test]
    fn only_a_preset_can_say_a_service_has_no_key() {
        // ADR-0006: a Provider written out by hand still wants one, so no typo
        // in this file can quietly turn authentication off for a service that
        // wanted it.
        let by_hand = "[[providers]]\nname = \"local\"\n\
                       base_url = \"http://localhost:1234/v1\"\nmodels = [{ id = \"a-model\" }]\n";

        assert!(matches!(
            key(by_hand, &Environment::default()),
            Key::Missing(_)
        ));
    }

    #[test]
    fn the_provider_is_read_from_the_file() {
        let provider = config(&file_only(), &Environment::default())
            .providers
            .remove(0);

        assert_eq!(provider.name, "deepseek");
        assert_eq!(provider.base_url, "https://api.deepseek.com/v1");
        assert_eq!(
            provider.models,
            vec![Model {
                id: "deepseek-chat".to_owned(),
                vision: false,
            }]
        );
    }

    #[test]
    fn several_providers_are_all_configured() {
        let both = format!(
            "{EVERY_SOURCE}\n[[providers]]\nname = \"openai\"\npreset = \"openai\"\n\
             api_key = \"another-key\"\nmodels = [{{ id = \"gpt-4o-mini\" }}]\n"
        );

        let names: Vec<String> = config(&both, &Environment::default())
            .providers
            .into_iter()
            .map(|provider| provider.name)
            .collect();

        assert_eq!(names, ["deepseek", "openai"]);
    }

    #[test]
    fn a_preset_fills_in_the_base_url() {
        let by_preset = "[[providers]]\nname = \"openai\"\npreset = \"openai\"\n\
                         api_key = \"a-key\"\n";

        assert_eq!(
            config(by_preset, &Environment::default()).providers[0].base_url,
            "https://api.openai.com/v1"
        );
    }

    #[test]
    fn a_stated_base_url_wins_over_the_presets() {
        // A proxy or a regional endpoint should not cost the user the preset's
        // other half.
        let proxied = EVERY_SOURCE.replace(
            "base_url = \"https://api.deepseek.com/v1\"",
            "base_url = \"https://proxy.internal/v1\"",
        );

        assert_eq!(
            config(&proxied, &Environment::default()).providers[0].base_url,
            "https://proxy.internal/v1"
        );
    }

    #[test]
    fn a_provider_with_no_address_at_all_is_reported() {
        let nowhere = "[[providers]]\nname = \"mine\"\napi_key = \"a-key\"\n";
        let ConfigError::Malformed(message) = error(nowhere, &Environment::default()) else {
            panic!("a Provider with no address should be reported as malformed");
        };

        assert!(message.contains("base_url"), "{message}");
        assert!(message.contains("preset"), "{message}");
    }

    #[test]
    fn a_model_does_not_accept_images_unless_it_says_so() {
        // The whole point of the flag: a name is not a capability.
        let named_like_one = EVERY_SOURCE.replace("deepseek-chat", "gpt-4o-vision-preview");

        assert!(!config(&named_like_one, &Environment::default()).providers[0].models[0].vision);
    }

    #[test]
    fn a_model_marked_vision_capable_carries_it() {
        let seeing = EVERY_SOURCE.replace(
            "{ id = \"deepseek-chat\" }",
            "{ id = \"deepseek-chat\", vision = true }",
        );

        assert!(config(&seeing, &Environment::default()).providers[0].models[0].vision);
    }

    #[test]
    fn a_model_is_found_by_the_provider_that_offers_it_and_its_own_name() {
        let config = config(EVERY_SOURCE, &Environment::default());

        let (provider, model) = config
            .model("deepseek/deepseek-chat")
            .expect("the Model should be found by its qualified name");

        assert_eq!(provider.name, "deepseek");
        assert_eq!(model.id, "deepseek-chat");
    }

    #[test]
    fn a_model_whose_own_name_holds_a_slash_is_still_found() {
        // Half of what an aggregating Provider offers is named this way.
        let routed = "[[providers]]\nname = \"openrouter\"\npreset = \"openrouter\"\n\
                      api_key = \"a-key\"\nmodels = [{ id = \"anthropic/claude-sonnet-4.5\" }]\n";
        let config = config(routed, &Environment::default());

        let (_, model) = config
            .model("openrouter/anthropic/claude-sonnet-4.5")
            .expect("the Model should be found by its qualified name");

        assert_eq!(model.id, "anthropic/claude-sonnet-4.5");
    }

    #[test]
    fn a_provider_whose_name_holds_a_slash_is_reported() {
        // It would make some Model's name ambiguous, and the user would be told
        // there is no such Model over a file that plainly holds it.
        let slashed = EVERY_SOURCE.replace("name = \"deepseek\"", "name = \"deep/seek\"");

        assert!(matches!(
            error(&slashed, &Environment::default()),
            ConfigError::Malformed(_)
        ));
    }

    #[test]
    fn two_providers_of_the_same_name_are_reported() {
        let twice = format!("{EVERY_SOURCE}\n{EVERY_SOURCE}");
        let ConfigError::Malformed(message) = error(&twice, &Environment::default()) else {
            panic!("two Providers of one name should be reported as malformed");
        };

        assert!(message.contains("deepseek"), "{message}");
    }

    #[test]
    fn a_model_listed_twice_by_one_provider_is_reported() {
        let twice = EVERY_SOURCE.replace(
            "models = [{ id = \"deepseek-chat\" }]",
            "models = [{ id = \"deepseek-chat\" }, { id = \"deepseek-chat\", vision = true }]",
        );

        assert!(matches!(
            error(&twice, &Environment::default()),
            ConfigError::Malformed(_)
        ));
    }

    #[test]
    fn the_two_defaults_are_read_from_the_file() {
        let nominated = format!(
            "default_model = \"deepseek/deepseek-chat\"\n\
             default_vision_model = \"openai/gpt-4o\"\n\n{EVERY_SOURCE}"
        );
        let config = config(&nominated, &Environment::default());

        assert_eq!(
            config.default_model.as_deref(),
            Some("deepseek/deepseek-chat")
        );
        assert_eq!(
            config.default_vision_model.as_deref(),
            Some("openai/gpt-4o")
        );
    }

    #[test]
    fn a_settings_file_is_created_when_there_is_none() {
        let dir = TempDir::new().unwrap();

        let _ = load(dir.path(), &Environment::default());

        assert!(dir.path().join(FILE_NAME).is_file());
    }

    #[test]
    fn the_directory_is_created_too() {
        let dir = TempDir::new().unwrap();
        let nested = dir.path().join("never/been/here");

        let _ = load(&nested, &Environment::default());

        assert!(nested.join(FILE_NAME).is_file());
    }

    #[cfg(unix)]
    #[test]
    fn a_created_settings_file_is_readable_by_nobody_else() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDir::new().unwrap();

        let _ = load(dir.path(), &Environment::default());

        let mode = fs::metadata(dir.path().join(FILE_NAME))
            .unwrap()
            .permissions()
            .mode();

        assert_eq!(mode & 0o777, 0o600, "mode was {:o}", mode & 0o777);
    }

    #[test]
    fn a_created_settings_file_configures_nothing_and_says_so() {
        let dir = TempDir::new().unwrap();

        assert!(matches!(
            load(dir.path(), &Environment::default()),
            Err(ConfigError::NoProvider(_))
        ));
    }

    #[test]
    fn the_example_the_template_offers_is_one_that_would_load() {
        // Uncommenting it is the whole of what a new user is asked to do.
        let config = config(EXAMPLE, &Environment::default());

        assert_eq!(
            config.default_model.as_deref(),
            Some("deepseek/deepseek-chat")
        );
        assert!(config
            .model("openai/gpt-4o")
            .is_some_and(|(_, model)| model.vision));

        // The local Provider the example offers is the one that asks for
        // nothing: uncommenting it is the whole of what it takes.
        let (local, _) = config
            .model("local/qwen/qwen3-8b")
            .expect("the example should offer a local Model");

        assert!(matches!(local.key, Key::NotNeeded));
    }

    #[test]
    fn every_preset_the_template_names_is_one_the_file_accepts_and_has_an_address_for() {
        // The names are written out for the template, and read back by serde
        // from the same word; a typo in either would be a preset the file names
        // and rejects.
        let written = template();

        for preset in Preset::ALL {
            let spec = preset.spec();
            assert!(written.contains(spec.name), "{written}");

            // Stating nothing, so that a preset which names no variable is
            // asked for no key and one that names a variable still is.
            let by_preset = format!(
                "[[providers]]\nname = \"mine\"\npreset = \"{}\"\n",
                spec.name
            );
            let provider = config(&by_preset, &Environment::default())
                .providers
                .remove(0);

            assert_eq!(provider.base_url, spec.base_url);

            match spec.auth {
                Auth::Variable(variable) => {
                    let Key::Missing(message) = provider.key else {
                        panic!("{} wants a key and should have asked for one", spec.name);
                    };

                    assert!(message.contains(variable), "{message}");
                }
                Auth::Nothing => assert!(
                    matches!(provider.key, Key::NotNeeded),
                    "{} has no keys and should have asked for none",
                    spec.name
                ),
            }
        }
    }

    #[test]
    fn the_template_holds_the_example_with_a_comment_marker_on_every_line() {
        let written = template();

        for line in EXAMPLE.lines().filter(|line| !line.is_empty()) {
            assert!(written.contains(&format!("# {line}\n")), "{written}");
        }
    }

    #[test]
    fn a_settings_file_that_is_already_there_is_left_alone() {
        let (dir, _) = load_with(&file_only(), &Environment::default());

        let written = fs::read_to_string(dir.path().join(FILE_NAME)).unwrap();

        assert!(written.contains("from-the-file"), "{written}");
    }

    #[test]
    fn a_file_that_is_not_valid_toml_names_itself() {
        let ConfigError::Malformed(message) =
            error("[[providers]\nname = ", &Environment::default())
        else {
            panic!("a broken file should be reported as malformed");
        };

        assert!(message.contains(FILE_NAME), "{message}");
    }

    #[test]
    fn a_broken_file_is_never_quoted_back() {
        // The message reaches a window that also renders untrusted model
        // output, and the line it would quote is the one the key is on:
        // ADR-0002's "The key never enters the webview".
        let unquoted = file_only().replace("\"from-the-file\"", "from-the-file");
        let ConfigError::Malformed(message) = error(&unquoted, &Environment::default()) else {
            panic!("an unquoted value should be reported as malformed");
        };

        assert!(!message.contains("from-the-file"), "{message}");
        assert!(message.contains("line 6"), "{message}");
    }

    #[test]
    fn a_misspelled_field_is_reported_rather_than_ignored() {
        // Silently ignoring `api_kye` would leave the user staring at a file
        // that plainly holds their key and a Demysto that cannot find it.
        let misspelled = file_only().replace("api_key =", "api_kye =");

        assert!(matches!(
            error(&misspelled, &Environment::default()),
            ConfigError::Malformed(_)
        ));
    }

    #[test]
    fn a_misspelled_field_on_a_model_is_reported_too() {
        let misspelled = EVERY_SOURCE.replace("id = \"deepseek-chat\"", "idd = \"deepseek-chat\"");

        assert!(matches!(
            error(&misspelled, &Environment::default()),
            ConfigError::Malformed(_)
        ));
    }

    #[test]
    fn a_file_from_a_newer_demysto_is_not_guessed_at() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join(FILE_NAME), "version = 99\n").unwrap();

        let Err(ConfigError::Malformed(message)) = load(dir.path(), &Environment::default()) else {
            panic!("a file from the future should not be acted on");
        };

        assert!(message.contains("99"), "{message}");
    }

    #[test]
    fn a_file_that_states_no_version_is_read_as_the_first() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join(FILE_NAME), file_only()).unwrap();

        assert!(load(dir.path(), &Environment::default()).is_ok());
    }
}
