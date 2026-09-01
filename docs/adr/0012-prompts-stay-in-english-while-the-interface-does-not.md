# ADR-0012: A built-in Action's prompt stays in English; everything the user reads does not

Status: accepted

## Context

Ticket 14 gives Demysto a catalogue per language and asks that no string be left
untranslated. A built-in Action has four user-facing pieces: its name, the label
of any Parameter it collects, the default that Parameter offers, and the prompt
template it sends. The first three are read by the user, in the Palette. The
fourth is read by a Model.

The obvious reading of "no untranslated string" translates all four, and a
Russian interface would then send

> Объясни текст ниже… Текст на языке {{selection_language}}; отвечай на
> {{ui_language}}.

where the English interface sends the English wording.

## Decision

The names, labels and defaults come from the catalogue. **The prompt templates
do not**: they stay in `action`, in English, in every interface language. The
`{{ui_language}}` they interpolate is the interface language by its English
name — "Russian", not "Русский" — which is the rule `language` already applies
to the language a Selection is detected as.

An Override still holds whatever the user wrote, in whatever language they wrote
it. This is about what Demysto ships, not about what the user may write.

## Consequences

A Russian user who opens the built-in Explain in Settings sees an English
prompt. That is the cost, and it is real: the prompt is editable, so it is
interface in the weak sense that it appears in a window.

What it buys is answer quality that does not vary by interface language.
Instruction-following is measurably better in English across the models Demysto
targets, and a prompt is a program rather than a sentence: "do not repeat the
text back" is an instruction whose translation can drift in ways nobody notices
until an answer is worse. Translating it would make the interface language a
hidden variable in what the Model produces, and the one thing a user changing
their interface language is not asking for is different answers.

Naming the language in English inside the prompt is the same argument at one
remove, and is already load-bearing: `language::detect` reports "German" rather
than "Deutsch" for exactly this reason.

The line, then, is who reads the string. A person reads the name, the label and
the default, and those are translated. A Model reads the template, and it is
not. The suite holds the line: `i18n::tests` fails over any identifier a
catalogue is missing, and the templates are not identifiers in it.
