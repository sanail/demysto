//! Resolve Models against a settings snapshot so this decision needs neither disk nor network access.

use std::fmt;

use crate::config::{
    self, Config, Key, Model, Provider, SkipReasoning, MODEL_SETTING, VISION_SETTING,
};
use crate::i18n::{say, Words};
use crate::run::RunError;
use crate::selection::Kind;

/// Own the Endpoint so a pending request does not hold Settings locked.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Endpoint {
    /// The name lets an authentication error point to the correct entry in Settings.
    pub(crate) provider: String,
    pub(crate) base_url: String,
    pub(crate) api_key: Option<String>,
    pub(crate) skip_reasoning: SkipReasoning,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Resolved {
    pub(crate) endpoint: Endpoint,
    pub(crate) model: String,
}

impl fmt::Debug for Endpoint {
    /// Keep the unwrapped key out of debug output, including logs and panic messages.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Endpoint")
            .field("provider", &self.provider)
            .field("base_url", &self.base_url)
            .field("api_key", &self.api_key.as_ref().map(|_| "<not shown>"))
            .field("skip_reasoning", &self.skip_reasoning)
            .finish()
    }
}

impl fmt::Debug for Resolved {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Resolved")
            .field("endpoint", &self.endpoint)
            .field("model", &self.model)
            .finish()
    }
}

pub(crate) fn resolve(
    config: &Config,
    binding: Option<&str>,
    kind: Kind,
    words: &Words,
) -> Result<Resolved, RunError> {
    let (provider, model) = chosen(config, binding, kind, words)?;

    Ok(Resolved {
        endpoint: endpoint_for(
            &provider.name,
            &provider.base_url,
            &provider.key,
            provider.skip_reasoning,
        )?,
        model: model.id.clone(),
    })
}

/// Accept the individual fields so Settings can verify a draft before it has been saved.
pub(crate) fn endpoint_for(
    provider: &str,
    base_url: &str,
    key: &Key,
    skip_reasoning: SkipReasoning,
) -> Result<Endpoint, RunError> {
    Ok(Endpoint {
        provider: provider.to_owned(),
        base_url: base_url.to_owned(),
        api_key: key_for(key)?.map(ToOwned::to_owned),
        skip_reasoning,
    })
}

/// Defer key errors so one Provider with a missing key does not prevent using the others.
pub(crate) fn key_for(key: &Key) -> Result<Option<&str>, RunError> {
    match key {
        Key::Found { key, .. } => Ok(Some(key)),
        Key::NotNeeded => Ok(None),
        Key::Missing(missing) => Err(RunError::Configuration {
            message: missing.clone(),
        }),
    }
}

fn chosen<'a>(
    config: &'a Config,
    binding: Option<&str>,
    kind: Kind,
    words: &Words,
) -> Result<(&'a Provider, &'a Model), RunError> {
    // An Action's explicit binding takes precedence for both text and images.
    if let Some(name) = binding {
        return config
            .model(name)
            .ok_or_else(|| bound_to_nothing(config, name, words));
    }

    if kind == Kind::Image {
        // A manually chosen binding can use a Model whose vision capability was left undeclared.
        let Some(name) = config.default_vision_model.as_deref() else {
            return Err(no_vision_model(config, words));
        };

        return config
            .model(name)
            .ok_or_else(|| nominates_nothing(config, VISION_SETTING, name, words));
    }

    default(config, words)
}

fn default<'a>(config: &'a Config, words: &Words) -> Result<(&'a Provider, &'a Model), RunError> {
    let Some(name) = config.default_model.as_deref() else {
        return Err(nothing_nominated(config, words));
    };

    config
        .model(name)
        .ok_or_else(|| nominates_nothing(config, MODEL_SETTING, name, words))
}

/// List available Models so the user can fix a name without looking one up.
fn offered(config: &Config, words: &Words) -> String {
    let names: Vec<String> = config
        .models()
        .map(|(provider, model)| config::qualified(provider, model))
        .collect();

    match names.is_empty() {
        true => say!(words, "model-none-configured"),
        false => say!(words, "model-configured-are", "models" = names.join(", ")),
    }
}

fn bound_to_nothing(config: &Config, name: &str, words: &Words) -> RunError {
    RunError::Configuration {
        message: say!(
            words,
            "model-action-binds-nothing",
            "model" = name.to_owned(),
            "path" = config.path.display().to_string(),
            "offered" = offered(config, words)
        ),
    }
}

fn nominates_nothing(config: &Config, setting: &str, name: &str, words: &Words) -> RunError {
    RunError::Configuration {
        message: say!(
            words,
            "model-setting-names-nothing",
            "setting" = setting.to_owned(),
            "path" = config.path.display().to_string(),
            "model" = name.to_owned(),
            "offered" = offered(config, words)
        ),
    }
}

/// A text-only fallback could confidently answer without seeing the picture.
fn no_vision_model(config: &Config, words: &Words) -> RunError {
    RunError::Configuration {
        message: say!(
            words,
            "model-no-vision-model",
            "setting" = VISION_SETTING,
            "path" = config.path.display().to_string(),
            "offered" = offered(config, words)
        ),
    }
}

fn nothing_nominated(config: &Config, words: &Words) -> RunError {
    RunError::Configuration {
        message: say!(
            words,
            "model-nothing-nominated",
            "setting" = MODEL_SETTING,
            "path" = config.path.display().to_string(),
            "offered" = offered(config, words)
        ),
    }
}

#[cfg(test)]
mod tests {
    //! Build Settings directly to exercise bindings independently of TOML parsing.

    use std::path::PathBuf;

    use super::*;

    const FILE: &str = "/somewhere/settings.toml";

    fn provider(name: &str, models: &[(&str, bool)]) -> Provider {
        Provider {
            name: name.to_owned(),
            base_url: format!("https://{name}.example/v1"),
            key: Key::Found {
                key: format!("{name}-key"),
                from: config::Origin::File,
            },
            models: models
                .iter()
                .map(|(id, vision)| Model {
                    id: (*id).to_owned(),
                    vision: *vision,
                })
                .collect(),
            skip_reasoning: SkipReasoning::Unmentioned,
        }
    }

    fn local(name: &str, models: &[(&str, bool)]) -> Provider {
        Provider {
            key: Key::NotNeeded,
            ..provider(name, models)
        }
    }

    fn wanting_a_key(name: &str, models: &[(&str, bool)]) -> Provider {
        Provider {
            key: Key::Missing(format!(
                "The Provider \"{name}\" has no API key: export SOMETHING."
            )),
            ..provider(name, models)
        }
    }

    fn settings(providers: Vec<Provider>, default: Option<&str>, vision: Option<&str>) -> Config {
        Config {
            path: PathBuf::from(FILE),
            providers,
            default_model: default.map(ToOwned::to_owned),
            default_vision_model: vision.map(ToOwned::to_owned),
            large_selection: None,
            language: None,
        }
    }

    fn one_model() -> Config {
        settings(
            vec![provider("cheap", &[("everyday", false)])],
            Some("cheap/everyday"),
            None,
        )
    }

    fn both() -> Config {
        settings(
            vec![
                provider("cheap", &[("everyday", false)]),
                provider("dear", &[("sharp", true)]),
            ],
            Some("cheap/everyday"),
            Some("dear/sharp"),
        )
    }

    fn resolved(config: &Config, binding: Option<&str>, kind: Kind) -> String {
        let resolved = resolve(config, binding, kind, &crate::i18n::english())
            .expect("the chain should resolve");

        // Include the Provider identity because the same Model ID may be offered at different addresses.
        let provider = config
            .providers
            .iter()
            .find(|provider| provider.base_url == resolved.endpoint.base_url)
            .expect("the endpoint should be one of the Providers configured");

        format!("{}/{}", provider.name, resolved.model)
    }

    fn failure(config: &Config, binding: Option<&str>, kind: Kind) -> String {
        let error = resolve(config, binding, kind, &crate::i18n::english())
            .expect_err("the chain should not resolve");

        assert!(
            matches!(error, RunError::Configuration { message: _ }),
            "a setting is what would fix it: {error:?}"
        );

        error.message().to_owned()
    }

    #[test]
    fn an_action_that_binds_a_model_gets_that_model() {
        assert_eq!(
            resolved(&both(), Some("dear/sharp"), Kind::Text),
            "dear/sharp"
        );
    }

    #[test]
    fn a_binding_wins_over_both_defaults() {
        assert_eq!(
            resolved(&both(), Some("cheap/everyday"), Kind::Image),
            "cheap/everyday"
        );
    }

    #[test]
    fn a_binding_to_a_model_nobody_offers_names_what_is_missing() {
        let message = failure(&both(), Some("dear/imagined"), Kind::Text);

        assert!(message.contains("dear/imagined"), "{message}");
        assert!(message.contains(FILE), "{message}");
        assert!(message.contains("cheap/everyday"), "{message}");
    }

    #[test]
    fn an_action_that_binds_nothing_gets_the_default_model() {
        assert_eq!(resolved(&both(), None, Kind::Text), "cheap/everyday");
    }

    #[test]
    fn an_image_gets_the_default_vision_model() {
        assert_eq!(resolved(&both(), None, Kind::Image), "dear/sharp");
    }

    #[test]
    fn text_is_not_sent_to_the_vision_model_merely_because_there_is_one() {
        assert_eq!(resolved(&both(), None, Kind::Text), "cheap/everyday");
    }

    #[test]
    fn an_image_with_no_vision_model_nominated_names_the_setting_that_fixes_it() {
        // A text-only fallback could answer confidently without seeing the picture.
        let message = failure(&one_model(), None, Kind::Image);

        assert!(message.contains("default_vision_model"), "{message}");
        assert!(message.contains(FILE), "{message}");
        assert!(message.contains("cheap/everyday"), "{message}");
    }

    #[test]
    fn a_vision_model_nobody_ticked_as_one_is_used_anyway() {
        // A binding expresses the user's choice even when vision capability was left undeclared.
        let config = settings(
            vec![provider("cheap", &[("everyday", false)])],
            Some("cheap/everyday"),
            Some("cheap/everyday"),
        );

        assert_eq!(resolved(&config, None, Kind::Image), "cheap/everyday");
    }

    #[test]
    fn a_default_vision_model_naming_nothing_names_the_setting_it_is() {
        let config = settings(
            vec![provider("cheap", &[("everyday", false)])],
            Some("cheap/everyday"),
            Some("dear/imagined"),
        );
        let message = failure(&config, None, Kind::Image);

        assert!(message.contains("default_vision_model"), "{message}");
        assert!(message.contains("dear/imagined"), "{message}");
    }

    #[test]
    fn a_default_model_naming_nothing_names_the_setting_it_is() {
        let config = settings(
            vec![provider("cheap", &[("everyday", false)])],
            Some("cheap/imagined"),
            None,
        );
        let message = failure(&config, None, Kind::Text);

        assert!(message.contains("default_model"), "{message}");
        assert!(message.contains("cheap/imagined"), "{message}");
        assert!(message.contains("cheap/everyday"), "{message}");
    }

    #[test]
    fn nothing_nominated_asks_for_a_default_model_and_lists_what_there_is() {
        let config = settings(
            vec![
                provider("cheap", &[("everyday", false)]),
                provider("dear", &[("sharp", true)]),
            ],
            None,
            None,
        );
        let message = failure(&config, None, Kind::Text);

        assert!(message.contains("default_model"), "{message}");
        assert!(message.contains("cheap/everyday"), "{message}");
        assert!(message.contains("dear/sharp"), "{message}");
    }

    #[test]
    fn a_provider_with_no_models_at_all_says_so() {
        let config = settings(vec![provider("cheap", &[])], None, None);
        let message = failure(&config, None, Kind::Text);

        assert!(message.contains("No Model is configured"), "{message}");
        assert!(message.contains(FILE), "{message}");
    }

    #[test]
    fn a_model_whose_provider_has_no_key_says_where_a_key_goes() {
        let config = settings(
            vec![wanting_a_key("cheap", &[("everyday", false)])],
            Some("cheap/everyday"),
            None,
        );
        let message = failure(&config, None, Kind::Text);

        assert!(message.contains("no API key"), "{message}");
    }

    #[test]
    fn a_model_whose_service_has_no_key_resolves_with_none_to_send() {
        let config = settings(
            vec![local("local", &[("a-model", false)])],
            Some("local/a-model"),
            None,
        );
        let resolved = resolve(&config, None, Kind::Text, &crate::i18n::english())
            .expect("the chain should resolve");

        assert_eq!(resolved.endpoint.api_key, None);
    }

    #[test]
    fn the_resolved_model_carries_the_provider_that_offers_it() {
        let config = both();
        let resolved = resolve(&config, None, Kind::Image, &crate::i18n::english())
            .expect("the chain should resolve");

        assert_eq!(resolved.endpoint.base_url, "https://dear.example/v1");
        assert_eq!(resolved.endpoint.api_key.as_deref(), Some("dear-key"));
        assert_eq!(resolved.model, "sharp");
    }
}
