//! Where Demysto keeps its configuration on each platform.

use std::ffi::OsStr;
use std::fmt;
use std::path::PathBuf;

/// The environment variable that overrides the platform configuration directory.
pub const CONFIG_DIR_ENV: &str = "DEMYSTO_CONFIG_DIR";

/// The directory name Demysto occupies inside the platform configuration root.
const DIR_NAME: &str = "demysto";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigDirError {
    /// The platform has no configuration directory and no override was given.
    NoPlatformConfigDir,
    /// The override was set but empty, which is more likely a mistake than an intent.
    EmptyOverride,
}

impl fmt::Display for ConfigDirError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoPlatformConfigDir => write!(
                f,
                "this platform reports no configuration directory; set {CONFIG_DIR_ENV} to choose one"
            ),
            Self::EmptyOverride => write!(f, "{CONFIG_DIR_ENV} is set but empty"),
        }
    }
}

impl std::error::Error for ConfigDirError {}

/// Resolves the configuration directory for the running platform.
pub fn config_dir() -> Result<PathBuf, ConfigDirError> {
    // `var_os` rather than `var`: a path that is not valid UTF-8 is still a path
    // the user chose, and falling back silently would put their keys somewhere
    // they never asked for.
    resolve(
        std::env::var_os(CONFIG_DIR_ENV).as_deref(),
        dirs::config_dir(),
    )
}

/// The resolution itself, with both inputs supplied, so that it can be tested
/// without mutating the environment of the whole test binary.
fn resolve(
    override_value: Option<&OsStr>,
    platform_root: Option<PathBuf>,
) -> Result<PathBuf, ConfigDirError> {
    match override_value {
        // An override names the directory itself, rather than a root to nest
        // inside: whoever sets it has already said where they want the files.
        Some(value) => match value.to_str() {
            Some(text) if text.trim().is_empty() => Err(ConfigDirError::EmptyOverride),
            Some(text) => Ok(PathBuf::from(text.trim())),
            // Not valid UTF-8, so it cannot be trimmed — take it verbatim.
            None => Ok(PathBuf::from(value)),
        },
        None => platform_root
            .map(|root| root.join(DIR_NAME))
            .ok_or(ConfigDirError::NoPlatformConfigDir),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolve_str(
        override_value: Option<&str>,
        platform_root: Option<&str>,
    ) -> Result<PathBuf, ConfigDirError> {
        resolve(
            override_value.map(OsStr::new),
            platform_root.map(PathBuf::from),
        )
    }

    #[test]
    fn nests_inside_the_platform_configuration_root() {
        let resolved = resolve_str(None, Some("/home/someone/.config")).unwrap();

        assert_eq!(resolved, PathBuf::from("/home/someone/.config/demysto"));
    }

    #[test]
    fn an_override_names_the_directory_itself() {
        let resolved = resolve_str(Some("/tmp/somewhere-else"), Some("/ignored")).unwrap();

        assert_eq!(resolved, PathBuf::from("/tmp/somewhere-else"));
    }

    #[test]
    fn an_override_wins_even_when_the_platform_has_no_root() {
        let resolved = resolve_str(Some("/tmp/somewhere-else"), None).unwrap();

        assert_eq!(resolved, PathBuf::from("/tmp/somewhere-else"));
    }

    #[test]
    fn surrounding_whitespace_in_an_override_is_not_part_of_the_path() {
        let resolved = resolve_str(Some("  /tmp/somewhere-else  "), None).unwrap();

        assert_eq!(resolved, PathBuf::from("/tmp/somewhere-else"));
    }

    #[test]
    fn an_empty_override_is_an_error_rather_than_a_silent_fallback() {
        assert_eq!(
            resolve_str(Some("   "), Some("/home/someone/.config")),
            Err(ConfigDirError::EmptyOverride)
        );
    }

    #[test]
    fn no_override_and_no_platform_root_is_an_error() {
        assert_eq!(
            resolve_str(None, None),
            Err(ConfigDirError::NoPlatformConfigDir)
        );
    }

    #[cfg(unix)]
    #[test]
    fn an_override_that_is_not_utf8_is_honoured_rather_than_ignored() {
        use std::os::unix::ffi::OsStrExt;

        let invalid = OsStr::from_bytes(b"/tmp/\xff\xfeodd");

        let resolved =
            resolve(Some(invalid), Some(PathBuf::from("/home/someone/.config"))).unwrap();

        assert_eq!(resolved, PathBuf::from(invalid));
    }
}
