//! The settings file as the settings window sees it, and how the window writes
//! one back.
//!
//! Two shapes rather than one, and the asymmetry is the point. ADR-0002 buys
//! keeping the key on disk with exactly one promise — "The key never enters the
//! webview" — and the window that edits keys is drawn in the same webview that
//! renders whatever a Model said. So a key travels in, when somebody types one,
//! and never back out: [`Settings`] carries a [`KeyStanding`], which says where
//! each key is rather than what it is, and [`Edit`] carries a [`KeyEdit`],
//! which says what to do about it.
//!
//! What is written goes through `toml_edit` rather than through serde, so that
//! the preamble a fresh installation is met by, the comments somebody added to
//! their own file, and anything a later Demysto put there all survive a save.
//! The file belongs to the user; the window is a guest in it — ADR-0007, which
//! also records why a save is read back before it is written.

use std::fmt;
use std::path::Path;

use serde::{Deserialize, Serialize};
use toml_edit::{value, Array, ArrayOfTables, DocumentMut, InlineTable, Item, Table, Value};

use crate::config::{
    self, Auth, Config, ConfigError, Environment, Key, ModelEntry, Origin, ProviderEntry,
};
use crate::i18n::{say, Interface, Words};
use crate::model::{self, Endpoint};
use crate::run::RunError;

/// What the settings file configures, as the window shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Settings {
    pub providers: Vec<ConfiguredProvider>,
    /// The Model an Action binding none of its own resolves to, by the name it
    /// is nominated with.
    pub default_model: Option<String>,
    /// And the one an image resolves to first.
    pub default_vision_model: Option<String>,
    /// The Hotkey that opens the Palette, `None` for the one Demysto comes
    /// with. Carried as the text it was written as, and judged nowhere here,
    /// for the reason `DefinedAction::hotkey` is: whether a combination can be
    /// claimed is a question only the desktop can answer.
    pub palette_hotkey: Option<String>,
    /// How many characters a Selection may hold before Demysto says so, `None`
    /// where the file states nothing and Demysto's own figure decides — which
    /// [`crate::Status`] reports, so that the window can name it.
    pub large_selection: Option<u64>,
    /// The language the file asks the interface to speak, `None` where it asks
    /// for none and the operating system decides.
    ///
    /// Also `None` where the file names one no catalogue answers to. The window
    /// offers a closed list of two, and there is no field on it that could show
    /// a third: a file hand-edited to `language = "de"` is reported as asking
    /// for nothing, which is what Demysto is in fact doing about it — and the
    /// next save from that window takes the line out. Unlike a Hotkey, which is
    /// carried as written because only the desktop can judge it, the set of
    /// languages Demysto speaks is a set of files in this repository.
    pub language: Option<String>,
}

/// One Provider as the file states it — with where its key is, and not the key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConfiguredProvider {
    pub name: String,
    /// What the file states, `None` when the preset supplies it. Left as the
    /// file has it rather than filled in from the preset, because the window
    /// edits the file: showing a preset's own base URL in the field would turn
    /// it into a stated one the first time somebody saved.
    pub base_url: Option<String>,
    /// The preset by the word the file names it with.
    pub preset: Option<String>,
    pub api_key_env: Option<String>,
    pub key: KeyStanding,
    pub models: Vec<ConfiguredModel>,
}

/// A Model of a Provider, and the one capability Demysto has to be told about.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfiguredModel {
    /// What the Provider calls it, which is what a request carries.
    pub id: String,
    #[serde(default)]
    pub vision: bool,
}

/// Where a Provider's key is, which is as much as the window is told about it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum KeyStanding {
    /// In the settings file.
    InFile,
    /// In an environment variable, which is named — so that the window can say
    /// which one rather than invite somebody to paste over a key that would go
    /// on being ignored (ADR-0002 puts the variable first).
    InEnvironment { variable: String },
    /// The service has none to send (ADR-0006).
    NotNeeded,
    /// The service wants one and none was found anywhere.
    Missing,
}

/// What the window hands back when the user saves.
///
/// The whole of the settings, every time: a Provider the file holds and this
/// does not is one the user removed.
#[derive(Debug, Clone, Deserialize)]
pub struct Edit {
    pub providers: Vec<ProviderEdit>,
    pub default_model: Option<String>,
    pub default_vision_model: Option<String>,
    pub palette_hotkey: Option<String>,
    /// `None` takes the setting out of the file and leaves Demysto's own figure
    /// deciding; zero is the user asking to be told nothing, which is a
    /// different thing and is written.
    #[serde(default)]
    pub large_selection: Option<u64>,
    /// The language to speak, `None` for following the operating system.
    #[serde(default)]
    pub language: Option<String>,
}

/// One Provider as the window would have it.
#[derive(Debug, Clone, Deserialize)]
pub struct ProviderEdit {
    /// What this Provider was called in the file, `None` for one being added.
    ///
    /// What the file already holds for it is found by this rather than by
    /// [`Self::name`], so that renaming a Provider keeps the key written under
    /// the old name — and the comments somebody put above it.
    #[serde(default)]
    pub was: Option<String>,
    pub name: String,
    pub base_url: Option<String>,
    pub preset: Option<String>,
    pub api_key_env: Option<String>,
    #[serde(default)]
    pub api_key: KeyEdit,
    #[serde(default)]
    pub models: Vec<ConfiguredModel>,
}

/// What a save does to the key the settings file holds for one Provider.
///
/// Three states rather than an `Option<String>`, because a window that is never
/// shown the key cannot hand it back: leaving it alone is the ordinary case,
/// and it has to be distinguishable from there being none.
#[derive(Clone, Default, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum KeyEdit {
    /// Leave whatever the file holds.
    #[default]
    Keep,
    /// Write this into the file.
    Set { key: String },
    /// Take the key out of the file.
    Forget,
}

impl fmt::Debug for KeyEdit {
    /// Written out rather than derived, for the reason `config::Provider`'s own
    /// is: this is the one type in the crate that carries a key on its way in
    /// from the window, and a key that can be printed is a key that reaches a
    /// panic message or a log (ADR-0010). [`ProviderEdit`] and
    /// [`Edit`] derive theirs, which is safe because this one does not.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Keep => "Keep",
            Self::Set { .. } => "Set(<not shown>)",
            Self::Forget => "Forget",
        })
    }
}

/// A service Demysto knows the conventions of, as the window offers it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Preset {
    /// The word the settings file names it by.
    pub name: String,
    pub base_url: String,
    /// The variable this service's own documentation says to export, `None`
    /// where it documents none.
    pub variable: Option<String>,
    /// Whether the service has keys at all. A separate fact from documenting no
    /// variable for one: ADR-0006 lets only the second turn authentication off.
    pub needs_key: bool,
}

/// Every preset there is, so that configuring a common service is picking a
/// name rather than looking up a base URL (user story 32).
pub(crate) fn presets() -> Vec<Preset> {
    config::Preset::ALL
        .into_iter()
        .map(|preset| {
            let spec = preset.spec();

            Preset {
                name: spec.name.to_owned(),
                base_url: spec.base_url.to_owned(),
                variable: match spec.auth {
                    Auth::Variable(variable) => Some(variable.to_owned()),
                    Auth::Nothing => None,
                },
                needs_key: matches!(spec.auth, Auth::Variable(_)),
            }
        })
        .collect()
}

/// The settings as the file now holds them.
pub(crate) fn read(
    config_dir: &Path,
    env: &Environment,
    words: &Words,
) -> Result<Settings, ConfigError> {
    let (path, text) = config::read(config_dir, words)?;
    let file = config::parse(&path, &text, words)?;

    let providers = file
        .providers
        .iter()
        .map(|entry| ConfiguredProvider {
            name: entry.name.clone(),
            base_url: entry.base_url.clone(),
            preset: entry.preset.map(|preset| preset.spec().name.to_owned()),
            api_key_env: entry.api_key_env.clone(),
            key: standing(&config::resolve_key(entry, env, &path, words)),
            models: entry
                .models
                .iter()
                .map(|model| ConfiguredModel {
                    id: model.id.clone(),
                    vision: model.vision,
                })
                .collect(),
        })
        .collect();

    Ok(Settings {
        providers,
        default_model: file.default_model,
        default_vision_model: file.default_vision_model,
        // Trimmed on the way out, unlike the two above it, because this is the
        // one field that goes to a parser which will not trim it for itself: a
        // Hotkey stated by hand as " F13" would read back looking right and
        // then refuse to be claimed, for a reason nothing on screen could show.
        palette_hotkey: config::stated(file.palette_hotkey),
        large_selection: file.large_selection,
        // Only a language Demysto speaks reaches the window: it has no field
        // that could show a third, and `i18n::chosen` is already passing the
        // same value over and letting the desktop decide.
        language: config::stated(file.language)
            .and_then(|tag| Interface::matching(&tag))
            .map(|language| language.tag().to_owned()),
    })
}

/// Writes what the window edited over the settings file, and answers with the
/// settings as the file then holds them.
///
/// Read back rather than reflected: what the window is shown afterwards is what
/// the next start of Demysto will read, keys and all, not what the window
/// believed it was saving.
pub(crate) fn write(
    config_dir: &Path,
    env: &Environment,
    edit: &Edit,
    words: &Words,
) -> Result<Settings, ConfigError> {
    let (path, text) = config::read(config_dir, words)?;

    // Asked of the file before it is edited, so that a file nobody can parse is
    // reported in `config`'s own words — which name the line it failed on
    // without quoting it back, that line being the key as often as not.
    config::parse(&path, &text, words)?;

    let rewritten = rewritten(&text, edit, &path, words)?;

    // Checked before anything reaches the disk. A window that had written a
    // file Demysto cannot read would be a window that could no longer open it,
    // leaving the user to repair by hand the one file this ticket exists to
    // spare them. It is also what checks the field names [`rewritten`] writes,
    // which serde cannot: the file denies unknown fields, so a name misspelt
    // there fails to parse here rather than reaching anybody's disk.
    let written = config::resolve(&path, config::parse(&path, &rewritten, words)?, env, words)?;

    nominating(
        &written,
        config::MODEL_SETTING,
        edit.default_model.as_deref(),
        words,
    )?;
    nominating(
        &written,
        config::VISION_SETTING,
        edit.default_vision_model.as_deref(),
        words,
    )?;

    config::write(&path, &rewritten, words)?;

    read(config_dir, env, words)
}

/// Where a request would reach the Provider a draft describes, and what it
/// would authenticate with — so that a key can be tried and a Model list
/// fetched before any of it is saved.
///
/// The key follows ADR-0002's order like any other, through the same
/// resolution: a key typed into the window is still beaten by a variable, which
/// is what the window has to be able to show somebody who wonders why theirs
/// made no difference.
pub(crate) fn endpoint(
    config_dir: &Path,
    env: &Environment,
    draft: &ProviderEdit,
    words: &Words,
) -> Result<Endpoint, RunError> {
    let configuration = |error: ConfigError| RunError::Configuration {
        message: error.to_string(),
    };

    let (path, text) = config::read(config_dir, words).map_err(configuration)?;
    let held = config::parse(&path, &text, words).map_err(configuration)?;

    // "Leave the key alone" means the key the file holds under the name this
    // Provider had, which is the one thing about a draft that is not in it.
    let stored = match draft.api_key {
        KeyEdit::Keep => draft
            .was
            .as_deref()
            .and_then(|was| held.providers.iter().find(|entry| entry.name == was))
            .and_then(|entry| entry.api_key.clone()),
        _ => None,
    };

    let entry = entry(draft, stored, words).map_err(configuration)?;

    model::endpoint_for(
        &draft.name,
        &config::base_url(&entry, &path, words).map_err(configuration)?,
        &config::resolve_key(&entry, env, &path, words),
    )
}

/// Refuses settings that nominate a Model no Provider in them offers.
///
/// Such a file loads and then fails at the first Run, and `model` says exactly
/// which setting is wrong when it does — the right answer for a file written by
/// hand, and the wrong one for a window that had the whole list of Models on
/// screen as it was written. It is also how renaming a Provider would silently
/// break the Default Model that named it: the key follows a rename, and the
/// nomination cannot, so this is where the user is asked to pick again.
fn nominating(
    config: &Config,
    setting: &str,
    name: Option<&str>,
    words: &Words,
) -> Result<(), ConfigError> {
    // Blank is nothing, for the reason `stating` writes a blank field as an
    // absent one: the window sends "None" as an empty string, and the two have
    // to agree — otherwise the file that is about to be written, which states
    // no default at all, is refused for naming a Model called "".
    let Some(name) = config::stated(name.map(ToOwned::to_owned)) else {
        return Ok(());
    };

    let name = name.as_str();

    if config.model(name).is_some() {
        return Ok(());
    }

    let offered: Vec<String> = config
        .models()
        .map(|(provider, model)| config::qualified(provider, model))
        .collect();

    Err(ConfigError::Malformed(match offered.is_empty() {
        true => say!(
            words,
            "model-nomination-none-configured",
            "setting" = setting.to_owned(),
            "model" = name.to_owned()
        ),
        false => say!(
            words,
            "model-nomination-unknown",
            "setting" = setting.to_owned(),
            "model" = name.to_owned(),
            "models" = offered.join(", ")
        ),
    }))
}

/// A draft as the settings file would have stated it, so that everything the
/// file's own rules decide — the base URL, and which of the three sources the
/// key comes from — is decided in one place for a draft and a saved Provider
/// alike.
fn entry(
    draft: &ProviderEdit,
    stored: Option<String>,
    words: &Words,
) -> Result<ProviderEntry, ConfigError> {
    Ok(ProviderEntry {
        name: draft.name.clone(),
        base_url: draft.base_url.clone(),
        preset: preset(draft, words)?,
        api_key: match &draft.api_key {
            KeyEdit::Keep => stored,
            KeyEdit::Set { key } => Some(key.clone()),
            KeyEdit::Forget => None,
        },
        // Through `config::stated` like everything else the window sends: it
        // sends a field it has no value for as an empty string, and naming the
        // variable "" would be `resolve_key` reading this Provider as one that
        // is authenticated — which for a keyless preset is the difference
        // between a Model list and a demand for a key that does not exist.
        api_key_env: config::stated(draft.api_key_env.clone()),
        models: draft
            .models
            .iter()
            .map(|model| ModelEntry {
                id: model.id.clone(),
                vision: model.vision,
            })
            .collect(),
    })
}

/// The preset a draft names. An error where nothing is called that: the window
/// offers the list, so a name off it is a version of Demysto that no longer has
/// one — and inventing a base URL for it would be worse than saying so.
fn preset(draft: &ProviderEdit, words: &Words) -> Result<Option<config::Preset>, ConfigError> {
    let Some(name) = draft
        .preset
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
    else {
        return Ok(None);
    };

    config::Preset::named(name).map(Some).ok_or_else(|| {
        ConfigError::Malformed(say!(
            words,
            "config-no-such-preset",
            "preset" = name.to_owned()
        ))
    })
}

/// The file with the window's edits written into it, and everything else about
/// it left where it was.
fn rewritten(text: &str, edit: &Edit, path: &Path, words: &Words) -> Result<String, ConfigError> {
    let mut document: DocumentMut = text.parse().map_err(|_| uneditable(path, words))?;

    // Stated outright rather than left to the default a file written by hand
    // gets, so that a file this build wrote says which build wrote it.
    document["version"] = value(i64::from(config::VERSION));

    let root = document.as_table_mut();
    stating(root, config::MODEL_SETTING, edit.default_model.as_deref());
    stating(
        root,
        config::VISION_SETTING,
        edit.default_vision_model.as_deref(),
    );
    stating(
        root,
        config::PALETTE_SETTING,
        edit.palette_hotkey.as_deref(),
    );
    counting(root, config::LARGE_SELECTION_SETTING, edit.large_selection);
    // Only a language Demysto speaks is written. The window offers a list, so
    // anything else is a field it had no business sending, and writing it would
    // put a setting in the file that nothing acts on.
    stating(
        root,
        config::LANGUAGE_SETTING,
        edit.language
            .as_deref()
            .and_then(Interface::matching)
            .map(Interface::tag),
    );

    let held = root
        .get("providers")
        .and_then(Item::as_array_of_tables)
        .cloned()
        .unwrap_or_default();

    let mut providers = ArrayOfTables::new();

    for (at, draft) in edit.providers.iter().enumerate() {
        // The table the file already holds for this Provider, carrying whatever
        // was written around it — a comment above it, a field a later Demysto
        // understands and this one does not.
        let mut table = draft
            .was
            .as_deref()
            .and_then(|was| {
                held.iter()
                    .find(|table| table.get("name").and_then(Item::as_str) == Some(was))
            })
            .cloned()
            .unwrap_or_default();

        // Where each table sits among the others, in the order the window has
        // them: a table cloned out of the file still remembers where it was,
        // and two that remember the same place order by neither.
        table.set_position(Some(at as isize));

        table["name"] = value(draft.name.trim());
        stating(&mut table, "base_url", draft.base_url.as_deref());
        stating(
            &mut table,
            "preset",
            preset(draft, words)?.map(|preset| preset.spec().name),
        );
        stating(&mut table, "api_key_env", draft.api_key_env.as_deref());

        match &draft.api_key {
            // The file's own key, untouched — and unread. This is the whole of
            // what "the key never enters the webview" costs to keep.
            KeyEdit::Keep => {}
            KeyEdit::Set { key } => stating(&mut table, "api_key", Some(key.as_str())),
            KeyEdit::Forget => {
                table.remove("api_key");
            }
        }

        table["models"] = value(models(&draft.models));

        providers.push(table);
    }

    root["providers"] = Item::ArrayOfTables(providers);

    Ok(document.to_string())
}

/// The Models of one Provider, as the file states them: one line, and `vision`
/// only where it is true — the shape the template's own example is written in.
fn models(models: &[ConfiguredModel]) -> Array {
    models
        .iter()
        .map(|model| {
            let mut stated = InlineTable::new();
            stated.insert("id", Value::from(model.id.trim()));

            if model.vision {
                stated.insert("vision", Value::from(true));
            }

            Value::InlineTable(stated)
        })
        .collect()
}

/// Writes a field somebody stated, and takes it out where they stated nothing.
///
/// Nothing but whitespace is nothing: an emptied field is the user clearing a
/// setting, and writing `base_url = ""` for it would be a Provider that answers
/// nowhere rather than one that takes its preset's address. It is also what
/// lets the window send a blank field rather than decide for itself that a
/// blank one means absent.
fn stating(table: &mut Table, key: &str, held: Option<&str>) {
    match held.map(str::trim).filter(|held| !held.is_empty()) {
        Some(held) => table[key] = value(held),
        None => {
            table.remove(key);
        }
    }
}

/// Writes a count somebody stated, and takes it out where they stated nothing.
///
/// Unlike [`stating`], zero is a value: a Selection warning turned off is the
/// user saying something, and a field emptied is the user saying nothing and
/// leaving Demysto's own figure to decide.
fn counting(table: &mut Table, key: &str, held: Option<u64>) {
    match held {
        Some(held) => table[key] = value(i64::try_from(held).unwrap_or(i64::MAX)),
        None => {
            table.remove(key);
        }
    }
}

fn standing(key: &Key) -> KeyStanding {
    match key {
        Key::Found {
            from: Origin::File, ..
        } => KeyStanding::InFile,
        Key::Found {
            from: Origin::Variable(variable),
            ..
        } => KeyStanding::InEnvironment {
            variable: variable.clone(),
        },
        Key::NotNeeded => KeyStanding::NotNeeded,
        Key::Missing(_) => KeyStanding::Missing,
    }
}

/// What is said about a file that parsed for `config` and not for the editor
/// that keeps its comments — which is nothing anybody can act on beyond where
/// it is. Neither the reason nor the line is given: the two parsers disagreeing
/// is Demysto's problem rather than the user's, and the file's own text is
/// never quoted into a window that also renders what a Model said.
fn uneditable(path: &Path, words: &Words) -> ConfigError {
    ConfigError::Malformed(say!(
        words,
        "config-uneditable",
        "path" = path.display().to_string()
    ))
}
