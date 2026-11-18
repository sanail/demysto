//! Settings own the environment snapshot so reloading the file cannot change key precedence mid-session.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::files;
use crate::i18n::{say, Words};

pub(crate) const FILE_NAME: &str = "settings.toml";

/// Reject newer formats because guessing at their meaning can send a key to the wrong endpoint.
pub(crate) const VERSION: u32 = 1;

/// A Provider name cannot contain the separator. A Model identifier may contain several.
const SEPARATOR: char = '/';

pub(crate) const MODEL_SETTING: &str = "default_model";
pub(crate) const VISION_SETTING: &str = "default_vision_model";
pub(crate) const PALETTE_SETTING: &str = "palette_hotkey";
pub(crate) const SCREEN_AREA_SETTING: &str = "screen_area_hotkey";
pub(crate) const LARGE_SELECTION_SETTING: &str = "large_selection";
pub(crate) const LANGUAGE_SETTING: &str = "language";
pub(crate) const WELCOMED_SETTING: &str = "welcomed";

/// Count characters so the warning can be shown before involving a Provider.
pub const DEFAULT_LARGE_SELECTION: u64 = 20_000;

/// Keep the example separate from the preamble so tests can parse the exact TOML users uncomment.
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

/// Localize the preamble so a fresh installation introduces itself in the interface language.
fn template(words: &Words) -> String {
    let example: String = EXAMPLE
        .lines()
        .map(|line| match line.is_empty() {
            true => "#\n".to_owned(),
            false => format!("# {line}\n"),
        })
        .collect();

    // Generate the list from the presets so the template always advertises the current choices.
    let named = Preset::ALL.map(|preset| match preset.spec() {
        Spec {
            name,
            auth: Auth::Nothing,
            ..
        } => say!(words, "settings-file-preset-keyless", "preset" = name),
        Spec { name, .. } => say!(words, "settings-file-preset", "preset" = name),
    });

    let preamble = say!(
        words,
        "settings-file-preamble",
        "presets" = named.join("\n"),
        "languageEnv" = crate::i18n::LANGUAGE_ENV,
        "largeSelection" = DEFAULT_LARGE_SELECTION.to_string(),
    );

    format!("{preamble}\n\nversion = {VERSION}\n\n{example}")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Config {
    /// Keep the path so errors while resolving a Run can point to the file the user can fix.
    pub(crate) path: PathBuf,
    pub(crate) providers: Vec<Provider>,
    pub(crate) default_model: Option<String>,
    /// Keep a separate default for images because the everyday Model may be text-only.
    pub(crate) default_vision_model: Option<String>,
    pub(crate) large_selection: Option<u64>,
    /// An omitted language lets the operating system decide.
    pub(crate) language: Option<String>,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Provider {
    pub(crate) name: String,
    pub(crate) base_url: String,
    /// Defer a missing-key error until a Run needs this Provider so other Providers can still work.
    pub(crate) key: Key,
    pub(crate) models: Vec<Model>,
    pub(crate) skip_reasoning: SkipReasoning,
}

/// Omit Debug because a key can otherwise reach a panic message or a log.
#[derive(Clone, PartialEq, Eq)]
pub(crate) enum Key {
    Found {
        key: String,
        from: Origin,
    },
    NotNeeded,
    /// The message tells the user where to supply the missing key.
    Missing(String),
}

/// The settings window can explain where the key came from without receiving the key itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Origin {
    File,
    Variable(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Model {
    pub(crate) id: String,
    /// The user supplies vision capability because an identifier cannot reliably tell us what a Model accepts.
    pub(crate) vision: bool,
}

impl fmt::Debug for ProviderEntry {
    /// A draft can hold a key as the user typed it. Keep it out of debug output.
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
    /// A configured key must stay out of panic messages and logs.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Provider")
            .field("name", &self.name)
            .field("base_url", &self.base_url)
            .field("models", &self.models)
            .field("skip_reasoning", &self.skip_reasoning)
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
    pub(crate) fn model(&self, name: &str) -> Option<(&Provider, &Model)> {
        let (provider, id) = name.split_once(SEPARATOR)?;
        let provider = self.providers.iter().find(|it| it.name == provider)?;

        provider
            .models
            .iter()
            .find(|model| model.id == id)
            .map(|model| (provider, model))
    }

    /// Zero lets the user disable a warning that otherwise adds an interruption before a Run.
    pub(crate) fn large_selection(&self) -> Option<u64> {
        match self.large_selection.unwrap_or(DEFAULT_LARGE_SELECTION) {
            0 => None,
            characters => Some(characters),
        }
    }

    /// Preserve the order in which the user configured the Providers and Models.
    pub(crate) fn models(&self) -> impl Iterator<Item = (&Provider, &Model)> {
        self.providers
            .iter()
            .flat_map(|provider| provider.models.iter().map(move |model| (provider, model)))
    }
}

/// A Provider's name distinguishes identical Model identifiers with different keys and bills.
pub(crate) fn qualified(provider: &Provider, model: &Model) -> String {
    format!("{}{SEPARATOR}{}", provider.name, model.id)
}

/// Include the settings file in actionable errors so the user can find where to fix them.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", content = "message", rename_all = "snake_case")]
pub enum ConfigError {
    Unreadable(String),
    Refused(String),
    /// A failed write is reported separately so users know their changes were never saved.
    Unwritable(String),
    Malformed(String),
    NoProvider(String),
}

impl ConfigError {
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

/// A snapshot keeps key lookup stable across settings reloads and lets tests avoid changing the process environment.
#[derive(Clone, Default)]
pub(crate) struct Environment(BTreeMap<String, String>);

impl Environment {
    pub(crate) fn snapshot() -> Self {
        Self(std::env::vars().collect())
    }

    pub(crate) fn get(&self, name: &str) -> Option<String> {
        self.0.get(name).cloned()
    }

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
    /// The shell can export keys for other applications. Keep its values out of debug output.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Environment").field(&"<not shown>").finish()
    }
}

pub(crate) fn load(
    config_dir: &Path,
    env: &Environment,
    words: &Words,
) -> Result<Config, ConfigError> {
    let (path, text) = read(config_dir, words)?;
    let config = resolve(&path, parse(&path, &text, words)?, env, words)?;

    // Saving must allow an empty file so the user can remove their last Provider.
    if config.providers.is_empty() {
        return Err(ConfigError::NoProvider(say!(
            words,
            "config-no-provider",
            "path" = path.display().to_string()
        )));
    }

    Ok(config)
}

/// Keep the text available for editing so comments and fields from another version survive a save.
pub(crate) fn read(config_dir: &Path, words: &Words) -> Result<(PathBuf, String), ConfigError> {
    let path = config_dir.join(FILE_NAME);
    let text = read_or_create(&path, words)?;

    Ok((path, text))
}

/// Resolve the language before producing any settings errors. Missing or malformed files leave that choice to the system.
pub(crate) fn stated_language(config_dir: &Path) -> Option<String> {
    #[derive(Deserialize)]
    struct Stating {
        language: Option<String>,
    }

    let text = fs::read_to_string(config_dir.join(FILE_NAME)).ok()?;

    toml::from_str::<Stating>(&text).ok()?.language
}

pub(crate) fn parse(path: &Path, text: &str, words: &Words) -> Result<File, ConfigError> {
    let file: File =
        toml::from_str(text).map_err(|error| unparseable(path, text, &error, words))?;

    if file.version > VERSION {
        return Err(ConfigError::Malformed(say!(
            words,
            "config-newer-version",
            "path" = path.display().to_string(),
            "stated" = file.version,
            "understood" = VERSION,
            "variable" = crate::paths::CONFIG_DIR_ENV,
        )));
    }

    Ok(file)
}

pub(crate) fn resolve(
    path: &Path,
    file: File,
    env: &Environment,
    words: &Words,
) -> Result<Config, ConfigError> {
    let mut providers: Vec<Provider> = Vec::with_capacity(file.providers.len());

    for entry in &file.providers {
        nameable(entry, &providers, path, words)?;

        providers.push(Provider {
            name: entry.name.clone(),
            base_url: base_url(entry, path, words)?,
            key: resolve_key(entry, env, path, words),
            skip_reasoning: skip_reasoning(entry),
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
        large_selection: file.large_selection,
        language: file.language,
    })
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct File {
    /// Files written before versioning use the first format.
    #[serde(default = "first_version")]
    version: u32,
    #[serde(default)]
    pub(crate) providers: Vec<ProviderEntry>,
    pub(crate) default_model: Option<String>,
    pub(crate) default_vision_model: Option<String>,
    /// Keep Hotkeys in the file schema so their presence does not make the file unreadable.
    /// An absent value uses the desktop default and an empty string disables the Hotkey.
    pub(crate) palette_hotkey: Option<String>,
    pub(crate) screen_area_hotkey: Option<String>,
    pub(crate) large_selection: Option<u64>,
    /// An unrecognized language tag leaves the choice to the system instead of preventing startup.
    pub(crate) language: Option<String>,
    /// A default keeps settings files written before Welcome readable.
    #[serde(default)]
    pub(crate) welcomed: bool,
}

fn first_version() -> u32 {
    VERSION
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProviderEntry {
    pub(crate) name: String,
    /// The preset can supply an omitted URL.
    pub(crate) base_url: Option<String>,
    pub(crate) preset: Option<Preset>,
    pub(crate) api_key: Option<String>,
    pub(crate) api_key_env: Option<String>,
    /// A user can configure a subset of the Provider's catalog.
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Preset {
    Deepseek,
    Lmstudio,
    Ollama,
    Openai,
    Openrouter,
}

pub(crate) struct Spec {
    pub(crate) name: &'static str,
    pub(crate) base_url: &'static str,
    pub(crate) auth: Auth,
    pub(crate) reasoning: SkipReasoning,
}

/// Skipping optional reasoning reduces the wait before an answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SkipReasoning {
    DeepSeek,
    OpenRouter,
    Unmentioned,
}

/// Only an explicitly keyless preset may dispense with authentication.
pub(crate) enum Auth {
    Variable(&'static str),
    Nothing,
}

impl Preset {
    /// Expose all presets to the template so users can discover their choices.
    pub(crate) const ALL: [Self; 5] = [
        Self::Deepseek,
        Self::Lmstudio,
        Self::Ollama,
        Self::Openai,
        Self::Openrouter,
    ];

    pub(crate) fn named(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|preset| preset.spec().name == name)
    }

    pub(crate) fn spec(self) -> Spec {
        match self {
            Self::Deepseek => Spec {
                name: "deepseek",
                base_url: "https://api.deepseek.com/v1",
                auth: Auth::Variable("DEEPSEEK_API_KEY"),
                reasoning: SkipReasoning::DeepSeek,
            },
            Self::Lmstudio => Spec {
                name: "lmstudio",
                base_url: "http://localhost:1234/v1",
                auth: Auth::Nothing,
                reasoning: SkipReasoning::Unmentioned,
            },
            Self::Ollama => Spec {
                name: "ollama",
                base_url: "http://localhost:11434/v1",
                auth: Auth::Nothing,
                reasoning: SkipReasoning::Unmentioned,
            },
            Self::Openai => Spec {
                name: "openai",
                base_url: "https://api.openai.com/v1",
                auth: Auth::Variable("OPENAI_API_KEY"),
                reasoning: SkipReasoning::Unmentioned,
            },
            Self::Openrouter => Spec {
                name: "openrouter",
                base_url: "https://openrouter.ai/api/v1",
                auth: Auth::Variable("OPENROUTER_API_KEY"),
                reasoning: SkipReasoning::OpenRouter,
            },
        }
    }
}

/// A custom URL must identify the official gateway before we send its extensions.
pub(crate) fn skip_reasoning(entry: &ProviderEntry) -> SkipReasoning {
    if let Some(preset) = entry.preset {
        return preset.spec().reasoning;
    }

    let openrouter = Preset::Openrouter.spec();
    if entry
        .base_url
        .as_deref()
        .is_some_and(|url| url.trim().trim_end_matches('/') == openrouter.base_url)
    {
        openrouter.reasoning
    } else {
        SkipReasoning::Unmentioned
    }
}

/// A URL override lets a proxy or regional endpoint keep the rest of the preset.
pub(crate) fn base_url(
    entry: &ProviderEntry,
    path: &Path,
    words: &Words,
) -> Result<String, ConfigError> {
    stated(entry.base_url.clone())
        .or_else(|| entry.preset.map(|preset| preset.spec().base_url.to_owned()))
        .ok_or_else(|| {
            ConfigError::Malformed(say!(
                words,
                "config-provider-no-base-url",
                "provider" = entry.name.clone(),
                "path" = path.display().to_string()
            ))
        })
}

/// Reject names that would make a Model unreachable or ambiguous before reporting errors about its nomination.
fn nameable(
    entry: &ProviderEntry,
    taken: &[Provider],
    path: &Path,
    words: &Words,
) -> Result<(), ConfigError> {
    let malformed = |reason: String| {
        Err(ConfigError::Malformed(say!(
            words,
            "config-in-file",
            "reason" = reason,
            "path" = path.display().to_string()
        )))
    };

    if entry.name.trim().is_empty() {
        return malformed(say!(words, "config-provider-no-name"));
    }

    if entry.name.contains(SEPARATOR) {
        return malformed(say!(
            words,
            "config-provider-name-has-separator",
            "provider" = entry.name.clone(),
            "separator" = SEPARATOR.to_string()
        ));
    }

    if taken.iter().any(|provider| provider.name == entry.name) {
        return malformed(say!(
            words,
            "config-two-providers-named",
            "provider" = entry.name.clone()
        ));
    }

    // Adding a row in Settings can leave an empty identifier if the user saves before filling it in.
    if entry.models.iter().any(|model| model.id.trim().is_empty()) {
        return malformed(say!(
            words,
            "config-provider-model-no-name",
            "provider" = entry.name.clone()
        ));
    }

    if let Some(duplicate) = duplicate_model(entry) {
        return malformed(say!(
            words,
            "config-provider-model-twice",
            "provider" = entry.name.clone(),
            "model" = duplicate.to_owned()
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

/// Treat an empty environment variable as missing so it does not override a usable key.
/// A key for a normally keyless preset is still useful when a proxy requires it.
pub(crate) fn resolve_key(
    entry: &ProviderEntry,
    env: &Environment,
    path: &Path,
    words: &Words,
) -> Key {
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
        // An explicit variable requires authentication even for a keyless preset. GUI launches may lack shell-profile exports.
        None if has_no_keys(entry) && entry.api_key_env.is_none() => Key::NotNeeded,
        None => Key::Missing(no_key(entry, path, words)),
    }
}

fn conventional(entry: &ProviderEntry) -> Option<&'static str> {
    match entry.preset?.spec().auth {
        Auth::Variable(name) => Some(name),
        Auth::Nothing => None,
    }
}

/// An omitted preset must never silently turn authentication off for a hand-written Provider.
fn has_no_keys(entry: &ProviderEntry) -> bool {
    entry
        .preset
        .is_some_and(|preset| matches!(preset.spec().auth, Auth::Nothing))
}

/// An absent Hotkey uses the default while an empty string disables it.
pub(crate) fn stated_hotkey(value: Option<String>) -> Option<String> {
    value.map(|value| value.trim().to_owned())
}

/// Keys pasted from web pages or read from files can carry a trailing newline.
pub(crate) fn stated(value: Option<String>) -> Option<String> {
    let value = value?;
    let trimmed = value.trim();

    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

fn no_key(entry: &ProviderEntry, path: &Path, words: &Words) -> String {
    // A Provider can name its preset's own variable in api_key_env.
    let variables: BTreeSet<&str> = entry
        .api_key_env
        .as_deref()
        .into_iter()
        .chain(conventional(entry))
        .collect();

    match variables.is_empty() {
        true => say!(
            words,
            "config-no-key-anywhere",
            "provider" = entry.name.clone(),
            "path" = path.display().to_string()
        ),
        false => say!(
            words,
            "config-no-key-export",
            "provider" = entry.name.clone(),
            "variables" = variables.into_iter().collect::<Vec<_>>().join(" or "),
            "path" = path.display().to_string()
        ),
    }
}

fn read_or_create(path: &Path, words: &Words) -> Result<String, ConfigError> {
    match fs::read_to_string(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            create(path, words)?;

            // Read the file the user will edit, including a file created by a concurrent caller.
            fs::read_to_string(path).map_err(|error| unreadable(path, &error, words))
        }
        read => read.map_err(|error| unreadable(path, &error, words)),
    }
}

fn create(path: &Path, words: &Words) -> Result<(), ConfigError> {
    use std::io::Write;

    if let Some(parent) = path.parent() {
        files::create_dir(parent).map_err(|error| unreadable(parent, &error, words))?;
    }

    let mut file = match files::options().create_new(true).write(true).open(path) {
        Ok(file) => file,
        // A concurrent caller may already have created a usable settings file.
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => return Ok(()),
        Err(error) => return Err(unreadable(path, &error, words)),
    };

    file.write_all(template(words).as_bytes())
        .map_err(|error| unreadable(path, &error, words))
}

/// An atomic replacement keeps the previous settings usable if saving fails.
pub(crate) fn write(path: &Path, text: &str, words: &Words) -> Result<(), ConfigError> {
    files::replace(path, text).map_err(|error| unwritable(path, &error, words))
}

fn unreadable(path: &Path, error: &io::Error, words: &Words) -> ConfigError {
    ConfigError::Unreadable(say!(
        words,
        "config-unreadable",
        "path" = path.display().to_string(),
        "detail" = error.to_string()
    ))
}

fn unwritable(path: &Path, error: &io::Error, words: &Words) -> ConfigError {
    ConfigError::Unwritable(say!(
        words,
        "config-unwritable",
        "path" = path.display().to_string(),
        "detail" = error.to_string()
    ))
}

/// TOML errors can quote the offending line, so pass only the reason and location to the webview.
fn unparseable(path: &Path, text: &str, error: &toml::de::Error, words: &Words) -> ConfigError {
    let line = error
        .span()
        .and_then(|span| text.get(..span.start))
        .map(|before| before.matches('\n').count() + 1);

    ConfigError::Malformed(match line {
        Some(line) => say!(
            words,
            "config-not-toml-at-line",
            "path" = path.display().to_string(),
            "line" = line,
            "detail" = error.message().to_owned()
        ),
        None => say!(
            words,
            "config-not-toml",
            "path" = path.display().to_string(),
            "detail" = error.message().to_owned()
        ),
    })
}

#[cfg(test)]
mod tests {

    use tempfile::TempDir;

    use super::*;

    fn load_with(body: &str, env: &Environment) -> (TempDir, Result<Config, ConfigError>) {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join(FILE_NAME), format!("version = 1\n\n{body}")).unwrap();

        let loaded = load(dir.path(), env, &crate::i18n::english());
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

    fn key(body: &str, env: &Environment) -> Key {
        config(body, env).providers.remove(0).key
    }

    fn found(body: &str, env: &Environment) -> String {
        match key(body, env) {
            Key::Found { key, .. } => key,
            other => panic!("the Provider should have found a key: {:?}", Named(&other)),
        }
    }

    fn no_key_message(body: &str, env: &Environment) -> String {
        match key(body, env) {
            Key::Missing(message) => message,
            other => panic!("the Provider should have wanted a key: {:?}", Named(&other)),
        }
    }

    /// Keep the production Key type out of debug output even when assertions need a description.
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

    /// Supply all three key sources so tests can isolate their precedence.
    const EVERY_SOURCE: &str = "\
[[providers]]
name = \"deepseek\"
preset = \"deepseek\"
base_url = \"https://api.deepseek.com/v1\"
api_key = \"from-the-file\"
api_key_env = \"MY_OWN_KEY\"
models = [{ id = \"deepseek-chat\" }]
";

    fn without_its_own_variable() -> String {
        EVERY_SOURCE.replace("api_key_env = \"MY_OWN_KEY\"\n", "")
    }

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
        // A keyless server may sit behind an authenticated proxy.
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
        // A missing shell variable must not turn off authentication for a Provider with an overridden base URL.
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
        // A typo in a manually configured Provider must not disable authentication.
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
    fn openrouter_reasoning_is_recognized_from_the_official_url_without_a_preset() {
        for (url, expected) in [
            ("https://openrouter.ai/api/v1", SkipReasoning::OpenRouter),
            (" https://openrouter.ai/api/v1/ ", SkipReasoning::OpenRouter),
            (
                "https://openrouter.ai.example.org/api/v1",
                SkipReasoning::Unmentioned,
            ),
            ("https://openrouter.ai/api/v2", SkipReasoning::Unmentioned),
        ] {
            let body = format!("[[providers]]\nname = \"custom\"\nbase_url = \"{url}\"\n");
            let loaded = config(&body, &Environment::default());

            assert_eq!(loaded.providers[0].skip_reasoning, expected, "{url}");
        }
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
        // A separator in a Provider name would make the Model reference ambiguous.
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

        let _ = load(dir.path(), &Environment::default(), &crate::i18n::english());

        assert!(dir.path().join(FILE_NAME).is_file());
    }

    #[test]
    fn the_directory_is_created_too() {
        let dir = TempDir::new().unwrap();
        let nested = dir.path().join("never/been/here");

        let _ = load(&nested, &Environment::default(), &crate::i18n::english());

        assert!(nested.join(FILE_NAME).is_file());
    }

    #[cfg(unix)]
    #[test]
    fn a_created_settings_file_is_readable_by_nobody_else() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDir::new().unwrap();

        let _ = load(dir.path(), &Environment::default(), &crate::i18n::english());

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
            load(dir.path(), &Environment::default(), &crate::i18n::english()),
            Err(ConfigError::NoProvider(_))
        ));
    }

    #[test]
    fn the_example_the_template_offers_is_one_that_would_load() {
        let config = config(EXAMPLE, &Environment::default());

        assert_eq!(
            config.default_model.as_deref(),
            Some("deepseek/deepseek-chat")
        );
        assert!(config
            .model("openai/gpt-4o")
            .is_some_and(|(_, model)| model.vision));

        let (local, _) = config
            .model("local/qwen/qwen3-8b")
            .expect("the example should offer a local Model");

        assert!(matches!(local.key, Key::NotNeeded));
    }

    #[test]
    fn every_preset_the_template_names_is_one_the_file_accepts_and_has_an_address_for() {
        // Check that names advertised in the template are accepted by the settings parser.
        let written = template(&crate::i18n::english());

        for preset in Preset::ALL {
            let spec = preset.spec();
            assert!(written.contains(spec.name), "{written}");

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
        let written = template(&crate::i18n::english());

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
        // A quoted source line could expose the key to the webview.
        let unquoted = file_only().replace("\"from-the-file\"", "from-the-file");
        let ConfigError::Malformed(message) = error(&unquoted, &Environment::default()) else {
            panic!("an unquoted value should be reported as malformed");
        };

        assert!(!message.contains("from-the-file"), "{message}");
        assert!(message.contains("line 6"), "{message}");
    }

    #[test]
    fn a_misspelled_field_is_reported_rather_than_ignored() {
        // A typo in api_key must be reported because it would leave the Provider without credentials.
        let misspelled = file_only().replace("api_key =", "api_kye =");

        assert!(matches!(
            error(&misspelled, &Environment::default()),
            ConfigError::Malformed(_)
        ));
    }

    #[test]
    fn a_settings_file_that_states_a_palette_hotkey_still_loads() {
        // A Hotkey must be declared because the file schema rejects unknown fields.
        let stating = format!("palette_hotkey = \"F13\"\n{}", file_only());

        config(&stating, &Environment::default());
    }

    #[test]
    fn a_misspelled_palette_hotkey_is_reported_rather_than_ignored() {
        let misspelled = format!("palette_hotkeys = \"F13\"\n{}", file_only());

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

        let Err(ConfigError::Malformed(message)) =
            load(dir.path(), &Environment::default(), &crate::i18n::english())
        else {
            panic!("a file from the future should not be acted on");
        };

        assert!(message.contains("99"), "{message}");
    }

    #[test]
    fn a_file_that_states_no_version_is_read_as_the_first() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join(FILE_NAME), file_only()).unwrap();

        assert!(load(dir.path(), &Environment::default(), &crate::i18n::english()).is_ok());
    }
}
