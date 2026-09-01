/**
 * The languages Demysto speaks, each written in its own name.
 *
 * The endonyms rather than translations of them: a list of languages written in
 * the language somebody cannot read is a list they cannot use to get out of it.
 * Held here rather than in a catalogue for the same reason — they are the same
 * words in every one.
 *
 * `Interface::ALL` in the core, in its order: English first, then the rest by
 * what they call themselves. Written by hand, and checked against the core by
 * `i18n::tests` — a language added there and forgotten here is a window with no
 * way to choose it.
 */
export const LANGUAGES = [
  { tag: "en", name: "English" },
  { tag: "de", name: "Deutsch" },
  { tag: "es", name: "Español" },
  { tag: "fr", name: "Français" },
  { tag: "ru", name: "Русский" },
];
