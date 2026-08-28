//! The bridge between the frontend and [`demysto_core`].
//!
//! Every command here is a thin adapter: it borrows the facade out of Tauri's
//! managed state, calls one method on it, and maps the result. Logic that is
//! worth testing belongs in `demysto-core`, behind the single test seam, not here.

use demysto_core::{CaptureOutcome, Demysto, RunOutcome, Status};
use tauri::{AppHandle, Runtime, State, WebviewWindow};

#[tauri::command]
pub fn status(demysto: State<'_, Demysto>) -> Status {
    demysto.status()
}

/// What the last Capture produced, for a Palette that mounted after it.
#[tauri::command]
pub fn last_capture(demysto: State<'_, Demysto>) -> Option<CaptureOutcome> {
    demysto.last_capture()
}

/// Runs the built-in explain Action over the last Capture.
///
/// The one command that does not call the facade directly: which window the
/// answer appears in, and which thread the waiting happens on, are Tauri's
/// business rather than the core's. It returns as soon as the Run has somewhere
/// to happen, and the answer reaches the result window as an event.
#[tauri::command]
pub fn run<R: Runtime>(app: AppHandle<R>) {
    crate::result::run(&app);
}

/// What the last Run produced, for a result window that mounted after it.
#[tauri::command]
pub fn last_run(demysto: State<'_, Demysto>) -> Option<RunOutcome> {
    demysto.last_run()
}

/// Hides the window this was invoked from, which is what Escape asks for in
/// both of them.
#[tauri::command]
pub fn dismiss<R: Runtime>(window: WebviewWindow<R>) {
    let _ = window.hide();
}
