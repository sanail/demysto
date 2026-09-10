/**
 * What this window holds while it is being edited, and how that turns back
 * into what the backend is told.
 *
 * Kept out of the panels that show it because the window saves what several of
 * them hold in one write: a Provider is edited on one panel and written by the
 * button under all of them.
 */

import type {
  ActionEdit,
  ActionStanding,
  ConfiguredModel,
  ConfiguredProvider,
  KeyEdit,
  KeyStanding,
  ProviderEdit,
} from "../lib/ipc";

/**
 * One Provider as this window has it: what will be written, plus what is only
 * ever on screen — where its key already is, what it answered when it was
 * last asked something, and whether it is being asked something now.
 */
export type Draft = {
  was: string | null;
  name: string;
  base_url: string;
  preset: string;
  api_key_env: string;
  models: ConfiguredModel[];
  /**
   * Where the key the file holds is. Never the key: this window is the one
   * ADR-0002 promises the key does not enter, and a field showing it would
   * be that promise broken for the sake of showing somebody their own
   * secret back.
   */
  standing: KeyStanding;
  /** What has been typed into the key field, which is the only way one gets in. */
  typed: string;
  /** Whether the file's own key is to be taken out on the next save. */
  forgetting: boolean;
  /** What this Provider said it offers, once somebody asked it. */
  offered: string[] | null;
  /** Which of its Models a verification puts its request to. */
  trying: string;
  /**
   * Which question is with the Provider now, `null` while none is.
   *
   * Named rather than counted, because the two questions are asked from two
   * places and each answer belongs under the one that asked it: the Models a
   * Provider offers are an extension of the list above, and whether a key
   * works is about the key.
   */
  asking: Asked | null;
  /** What each question last got back, and whether that was good news. */
  said: Record<Asked, Said | null>;
};

/** The two things this window asks a Provider. */
export type Asked = "models" | "key";

export type Said = { well: boolean; message: string };

/**
 * One Action being edited, and where its definition stood before the editing
 * began — `null` for one being written, which stands nowhere yet.
 *
 * One at a time, and not a draft each: an Action is a file of its own and is
 * saved on its own, so there is never more than one unsaved.
 */
export type Editing = {
  draft: ActionEdit;
  standing: ActionStanding | null;
  /**
   * The draft as it was opened, written out, so that what has been done to it
   * since can be told from what has not. An Action opened and read is not an
   * Action with something in it to save, and a mark saying otherwise would be
   * one nobody could ever clear.
   */
  asOpened: string;
};

/** Whether an Action being edited has been changed since it was opened. */
export function changed(editing: Editing | null): boolean {
  return editing !== null && JSON.stringify(editing.draft) !== editing.asOpened;
}

export function drafted(provider: ConfiguredProvider): Draft {
  return {
    was: provider.name,
    name: provider.name,
    base_url: provider.base_url ?? "",
    preset: provider.preset ?? "",
    api_key_env: provider.api_key_env ?? "",
    models: provider.models.map((model) => ({ ...model })),
    standing: provider.key,
    typed: "",
    forgetting: false,
    offered: null,
    trying: provider.models[0]?.id ?? "",
    asking: null,
    said: { models: null, key: null },
  };
}

/** What a Provider being written from nothing starts as. */
export function fresh(): Draft {
  return {
    was: null,
    name: "",
    base_url: "",
    preset: "",
    api_key_env: "",
    models: [],
    standing: { state: "missing" },
    typed: "",
    forgetting: false,
    offered: null,
    trying: "",
    asking: null,
    said: { models: null, key: null },
  };
}

/** What of a draft gets written. */
export function edited(draft: Draft): ProviderEdit {
  return {
    was: draft.was,
    name: draft.name,
    base_url: draft.base_url,
    preset: draft.preset,
    api_key_env: draft.api_key_env,
    api_key: key(draft),
    models: draft.models,
  };
}

/**
 * What a save does to this Provider's key. Typing one replaces whatever the
 * file holds; typing nothing leaves it alone, which is the ordinary case and
 * the reason the field can start empty at all.
 */
function key(draft: Draft): KeyEdit {
  if (draft.typed.trim() !== "") return { action: "set", key: draft.typed };

  return draft.forgetting ? { action: "forget" } : { action: "keep" };
}
