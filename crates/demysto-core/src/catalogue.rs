//! The Action catalogue as it stands on disk: the user's own Actions, the
//! Overrides they have made to built-in ones, and the effective set the two
//! produce together.
//!
//! ADR-0005 fixes the shape. Built-in Actions are compiled in and never seeded
//! into the configuration directory, so `actions/` holds only what the user
//! wrote: one file per Action they authored, plus one per built-in they changed.
//! A built-in added by a later version therefore reaches somebody who installed
//! an earlier one, and "reset to default" is deleting a file.
//!
//! One file per Action rather than a list in one file, so that an Action can be
//! sent to somebody as a file. The file is written by serialising it, unlike the
//! settings file next door — ADR-0009 says why the two are not written the same
//! way.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::action::{self, Action, Parameter};
use crate::files;
use crate::selection::Kind;

/// The directory Actions are kept in, inside the configuration directory.
pub(crate) const DIR_NAME: &str = "actions";

/// What one is kept in, and the only extension read.
const EXTENSION: &str = "toml";

/// The shape of an Action file this build understands. A file claiming a higher
/// one was written by a newer Demysto, and guessing at what it means would be
/// running a prompt nobody wrote.
const VERSION: u32 = 1;

/// The line every file this writes opens with, so that somebody who comes
/// across one knows what they are looking at.
const PREAMBLE: &str = "# An Action Demysto runs. Edit it here, or in Demysto's Settings.";

/// How far the search for an unused identifier goes before it gives up. Far
/// past anything a person would do by hand, and short of a loop.
const ATTEMPTS: u32 = 1000;

/// What a name may not become, because Windows will not open a file called it
/// whatever the extension.
const RESERVED: [&str; 22] = [
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
    "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

/// The Actions as the window that edits them sees them, and whatever in
/// `actions/` could not be read.
///
/// The unreadable files travel with the Actions rather than replacing them: one
/// file nobody can parse is no reason for the rest of somebody's Actions to
/// disappear, and no reason to leave them wondering where one went.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Catalogue {
    /// The effective set: the built-ins with their Overrides applied, in the
    /// order the Palette lists them, then the user's own by name.
    pub actions: Vec<DefinedAction>,
    /// What went wrong with the files that are not in it, in whole sentences.
    pub unreadable: Vec<String>,
}

/// One Action with everything about it, for the window that writes it.
///
/// [`Action`] keeps the prompt and the Model to itself, because the Palette has
/// no business with either. This is the other view of the same thing: the one
/// held by the window whose whole purpose is to change them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DefinedAction {
    /// What this Action is asked for by, and what its file is called. Fixed
    /// when the Action is created and unchanged by renaming it, because an
    /// Override is keyed on it, a Hotkey is bound to it, and neither should
    /// follow a renaming.
    pub id: String,
    pub name: String,
    /// What it says to the Model, with `{{selection}}` and the rest standing in
    /// for what a Run fills in.
    pub template: String,
    pub parameters: Vec<Parameter>,
    /// The Model it runs on whatever the defaults say, `None` when it takes
    /// whichever Model resolution arrives at.
    pub model: Option<String>,
    /// The Hotkey that runs it without the Palette, `None` for an Action
    /// reached through the Palette like any other. Carried as the text it was
    /// written as: whether a combination can be claimed is a question only the
    /// desktop can answer, so the shell reads this and reports what it could
    /// not claim.
    pub hotkey: Option<String>,
    /// The Selection kinds it will run on. Text is the only one v1 captures;
    /// the field is here because the file states it and a save must not flatten
    /// a file written for a Demysto that captures more.
    pub accepts: Vec<Kind>,
    pub standing: ActionStanding,
    /// The file it is stored in, `None` for a built-in nobody has changed.
    pub path: Option<PathBuf>,
}

/// Where an Action's definition comes from, which is what the window offers
/// "reset" for and what it offers "delete" for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionStanding {
    /// Compiled into this build, exactly as it was written.
    BuiltIn,
    /// Compiled in, with the user's Override applied over it.
    Overridden,
    /// The user's own, and in no build.
    Authored,
}

/// What the window hands back when it saves one Action.
#[derive(Debug, Clone, Deserialize)]
pub struct ActionEdit {
    /// The Action this edits, `None` for one being created — which is what asks
    /// for an identifier to be found for it.
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    pub template: String,
    #[serde(default)]
    pub parameters: Vec<Parameter>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub hotkey: Option<String>,
    #[serde(default = "text")]
    pub accepts: Vec<Kind>,
}

fn text() -> Vec<Kind> {
    vec![Kind::Text]
}

/// What went wrong between the window and an Action on disk.
///
/// Every variant carries the whole sentence the user is shown, for the reason
/// [`crate::ConfigError`]'s do: the interface offers a different affordance per
/// kind, and composes none of the wording.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "message", rename_all = "snake_case")]
pub enum ActionError {
    /// The Action directory could not be read.
    Unreadable(String),
    /// The file could not be written, or could not be deleted.
    Unwritable(String),
    /// What the window asked to save is not an Action Demysto could run, so
    /// nothing was written.
    Refused(String),
    /// The Action the window asked about is not one there is a file for.
    NoSuchAction(String),
}

impl ActionError {
    /// The sentence the user is shown.
    pub fn message(&self) -> &str {
        match self {
            Self::Unreadable(message)
            | Self::Unwritable(message)
            | Self::Refused(message)
            | Self::NoSuchAction(message) => message,
        }
    }
}

impl fmt::Display for ActionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for ActionError {}

impl DefinedAction {
    /// The same Action as a Run and the Palette see it.
    fn runnable(&self) -> Action {
        action::assembled(
            &self.id,
            &self.name,
            &self.template,
            &self.parameters,
            self.model.as_deref(),
            &self.accepts,
        )
    }
}

/// The effective set: what the Palette lists and what a Run looks an Action up
/// in.
///
/// Read from the directory every time rather than held from startup, for the
/// reason the settings window reads the settings file every time: the files are
/// the user's, an Action can arrive in that directory as a file somebody sent
/// them, and a catalogue that needed a restart to notice would make "one file
/// each, portable" a promise with a footnote. It is a handful of small files,
/// and the Palette opens on a Hotkey — this is measured in microseconds.
pub(crate) fn runnable(config_dir: &Path) -> Vec<Action> {
    read(config_dir)
        .actions
        .iter()
        .map(DefinedAction::runnable)
        .collect()
}

/// The Action an interface asked for, or `None` when there is no such Action.
pub(crate) fn named(config_dir: &Path, id: &str) -> Option<Action> {
    runnable(config_dir)
        .into_iter()
        .find(|action| action.id == id)
}

/// Every Action there is, with everything about it, for the window that edits
/// them.
pub(crate) fn read(config_dir: &Path) -> Catalogue {
    let dir = config_dir.join(DIR_NAME);
    let mut unreadable = Vec::new();
    let mut held = stated(&dir, &mut unreadable);

    // The built-ins first and in their own order, which is how often they are
    // reached for: an Override renames an Action, it does not re-rank it.
    let mut actions: Vec<DefinedAction> = action::built_in()
        .into_iter()
        .map(|built_in| {
            let stated = held.remove(&built_in.id);
            overridden(built_in, stated, &dir)
        })
        .collect();

    // Then the user's own, by name: nothing about them says which matters most,
    // and the order a directory is read in is not an answer.
    let mut authored: Vec<DefinedAction> = held
        .into_iter()
        .filter_map(|(id, file)| match authored(&id, file, &dir) {
            Ok(action) => Some(action),
            Err(message) => {
                unreadable.push(message);
                None
            }
        })
        .collect();

    authored.sort_by(|one, other| one.name.cmp(&other.name).then(one.id.cmp(&other.id)));
    actions.append(&mut authored);

    Catalogue {
        actions,
        unreadable,
    }
}

/// Writes one Action, and answers with the catalogue as the directory then
/// holds it — the window shows what was read back rather than what it sent, for
/// the reason a saved settings file is read back.
pub(crate) fn write(
    config_dir: &Path,
    edit: &ActionEdit,
    offered: &[String],
) -> Result<Catalogue, ActionError> {
    let dir = config_dir.join(DIR_NAME);
    let stated = checked(edit, offered)?;
    let id = identifier(edit, &dir)?;

    let file = match action::built_in()
        .into_iter()
        .find(|built_in| built_in.id == id)
    {
        // Only what the user changed is written, so that a built-in improved by
        // a later version still reaches somebody who only ever bound a Model to
        // it. An Override stating nothing is an Override of nothing: the file
        // goes, and the built-in is back — which is what "reset" does too.
        Some(built_in) => match differing(&stated, &as_stated(&overridden(built_in, None, &dir))) {
            // Saving a built-in exactly as it was written is the same
            // instruction as resetting it: there is nothing to hold, so there
            // is no file to hold it in. Whether one was there is not the user's
            // concern, so its absence is not an error here.
            file if file.states_nothing() => match removing(&dir, &id) {
                Ok(()) | Err(ActionError::NoSuchAction(_)) => return Ok(read(config_dir)),
                Err(error) => return Err(error),
            },
            file => file,
        },
        None => stated,
    };

    let path = path(&dir, &id);
    let body = toml::to_string(&file)
        .map_err(|error| ActionError::Refused(unwritable_shape(&path, &error)))?;

    files::replace(&path, &format!("{PREAMBLE}\n{body}"))
        .map_err(|error| ActionError::Unwritable(unwritable(&path, &error)))?;

    Ok(read(config_dir))
}

/// Takes an Action off the user: deletes their own, or removes an Override and
/// leaves the built-in it was over.
pub(crate) fn delete(config_dir: &Path, id: &str) -> Result<Catalogue, ActionError> {
    let dir = config_dir.join(DIR_NAME);

    removing(&dir, id)?;

    Ok(read(config_dir))
}

/// Deletes the file an Action is in.
fn removing(dir: &Path, id: &str) -> Result<(), ActionError> {
    let path = path(dir, id);

    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            Err(ActionError::NoSuchAction(format!(
                "There is no Action called \"{id}\" to remove. It may already have been \
                 deleted; reopen this window."
            )))
        }
        Err(error) => Err(ActionError::Unwritable(unwritable(&path, &error))),
    }
}

/// Every file in `actions/` that parsed, by the identifier its name gives it,
/// with a sentence pushed onto `unreadable` for every one that did not.
fn stated(dir: &Path, unreadable: &mut Vec<String>) -> BTreeMap<String, ActionFile> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        // Nothing has been authored yet, which is what a fresh installation
        // looks like and is not a fault. The directory is created by the first
        // save rather than at startup: ADR-0005 leaves the configuration
        // directory to the user, and an empty directory nobody asked for is
        // still something written into it.
        Err(error) if error.kind() == io::ErrorKind::NotFound => return BTreeMap::new(),
        Err(error) => {
            unreadable.push(unreadable_dir(dir, &error));
            return BTreeMap::new();
        }
    };

    let mut held = BTreeMap::new();

    for entry in entries {
        let path = match entry {
            Ok(entry) => entry.path(),
            Err(error) => {
                unreadable.push(unreadable_dir(dir, &error));
                continue;
            }
        };

        // A file being written is not a file to read, and neither is anything
        // that is not an Action: a directory, a backup, a note to self.
        if files::is_half_written(&path) || path.extension().is_none_or(|it| it != EXTENSION) {
            continue;
        }

        let Some(id) = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
        else {
            continue;
        };

        match parse(&path) {
            Ok(file) => {
                held.insert(id, file);
            }
            Err(message) => unreadable.push(message),
        }
    }

    held
}

/// One file, checked only for being a shape this build knows.
fn parse(path: &Path) -> Result<ActionFile, String> {
    let text = fs::read_to_string(path).map_err(|error| unreadable(path, &error))?;
    let file: ActionFile =
        toml::from_str(&text).map_err(|error| unparseable(path, &text, &error))?;

    if file.version > VERSION {
        return Err(format!(
            "{} says it is version {}, and this Demysto understands version {VERSION}. \
             Update Demysto, or take the file out of that directory.",
            path.display(),
            file.version
        ));
    }

    Ok(file)
}

/// A built-in with whatever its Override says written over it, or as it was
/// written where there is none.
fn overridden(built_in: Action, file: Option<ActionFile>, dir: &Path) -> DefinedAction {
    let action::Parts {
        id,
        name,
        template,
        parameters,
        accepts,
    } = action::parts(built_in);

    let Some(file) = file else {
        return DefinedAction {
            id,
            name,
            template,
            parameters,
            model: None,
            hotkey: None,
            accepts,
            standing: ActionStanding::BuiltIn,
            path: None,
        };
    };

    DefinedAction {
        path: Some(path(dir, &id)),
        id,
        name: file.name.unwrap_or(name),
        template: file.template.unwrap_or(template),
        parameters: file.parameters.unwrap_or(parameters),
        model: file.model,
        hotkey: file.hotkey,
        accepts: file.accepts.unwrap_or(accepts),
        standing: ActionStanding::Overridden,
    }
}

/// An Action of the user's own, which — being in no build — has to state the
/// two things there is nothing to fall back to.
fn authored(id: &str, file: ActionFile, dir: &Path) -> Result<DefinedAction, String> {
    let path = path(dir, id);
    let missing = |field: &str| {
        format!(
            "{} states no {field}. An Action Demysto does not already have must state its \
             name and its template.",
            path.display()
        )
    };

    Ok(DefinedAction {
        id: id.to_owned(),
        name: file.name.ok_or_else(|| missing("name"))?,
        template: file.template.ok_or_else(|| missing("template"))?,
        parameters: file.parameters.unwrap_or_default(),
        model: file.model,
        hotkey: file.hotkey,
        accepts: file.accepts.unwrap_or_else(text),
        standing: ActionStanding::Authored,
        path: Some(path),
    })
}

/// What an edit states, as a file would state it — before anything is asked
/// about whether it is an Override or an Action of the user's own.
fn checked(edit: &ActionEdit, offered: &[String]) -> Result<ActionFile, ActionError> {
    let refused = |reason: String| ActionError::Refused(reason);

    let name = edit.name.trim();
    if name.is_empty() {
        return Err(refused(
            "An Action needs a name to be listed under.".to_owned(),
        ));
    }

    let template = edit.template.trim();
    if template.is_empty() {
        return Err(refused(
            "An Action needs a prompt: what it says to the Model, with {{selection}} where \
             the Selection goes."
                .to_owned(),
        ));
    }

    if edit.accepts.is_empty() {
        return Err(refused(
            "An Action that accepts no kind of Selection could never appear in the Palette."
                .to_owned(),
        ));
    }

    let parameters = parameters(&edit.parameters).map_err(refused)?;
    let model = binding(edit.model.as_deref(), offered).map_err(refused)?;

    Ok(ActionFile {
        version: VERSION,
        name: Some(name.to_owned()),
        model,
        hotkey: stated_value(edit.hotkey.as_deref()),
        accepts: Some(edit.accepts.clone()),
        template: Some(template.to_owned()),
        parameters: Some(parameters),
    })
}

/// The Parameters an Action declares, refusing the ones a Run could not collect.
fn parameters(declared: &[Parameter]) -> Result<Vec<Parameter>, String> {
    let mut seen = BTreeSet::new();
    let mut parameters = Vec::with_capacity(declared.len());

    for parameter in declared {
        let id = parameter.id.trim();
        let label = parameter.label.trim();

        if id.is_empty() {
            return Err(
                "A Parameter needs a name to be written as {{like_this}} in the prompt.".to_owned(),
            );
        }

        if action::is_a_variable(id) {
            return Err(format!(
                "A Parameter cannot be called \"{id}\": that is what a prompt writes to reach \
                 something Demysto fills in, so nothing would ever collect it."
            ));
        }

        if label.is_empty() {
            return Err(format!(
                "The Parameter \"{id}\" needs a label, which is what the Palette asks for it."
            ));
        }

        if !seen.insert(id.to_owned()) {
            return Err(format!(
                "Two Parameters are called \"{id}\", so {{{{{id}}}}} in the prompt could mean \
                 either."
            ));
        }

        parameters.push(trim(parameter));
    }

    Ok(parameters)
}

/// A Parameter as it is written down, with the whitespace around what somebody
/// typed taken off.
fn trim(parameter: &Parameter) -> Parameter {
    Parameter {
        id: parameter.id.trim().to_owned(),
        label: parameter.label.trim().to_owned(),
        default: parameter.default.trim().to_owned(),
    }
}

fn trimmed(parameters: &[Parameter]) -> Vec<Parameter> {
    parameters.iter().map(trim).collect()
}

/// The Model an Action binds, refused where no Provider offers one by that name.
///
/// Asked here rather than left to the Run, for the reason `settings::nominating`
/// asks it of the two defaults: the window had the whole list of Models on
/// screen as this was written, and a binding that resolves to nothing is a
/// failure the user would meet at the next Run instead.
fn binding(model: Option<&str>, offered: &[String]) -> Result<Option<String>, String> {
    let Some(model) = stated_value(model) else {
        return Ok(None);
    };

    if offered.contains(&model) {
        return Ok(Some(model));
    }

    Err(match offered.is_empty() {
        true => {
            format!("This Action binds the Model \"{model}\", and no Model is configured at all.")
        }
        false => format!(
            "This Action binds the Model \"{model}\", and no Provider offers one by that name. \
             The Models configured are: {}.",
            offered.join(", ")
        ),
    })
}

/// What an Override has to state: the fields this edit puts somewhere other
/// than where the built-in left them, and nothing else.
///
/// Both sides come through [`checked`]'s normalisation — see [`as_stated`] —
/// so that "the same as the built-in" cannot turn on a space nobody typed.
fn differing(stated: &ActionFile, built_in: &ActionFile) -> ActionFile {
    let unless = |same: bool, held: &Option<String>| (!same).then(|| held.clone()).flatten();

    ActionFile {
        version: VERSION,
        name: unless(stated.name == built_in.name, &stated.name),
        model: unless(stated.model == built_in.model, &stated.model),
        hotkey: unless(stated.hotkey == built_in.hotkey, &stated.hotkey),
        accepts: match stated.accepts == built_in.accepts {
            true => None,
            false => stated.accepts.clone(),
        },
        template: unless(stated.template == built_in.template, &stated.template),
        parameters: match stated.parameters == built_in.parameters {
            true => None,
            false => stated.parameters.clone(),
        },
    }
}

/// A built-in as an Override of it would have to state it.
///
/// Put through the same trimming [`checked`] puts the window's edit through,
/// because the two are about to be compared field by field: a built-in whose
/// template happened to end in a newline would otherwise differ from itself the
/// moment somebody saved it unaltered, and leave an Override behind that says
/// nothing.
fn as_stated(built_in: &DefinedAction) -> ActionFile {
    ActionFile {
        version: VERSION,
        name: stated_value(Some(&built_in.name)),
        model: stated_value(built_in.model.as_deref()),
        hotkey: stated_value(built_in.hotkey.as_deref()),
        accepts: Some(built_in.accepts.clone()),
        template: stated_value(Some(&built_in.template)),
        parameters: Some(trimmed(&built_in.parameters)),
    }
}

/// What this Action is to be filed under: the one it already had, or one found
/// for it from its name.
fn identifier(edit: &ActionEdit, dir: &Path) -> Result<String, ActionError> {
    let Some(id) = edit
        .id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty())
    else {
        return Ok(unused(&edit.name, dir));
    };

    match usable_as_a_file_name(id) {
        true => Ok(id.to_owned()),
        false => Err(ActionError::Refused(format!(
            "\"{id}\" cannot be the name of a file, so no Action can be kept under it."
        ))),
    }
}

/// An identifier nothing answers to yet, from the name the user gave.
///
/// This is where a user Action that collides with a built-in is separated from
/// an Override of one: an Action somebody creates and calls "Explain" is a
/// second Action called Explain, not a rewriting of the first, so it is filed
/// under an identifier of its own and both appear.
fn unused(name: &str, dir: &Path) -> String {
    let stem = slug(name);

    if !taken(&stem, dir) {
        return stem;
    }

    (2..=ATTEMPTS)
        .map(|at| format!("{stem}-{at}"))
        .find(|candidate| !taken(candidate, dir))
        // Past a thousand Actions of one name, the file this would overwrite is
        // the least of what has gone wrong.
        .unwrap_or(stem)
}

/// Whether an identifier is spoken for: by a built-in, by a file, or by the
/// operating system.
fn taken(id: &str, dir: &Path) -> bool {
    !usable_as_a_file_name(id)
        || path(dir, id).exists()
        || action::built_in().iter().any(|built_in| built_in.id == id)
}

/// A name as a file can be called, which is what an identifier has to be.
///
/// Lowercased and hyphenated so that the directory reads as one thing, and
/// letters of any alphabet kept: somebody writing their Actions in Russian
/// should not find them all called `action-4`.
fn slug(name: &str) -> String {
    let mut slug = String::with_capacity(name.len());

    for character in name.chars().flat_map(char::to_lowercase) {
        match character.is_alphanumeric() {
            true => slug.push(character),
            false if slug.ends_with('-') || slug.is_empty() => {}
            false => slug.push('-'),
        }
    }

    let slug = slug.trim_end_matches('-');

    match slug.is_empty() {
        // A name of nothing but punctuation is a name, and its Action still has
        // to go somewhere.
        true => "action".to_owned(),
        false => slug.to_owned(),
    }
}

/// Whether an identifier names a file that can be created and opened
/// everywhere Demysto runs.
///
/// The separators would put the file somewhere else entirely, a leading dot
/// hides it, and the reserved words are devices Windows will not let anything
/// be called whatever extension follows them.
fn usable_as_a_file_name(id: &str) -> bool {
    !id.is_empty()
        && !id.starts_with('.')
        && !id.contains(['/', '\\', ':'])
        && !id.chars().any(char::is_control)
        && !RESERVED.contains(&id.to_lowercase().as_str())
}

fn path(dir: &Path, id: &str) -> PathBuf {
    dir.join(format!("{id}.{EXTENSION}"))
}

/// A value somebody actually stated, trimmed, and `None` where there was
/// nothing but whitespace — so that a field the window sends empty is the user
/// clearing it rather than a Model bound to "".
fn stated_value(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

/// One Action as its file states it.
///
/// Every field but the version is optional, because one shape serves both
/// things the directory holds: an Action of the user's own states all of it,
/// and an Override of a built-in states only what it changes.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ActionFile {
    /// Absent in a file written by hand, which is the same as the first version.
    #[serde(default = "first_version")]
    version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    hotkey: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    accepts: Option<Vec<Kind>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    template: Option<String>,
    /// Last, because TOML puts every plain value of a table before the tables
    /// inside it, and this is the one field that is tables.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    parameters: Option<Vec<Parameter>>,
}

fn first_version() -> u32 {
    VERSION
}

impl ActionFile {
    /// Whether this file would say nothing about the Action it is filed under —
    /// which, for an Override, is the built-in back exactly as it was written.
    fn states_nothing(&self) -> bool {
        let Self {
            version: _,
            name,
            model,
            hotkey,
            accepts,
            template,
            parameters,
        } = self;

        name.is_none()
            && model.is_none()
            && hotkey.is_none()
            && accepts.is_none()
            && template.is_none()
            && parameters.is_none()
    }
}

fn unreadable(path: &Path, error: &io::Error) -> String {
    format!("{} could not be read: {error}", path.display())
}

fn unreadable_dir(dir: &Path, error: &io::Error) -> String {
    format!(
        "{} could not be read, so the Actions in it are not listed: {error}",
        dir.display()
    )
}

fn unwritable(path: &Path, error: &io::Error) -> String {
    format!("{} could not be written: {error}", path.display())
}

fn unwritable_shape(path: &Path, error: &toml::ser::Error) -> String {
    format!("{} could not be written as TOML: {error}", path.display())
}

/// What a parse failure says, and where — but never the line it happened on.
///
/// `toml`'s own `Display` quotes the offending source line back. An Action file
/// holds no key, but this sentence is shown in a window beside the settings
/// file's own errors, and `config::unparseable` withholds the line for a reason
/// that is worth applying to both rather than remembering the difference.
fn unparseable(path: &Path, text: &str, error: &toml::de::Error) -> String {
    let line = error
        .span()
        .and_then(|span| text.get(..span.start))
        .map(|before| before.matches('\n').count() + 1);

    match line {
        Some(line) => format!(
            "{} is not a valid Action at line {line}: {}",
            path.display(),
            error.message()
        ),
        None => format!(
            "{} is not a valid Action: {}",
            path.display(),
            error.message()
        ),
    }
}
