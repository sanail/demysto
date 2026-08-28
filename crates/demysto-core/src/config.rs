//! The settings file: what Demysto is configured with, and where the key for a
//! Provider comes from.
//!
//! Read once, at startup, and nothing else in the crate reads the environment —
//! per the spec's *Core modules*. Ticket 07 turns the single Provider below into
//! several, with Models and a resolution chain; ticket 08 gives the file a
//! window to be edited from.

use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use serde::Deserialize;

/// The file Demysto reads, inside the configuration directory.
pub(crate) const FILE_NAME: &str = "settings.toml";

/// The shape of the file this build understands. A file claiming a higher one
/// was written by a newer Demysto, and guessing at what it means would be a
/// good way to send somebody's key to the wrong place.
const VERSION: u32 = 1;

/// What a fresh installation gets: a file that parses, says what goes in it,
/// and configures nothing until the user uncomments a Provider.
const TEMPLATE: &str = "\
# Demysto's settings.
#
# Read once, when Demysto starts, so restart it after an edit.

version = 1

# Uncomment one of these and fill it in. `preset` names the service so that
# Demysto knows which environment variable it conventionally uses; `base_url`
# and `model` are yours to state, whatever the service.
#
# The key is looked for in the variable `api_key_env` names, then in the
# preset's own variable, then in `api_key` here. Leaving `api_key` out and
# exporting the variable instead keeps the secret out of this file.
#
# [[providers]]
# name = \"deepseek\"
# preset = \"deepseek\"
# base_url = \"https://api.deepseek.com/v1\"
# model = \"deepseek-chat\"
# api_key = \"sk-...\"
";

/// The configuration Demysto is running on, with the key already resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Config {
    pub(crate) provider: Provider,
}

/// A configured LLM endpoint, ready to be asked something.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Provider {
    pub(crate) base_url: String,
    pub(crate) model: String,
    pub(crate) api_key: String,
}

impl fmt::Debug for Provider {
    /// Written out rather than derived, so that the key cannot arrive somewhere
    /// nobody meant to send it through a panic message or, once ticket 11 has
    /// them, a log. ADR-0002 leaves it readable on disk by its owner; that is
    /// the whole of what it leaves readable.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Provider")
            .field("base_url", &self.base_url)
            .field("model", &self.model)
            .field("api_key", &"<not shown>")
            .finish()
    }
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
    /// A Provider is configured and no key for it could be found.
    NoKey(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Unreadable(message)
            | Self::Malformed(message)
            | Self::NoProvider(message)
            | Self::NoKey(message) => message,
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
/// the Provider it configures.
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

    // The first, rather than a named one: ticket 07 is where several Providers
    // coexist and an Action resolves to one of them.
    let entry = file.providers.into_iter().next().ok_or_else(|| {
        ConfigError::NoProvider(format!(
            "no Provider is configured; open {} and fill in the example it holds",
            path.display()
        ))
    })?;

    let api_key =
        resolve_key(&entry, env).ok_or_else(|| ConfigError::NoKey(no_key(&entry, &path)))?;

    Ok(Config {
        provider: Provider {
            base_url: entry.base_url,
            model: entry.model,
            api_key,
        },
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
}

fn first_version() -> u32 {
    VERSION
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProviderEntry {
    /// What the user calls this Provider. Unused until ticket 07 has more than
    /// one to tell apart, but a Provider nobody can name is not a Provider.
    #[allow(dead_code)]
    name: String,
    base_url: String,
    model: String,
    preset: Option<Preset>,
    api_key: Option<String>,
    api_key_env: Option<String>,
}

/// A service Demysto knows the conventions of.
///
/// The preset says which environment variable the service's own documentation
/// tells people to export — nothing more in this ticket. Ticket 07 gives it the
/// base URL as well, so that setup becomes picking a name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Preset {
    Deepseek,
    Openai,
    Openrouter,
}

impl Preset {
    fn conventional_key_env(self) -> &'static str {
        match self {
            Self::Deepseek => "DEEPSEEK_API_KEY",
            Self::Openai => "OPENAI_API_KEY",
            Self::Openrouter => "OPENROUTER_API_KEY",
        }
    }
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
                .and_then(|preset| from_env(preset.conventional_key_env()))
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
        .chain(entry.preset.map(Preset::conventional_key_env))
        .collect();

    match variables.is_empty() {
        true => format!(
            "no API key is configured; set api_key in {}, or name an environment variable in api_key_env",
            path.display()
        ),
        false => format!(
            "no API key is configured; export {}, or set api_key in {}",
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

    file.write_all(TEMPLATE.as_bytes())
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

    /// A settings file with the given Provider block, and the config it loads to.
    fn load_with(providers: &str, env: &FakeEnv) -> (TempDir, Result<Config, ConfigError>) {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join(FILE_NAME),
            format!("version = 1\n\n{providers}"),
        )
        .unwrap();

        let loaded = load(dir.path(), env);
        (dir, loaded)
    }

    fn config(providers: &str, env: &FakeEnv) -> Config {
        let (_dir, loaded) = load_with(providers, env);
        loaded.expect("the settings should have loaded")
    }

    fn error(providers: &str, env: &FakeEnv) -> ConfigError {
        let (_dir, loaded) = load_with(providers, env);
        loaded.expect_err("the settings should not have loaded")
    }

    /// A Provider naming its own variable, carrying a preset, and holding a key
    /// in the file — all three sources at once, so that a test can take one away.
    const EVERY_SOURCE: &str = "\
[[providers]]
name = \"deepseek\"
preset = \"deepseek\"
base_url = \"https://api.deepseek.com/v1\"
model = \"deepseek-chat\"
api_key = \"from-the-file\"
api_key_env = \"MY_OWN_KEY\"
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
            config(EVERY_SOURCE, &env).provider.api_key,
            "from-my-own-variable"
        );
    }

    #[test]
    fn the_key_comes_from_the_presets_conventional_variable() {
        let env = FakeEnv::holding(&[("DEEPSEEK_API_KEY", "from-the-preset")]);

        assert_eq!(
            config(&without_its_own_variable(), &env).provider.api_key,
            "from-the-preset"
        );
    }

    #[test]
    fn the_key_comes_from_the_file_when_the_environment_holds_none() {
        assert_eq!(
            config(&file_only(), &FakeEnv::default()).provider.api_key,
            "from-the-file"
        );
    }

    #[test]
    fn the_variable_the_provider_names_wins_over_the_presets() {
        let env = FakeEnv::holding(&[
            ("MY_OWN_KEY", "from-my-own-variable"),
            ("DEEPSEEK_API_KEY", "from-the-preset"),
        ]);

        assert_eq!(
            config(EVERY_SOURCE, &env).provider.api_key,
            "from-my-own-variable"
        );
    }

    #[test]
    fn the_presets_variable_wins_over_the_file() {
        let env = FakeEnv::holding(&[("DEEPSEEK_API_KEY", "from-the-preset")]);

        assert_eq!(
            config(EVERY_SOURCE, &env).provider.api_key,
            "from-the-preset"
        );
    }

    #[test]
    fn a_variable_that_is_set_but_empty_is_not_a_key() {
        // Exported and left empty is a common state of a shell profile, and
        // reading it as a key would turn a working configuration into a 401.
        let env = FakeEnv::holding(&[("MY_OWN_KEY", ""), ("DEEPSEEK_API_KEY", "   ")]);

        assert_eq!(config(EVERY_SOURCE, &env).provider.api_key, "from-the-file");
    }

    #[test]
    fn a_key_arrives_without_the_whitespace_around_it() {
        let env = FakeEnv::holding(&[("MY_OWN_KEY", "  from-my-own-variable\n")]);

        assert_eq!(
            config(EVERY_SOURCE, &env).provider.api_key,
            "from-my-own-variable"
        );
    }

    #[test]
    fn no_key_anywhere_names_every_variable_that_was_looked_at() {
        let ConfigError::NoKey(message) = error(
            &EVERY_SOURCE.replace("api_key = \"from-the-file\"\n", ""),
            &FakeEnv::default(),
        ) else {
            panic!("a Provider with no key should fail for want of one");
        };

        assert!(message.contains("MY_OWN_KEY"), "{message}");
        assert!(message.contains("DEEPSEEK_API_KEY"), "{message}");
        assert!(message.contains("api_key"), "{message}");
    }

    #[test]
    fn a_provider_with_no_variables_to_name_still_says_where_a_key_goes() {
        let missing = file_only().replace("api_key = \"from-the-file\"\n", "");
        let ConfigError::NoKey(message) = error(&missing, &FakeEnv::default()) else {
            panic!("a Provider with no key should fail for want of one");
        };

        assert!(message.contains(FILE_NAME), "{message}");
        assert!(message.contains("api_key_env"), "{message}");
    }

    #[test]
    fn the_provider_is_read_from_the_file() {
        let provider = config(&file_only(), &FakeEnv::default()).provider;

        assert_eq!(provider.base_url, "https://api.deepseek.com/v1");
        assert_eq!(provider.model, "deepseek-chat");
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
        assert!(message.contains("line 7"), "{message}");
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
