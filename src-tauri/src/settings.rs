//! The Settings window: where everything Demysto is configured with is edited.
//!
//! An ordinary window rather than the Palette's panel — it is somewhere the
//! user works for a minute, not something that floats over what they were doing.

use demysto_core::Demysto;
use tauri::{AppHandle, Emitter, Manager, Runtime};

/// The window label, fixed in `tauri.conf.json`.
pub const LABEL: &str = "settings";

/// Emitted with the name of the Provider Settings should open at, so that a
/// refused key can be fixed where it is reported (user story 45).
const PROVIDER_EVENT: &str = "settings://provider";

/// Brings Settings in front of the user, wherever it was left.
///
/// Shown rather than created: the window is declared with the others and hidden
/// instead of closed, which is what lets it come back up where the user put it.
pub fn reveal<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = app.get_webview_window(LABEL) else {
        return;
    };

    names_itself(app);

    // Before it is shown, for the reason a Conversation is: an accessory
    // application's windows are in no switcher.
    crate::dock::follows_the_windows(app, crate::dock::Change::Showing(LABEL));

    let _ = window.show();
    let _ = window.set_focus();
}

/// Puts the window's own name on it, in the language Demysto is speaking.
///
/// The title in `tauri.conf.json` is fixed when the application is built, so it
/// is the English one until this runs. Called where the window is shown and
/// where the language may have changed under it — which are the two moments it
/// can be wrong, and between them the window is not on screen.
///
/// The other two windows are called "Demysto", which is the same word in every
/// catalogue and needs none of this.
pub fn names_itself<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = app.get_webview_window(LABEL) else {
        return;
    };

    let _ = window.set_title(&app.state::<Demysto>().words().text("settings-window-title"));
}

/// Brings Settings up at one Provider, which is where a refused key is fixed.
///
/// The window is declared with the others and loaded at startup rather than
/// created here, so it is listening by the time this is called; a name it
/// misses costs the scroll and not the window.
pub fn reveal_at<R: Runtime>(app: &AppHandle<R>, provider: Option<String>) {
    reveal(app);

    if let Some(provider) = provider {
        let _ = app.emit_to(LABEL, PROVIDER_EVENT, provider);
    }
}
