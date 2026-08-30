import { Channel, invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

/** Mirrors `demysto_core::Status`. */
export type Status = {
  version: string;
  config_dir: string;
  /**
   * How long a Selection may be before Demysto says so, where the settings
   * state nothing — so that the window can name it rather than repeat a number
   * of its own.
   */
  large_selection_default: number;
};

/** Mirrors `demysto_core::Selection`. */
export type Selection = { kind: "text"; text: string };

/** Mirrors `demysto_core::Captured`. */
export type Captured =
  | { origin: "selection"; selection: Selection }
  | { origin: "clipboard"; selection: Selection }
  | { origin: "nothing" };

/**
 * Mirrors `demysto_core::CaptureError`.
 *
 * The kinds exist so that a window can offer a different affordance per kind —
 * `permission` is the one with somewhere to be sent. The message is composed in
 * the backend and shown as it is.
 */
export type CaptureError = {
  kind: "clipboard" | "keystroke" | "permission";
  message: string;
};

/** Mirrors `demysto_core::CaptureOutcome`. */
export type CaptureOutcome =
  | { status: "captured"; detail: Captured }
  | { status: "failed"; detail: CaptureError };

/** Mirrors `demysto_core::Parameter`. */
export type Parameter = {
  id: string;
  label: string;
  /** What the field holds before the user types, and when they type nothing. */
  default: string;
};

/** Mirrors `demysto_core::Kind`: what an Action will run on. */
export type Kind = "text" | "image";

/** Mirrors `demysto_core::Action`. */
export type Action = {
  id: string;
  name: string;
  parameters: Parameter[];
};

export function status(): Promise<Status> {
  return invoke<Status>("status");
}

/** What the last Capture produced, for a Palette that mounted after it. */
export function lastCapture(): Promise<CaptureOutcome | null> {
  return invoke<CaptureOutcome | null>("last_capture");
}

/**
 * The Actions the Palette lists: the ones that accept the last Capture, in the
 * order they are listed in. Filtered by the backend against the Selection it
 * read, rather than here against the one the window is showing.
 */
export function actions(): Promise<Action[]> {
  return invoke<Action[]>("actions");
}

/** Hides the window this is called from, which is what Escape asks for. */
export function dismiss(): Promise<void> {
  return invoke<void>("dismiss");
}

/**
 * Every Capture from here on that has begun but not yet finished, so that the
 * Palette can stop showing the one before it.
 */
export function onCapturing(handle: () => void): Promise<UnlistenFn> {
  return listen<null>("palette://capturing", () => handle());
}

/** Every Capture from here on, as the Hotkey produces them. */
export function onCapture(
  handle: (outcome: CaptureOutcome) => void,
): Promise<UnlistenFn> {
  return listen<CaptureOutcome>("palette://captured", (event) =>
    handle(event.payload),
  );
}

/**
 * Mirrors `demysto_core::RunError`: why a Turn produced no answer, or stopped
 * producing one.
 *
 * The kinds exist so that this window can offer a different affordance per
 * kind — the message is composed in the backend and shown as it is.
 */
export type RunError =
  /** The Provider refused the credentials, and named itself so this can link there. */
  | { kind: "authentication"; message: string; provider: string }
  /**
   * The Provider's answer was not the contract's shape. `reason` is what was
   * wrong with it without the quotation of what arrived that `message` carries
   * — it is what the log is given, and this window shows the message.
   */
  | { kind: "malformed"; message: string; reason: string }
  /**
   * The operating system is withholding what the Capture this Run would have
   * operated on needs — on macOS, the Accessibility permission. Offered the way
   * to the pane that grants it rather than a retry that cannot help.
   */
  | { kind: "permission"; message: string }
  | {
      kind:
        | "configuration"
        | "nothing_to_run"
        | "no_such_action"
        | "unreachable"
        | "timed_out"
        | "provider";
      message: string;
    };

/** Mirrors `demysto_core::RunOutcome`. */
export type RunOutcome =
  | { status: "answered"; detail: string }
  /** The user stopped it, keeping whatever had already arrived. */
  | { status: "stopped"; detail: string }
  /**
   * The answer began and then broke off. What arrived is kept, and the Model
   * can be asked for the rest of it.
   */
  | { status: "interrupted"; detail: { text: string; error: RunError } }
  | { status: "failed"; detail: RunError };

/** Mirrors `demysto_core::Turn`. */
export type Turn = {
  /**
   * What the user asked in their own words, `null` for the Turn that opened
   * the Conversation — which the Action asked on their behalf.
   */
  question: string | null;
  /** What it produced, `null` while the Model is still answering. */
  outcome: RunOutcome | null;
};

/** Mirrors `demysto_core::Conversation`. */
export type Conversation = {
  id: number;
  /** The Action the opening Run ran, `null` when it was not one Demysto has. */
  action: Action | null;
  turns: Turn[];
  /** The Model this Conversation was switched to, `null` while it has not been. */
  model: string | null;
  /**
   * What the user was told about the Selection before anything was sent — that
   * it is unusually large. `null` when there was nothing to say.
   */
  warning: string | null;
};

/** Mirrors `demysto_core::Summary`: one line of the list of Conversations. */
export type Summary = {
  id: number;
  name: string | null;
  /** The opening words of what it is about. */
  about: string;
};

/**
 * Runs one Action over the last Capture, with what the Palette collected for
 * the Parameters it declares.
 *
 * Answers as soon as the Run has somewhere to happen, not when the Model has:
 * the answer arrives in the result window, through the events below.
 */
export function run(
  action: string,
  parameters: Record<string, string>,
): Promise<void> {
  return invoke<void>("run", { action, parameters });
}

/**
 * Asks a follow-up in the Conversation the window is showing.
 *
 * Answers for the same reason [`run`] does: the reply arrives through the
 * events and the channel below, not as this call's result.
 */
export function followUp(question: string): Promise<void> {
  return invoke<void>("follow_up", { question });
}

/** Stops the Turn under way, keeping what has already arrived. */
export function stop(): Promise<void> {
  return invoke<void>("stop");
}

/**
 * Asks the last Turn of the Conversation again, optionally of another Model.
 *
 * The retry and the Model switch are one call because switching a Model without
 * asking anything would leave the user looking at the same failure. A Model
 * named here stands for the rest of the Conversation.
 *
 * Answers for the reason [`run`] does: the reply arrives through the events.
 */
export function retry(model?: string): Promise<void> {
  return invoke<void>("retry", { model: model ?? null });
}

/** Asks the Model for the rest of an answer that broke off part-way. */
export function continueAnswer(): Promise<void> {
  return invoke<void>("continue_answer");
}

/** Every Model configured, by the name a Conversation is switched to. */
export function models(): Promise<string[]> {
  return invoke<string[]>("models");
}

/**
 * Brings Settings up, at one Provider where one is named — which is how a
 * refused key is fixed from where it is reported.
 */
export function openSettings(provider?: string): Promise<void> {
  return invoke<void>("open_settings", { provider: provider ?? null });
}

/**
 * Every Provider Settings should open at, as the backend asks for one.
 */
export function onProviderWanted(
  handle: (provider: string) => void,
): Promise<UnlistenFn> {
  return listen<string>("settings://provider", (event) => handle(event.payload));
}

/**
 * Opens the settings pane where the Accessibility permission is granted, which
 * is how a Capture the system refused is fixed from where it is reported.
 * Rejects with a whole sentence when the pane could not be reached.
 */
export function openAccessibility(): Promise<void> {
  return invoke<void>("open_accessibility");
}

/**
 * Opens the folder Demysto writes its logs in, so that a bug report can carry
 * them. Rejects with a whole sentence when the file manager could not be
 * reached.
 */
export function openLogs(): Promise<void> {
  return invoke<void>("open_logs");
}

/**
 * The Conversation the window is showing, `null` before there has been one.
 *
 * Asked for rather than carried on the events, because it is the one answer
 * that is right whether or not the window was loaded for the events before it.
 */
export function conversation(): Promise<Conversation | null> {
  return invoke<Conversation | null>("conversation");
}

/** This session's Conversations, newest first. */
export function conversations(): Promise<Summary[]> {
  return invoke<Summary[]>("conversations");
}

/** Puts an earlier Conversation on screen. */
export function showConversation(id: number): Promise<Conversation | null> {
  return invoke<Conversation | null>("show_conversation", { id });
}

/**
 * Every hand-over of an answer still arriving, carrying the whole of it so far
 * rather than the piece that just landed — so a window that missed one is put
 * right by the next. It is render-ready Markdown: see `demysto_core::stream`
 * for why the assembling happens there and not here.
 *
 * A channel rather than an event, per the spec's *Shape*, because an answer
 * crosses the bridge some hundreds of times and an event would wake the Palette
 * for every one of them. It is opened here so that the window asking for the
 * answer looks the same as the windows listening for everything else.
 */
export function onStreaming(
  handle: (answer: string) => void,
): Promise<UnlistenFn> {
  const channel = new Channel<string>();
  channel.onmessage = handle;

  return invoke<void>("show_answers_on", { channel }).then(
    () => () => {
      // The backend holds the channel until another window claims it, so the
      // only thing to stop is this window acting on what comes down it.
      channel.onmessage = () => {};
    },
  );
}

/** Every Turn from here on that has begun but not yet finished. */
export function onRunning(handle: () => void): Promise<UnlistenFn> {
  return listen<null>("result://running", () => handle());
}

/**
 * Every Turn from here on, as it finishes.
 *
 * Neither event carries what changed: the window asks for the Conversation as
 * it now stands, which spares it having to reconcile a payload against what it
 * already had.
 */
export function onAnswered(handle: () => void): Promise<UnlistenFn> {
  return listen<null>("result://answered", () => handle());
}

/** Mirrors `demysto_core::ConfigError`. */
export type ConfigError = {
  kind: "unreadable" | "refused" | "unwritable" | "malformed" | "no_provider";
  message: string;
};

/** Mirrors `demysto_core::ConfiguredModel`. */
export type ConfiguredModel = { id: string; vision: boolean };

/**
 * Mirrors `demysto_core::KeyStanding`: where a Provider's key is.
 *
 * Where, and never what. ADR-0002 keeps the key on disk in exchange for one
 * promise — that it never enters the webview — and this window is drawn in the
 * same webview that renders whatever a Model said.
 */
export type KeyStanding =
  | { state: "in_file" }
  | { state: "in_environment"; variable: string }
  | { state: "not_needed" }
  | { state: "missing" };

/** Mirrors `demysto_core::ConfiguredProvider`. */
export type ConfiguredProvider = {
  name: string;
  /** What the file states, `null` when the preset supplies it. */
  base_url: string | null;
  preset: string | null;
  api_key_env: string | null;
  key: KeyStanding;
  models: ConfiguredModel[];
};

/** Mirrors `demysto_core::Settings`. */
export type Settings = {
  providers: ConfiguredProvider[];
  default_model: string | null;
  default_vision_model: string | null;
  /** The Hotkey that opens the Palette, `null` for the one Demysto comes with. */
  palette_hotkey: string | null;
  /**
   * How many characters a Selection may hold before Demysto says so, `null`
   * where the file states nothing and `Status.large_selection_default` decides.
   * Zero is a user who would rather not be told.
   */
  large_selection: number | null;
};

/** Mirrors `demysto_core::KeyEdit`: what a save does to a Provider's key. */
export type KeyEdit =
  | { action: "keep" }
  | { action: "set"; key: string }
  | { action: "forget" };

/** Mirrors `demysto_core::ProviderEdit`. */
export type ProviderEdit = {
  /** What this Provider was called in the file, `null` for one being added. */
  was: string | null;
  name: string;
  base_url: string | null;
  preset: string | null;
  api_key_env: string | null;
  api_key: KeyEdit;
  models: ConfiguredModel[];
};

/** Mirrors `demysto_core::Edit`: the whole of the settings, every time. */
export type Edit = {
  providers: ProviderEdit[];
  default_model: string | null;
  default_vision_model: string | null;
  palette_hotkey: string | null;
  /** `null` takes the setting out of the file; zero is being told nothing. */
  large_selection: number | null;
};

/**
 * Mirrors the backend's `Hotkeys`: the two things about Hotkeys this window
 * cannot work out for itself.
 */
export type Hotkeys = {
  /** The Palette's Hotkey where the settings state none, as it is read. */
  palette_default: string;
  /** The keys a Hotkey may be on its own, because they type nothing. */
  no_modifier_needed: string[];
};

/**
 * What a Hotkey may be, asked of the backend because the backend decides it:
 * which keys need no modifier, and what opens the Palette when nothing states
 * otherwise.
 */
export function hotkeys(): Promise<Hotkeys> {
  return invoke<Hotkeys>("hotkeys");
}

/** Mirrors `demysto_core::Preset`. */
export type Preset = {
  name: string;
  base_url: string;
  /** What the service's own documentation says to export, `null` where none. */
  variable: string | null;
  /** Whether the service has keys at all — see ADR-0006. */
  needs_key: boolean;
};

/** Mirrors `demysto_core::ActionStanding`: where a definition comes from. */
export type ActionStanding = "built_in" | "overridden" | "authored";

/**
 * Mirrors `demysto_core::DefinedAction`: one Action with everything about it.
 *
 * The other view of what `Action` above is: that one is what the Palette lists,
 * and keeps the prompt to itself; this is what the window whose whole purpose is
 * to change the prompt holds.
 */
export type DefinedAction = {
  id: string;
  name: string;
  template: string;
  parameters: Parameter[];
  model: string | null;
  /** The Hotkey that runs it without the Palette, `null` for one that has none. */
  hotkey: string | null;
  accepts: Kind[];
  standing: ActionStanding;
  /** The file it is in, `null` for a built-in nobody has changed. */
  path: string | null;
};

/**
 * Mirrors the backend's `Actions`: `demysto_core::Catalogue`, flattened, plus
 * what came of claiming the Hotkeys the Actions in it state.
 */
export type Catalogue = {
  actions: DefinedAction[];
  /** What went wrong with the files that are not in it, in whole sentences. */
  unreadable: string[];
  /**
   * The stated Hotkeys Demysto does not answer to, in whole sentences: one
   * another application already has, one two Actions both ask for, one written
   * as something that is not a combination at all.
   */
  unclaimed: string[];
};

/** Mirrors `demysto_core::ActionEdit`: what the window saves for one Action. */
export type ActionEdit = {
  /** The Action this edits, `null` for one being created. */
  id: string | null;
  name: string;
  template: string;
  parameters: Parameter[];
  model: string | null;
  hotkey: string | null;
  accepts: Kind[];
};

/** Mirrors `demysto_core::ActionError`. */
export type ActionError = {
  kind: "unreadable" | "unwritable" | "refused" | "no_such_action";
  message: string;
};

/**
 * Every Action there is, with everything about it — and, because asking claims
 * the Hotkeys they state, what could not be claimed.
 */
export function catalogue(): Promise<Catalogue> {
  return invoke<Catalogue>("catalogue");
}

/**
 * Writes one Action, and answers with the catalogue as the directory then holds
 * it — which is what the window shows next, for the reason a saved settings
 * file is read back.
 *
 * Rejects with an `ActionError` when what was edited is not an Action Demysto
 * could run; nothing is written in that case.
 */
export function saveAction(edit: ActionEdit): Promise<Catalogue> {
  return invoke<Catalogue>("save_action", { edit });
}

/**
 * Deletes an Action of the user's own, or removes the Override over a built-in
 * and leaves the built-in as it was written.
 */
export function deleteAction(id: string): Promise<Catalogue> {
  return invoke<Catalogue>("delete_action", { id });
}

/** The settings as the file now holds them. */
export function settings(): Promise<Settings> {
  return invoke<Settings>("settings");
}

/**
 * Writes the settings, and answers with them as the file then holds them —
 * which is what the window shows next: a save is only finished when it reads
 * back.
 *
 * Every key typed here is put to its Provider first, so this waits on the
 * network. Rejects with a `ConfigError` when a Provider refused a key, or when
 * what was edited is something Demysto could not read again; nothing is written
 * in either case.
 */
export function saveSettings(edit: Edit): Promise<Settings> {
  return invoke<Settings>("save_settings", { edit });
}

/** The services Demysto knows the conventions of. */
export function presets(): Promise<Preset[]> {
  return invoke<Preset[]>("presets");
}

/**
 * The Models a Provider says it offers, asked of the Provider as this window
 * has it now — a key just typed included — rather than as the file holds it.
 */
export function providerModels(provider: ProviderEdit): Promise<string[]> {
  return invoke<string[]>("provider_models", { provider });
}

/**
 * Whether a Provider accepts a key, asked of the Provider itself against one of
 * its Models. Rejects with a `RunError` carrying the Provider's own words.
 */
export function verifyProvider(
  provider: ProviderEdit,
  model: string,
): Promise<void> {
  return invoke<void>("verify_provider", { provider, model });
}
