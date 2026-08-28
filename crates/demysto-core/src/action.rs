//! The Action catalogue, and the prompt an Action assembles around a Selection.
//!
//! Built-in Actions are compiled in rather than seeded onto disk, per ADR-0005:
//! the config directory belongs to the user, and an Action written into it can
//! never be improved by a later version. Ticket 09 puts user-authored Actions
//! and Overrides of these alongside them; the shape is already the one they get,
//! because a built-in Action is not a privileged variety of anything.

use std::collections::BTreeMap;

use crate::language::{self, Language};
use crate::selection::{Kind, Selection};

/// What a template writes around the value it wants.
const OPEN: &str = "{{";
const CLOSE: &str = "}}";

/// The Selection the Run operates on.
const SELECTION: &str = "selection";
/// The language Demysto's interface speaks.
const UI_LANGUAGE: &str = "ui_language";
/// The language the Selection turns out to be written in.
const SELECTION_LANGUAGE: &str = "selection_language";

/// A named, user-runnable operation.
///
/// Serialised for the Palette, which needs the name it lists and the Parameters
/// it collects. What the Action accepts and what it says to the Model stay
/// here: the Palette is handed the Actions that already accept what was
/// captured, and the prompt is nobody's business but the Run's.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Action {
    /// What an Action is asked for by. Stable across renamings and across
    /// interface languages, because Overrides and Hotkeys are keyed on it.
    pub id: String,
    /// What the Palette lists, and what typing in it filters on.
    pub name: String,
    /// What the Palette collects before the Run starts.
    pub parameters: Vec<Parameter>,
    /// The Model this Action runs on whatever the defaults say, `None` when it
    /// takes whichever Model resolution arrives at. No built-in binds
    /// one — a built-in that insisted on somebody's expensive Model would be a
    /// built-in most people could not run — and ticket 09 lets an Override give
    /// one to any Action.
    #[serde(skip)]
    pub(crate) model: Option<String>,
    #[serde(skip)]
    accepts: Vec<Kind>,
    #[serde(skip)]
    template: String,
}

/// A value an Action declares and collects before running, beyond the Selection
/// itself.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Parameter {
    /// What the template refers to it by, written `{{like_this}}`.
    pub id: String,
    /// What the Palette asks the user for it.
    pub label: String,
    /// What the field holds before the user types, and therefore what is used
    /// when they type nothing. Empty when there is nothing sensible to offer.
    pub default: String,
}

/// The built-in Actions, in the order the Palette lists them.
///
/// Ordered by how often they are reached for rather than alphabetically: the
/// first is the one Enter runs without the user reading anything.
pub(crate) fn built_in() -> Vec<Action> {
    vec![
        Action {
            id: "explain".to_owned(),
            name: "Explain".to_owned(),
            parameters: Vec::new(),
            model: None,
            accepts: vec![Kind::Text],
            template: EXPLAIN.to_owned(),
        },
        Action {
            id: "translate".to_owned(),
            name: "Translate".to_owned(),
            parameters: vec![Parameter {
                id: "target".to_owned(),
                label: "Into which language?".to_owned(),
                // The overwhelmingly common translation is into the language
                // the user reads, so the field comes up holding it and the
                // Action costs one more keystroke than one with no Parameter
                // at all. Typing over it is what the other cases are for.
                default: Language::INTERFACE.name().to_owned(),
            }],
            model: None,
            accepts: vec![Kind::Text],
            template: TRANSLATE.to_owned(),
        },
        Action {
            id: "summarize".to_owned(),
            name: "Summarize".to_owned(),
            parameters: Vec::new(),
            model: None,
            accepts: vec![Kind::Text],
            template: SUMMARIZE.to_owned(),
        },
    ]
}

/// The Action an interface asked for, or `None` when there is no such Action.
pub(crate) fn named(id: &str) -> Option<Action> {
    built_in().into_iter().find(|action| action.id == id)
}

impl Action {
    /// Whether this Action will run on a Selection of `kind`, which is what
    /// keeps it out of a Palette where it could not run.
    pub(crate) fn accepts(&self, kind: Kind) -> bool {
        self.accepts.contains(&kind)
    }

    /// The prompt this Action sends for `selection`, with `given` standing in
    /// for the Parameters the Palette collected.
    pub(crate) fn prompt(&self, selection: &Selection, given: &BTreeMap<String, String>) -> String {
        render(&self.template, |name| match name {
            SELECTION => Some(selection.as_text().to_owned()),
            UI_LANGUAGE => Some(Language::INTERFACE.name().to_owned()),
            // Detected here rather than at Capture, and only when a template
            // asks: most Actions never mention it, and every Selection would
            // otherwise be read twice for a variable nobody used.
            SELECTION_LANGUAGE => Some(language::detect(selection.as_text()).name().to_owned()),
            _ => self
                .parameters
                .iter()
                .find(|parameter| parameter.id == name)
                .map(|parameter| collected(parameter, given)),
        })
    }
}

/// What the user gave for a Parameter, or what it offered when they gave
/// nothing. Whitespace is nothing: a field cleared and left is a field the user
/// had no answer for.
fn collected(parameter: &Parameter, given: &BTreeMap<String, String>) -> String {
    given
        .get(&parameter.id)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .unwrap_or(&parameter.default)
        .to_owned()
}

/// Substitutes every `{{name}}` a template holds.
///
/// A name nothing answers to is left standing rather than dropped. None of the
/// templates here has one; ticket 09 lets the user write one, and a Run that
/// quietly deleted part of their prompt would be harder to find than one whose
/// answer shows the mistake back to them.
fn render(template: &str, mut resolve: impl FnMut(&str) -> Option<String>) -> String {
    let mut prompt = String::with_capacity(template.len());
    let mut rest = template;

    while let Some(open) = rest.find(OPEN) {
        let (text, from_open) = rest.split_at(open);
        prompt.push_str(text);

        let after_open = &from_open[OPEN.len()..];
        let Some(close) = after_open.find(CLOSE) else {
            // Braces that never close are not a variable, they are text.
            prompt.push_str(from_open);
            return prompt;
        };

        match resolve(after_open[..close].trim()) {
            Some(value) => prompt.push_str(&value),
            None => prompt.push_str(&from_open[..OPEN.len() + close + CLOSE.len()]),
        }

        rest = &after_open[close + CLOSE.len()..];
    }

    prompt.push_str(rest);
    prompt
}

const EXPLAIN: &str = "\
Explain the text below to somebody who has just run into it while reading. Say \
what it means and unpack anything in it that is not obvious. Be brief and \
concrete, and do not repeat the text back. The text is in {{selection_language}}; \
answer in {{ui_language}}.

{{selection}}";

const TRANSLATE: &str = "\
Translate the text below into {{target}}. Answer with the translation and \
nothing else: no commentary, no notes, no transliteration, and no repetition of \
the original. Keep whatever formatting the text already has.

{{selection}}";

const SUMMARIZE: &str = "\
Summarize the text below for somebody deciding whether to read it. Lead with \
what it is about, then the points that carry it. Be brief, and prefer the text's \
own terms to paraphrases of them. The text is in {{selection_language}}; answer \
in {{ui_language}}.

{{selection}}";
