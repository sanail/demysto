//! The bridge between the frontend and [`demysto_core`].
//!
//! Every command here is a thin adapter: it borrows the facade out of Tauri's
//! managed state, calls one method on it, and maps the result. Logic that is
//! worth testing belongs in `demysto-core`, behind the single test seam, not here.

use demysto_core::{CaptureOutcome, Demysto, Status};
use tauri::{Runtime, State, WebviewWindow};

#[tauri::command]
pub fn status(demysto: State<'_, Demysto>) -> Status {
    demysto.status()
}

/// What the last Capture produced, for a Palette that mounted after it.
#[tauri::command]
pub fn last_capture(demysto: State<'_, Demysto>) -> Option<CaptureOutcome> {
    demysto.last_capture()
}

#[tauri::command]
pub fn dismiss_palette<R: Runtime>(window: WebviewWindow<R>) {
    let _ = window.hide();
}
