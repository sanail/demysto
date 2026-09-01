/**
 * The words this window says, out of the same catalogue the Rust layer reads.
 *
 * Not a copy of it and not a shape derived from it: Vite imports `i18n/*.ftl`
 * as text and `@fluent/bundle` parses it here, while `demysto-core` compiles
 * the very same files in with `include_str!`. One file per language, and the
 * suite fails the build over an identifier one catalogue holds and another does
 * not — see `i18n::tests` in the core.
 *
 * Which language is the backend's answer, because the backend is where it is
 * decided: the environment variable, then the settings file, then the operating
 * system. A window asks once as it mounts, and is told again whenever a save
 * changes it.
 */

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { FluentBundle, FluentResource } from "@fluent/bundle";
import english from "../../i18n/en.ftl?raw";
import german from "../../i18n/de.ftl?raw";
import spanish from "../../i18n/es.ftl?raw";
import french from "../../i18n/fr.ftl?raw";
import russian from "../../i18n/ru.ftl?raw";

/**
 * The catalogues, by the tag the backend names one with.
 *
 * The same set as `Interface` in the core, and in the same order. A language
 * added there and forgotten here is a window that stays English while the tray
 * menu changes — which is the whole failure this file exists to avoid, and the
 * one thing the suite cannot see, because it reads the catalogues rather than
 * this list.
 */
const CATALOGUES: Record<string, string> = {
  en: english,
  de: german,
  es: spanish,
  fr: french,
  ru: russian,
};

/** What a language with no catalogue of its own falls back to. */
const ENGLISH = "en";

/** Emitted by the backend when a save changes the language. */
const LANGUAGE_EVENT = "language://spoken";

/** What a message may have filled into it. */
export type Filling = Record<string, string | number>;

/**
 * The language being spoken, as something a template can depend on.
 *
 * Read by [`t`] before it says anything, which is what makes every string in
 * every window follow a change without the window being reopened: Svelte re-runs
 * whatever read this.
 *
 * English before the backend has answered, so that a window has something to
 * say from its first frame — the Palette is drawn on a Hotkey press, and one
 * that waited on a round trip to know what to call a button would be a Palette
 * that flickers.
 */
const spoken = $state({ tag: ENGLISH });

function bundled(tag: string): FluentBundle {
  const bundle = new FluentBundle(tag, {
    // Off, as it is on the Rust side and for the same reason: Fluent wraps
    // every placeable in the directional isolates U+2068 and U+2069 by default,
    // and a button whose label carries two invisible control characters is a
    // button nobody can search the source for.
    useIsolating: false,
  });

  bundle.addResource(new FluentResource(CATALOGUES[tag] ?? english));

  return bundle;
}

/** The catalogue being read, and English underneath it. */
let speaking = bundled(ENGLISH);
let beneath = speaking;

/**
 * One message, with whatever it needs filled into it.
 *
 * Answers with the identifier in brackets for a message no catalogue holds,
 * rather than throwing: that is a fault the suite is meant to have caught, and
 * a window that says an identifier is a bug report somebody can send — one that
 * failed to render is not.
 */
export function t(id: string, filling?: Filling): string {
  // The dependency, and the whole of why this is a function rather than a
  // lookup: nothing below reads it, and everything that called this re-runs
  // when it changes.
  void spoken.tag;

  const held = speaking.getMessage(id);
  const message = held ?? beneath.getMessage(id);

  if (!message?.value) return `[${id}]`;

  // The third argument is what keeps this from throwing. `@fluent/bundle`
  // rethrows a resolution error when it has nowhere to put one — a message
  // whose placeable nothing filled in would take the whole render down — and
  // collecting them instead leaves the partial string, which says where it went
  // wrong. The Rust side makes the same bargain in `Words::held`.
  return (held ? speaking : beneath).formatPattern(message.value, filling, []);
}

/**
 * The tag of the language being spoken, for whatever has to notice it changing
 * rather than merely be said in it.
 *
 * Reactive like [`t`]: an effect that reads this re-runs when the language
 * changes.
 */
export function spokenTag(): string {
  return spoken.tag;
}

/**
 * Takes the language from the backend, and follows it from then on.
 *
 * Awaited before a window mounts, so that nothing is drawn twice: the first
 * frame is already in the right language. The listener afterwards is what lets
 * a Conversation left open behind Settings change language with everything else
 * (user story 59).
 */
export async function speak(): Promise<void> {
  adopt(await invoke<string>("language"));

  await listen<string>(LANGUAGE_EVENT, (event) => adopt(event.payload));
}

function adopt(tag: string) {
  const held = tag in CATALOGUES ? tag : ENGLISH;
  if (held === spoken.tag) return;

  speaking = bundled(held);
  beneath = held === ENGLISH ? speaking : bundled(ENGLISH);
  spoken.tag = held;

  // The document's own language, which nothing on screen shows and a screen
  // reader reads everything through: a page marked `lang="en"` holding Russian
  // is a page read out in the wrong voice, letter by letter.
  document.documentElement.lang = held;
}
