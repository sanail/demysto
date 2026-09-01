//! The words Demysto says, and which language it says them in.
//!
//! One Fluent catalogue per language in `i18n/`, compiled into the binary and
//! read by both layers: the windows import the same files through Vite, and
//! everything native — the tray menu, a notification, every sentence an error
//! carries across the channel — is said from here. Two sources of strings would
//! be two places to forget, and forgetting in this one shows up as a tray menu
//! in English under a Russian interface.
//!
//! Fluent rather than a flat map of strings, because Russian is why: "1 символ,
//! 2 символа, 5 символов" is not a plural rule a format string can express, and
//! a catalogue that cannot express it produces an interface that reads as
//! machine-translated (user story 60).
//!
//! What is deliberately *not* here: the prompt templates the built-in Actions
//! send. Those are addressed to a Model rather than to a person, and `language`
//! says why they name languages in English wherever they name one at all. Their
//! names and the Parameters they collect are interface, and are here.

use fluent::concurrent::FluentBundle;
use fluent::{FluentArgs, FluentResource};
use unic_langid::LanguageIdentifier;

/// The environment variable that fixes the interface language, whatever the
/// settings and the operating system say.
///
/// First in the order for the reason `DEMYSTO_CONFIG_DIR` is: somebody who
/// exports it has already said what they want, and a launcher or a script is
/// where they say it. Unlike the settings, nothing writes it back.
pub const LANGUAGE_ENV: &str = "DEMYSTO_LANGUAGE";

/// The language Demysto's interface speaks.
///
/// A closed set rather than a tag, because a catalogue is a file in this
/// repository: there is no language Demysto can be asked for that it does not
/// already hold the words of, and anything else is English (user story 58).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interface {
    English,
    German,
    Spanish,
    French,
    Russian,
}

/// Demysto's own words in one language, ready to be asked for one at a time.
///
/// Carries English underneath whenever it is not English itself. The suite
/// fails over a message one catalogue holds and another does not, so nothing
/// should ever reach that fallback — but a build that got there anyway should
/// say something a user can read rather than an identifier.
pub struct Words {
    interface: Interface,
    bundle: FluentBundle<FluentResource>,
    /// English, for a message this language turned out not to hold. `None` when
    /// this language *is* English, which is the only catalogue with nothing
    /// underneath it.
    beneath: Option<FluentBundle<FluentResource>>,
}

const ENGLISH: &str = include_str!("../../../i18n/en.ftl");
const GERMAN: &str = include_str!("../../../i18n/de.ftl");
const SPANISH: &str = include_str!("../../../i18n/es.ftl");
const FRENCH: &str = include_str!("../../../i18n/fr.ftl");
const RUSSIAN: &str = include_str!("../../../i18n/ru.ftl");

impl Interface {
    /// Every language Demysto speaks, in the order Settings offers them:
    /// English first, because it is the catalogue every other is written
    /// against and the one anything missing falls back to, and then the rest by
    /// what they call themselves.
    pub const ALL: [Self; 5] = [
        Self::English,
        Self::German,
        Self::Spanish,
        Self::French,
        Self::Russian,
    ];

    /// The tag the settings file states this language as, and the one the
    /// windows pick a catalogue by.
    pub fn tag(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::German => "de",
            Self::Spanish => "es",
            Self::French => "fr",
            Self::Russian => "ru",
        }
    }

    /// What this language calls itself.
    ///
    /// The same word in every catalogue, and therefore not in one: a list of
    /// languages written in the language somebody cannot read is a list they
    /// cannot use to get out of it.
    pub fn endonym(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::German => "Deutsch",
            Self::Spanish => "Español",
            Self::French => "Français",
            Self::Russian => "Русский",
        }
    }

    /// What a prompt calls this language.
    ///
    /// English names, for the reason `language::Language` gives: this goes to a
    /// Model rather than onto the screen, and a Model asked to answer in
    /// "Русский" is being told the same thing in a less reliable way than one
    /// asked to answer in "Russian".
    pub(crate) fn prompt_name(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::German => "German",
            Self::Spanish => "Spanish",
            Self::French => "French",
            Self::Russian => "Russian",
        }
    }

    /// The language a tag names, or `None` for one no catalogue answers to.
    ///
    /// Matched on the primary subtag alone, and on whatever the platform hands
    /// over: `ru`, `ru-RU` and the `ru_RU.UTF-8` a Unix `LANG` carries are one
    /// answer, because none of them is a different language.
    pub fn matching(tag: &str) -> Option<Self> {
        let primary = tag
            .split(['-', '_', '.', '@'])
            .next()
            .unwrap_or_default()
            .trim();

        Self::ALL
            .into_iter()
            .find(|language| primary.eq_ignore_ascii_case(language.tag()))
    }

    fn catalogue(self) -> &'static str {
        match self {
            Self::English => ENGLISH,
            Self::German => GERMAN,
            Self::Spanish => SPANISH,
            Self::French => FRENCH,
            Self::Russian => RUSSIAN,
        }
    }

    fn identifier(self) -> LanguageIdentifier {
        self.tag()
            .parse()
            .expect("a language Demysto holds a catalogue for has a tag Fluent can parse")
    }
}

/// The language the interface speaks, given everything that has a say in it.
///
/// In order, and the first that names a language Demysto holds wins: the
/// environment variable, the settings file, then the operating system. English
/// is what is left — a system language with no catalogue is not a reason to
/// refuse to start, or to make somebody choose before they have seen the
/// application (user story 58).
///
/// A source naming a language Demysto does not speak is passed over rather than
/// treated as English outright, so that `DEMYSTO_LANGUAGE=xh` on a Russian
/// desktop still gets Russian: the variable said nothing Demysto understood, and
/// the desktop did.
pub(crate) fn chosen(
    exported: Option<String>,
    stated: Option<&str>,
    system: Option<String>,
) -> Interface {
    [exported, stated.map(ToOwned::to_owned), system]
        .into_iter()
        .flatten()
        .find_map(|tag| Interface::matching(&tag))
        .unwrap_or(Interface::English)
}

/// Demysto's own words in English, which is what the suite reads its
/// assertions in — every module but this one is testing something other than
/// which language it is being said in.
#[cfg(test)]
pub(crate) fn english() -> Words {
    Words::spoken(Interface::English)
}

/// What the operating system says the user reads.
pub(crate) fn system_language() -> Option<String> {
    sys_locale::get_locale()
}

impl Words {
    /// The catalogue for one language, parsed.
    ///
    /// Parsed at every call rather than once into a static: this happens at
    /// startup and again when somebody changes the language in Settings, which
    /// is twice in a session that changes it, and a lock around a global is a
    /// larger thing to own than two parses of a file measured in kilobytes.
    pub fn spoken(interface: Interface) -> Self {
        Self {
            interface,
            bundle: bundle(interface),
            beneath: (interface != Interface::English).then(|| bundle(Interface::English)),
        }
    }

    /// The language these words are in.
    pub fn interface(&self) -> Interface {
        self.interface
    }

    /// One message, as it stands with nothing filled into it.
    pub fn text(&self, id: &str) -> String {
        self.filled(id, FluentArgs::new())
    }

    /// One message with its placeables filled in.
    ///
    /// Reached through [`say!`] rather than called, on both sides of the
    /// crate boundary: the macro is what keeps the identifier and its
    /// arguments next to each other where the suite can find them.
    pub fn filled(&self, id: &str, args: FluentArgs) -> String {
        self.held(&self.bundle, id, &args)
            .or_else(|| {
                self.beneath
                    .as_ref()
                    .and_then(|beneath| self.held(beneath, id, &args))
            })
            // Not a panic: a message the catalogues turn out not to hold is a
            // fault the suite is meant to have caught, and a window that says
            // an identifier is a bug report somebody can send. One that took
            // the application down with it is not.
            .unwrap_or_else(|| format!("[{id}]"))
    }

    fn held(
        &self,
        bundle: &FluentBundle<FluentResource>,
        id: &str,
        args: &FluentArgs,
    ) -> Option<String> {
        let pattern = bundle.get_message(id)?.value()?;
        let mut errors = Vec::new();
        let said = bundle.format_pattern(pattern, Some(args), &mut errors);

        // A pattern that resolved with errors is still a pattern that resolved,
        // and what it produced names what went wrong where the placeable was.
        // Which is the same bargain the missing message above makes.
        Some(said.into_owned())
    }
}

fn bundle(interface: Interface) -> FluentBundle<FluentResource> {
    let resource = FluentResource::try_new(interface.catalogue().to_owned())
        .expect("a catalogue in this repository parses as Fluent");

    let mut bundle = FluentBundle::new_concurrent(vec![interface.identifier()]);

    // Off, on both sides of the application. Fluent wraps every placeable in
    // the directional isolates U+2068 and U+2069 by default, which is right for
    // a browser laying out mixed scripts and wrong for everywhere Demysto puts
    // these words: a tray menu item, a system notification, a test that asserts
    // what a sentence says. The frontend turns it off for the same reason.
    bundle.set_use_isolating(false);

    bundle
        .add_resource(resource)
        .expect("a catalogue in this repository states each message once");

    bundle
}

/// Says one message, with whatever it needs filled into it.
///
/// `say!(words, "id")` for a sentence that stands on its own, and
/// `say!(words, "id", "name" = value)` for one that does not.
///
/// Exported, because the shell says things too: the tray menu, a notification
/// and every sentence about a Hotkey the desktop would not give up are all
/// native, and all of them come out of the same catalogue as the windows.
#[macro_export]
macro_rules! say {
    ($words:expr, $id:literal) => {
        $crate::Words::text($words, $id)
    };
    ($words:expr, $id:literal, $($name:literal = $value:expr),+ $(,)?) => {{
        let mut args = $crate::Args::new();
        $(args.set($name, $value);)+
        $crate::Words::filled($words, $id, args)
    }};
}

pub(crate) use crate::say;

#[cfg(test)]
mod tests {
    //! What keeps the catalogues honest.
    //!
    //! Two questions, and neither can be answered by reading one file: does
    //! every catalogue hold the same messages, and does every message the
    //! sources ask for exist? A translation is only ever missing by omission,
    //! and an omission is invisible at the place it was made — it shows up as
    //! one English sentence in a Russian window, months later, in front of
    //! somebody who is not going to file a bug about it.
    //!
    //! So the suite reads the repository. The catalogues are compared to each
    //! other, and both are compared to every `say!` in the Rust and every `t(`
    //! in the frontend. Reaching outside the crate is unusual and deliberate:
    //! the frontend reads these same two files, so a test that looked only at
    //! Rust would be checking half of the thing the ticket is about.

    use std::collections::BTreeSet;
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::*;

    /// The repository, from the crate this test is compiled in.
    fn repository() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("the crate sits inside the repository")
    }

    /// Every message one catalogue states, by identifier.
    fn stated(catalogue: &str) -> BTreeSet<String> {
        fluent_syntax::parser::parse(catalogue)
            .expect("a catalogue in this repository parses as Fluent")
            .body
            .into_iter()
            .filter_map(|entry| match entry {
                fluent_syntax::ast::Entry::Message(message) => Some(message.id.name.to_owned()),
                _ => None,
            })
            .collect()
    }

    /// Every message the sources ask for, wherever they ask for it.
    fn asked_for() -> BTreeSet<String> {
        let repository = repository();
        let mut asked = BTreeSet::new();

        // `say!` and `Words::text` on this side of the channel, and `t` on the
        // other: three ways of asking for a message, and every one of them puts
        // the identifier in the first string after the opener.
        for (directory, extensions, openers) in [
            (
                "crates/demysto-core/src",
                &["rs"][..],
                &["say!(", ".text("][..],
            ),
            ("src-tauri/src", &["rs"][..], &["say!(", ".text("][..]),
            ("src", &["ts", "svelte"][..], &["t("][..]),
        ] {
            for file in sources(&repository.join(directory), extensions) {
                // Not this file: the macro's own documentation shows how it is
                // called, and an example is not a message anybody says.
                if file.ends_with("i18n.rs") {
                    continue;
                }

                let text = fs::read_to_string(&file).expect("a source file in this repository");

                for opener in openers {
                    asked.extend(identifiers(&text, opener));
                }
            }
        }

        asked
    }

    /// Every source file under `dir` with one of these extensions.
    fn sources(dir: &Path, extensions: &[&str]) -> Vec<PathBuf> {
        let mut found = Vec::new();

        for entry in fs::read_dir(dir).expect("a source directory in this repository") {
            let path = entry.expect("a readable directory entry").path();

            if path.is_dir() {
                found.extend(sources(&path, extensions));
            } else if path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extensions.contains(&extension))
            {
                found.push(path);
            }
        }

        found
    }

    /// The identifiers every `opener` in `text` is followed by.
    ///
    /// A scan rather than a parse: what is being looked for is one literal in a
    /// fixed position — the first string after `say!(words,` and the first
    /// after `t(` — and everything that is not that shape is skipped rather
    /// than guessed at.
    fn identifiers(text: &str, opener: &str) -> Vec<String> {
        let mut found = Vec::new();
        let mut rest = text;

        while let Some(at) = rest.find(opener) {
            let before = rest[..at].chars().next_back();
            let after = &rest[at + opener.len()..];
            rest = after;

            // `split(`, `format!(` and every other call whose name happens to
            // end in the opener's first letter. A message is asked for by the
            // macro or by the function, and neither is the tail of a longer
            // name — unless the opener begins with the separator itself, which
            // `.text(` does, and which is preceded by whatever it is called on.
            let joined = opener.starts_with('.');

            if !joined
                && before.is_some_and(|char| char.is_alphanumeric() || char == '_' || char == '.')
            {
                continue;
            }

            // `say!` takes the words first; `t` takes the identifier straight
            // away. Either way what is wanted is the first string literal, and
            // only when nothing but the argument separator stands before it.
            let literal = match after.find('"') {
                Some(quote)
                    if after[..quote].chars().all(|char| {
                        char.is_whitespace() || char.is_alphanumeric() || ",_&*.".contains(char)
                    }) =>
                {
                    &after[quote + 1..]
                }
                _ => continue,
            };

            if let Some(end) = literal.find('"') {
                let id = &literal[..end];

                if !id.is_empty()
                    && id.chars().all(|char| {
                        char.is_ascii_lowercase() || char.is_ascii_digit() || char == '-'
                    })
                {
                    found.push(id.to_owned());
                }
            }
        }

        found
    }

    /// Every variable one message names, wherever in it they are named.
    fn named(catalogue: &str, id: &str) -> BTreeSet<String> {
        fn walk(pattern: &fluent_syntax::ast::Pattern<&str>, found: &mut BTreeSet<String>) {
            use fluent_syntax::ast::{Expression, InlineExpression, PatternElement};

            for element in &pattern.elements {
                let PatternElement::Placeable { expression } = element else {
                    continue;
                };

                let inline = match expression {
                    Expression::Inline(inline) => inline,
                    Expression::Select { selector, variants } => {
                        for variant in variants {
                            walk(&variant.value, found);
                        }

                        selector
                    }
                };

                if let InlineExpression::VariableReference { id } = inline {
                    found.insert(id.name.to_owned());
                }
            }
        }

        let mut found = BTreeSet::new();

        for entry in fluent_syntax::parser::parse(catalogue)
            .expect("a catalogue in this repository parses as Fluent")
            .body
        {
            if let fluent_syntax::ast::Entry::Message(message) = entry {
                if message.id.name == id {
                    if let Some(value) = &message.value {
                        walk(value, &mut found);
                    }
                }
            }
        }

        found
    }

    /// A translation may leave a variable out — German says "Zeichen" for one
    /// and for many, and needs no count to choose with — but it may not name one
    /// English does not have. That is a misspelling rather than a decision, and
    /// what it produces is a sentence with the placeable's own name in the hole
    /// where the path or the reason was meant to be.
    #[test]
    fn no_catalogue_names_a_variable_english_does_not() {
        for id in stated(ENGLISH) {
            let english = named(ENGLISH, &id);

            for language in Interface::ALL {
                let held = named(language.catalogue(), &id);

                assert_eq!(
                    held.difference(&english).collect::<Vec<_>>(),
                    Vec::<&String>::new(),
                    "{} names something in {id} that English does not",
                    language.tag()
                );
            }
        }
    }

    /// What one hand-written list in the frontend holds, between its brackets.
    fn listed(file: &str, opener: &str, open: char, close: char) -> String {
        let text =
            fs::read_to_string(repository().join(file)).expect("a source file in this repository");
        let from = text
            .find(opener)
            .unwrap_or_else(|| panic!("{file} states {opener}"));
        let body = &text[from..];
        let body = &body[body.find(open).expect("the list opens") + 1..];

        body[..body.find(close).expect("the list closes")].to_owned()
    }

    /// The string after each `key` in one such list, in the order they appear.
    fn quoted(listing: &str, key: &str) -> Vec<String> {
        let mut found = Vec::new();
        let mut rest = listing;

        while let Some(at) = rest.find(key) {
            let after = &rest[at + key.len()..];
            let end = after.find('"').expect("a quoted value is closed");

            found.push(after[..end].to_owned());
            rest = &after[end..];
        }

        found
    }

    /// A language the core speaks has to be one the windows can read and one
    /// they offer, and both of those are lists somebody writes by hand.
    /// Nothing else can catch a language added to `Interface` alone: it is not
    /// a build failure, it is four windows staying English while the tray menu
    /// changes underneath them, for the one user who chose that language.
    #[test]
    fn the_frontend_holds_every_language_the_core_speaks() {
        let spoken: Vec<&str> = Interface::ALL.into_iter().map(Interface::tag).collect();

        let catalogues: Vec<String> =
            listed("src/lib/i18n.svelte.ts", "const CATALOGUES", '{', '}')
                .split(',')
                .filter_map(|entry| entry.split(':').next())
                .map(str::trim)
                .filter(|tag| !tag.is_empty())
                .map(ToOwned::to_owned)
                .collect();

        assert_eq!(catalogues, spoken, "the catalogues the windows import");

        let offered = listed("src/lib/languages.ts", "export const LANGUAGES", '[', ']');

        assert_eq!(
            quoted(&offered, "tag: \""),
            spoken,
            "what the windows offer"
        );

        // And by the name each language calls itself, which is the whole of
        // what somebody who cannot read the current one has to go on.
        assert_eq!(
            quoted(&offered, "name: \""),
            Interface::ALL
                .into_iter()
                .map(Interface::endonym)
                .collect::<Vec<_>>(),
            "what the windows call them"
        );
    }

    #[test]
    fn every_catalogue_states_the_same_messages() {
        let english = stated(ENGLISH);

        for language in Interface::ALL {
            let held = stated(language.catalogue());

            assert_eq!(
                english.difference(&held).collect::<Vec<_>>(),
                Vec::<&String>::new(),
                "{} is missing messages English holds",
                language.tag()
            );
            assert_eq!(
                held.difference(&english).collect::<Vec<_>>(),
                Vec::<&String>::new(),
                "{} holds messages English does not",
                language.tag()
            );
        }
    }

    #[test]
    fn every_message_the_sources_ask_for_is_in_the_catalogues() {
        let english = stated(ENGLISH);
        let missing: Vec<String> = asked_for()
            .into_iter()
            .filter(|id| !english.contains(id))
            .collect();

        assert_eq!(missing, Vec::<String>::new());
    }

    #[test]
    fn every_message_the_catalogues_state_is_asked_for() {
        let asked = asked_for();
        let unused: Vec<String> = stated(ENGLISH)
            .into_iter()
            .filter(|id| !asked.contains(id))
            .collect();

        assert_eq!(unused, Vec::<String>::new());
    }

    #[test]
    fn a_tag_names_the_language_it_names_however_the_platform_writes_it() {
        for tag in ["ru", "RU", "ru-RU", "ru_RU.UTF-8", "ru@petr1708"] {
            assert_eq!(Interface::matching(tag), Some(Interface::Russian), "{tag}");
        }

        // A region is not a language, and every one of these is somebody whose
        // desktop would otherwise have come up in English.
        assert_eq!(Interface::matching("en-GB"), Some(Interface::English));
        assert_eq!(Interface::matching("de-AT"), Some(Interface::German));
        assert_eq!(Interface::matching("es_MX.UTF-8"), Some(Interface::Spanish));
        assert_eq!(Interface::matching("fr-CA"), Some(Interface::French));
    }

    /// User story 58: a system language Demysto does not speak is English
    /// rather than a question the user has to answer before they have seen the
    /// application.
    #[test]
    fn a_language_no_catalogue_holds_is_english() {
        assert_eq!(Interface::matching("xh"), None);
        assert_eq!(
            chosen(None, None, Some("xh-ZA".to_owned())),
            Interface::English
        );
        assert_eq!(chosen(None, None, None), Interface::English);
    }

    #[test]
    fn the_environment_is_asked_before_the_settings_and_the_settings_before_the_system() {
        assert_eq!(
            chosen(Some("en".to_owned()), Some("ru"), Some("ru-RU".to_owned())),
            Interface::English
        );
        assert_eq!(
            chosen(None, Some("en"), Some("ru-RU".to_owned())),
            Interface::English
        );
        assert_eq!(
            chosen(None, None, Some("ru-RU".to_owned())),
            Interface::Russian
        );
    }

    /// A source that names a language Demysto does not speak has said nothing,
    /// rather than said English: the desktop underneath it may still know.
    #[test]
    fn a_source_naming_a_language_demysto_does_not_speak_is_passed_over() {
        assert_eq!(
            chosen(Some("xh".to_owned()), None, Some("ru-RU".to_owned())),
            Interface::Russian
        );
    }

    /// The warning about a large Selection, in one language, for a count.
    ///
    /// The one message every catalogue has to get grammatically right, and the
    /// reason the catalogues are Fluent rather than a map of format strings.
    fn counted(language: Interface, characters: u64) -> String {
        say!(
            &Words::spoken(language),
            "run-large-selection",
            "characters" = characters,
            "shown" = characters.to_string(),
            "limit" = "100",
            "setting" = "large_selection",
            "path" = "settings.toml",
        )
    }

    /// User story 60. One, few and many are three different words in Russian,
    /// and a catalogue that cannot tell them apart is what makes an interface
    /// read as machine-translated.
    #[test]
    fn russian_counts_read_grammatically() {
        let said = |characters| counted(Interface::Russian, characters);

        assert!(said(1).contains("1 символ,"), "{}", said(1));
        assert!(said(2).contains("2 символа,"), "{}", said(2));
        assert!(said(5).contains("5 символов,"), "{}", said(5));
        assert!(said(21).contains("21 символ,"), "{}", said(21));
        assert!(said(112).contains("112 символов,"), "{}", said(112));
    }

    #[test]
    fn english_counts_read_grammatically() {
        let said = |characters| counted(Interface::English, characters);

        assert!(said(1).contains("1 character "), "{}", said(1));
        assert!(said(2).contains("2 characters "), "{}", said(2));
    }

    /// And so do the three added after, each in its own way: Spanish splits one
    /// from many where English does, French counts zero as one, and German has
    /// a single word for any number of them — which is why its message carries
    /// no selector at all, and why that is checked here rather than assumed.
    #[test]
    fn the_languages_added_later_count_grammatically() {
        let spanish = |characters| counted(Interface::Spanish, characters);

        assert!(spanish(1).contains("1 carácter,"), "{}", spanish(1));
        assert!(spanish(2).contains("2 caracteres,"), "{}", spanish(2));

        let french = |characters| counted(Interface::French, characters);

        assert!(french(0).contains("0 caractère,"), "{}", french(0));
        assert!(french(1).contains("1 caractère,"), "{}", french(1));
        assert!(french(2).contains("2 caractères,"), "{}", french(2));

        let german = |characters| counted(Interface::German, characters);

        assert!(german(1).contains("1 Zeichen lang"), "{}", german(1));
        assert!(german(2).contains("2 Zeichen lang"), "{}", german(2));
    }

    /// User story 56: the sentence is the whole of what a Wayland user is
    /// given, so it has to name the limitation and the way round it — in every
    /// language. One that says only that something is unavailable reads as a
    /// broken tool.
    ///
    /// Asserted here rather than in `desktop`, which used to hold the sentence:
    /// that module's business is which session gets it, and there is no longer
    /// one sentence for it to check.
    #[test]
    fn the_clipboard_only_sentence_says_what_to_do_instead() {
        for language in Interface::ALL {
            let said = Words::spoken(language).text("capture-clipboard-only");

            // The two things a translation cannot translate away: what is
            // imposing the limitation, and the keys that get round it.
            assert!(said.contains("Wayland"), "{said}");
            assert!(said.contains("Ctrl+C"), "{said}");
        }
    }

    /// And the one macOS imposes has to name the pane it is granted in, because
    /// a notification is one of the places it is read and there is no button
    /// beside it there (user story 55).
    #[test]
    fn the_accessibility_sentence_names_where_the_permission_is_granted() {
        for language in Interface::ALL {
            let said = Words::spoken(language).text("capture-no-accessibility");

            assert!(said.contains("macOS"), "{said}");
            assert!(said.contains("Demysto"), "{said}");
        }
    }

    /// Fluent wraps a placeable in directional isolates unless told not to, and
    /// a tray menu item carrying two invisible control characters is a tray
    /// menu item nobody can search the source for.
    #[test]
    fn nothing_carries_the_isolates_fluent_adds_by_default() {
        for language in Interface::ALL {
            let words = Words::spoken(language);
            let said = say!(
                &words,
                "result-open-provider-settings",
                "provider" = "openai"
            );

            assert!(said.contains("openai"), "{said}");
            assert!(!said.contains('\u{2068}'), "{said:?}");
        }
    }
}
