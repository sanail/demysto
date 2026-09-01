//! The first run: the flow a fresh installation is met by instead of a tray
//! icon it has to work out for itself.
//!
//! An ordinary window like Settings, shown once — at the end of it there is a
//! Provider configured, a key the Provider itself has accepted, an answer to
//! the autostart question, and, on macOS, a walk to the Accessibility
//! permission. What each step says is the window's; what this module owns is
//! when it appears and when it is over (user story 57).

use demysto_core::Demysto;
use tauri::{AppHandle, Manager, Runtime};

/// The window label, fixed in `tauri.conf.json`.
pub const LABEL: &str = "welcome";

/// Brings the flow up, which is what a fresh installation starts with.
///
/// Shown rather than created, like every other window: it is declared with the
/// others and hidden, so the one round trip it makes to learn the language has
/// already happened by the time anybody sees it.
pub fn reveal<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = app.get_webview_window(LABEL) else {
        return;
    };

    // Before it is shown, for the reason a Conversation is: an accessory
    // application's windows are in no switcher, and this is the first window
    // Demysto ever puts in front of anybody.
    crate::dock::follows_the_windows(app, crate::dock::Change::Showing(LABEL));

    let _ = window.show();
    let _ = window.set_focus();
}

/// Records the flow as over when the window that has gone away is its own.
///
/// However it went: the button at the end of it, Escape, and the close button
/// are the three ways out, and all of them are somebody who has been asked what
/// the flow exists to ask. Coming back at every launch until the last step is
/// reached would be the tool nagging, and everything the flow offers is in
/// Settings afterwards.
///
/// Said to the log rather than to the user when it cannot be recorded: the
/// window is already on its way out, there is nothing to act on, and the cost
/// is being met by the flow once more.
pub fn gone<R: Runtime>(app: &AppHandle<R>, label: &str) {
    if label != LABEL {
        return;
    }

    let demysto = app.state::<Demysto>();

    if let Err(error) = demysto.welcome_finished() {
        demysto.note(&error.to_string());
    }
}
