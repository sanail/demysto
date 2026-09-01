//! The first run: the flow a fresh installation is met by instead of a tray
//! icon it has to work out for itself.
//!
//! An ordinary window like Settings, shown once — at the end of it there is a
//! Provider configured, a key the Provider itself has accepted, an answer to
//! the autostart question, and, on macOS, a walk to the Accessibility
//! permission. What each step says is the window's; what this module owns is
//! when it appears and when it is over (user story 57).

use std::time::Duration;

use demysto_core::Demysto;
use tauri::{AppHandle, Manager, Runtime};

/// The window label, fixed in `tauri.conf.json`.
pub const LABEL: &str = "welcome";

/// Brings the flow up, a moment after the application has started.
///
/// The moment is the point, and it is the whole of why this is not two lines.
/// On a Linux desktop with no graphics acceleration — a virtual machine whose
/// Mesa answers `Accelerated: no`, which is where this was watched — a window
/// shown in the first instants of the process comes up as a correct, complete,
/// white rectangle: the page is loaded, the accessibility tree reads every line
/// of it back, the elements report their places on screen, and not one pixel is
/// drawn. A resize does not bring it back. Shown a beat later, it draws.
///
/// The three windows Demysto has always had never met this, and that is the
/// same fact rather than luck: nothing shows any of them until somebody asks
/// for one, which is never in the first instants. This window is the exception
/// because it is the one Demysto opens by itself.
///
/// A second, against a threshold measured under a fifth of one on that desktop.
/// It is paid once in the life of an installation, by somebody who has just
/// launched an application and is watching it start.
///
/// The alternative was `WEBKIT_DISABLE_DMABUF_RENDERER`, which fixes it
/// outright — and which every machine with a working graphics card would then
/// pay for, in a slower path it never needed. Tauri's own guidance is not to
/// ship such an override for a fault the application can avoid.
pub fn reveal<R: Runtime>(app: &AppHandle<R>) {
    let waiting = app.clone();

    std::thread::spawn(move || {
        std::thread::sleep(SETTLE);

        let showing = waiting.clone();
        let _ = waiting.run_on_main_thread(move || on_screen(&showing));
    });
}

/// How long WebKit is given before the flow is put on screen.
const SETTLE: Duration = Duration::from_secs(1);

fn on_screen<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = app.get_webview_window(LABEL) else {
        return;
    };

    // Before it is on screen, for the reason a Conversation is: an accessory
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
