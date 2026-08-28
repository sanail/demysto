import { Channel, invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

/** Mirrors `demysto_core::Status`. */
export type Status = {
  version: string;
  config_dir: string;
};

/** Mirrors `demysto_core::Selection`. */
export type Selection = { kind: "text"; text: string };

/** Mirrors `demysto_core::Captured`. */
export type Captured =
  | { origin: "selection"; selection: Selection }
  | { origin: "clipboard"; selection: Selection }
  | { origin: "nothing" };

/** Mirrors `demysto_core::CaptureError`. */
export type CaptureError = {
  kind: "clipboard" | "keystroke";
  message: string;
};

/** Mirrors `demysto_core::CaptureOutcome`. */
export type CaptureOutcome =
  | { status: "captured"; detail: Captured }
  | { status: "failed"; detail: CaptureError };

export function status(): Promise<Status> {
  return invoke<Status>("status");
}

/** What the last Capture produced, for a Palette that mounted after it. */
export function lastCapture(): Promise<CaptureOutcome | null> {
  return invoke<CaptureOutcome | null>("last_capture");
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

/** Mirrors `demysto_core::RunError`. */
export type RunError = {
  kind:
    | "configuration"
    | "nothing_to_run"
    | "unreachable"
    | "provider"
    | "malformed";
  message: string;
};

/** Mirrors `demysto_core::RunOutcome`. */
export type RunOutcome =
  | { status: "answered"; detail: string }
  | { status: "failed"; detail: RunError };

/**
 * Runs the built-in explain Action over the last Capture.
 *
 * Answers as soon as the Run has somewhere to happen, not when the Model has:
 * the answer arrives in the result window, through the events below.
 */
export function run(): Promise<void> {
  return invoke<void>("run");
}

/** What the last Run produced, for a result window that mounted after it. */
export function lastRun(): Promise<RunOutcome | null> {
  return invoke<RunOutcome | null>("last_run");
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

/** Every Run from here on that has begun but not yet finished. */
export function onRunning(handle: () => void): Promise<UnlistenFn> {
  return listen<null>("result://running", () => handle());
}

/** Every Run from here on, as it finishes. */
export function onAnswered(
  handle: (outcome: RunOutcome) => void,
): Promise<UnlistenFn> {
  return listen<RunOutcome>("result://answered", (event) =>
    handle(event.payload),
  );
}
