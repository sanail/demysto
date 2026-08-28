//! The Settings window: where everything Demysto is configured with is edited.
//!
//! An ordinary window rather than the Palette's panel — it is somewhere the
//! user works for a minute, not something that floats over what they were doing.

use tauri::{AppHandle, Manager, Runtime};

/// The window label, fixed in `tauri.conf.json`.
pub const LABEL: &str = "settings";

/// Brings Settings in front of the user, wherever it was left.
///
/// Shown rather than created: the window is declared with the others and hidden
/// instead of closed, which is what lets it come back up where the user put it.
pub fn reveal<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = app.get_webview_window(LABEL) else {
        return;
    };

    let _ = window.show();
    let _ = window.set_focus();
}
