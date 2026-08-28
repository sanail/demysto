//! The result window: where the answer to a Run appears.
//!
//! An ordinary window rather than the Palette's panel — it is somewhere the
//! user reads and copies from, not something that floats over their work. Ticket
//! 06 turns it into the Conversation window, and ticket 04 fills it as the
//! answer streams rather than when it is finished.

use std::sync::atomic::AtomicBool;

use demysto_core::Demysto;
use tauri::{AppHandle, Emitter, Manager, Runtime, WebviewWindow};

use crate::underway::Underway;

/// The window label, fixed in `tauri.conf.json`.
pub const LABEL: &str = "result";

/// Emitted when a Run begins, so that the window stops showing the one before it.
const RUNNING_EVENT: &str = "result://running";

/// Emitted when a Run ends, however it ended.
const ANSWERED_EVENT: &str = "result://answered";

/// Whether a Run is under way. Held through [`Underway`]; see that module.
static RUNNING: AtomicBool = AtomicBool::new(false);

/// Runs the built-in explain Action over the last Capture and shows the answer.
pub fn run<R: Runtime>(app: &AppHandle<R>) {
    let app = app.clone();

    // A Run waits on a Provider across the network, which is far longer than
    // the thread that draws every window Demysto has can be made to wait.
    std::thread::spawn(move || {
        // The Palette has done its part. Hidden here rather than by the window
        // that is about to take the focus from it, so that it goes even if
        // showing the result window turns out to fail.
        if let Some(palette) = app.get_webview_window(crate::palette::LABEL) {
            let _ = palette.hide();
        }

        let Some(window) = app.get_webview_window(LABEL) else {
            return;
        };

        // A Hotkey and an Enter are both keys the user can press twice in a
        // hurry, and a second Run is not a free mistake: it is another request,
        // paid for, whose answer would race the first one into the same window.
        // The window still comes up, saying what it is already doing.
        let Some(_running) = Underway::claim(&RUNNING) else {
            reveal(&window);
            return;
        };

        // Forgotten before the window is shown rather than when the Run begins:
        // a window loading for this Run asks the core what the last one
        // produced, and the answer to the question before is what it must not
        // come up holding.
        let demysto = app.state::<Demysto>();
        demysto.forget_last_run();

        // Shown before the answer exists, and told that one is on its way: the
        // whole point of the tracer bullet is that the user sees something
        // immediately rather than after however long the Model takes.
        let _ = window.emit(RUNNING_EVENT, ());
        reveal(&window);

        let outcome = demysto.run();

        // A window that has never loaded hears neither event and asks the core
        // for the last Run as it mounts, which is why the core keeps it.
        let _ = window.emit(ANSWERED_EVENT, &outcome);
    });
}

/// Brings the result window in front of the user, wherever it was.
fn reveal<R: Runtime>(window: &WebviewWindow<R>) {
    let _ = window.show();
    let _ = window.set_focus();
}
