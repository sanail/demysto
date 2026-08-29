//! Model resolution: from an Action and a Selection to exactly one Model, or to
//! an error saying which setting is missing.
//!
//! The chain the spec's *Model resolution* fixes is the Action's own binding,
//! then the Default Vision Model when the Selection is an image, then the
//! Default Model. Nothing here reaches the disk or the network: it works on the
//! settings as `config` read them, which is what lets the whole chain — the
//! branches a Run reaches today and the one v1 has no Selection kind for yet —
//! be tested without either.
//!
//! Every failure here is a [`RunError::Configuration`], because every one of
//! them is fixed in the settings file rather than by trying again.

use std::fmt;

use crate::config::{self, Config, Key, Model, Provider, MODEL_SETTING, VISION_SETTING};
use crate::run::RunError;
use crate::selection::Kind;

/// A Provider as a request reaches it: where it answers, and what to
/// authenticate with.
///
/// Owned rather than borrowed out of the settings, because the settings are no
/// longer read once and left alone: the window writes them while Demysto runs,
/// and nothing may hold them locked for the two minutes a Provider is allowed
/// to take over an answer.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Endpoint {
    /// What the user calls this Provider in their settings file.
    ///
    /// Carried so that a refused key can name the Provider whose settings fix
    /// it: the base URL is what a message about reaching an address should say,
    /// and the name is what the window opens a section by.
    pub(crate) provider: String,
    pub(crate) base_url: String,
    /// `None` for a service that has no key to send — see [`key_for`].
    pub(crate) api_key: Option<String>,
}

/// An Endpoint and the Model on it: everything a Run needs in order to ask
/// something.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Resolved {
    pub(crate) endpoint: Endpoint,
    /// What the Provider calls the Model, which is what the request carries.
    pub(crate) model: String,
}

impl fmt::Debug for Endpoint {
    /// Written out rather than derived, for the reason [`Provider`]'s own is:
    /// this is the one place the key sits unwrapped, and a key that can be
    /// printed is a key that reaches a panic message or a log (ADR-0010).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Endpoint")
            .field("provider", &self.provider)
            .field("base_url", &self.base_url)
            .field("api_key", &self.api_key.as_ref().map(|_| "<not shown>"))
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

/// The one Model a Run of this Action against a Selection of this kind uses.
///
/// `binding` is the Model the Action names. No built-in names one, and ticket
/// 09 is where an Override gives an Action its own.
pub(crate) fn resolve(
    config: &Config,
    binding: Option<&str>,
    kind: Kind,
) -> Result<Resolved, RunError> {
    let (provider, model) = chosen(config, binding, kind)?;

    Ok(Resolved {
        endpoint: endpoint_for(&provider.name, &provider.base_url, &provider.key)?,
        model: model.id.clone(),
    })
}

/// Where a Provider answers and what to authenticate with, taken out of the
/// settings so that the request can be made without them.
///
/// Takes the two fields it reaches a Provider by rather than the Provider, so
/// that the settings window can ask this of a Provider that is still being
/// typed — one that has no Models yet, and is in no file.
pub(crate) fn endpoint_for(
    provider: &str,
    base_url: &str,
    key: &Key,
) -> Result<Endpoint, RunError> {
    Ok(Endpoint {
        provider: provider.to_owned(),
        base_url: base_url.to_owned(),
        api_key: key_for(key)?.map(ToOwned::to_owned),
    })
}

/// The key to reach a Provider with: `None` where the service has none to send,
/// and an error where it wants one and the user has not supplied it.
///
/// The one place a key is taken out of the settings, which is what ADR-0002
/// asks for in exchange for keeping it on disk: "all key access goes through a
/// single interface in the Rust layer". Asked at the moment a Provider is
/// reached for rather than at load, so that one Provider missing its key costs
/// the user only the Models that Provider offers.
pub(crate) fn key_for(key: &Key) -> Result<Option<&str>, RunError> {
    match key {
        Key::Found { key, .. } => Ok(Some(key)),
        Key::NotNeeded => Ok(None),
        Key::Missing(missing) => Err(RunError::Configuration {
            message: missing.clone(),
        }),
    }
}

/// Which Model the chain arrives at, before anything is asked about reaching it.
fn chosen<'a>(
    config: &'a Config,
    binding: Option<&str>,
    kind: Kind,
) -> Result<(&'a Provider, &'a Model), RunError> {
    // What the Action names wins outright, images included: a binding is the
    // user saying which Model, and Demysto is in no position to know better.
    if let Some(name) = binding {
        return config
            .model(name)
            .ok_or_else(|| bound_to_nothing(config, name));
    }

    if kind == Kind::Image {
        if let Some(name) = config.default_vision_model.as_deref() {
            return config
                .model(name)
                .ok_or_else(|| nominates_nothing(config, VISION_SETTING, name));
        }
    }

    // An image with no Default Vision Model nominated goes to the Default Model
    // like anything else. Whether Demysto should refuse to send it to a Model
    // that cannot see is a real question, and one for v1.1: it has no Selection
    // to ask it of yet, and the answer is worth deciding against a picture
    // rather than against a guess.
    default(config)
}

/// The Model an Action that binds none resolves to.
fn default(config: &Config) -> Result<(&Provider, &Model), RunError> {
    let Some(name) = config.default_model.as_deref() else {
        return Err(nothing_nominated(config));
    };

    config
        .model(name)
        .ok_or_else(|| nominates_nothing(config, MODEL_SETTING, name))
}

/// The Models there are to choose from, so that the user can fix a name without
/// going to look one up.
fn offered(config: &Config) -> String {
    let names: Vec<String> = config
        .models()
        .map(|(provider, model)| config::qualified(provider, model))
        .collect();

    match names.is_empty() {
        true => "No Model is configured at all; add one to a Provider there.".to_owned(),
        false => format!("The Models configured there are: {}.", names.join(", ")),
    }
}

fn bound_to_nothing(config: &Config, name: &str) -> RunError {
    RunError::Configuration {
        message: format!(
        "This Action is bound to the Model \"{name}\", and no Provider in {} offers one by that \
         name. {}",
        config.path.display(),
        offered(config)
    ),
    }
}

fn nominates_nothing(config: &Config, setting: &str, name: &str) -> RunError {
    RunError::Configuration {
        message: format!(
            "{setting} in {} names the Model \"{name}\", and no Provider there offers one by that \
         name. {}",
            config.path.display(),
            offered(config)
        ),
    }
}

fn nothing_nominated(config: &Config) -> RunError {
    RunError::Configuration {
        message: format!(
            "No {MODEL_SETTING} is nominated in {}. {}",
            config.path.display(),
            offered(config)
        ),
    }
}

#[cfg(test)]
mod tests {
    //! The chain the spec's *Testing Decisions* asks to see tested "down the
    //! full chain, including the unresolvable case", tested here rather than at
    //! the facade — because two of its three legs cannot be reached from there
    //! in v1. No built-in Action binds a Model, and no Capture produces an
    //! image, so a seam-level test could exercise only the Default Model. What
    //! a Run does reach is asserted at the facade alongside everything else;
    //! this is the rest of the chain, and it follows `config`'s own precedent
    //! of testing key resolution beside the code that resolves.
    //!
    //! Settings are built here rather than parsed, so that these are about
    //! which Model the chain arrives at and not about what TOML says.

    use std::path::PathBuf;

    use super::*;

    const FILE: &str = "/somewhere/settings.toml";

    /// A Provider with a key, offering the Models named — each with whether it
    /// accepts images.
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
        }
    }

    /// A Provider whose service has no keys at all — a server on this machine.
    fn local(name: &str, models: &[(&str, bool)]) -> Provider {
        Provider {
            key: Key::NotNeeded,
            ..provider(name, models)
        }
    }

    /// A Provider whose service wants a key and where none was found.
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
        }
    }

    /// One Provider offering one blind Model, nominated as the Default Model —
    /// the whole of what most users configure.
    fn one_model() -> Config {
        settings(
            vec![provider("cheap", &[("everyday", false)])],
            Some("cheap/everyday"),
            None,
        )
    }

    /// A cheap blind Model and an expensive seeing one, at two Providers.
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

    /// The Model the chain arrives at, by the name it is nominated by.
    fn resolved(config: &Config, binding: Option<&str>, kind: Kind) -> String {
        let resolved = resolve(config, binding, kind).expect("the chain should resolve");

        // The Provider is named from the endpoint it answers at, which is what
        // resolving to a Model at one Provider rather than another comes to.
        let provider = config
            .providers
            .iter()
            .find(|provider| provider.base_url == resolved.endpoint.base_url)
            .expect("the endpoint should be one of the Providers configured");

        format!("{}/{}", provider.name, resolved.model)
    }

    /// What the user is told when it arrives at nothing.
    fn failure(config: &Config, binding: Option<&str>, kind: Kind) -> String {
        let error = resolve(config, binding, kind).expect_err("the chain should not resolve");

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
        // Including for an image, where the Default Vision Model would
        // otherwise have it: a binding is the user saying which Model.
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
    fn an_image_with_no_vision_model_nominated_falls_back_to_the_default_model() {
        // The end of the chain is the Default Model, whatever the Selection is.
        // Whether an image should be kept from a Model that cannot see is
        // v1.1's to decide, against a picture rather than against a guess.
        assert_eq!(resolved(&one_model(), None, Kind::Image), "cheap/everyday");
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
        let resolved = resolve(&config, None, Kind::Text).expect("the chain should resolve");

        assert_eq!(resolved.endpoint.api_key, None);
    }

    #[test]
    fn the_resolved_model_carries_the_provider_that_offers_it() {
        // Two Providers, two keys, two addresses: resolving to the Model is
        // only useful if it says which of them to ask.
        let config = both();
        let resolved = resolve(&config, None, Kind::Image).expect("the chain should resolve");

        assert_eq!(resolved.endpoint.base_url, "https://dear.example/v1");
        assert_eq!(resolved.endpoint.api_key.as_deref(), Some("dear-key"));
        assert_eq!(resolved.model, "sharp");
    }
}
