import { invoke } from "@tauri-apps/api/core";
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

export function dismissPalette(): Promise<void> {
  return invoke<void>("dismiss_palette");
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
