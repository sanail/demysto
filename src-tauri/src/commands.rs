//! The bridge between the frontend and [`demysto_core`].
//!
//! Every command here is a thin adapter: it borrows the facade out of Tauri's
//! managed state, calls one method on it, and maps the result. Logic that is
//! worth testing belongs in `demysto-core`, behind the single test seam, not here.

use demysto_core::{Demysto, Status};
use tauri::State;

#[tauri::command]
pub fn status(demysto: State<'_, Demysto>) -> Status {
    demysto.status()
}
