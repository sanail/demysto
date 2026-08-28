//! The settings file: what Demysto is configured with, and where the key for a
//! Provider comes from.
//!
//! Read once, at startup, and nothing else in the crate reads the environment —
//! per the spec's *Core modules*. Which of the Models configured here a given
//! Run uses is `model`'s; ticket 08 gives the file a window to be edited from.

use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// The file Demysto reads, inside the configuration directory.
pub(crate) const FILE_NAME: &str = "settings.toml";

/// The shape of the file this build understands. A file claiming a higher one
/// was written by a newer Demysto, and guessing at what it means would be a
/// good way to send somebody's key to the wrong place.
const VERSION: u32 = 1;

/// What separates the Provider from the Model in the name a Model is nominated
/// or bound by.
///
/// A Provider's own name may not hold one, so the first is always the divide —
/// which matters because a Model's identifier routinely holds several
/// (`anthropic/claude-sonnet-4.5` is one Model at one Provider).
const SEPARATOR: char = '/';

/// Where the preamble names every preset there is, filled in from the presets
/// themselves so that adding one cannot leave the file describing the old set.
const PRESETS: &str = "{presets}";

/// The prose a fresh installation is met by: what the file is, and what each
/// field in the example under it means.
const PREAMBLE: &str = r#"# Demysto's settings.
#
# Read once, when Demysto starts, so restart it after an edit.
#
# Uncomment the example below and fill it in.
#
# `preset` names a service Demysto knows the conventions of: it fills in
# `base_url`, and it says which environment variable that service's own
# documentation tells people to export. The presets are {presets}. State
# `base_url` yourself for a service that has none of its own, or to override
# what a preset fills in.
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

    let named = Preset::ALL.map(|preset| preset.spec().name);
    let preamble = PREAMBLE.replace(PRESETS, &named.join(", "));

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
    /// The key, or the sentence telling the user where to put one.
    ///
    /// Composed at load, while the file's path and the Provider's own fields
    /// are still at hand, and shown only when a Run resolves to this Provider:
    /// one Provider missing its key is no reason for another Provider's Models
    /// to stop working.
    pub(crate) api_key: Result<String, String>,
    pub(crate) models: Vec<Model>,
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
                "api_key",
                match &self.api_key {
                    Ok(_) => &"<not shown>",
                    Err(_) => &"<none>",
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

    /// The Provider called `name`. `None` when the file configures none.
    pub(crate) fn provider(&self, name: &str) -> Option<&Provider> {
        self.providers.iter().find(|provider| provider.name == name)
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ConfigError {
    /// The file could not be read, or could not be created.
    Unreadable(String),
    /// The file was read but is not something Demysto can act on.
    Malformed(String),
    /// The file is valid and configures no Provider.
    NoProvider(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Unreadable(message) | Self::Malformed(message) | Self::NoProvider(message) => {
                message
            }
        })
    }
}

impl std::error::Error for ConfigError {}

/// The environment, behind a trait so that key resolution can be tested without
/// mutating the environment of the whole test binary — the same reason
/// [`crate::paths::config_dir`] takes its inputs rather than reading them.
pub(crate) trait Env {
    fn get(&self, name: &str) -> Option<String>;
}

/// The environment of the running process.
pub(crate) struct SystemEnv;

impl Env for SystemEnv {
    fn get(&self, name: &str) -> Option<String> {
        std::env::var(name).ok()
    }
}

/// Reads the settings file, creating it when it is not there yet, and resolves
/// the Providers it configures.
pub(crate) fn load(config_dir: &Path, env: &dyn Env) -> Result<Config, ConfigError> {
    let path = config_dir.join(FILE_NAME);
    let text = read_or_create(&path)?;
    let file: File = toml::from_str(&text).map_err(|error| unparseable(&path, &text, &error))?;

    if file.version > VERSION {
        return Err(ConfigError::Malformed(format!(
            "{} says it is version {}, and this Demysto understands version {VERSION}; \
             update Demysto, or point {} at another directory",
            path.display(),
            file.version,
            crate::paths::CONFIG_DIR_ENV,
        )));
    }

    if file.providers.is_empty() {
        return Err(ConfigError::NoProvider(format!(
            "no Provider is configured; open {} and fill in the example it holds",
            path.display()
        )));
    }

    let mut providers: Vec<Provider> = Vec::with_capacity(file.providers.len());

    for entry in &file.providers {
        nameable(entry, &providers, &path)?;

        providers.push(Provider {
            name: entry.name.clone(),
            base_url: base_url(entry, &path)?,
            api_key: resolve_key(entry, env).ok_or_else(|| no_key(entry, &path)),
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
        path,
        providers,
        default_model: file.default_model,
        default_vision_model: file.default_vision_model,
    })
}

/// The settings file as it is written, before anything is resolved.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    /// Absent in a file written by hand, which is the same as the first version.
    #[serde(default = "first_version")]
    version: u32,
    #[serde(default)]
    providers: Vec<ProviderEntry>,
    default_model: Option<String>,
    default_vision_model: Option<String>,
}

fn first_version() -> u32 {
    VERSION
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProviderEntry {
    /// What the user calls this Provider, and what the first half of a Model's
    /// name refers to.
    name: String,
    /// Absent when the preset supplies it.
    base_url: Option<String>,
    preset: Option<Preset>,
    api_key: Option<String>,
    api_key_env: Option<String>,
    /// The Models of this Provider the user wants to use — not everything it
    /// offers, which is what the Model list is fetched for.
    #[serde(default)]
    models: Vec<ModelEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ModelEntry {
    id: String,
    #[serde(default)]
    vision: bool,
}

/// A service Demysto knows the conventions of.
///
/// The three ADR-0002 fixes the key order for, and no more: a preset is a
/// decision about where somebody's key goes, and inventing one here would be
/// inventing a decision nothing recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Preset {
    Deepseek,
    Openai,
    Openrouter,
}

/// What Demysto knows about a service: the word the file names it by, where it
/// answers, and the environment variable its own documentation tells people to
/// export.
struct Spec {
    name: &'static str,
    base_url: &'static str,
    key_env: &'static str,
}

impl Preset {
    /// Every preset there is, so that the template can name them: a preset
    /// nobody has heard of is a base URL somebody looks up anyway.
    const ALL: [Self; 3] = [Self::Deepseek, Self::Openai, Self::Openrouter];

    /// Everything known about one service, in one place: a preset added here is
    /// a preset added once, and the match keeps the compiler asking.
    fn spec(self) -> Spec {
        match self {
            Self::Deepseek => Spec {
                name: "deepseek",
                base_url: "https://api.deepseek.com/v1",
                key_env: "DEEPSEEK_API_KEY",
            },
            Self::Openai => Spec {
                name: "openai",
                base_url: "https://api.openai.com/v1",
                key_env: "OPENAI_API_KEY",
            },
            Self::Openrouter => Spec {
                name: "openrouter",
                base_url: "https://openrouter.ai/api/v1",
                key_env: "OPENROUTER_API_KEY",
            },
        }
    }
}

/// What a Provider is reached at: what it states, else what its preset knows.
///
/// A stated base URL wins over the preset's, so that a proxy or a regional
/// endpoint does not cost the user the preset's other half.
fn base_url(entry: &ProviderEntry, path: &Path) -> Result<String, ConfigError> {
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
fn resolve_key(entry: &ProviderEntry, env: &dyn Env) -> Option<String> {
    let from_env = |name: &str| stated(env.get(name));

    entry
        .api_key_env
        .as_deref()
        .and_then(from_env)
        .or_else(|| {
            entry
                .preset
                .and_then(|preset| from_env(preset.spec().key_env))
        })
        .or_else(|| stated(entry.api_key.clone()))
}

/// A value somebody actually stated: trimmed, and `None` when there was nothing
/// there but whitespace.
///
/// A key pasted out of a web page or read from a file arrives with a newline on
/// it often enough that trimming is the kinder default.
fn stated(value: Option<String>) -> Option<String> {
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
        .chain(entry.preset.map(|preset| preset.spec().key_env))
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
        create_dir(parent)?;
    }

    let mut file = match options().create_new(true).write(true).open(path) {
        Ok(file) => file,
        // Somebody else got there between the read and this line. Their file is
        // as good as ours, and better than an error the user cannot act on.
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => return Ok(()),
        Err(error) => return Err(unreadable(path, &error)),
    };

    file.write_all(template().as_bytes())
        .map_err(|error| unreadable(path, &error))
}

/// The file carries a key, so it is created readable by nobody else — the whole
/// of what ADR-0002 asks in exchange for keeping the key out of the keychain.
#[cfg(unix)]
fn options() -> fs::OpenOptions {
    use std::os::unix::fs::OpenOptionsExt;

    let mut options = fs::OpenOptions::new();
    options.mode(0o600);
    options
}

#[cfg(not(unix))]
fn options() -> fs::OpenOptions {
    fs::OpenOptions::new()
}

/// The directory the file goes in, owner-only for the same reason.
#[cfg(unix)]
fn create_dir(path: &Path) -> Result<(), ConfigError> {
    use std::os::unix::fs::DirBuilderExt;

    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path)
        .map_err(|error| unreadable(path, &error))
}

#[cfg(not(unix))]
fn create_dir(path: &Path) -> Result<(), ConfigError> {
    fs::create_dir_all(path).map_err(|error| unreadable(path, &error))
}

fn unreadable(path: &Path, error: &io::Error) -> ConfigError {
    ConfigError::Unreadable(format!("{} could not be read: {error}", path.display()))
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

    use std::collections::HashMap;

    use tempfile::TempDir;

    use super::*;

    /// An environment holding exactly what a test put in it.
    #[derive(Default)]
    struct FakeEnv(HashMap<String, String>);

    impl FakeEnv {
        fn holding(variables: &[(&str, &str)]) -> Self {
            Self(
                variables
                    .iter()
                    .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
                    .collect(),
            )
        }
    }

    impl Env for FakeEnv {
        fn get(&self, name: &str) -> Option<String> {
            self.0.get(name).cloned()
        }
    }

    /// A settings file holding `body` under the version line, and the config it
    /// loads to.
    fn load_with(body: &str, env: &FakeEnv) -> (TempDir, Result<Config, ConfigError>) {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join(FILE_NAME), format!("version = 1\n\n{body}")).unwrap();

        let loaded = load(dir.path(), env);
        (dir, loaded)
    }

    fn config(body: &str, env: &FakeEnv) -> Config {
        let (_dir, loaded) = load_with(body, env);
        loaded.expect("the settings should have loaded")
    }

    fn error(body: &str, env: &FakeEnv) -> ConfigError {
        let (_dir, loaded) = load_with(body, env);
        loaded.expect_err("the settings should not have loaded")
    }

    /// The key the first Provider resolved to, for the tests about where a key
    /// comes from.
    fn key(body: &str, env: &FakeEnv) -> Result<String, String> {
        config(body, env).providers.remove(0).api_key
    }

    /// The sentence a Provider with no key carries.
    fn no_key_message(body: &str, env: &FakeEnv) -> String {
        key(body, env).expect_err("the Provider should have found no key")
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
        let env = FakeEnv::holding(&[("MY_OWN_KEY", "from-my-own-variable")]);

        assert_eq!(
            key(EVERY_SOURCE, &env).as_deref(),
            Ok("from-my-own-variable")
        );
    }

    #[test]
    fn the_key_comes_from_the_presets_conventional_variable() {
        let env = FakeEnv::holding(&[("DEEPSEEK_API_KEY", "from-the-preset")]);

        assert_eq!(
            key(&without_its_own_variable(), &env).as_deref(),
            Ok("from-the-preset")
        );
    }

    #[test]
    fn the_key_comes_from_the_file_when_the_environment_holds_none() {
        assert_eq!(
            key(&file_only(), &FakeEnv::default()).as_deref(),
            Ok("from-the-file")
        );
    }

    #[test]
    fn the_variable_the_provider_names_wins_over_the_presets() {
        let env = FakeEnv::holding(&[
            ("MY_OWN_KEY", "from-my-own-variable"),
            ("DEEPSEEK_API_KEY", "from-the-preset"),
        ]);

        assert_eq!(
            key(EVERY_SOURCE, &env).as_deref(),
            Ok("from-my-own-variable")
        );
    }

    #[test]
    fn the_presets_variable_wins_over_the_file() {
        let env = FakeEnv::holding(&[("DEEPSEEK_API_KEY", "from-the-preset")]);

        assert_eq!(key(EVERY_SOURCE, &env).as_deref(), Ok("from-the-preset"));
    }

    #[test]
    fn a_variable_that_is_set_but_empty_is_not_a_key() {
        // Exported and left empty is a common state of a shell profile, and
        // reading it as a key would turn a working configuration into a 401.
        let env = FakeEnv::holding(&[("MY_OWN_KEY", ""), ("DEEPSEEK_API_KEY", "   ")]);

        assert_eq!(key(EVERY_SOURCE, &env).as_deref(), Ok("from-the-file"));
    }

    #[test]
    fn a_key_arrives_without_the_whitespace_around_it() {
        let env = FakeEnv::holding(&[("MY_OWN_KEY", "  from-my-own-variable\n")]);

        assert_eq!(
            key(EVERY_SOURCE, &env).as_deref(),
            Ok("from-my-own-variable")
        );
    }

    #[test]
    fn no_key_anywhere_names_every_variable_that_was_looked_at() {
        let missing = EVERY_SOURCE.replace("api_key = \"from-the-file\"\n", "");
        let message = no_key_message(&missing, &FakeEnv::default());

        assert!(message.contains("MY_OWN_KEY"), "{message}");
        assert!(message.contains("DEEPSEEK_API_KEY"), "{message}");
        assert!(message.contains("api_key"), "{message}");
    }

    #[test]
    fn a_provider_with_no_variables_to_name_still_says_where_a_key_goes() {
        let missing = file_only().replace("api_key = \"from-the-file\"\n", "");
        let message = no_key_message(&missing, &FakeEnv::default());

        assert!(message.contains(FILE_NAME), "{message}");
        assert!(message.contains("api_key_env"), "{message}");
    }

    #[test]
    fn a_provider_with_no_key_says_which_provider_it_is() {
        // Several Providers may be configured, and only one of them is broken.
        let missing = file_only().replace("api_key = \"from-the-file\"\n", "");
        let message = no_key_message(&missing, &FakeEnv::default());

        assert!(message.contains("deepseek"), "{message}");
    }

    #[test]
    fn a_provider_missing_its_key_leaves_the_others_configured() {
        let both = format!(
            "{}\n[[providers]]\nname = \"openai\"\npreset = \"openai\"\n\
             models = [{{ id = \"gpt-4o-mini\" }}]\n",
            file_only().replace("api_key = \"from-the-file\"\n", "")
        );

        let config = config(&both, &FakeEnv::holding(&[("OPENAI_API_KEY", "a-key")]));

        assert!(config.providers[0].api_key.is_err());
        assert_eq!(config.providers[1].api_key.as_deref(), Ok("a-key"));
    }

    #[test]
    fn the_provider_is_read_from_the_file() {
        let provider = config(&file_only(), &FakeEnv::default())
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

        let names: Vec<String> = config(&both, &FakeEnv::default())
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
            config(by_preset, &FakeEnv::default()).providers[0].base_url,
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
            config(&proxied, &FakeEnv::default()).providers[0].base_url,
            "https://proxy.internal/v1"
        );
    }

    #[test]
    fn a_provider_with_no_address_at_all_is_reported() {
        let nowhere = "[[providers]]\nname = \"mine\"\napi_key = \"a-key\"\n";
        let ConfigError::Malformed(message) = error(nowhere, &FakeEnv::default()) else {
            panic!("a Provider with no address should be reported as malformed");
        };

        assert!(message.contains("base_url"), "{message}");
        assert!(message.contains("preset"), "{message}");
    }

    #[test]
    fn a_model_does_not_accept_images_unless_it_says_so() {
        // The whole point of the flag: a name is not a capability.
        let named_like_one = EVERY_SOURCE.replace("deepseek-chat", "gpt-4o-vision-preview");

        assert!(!config(&named_like_one, &FakeEnv::default()).providers[0].models[0].vision);
    }

    #[test]
    fn a_model_marked_vision_capable_carries_it() {
        let seeing = EVERY_SOURCE.replace(
            "{ id = \"deepseek-chat\" }",
            "{ id = \"deepseek-chat\", vision = true }",
        );

        assert!(config(&seeing, &FakeEnv::default()).providers[0].models[0].vision);
    }

    #[test]
    fn a_model_is_found_by_the_provider_that_offers_it_and_its_own_name() {
        let config = config(EVERY_SOURCE, &FakeEnv::default());

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
        let config = config(routed, &FakeEnv::default());

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
            error(&slashed, &FakeEnv::default()),
            ConfigError::Malformed(_)
        ));
    }

    #[test]
    fn two_providers_of_the_same_name_are_reported() {
        let twice = format!("{EVERY_SOURCE}\n{EVERY_SOURCE}");
        let ConfigError::Malformed(message) = error(&twice, &FakeEnv::default()) else {
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
            error(&twice, &FakeEnv::default()),
            ConfigError::Malformed(_)
        ));
    }

    #[test]
    fn the_two_defaults_are_read_from_the_file() {
        let nominated = format!(
            "default_model = \"deepseek/deepseek-chat\"\n\
             default_vision_model = \"openai/gpt-4o\"\n\n{EVERY_SOURCE}"
        );
        let config = config(&nominated, &FakeEnv::default());

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

        let _ = load(dir.path(), &FakeEnv::default());

        assert!(dir.path().join(FILE_NAME).is_file());
    }

    #[test]
    fn the_directory_is_created_too() {
        let dir = TempDir::new().unwrap();
        let nested = dir.path().join("never/been/here");

        let _ = load(&nested, &FakeEnv::default());

        assert!(nested.join(FILE_NAME).is_file());
    }

    #[cfg(unix)]
    #[test]
    fn a_created_settings_file_is_readable_by_nobody_else() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDir::new().unwrap();

        let _ = load(dir.path(), &FakeEnv::default());

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
            load(dir.path(), &FakeEnv::default()),
            Err(ConfigError::NoProvider(_))
        ));
    }

    #[test]
    fn the_example_the_template_offers_is_one_that_would_load() {
        // Uncommenting it is the whole of what a new user is asked to do.
        let config = config(EXAMPLE, &FakeEnv::default());

        assert_eq!(
            config.default_model.as_deref(),
            Some("deepseek/deepseek-chat")
        );
        assert!(config
            .model("openai/gpt-4o")
            .is_some_and(|(_, model)| model.vision));
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

            let by_preset = format!(
                "[[providers]]\nname = \"mine\"\npreset = \"{}\"\napi_key = \"a-key\"\n",
                spec.name
            );

            assert_eq!(
                config(&by_preset, &FakeEnv::default()).providers[0].base_url,
                spec.base_url
            );
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
        let (dir, _) = load_with(&file_only(), &FakeEnv::default());

        let written = fs::read_to_string(dir.path().join(FILE_NAME)).unwrap();

        assert!(written.contains("from-the-file"), "{written}");
    }

    #[test]
    fn a_file_that_is_not_valid_toml_names_itself() {
        let ConfigError::Malformed(message) = error("[[providers]\nname = ", &FakeEnv::default())
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
        let ConfigError::Malformed(message) = error(&unquoted, &FakeEnv::default()) else {
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
            error(&misspelled, &FakeEnv::default()),
            ConfigError::Malformed(_)
        ));
    }

    #[test]
    fn a_misspelled_field_on_a_model_is_reported_too() {
        let misspelled = EVERY_SOURCE.replace("id = \"deepseek-chat\"", "idd = \"deepseek-chat\"");

        assert!(matches!(
            error(&misspelled, &FakeEnv::default()),
            ConfigError::Malformed(_)
        ));
    }

    #[test]
    fn a_file_from_a_newer_demysto_is_not_guessed_at() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join(FILE_NAME), "version = 99\n").unwrap();

        let Err(ConfigError::Malformed(message)) = load(dir.path(), &FakeEnv::default()) else {
            panic!("a file from the future should not be acted on");
        };

        assert!(message.contains("99"), "{message}");
    }

    #[test]
    fn a_file_that_states_no_version_is_read_as_the_first() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join(FILE_NAME), file_only()).unwrap();

        assert!(load(dir.path(), &FakeEnv::default()).is_ok());
    }
}
