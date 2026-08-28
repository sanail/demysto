//! The languages a prompt names: the one Demysto's interface speaks, and the
//! one a Selection turns out to be written in.

/// A language, by the name a prompt calls it.
///
/// English names throughout, because the name goes into a prompt rather than
/// onto the screen: a Model asked to answer in "Русский" is being told the same
/// thing in a less reliable way than one asked to answer in "Russian".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Language(&'static str);

impl Language {
    /// The language the interface speaks, and the one an explanation is asked
    /// for — user story 30, where reading foreign material does not mean
    /// reading a foreign explanation.
    ///
    /// Fixed here rather than taken from the operating system: ticket 14 owns
    /// the catalogue, the system's choice, the Settings override and the
    /// environment variable, and this constant is the one thing it replaces.
    pub(crate) const INTERFACE: Self = Self("English");

    /// What a Selection nothing can be told from is called in a prompt.
    ///
    /// A sentence rather than a name, because that is how it reads where it
    /// lands: "The text is in an unknown language".
    const UNKNOWN: Self = Self("an unknown language");

    pub(crate) fn name(self) -> &'static str {
        self.0
    }
}

/// The language a Selection is written in, as far as it can be told.
///
/// Only a detection the detector itself calls reliable is taken. That is a
/// strict bar, and deliberately so: it is cleared by the paragraph or sentence
/// somebody actually hits a wall on, and not by the fragments where a guess
/// goes wrong — "borrow checker" is detected as Shona, and `let x = &mut y;` as
/// Hungarian. A prompt told the wrong language argues with what the Model can
/// plainly see, while one told nothing simply reads what is in front of it.
pub(crate) fn detect(text: &str) -> Language {
    whatlang::detect(text)
        .filter(whatlang::Info::is_reliable)
        .map_or(Language::UNKNOWN, |info| Language(info.lang().eng_name()))
}
