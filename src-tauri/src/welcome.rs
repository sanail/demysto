//! The first run: the flow a fresh installation is met by instead of a tray
//! icon it has to work out for itself.
//!
//! An ordinary window like Settings, shown once — at the end of it there is a
//! Provider configured, a key the Provider itself has accepted, an answer to
//! the autostart question, and, on macOS, a walk to the Accessibility
//! permission. What each step says is the window's; what this module owns is
//! when it appears and when it is over (user story 57).

use demysto_core::Demysto;
use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};

/// The window label. Not declared in `tauri.conf.json` with the other three —
/// see [`reveal`].
pub const LABEL: &str = "welcome";

/// Brings the flow up, which is what a fresh installation starts with.
///
/// Built here rather than declared in `tauri.conf.json` and hidden, which is
/// how the Palette, the Conversation and Settings are done. Two reasons, and
/// the second is the one that decided it:
///
/// This window exists once in the life of an installation, so a window loaded
/// at every launch for it is a page parsed every time to be thrown away.
///
/// And a window declared hidden and shown at startup does not paint on
/// WebKitGTK. It comes up as a correct, complete, white rectangle — the page is
/// there, the accessibility tree reads it back in full, and nothing at all is
/// on screen; a resize does not bring it back. macOS and Windows draw it
/// either way. Built visible, it draws on all three.
pub fn reveal<R: Runtime>(app: &AppHandle<R>) {
    // Before it is on screen, for the reason a Conversation is: an accessory
    // application's windows are in no switcher, and this is the first window
    // Demysto ever puts in front of anybody.
    crate::dock::follows_the_windows(app, crate::dock::Change::Showing(LABEL));

    if let Some(window) = app.get_webview_window(LABEL) {
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }

    let built = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("welcome.html".into()))
        .title("Demysto")
        .inner_size(620.0, 560.0)
        .min_inner_size(460.0, 420.0)
        .center()
        .build();

    // A flow that could not be built is a flow nobody sees, and there is
    // nothing on screen to say so on: the tray is there, Settings configures
    // everything the flow would have, and the log is where this belongs.
    if let Err(error) = built {
        app.state::<Demysto>().note(&error.to_string());
    }
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
