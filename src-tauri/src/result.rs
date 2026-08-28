//! The result window: where the answer to a Run appears.
//!
//! An ordinary window rather than the Palette's panel — it is somewhere the
//! user reads and copies from, not something that floats over their work. Ticket
//! 06 turns it into the Conversation window.

use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use demysto_core::Demysto;
use tauri::ipc::Channel;
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

/// Where an answer still arriving is sent.
///
/// A channel rather than an event, per the spec's *Shape*: an answer crosses
/// the bridge some hundreds of times, and an event is broadcast to every window
/// — so the Palette would be woken by every hand-over of a stream it is not
/// showing. The two events either side of a Run stay events, because a state
/// changing twice is exactly what the event system is for.
///
/// A `static` rather than managed state for the reason [`RUNNING`] is one:
/// single instance is enforced, so there is one result window to send to.
static SHOWING: Mutex<Option<Channel<String>>> = Mutex::new(None);

/// Takes the channel a result window wants its answers on, replacing whatever
/// window said so before it.
///
/// The window says so as it mounts, which may be after the Run it is about to
/// show has started. Nothing is replayed: every hand-over carries the whole
/// answer so far, so the first one to arrive after this puts the window right.
pub fn show_answers_on(channel: Channel<String>) {
    *SHOWING.lock().unwrap() = Some(channel);
}

/// Runs one Action over the last Capture and shows the answer.
pub fn run<R: Runtime>(app: &AppHandle<R>, action: String, parameters: BTreeMap<String, String>) {
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

        // Declared before the window is shown rather than when the Run begins:
        // a window loading for this Run asks the core what it is looking at,
        // and the question before this one is what it must not come up holding
        // — neither that question's answer nor its name.
        let demysto = app.state::<Demysto>();
        demysto.about_to_run(&action);

        // Shown before the answer exists, and told that one is on its way: the
        // whole point of the tracer bullet is that the user sees something
        // immediately rather than after however long the Model takes.
        let _ = window.emit(RUNNING_EVENT, ());
        reveal(&window);

        // The whole answer so far crosses on every hand-over rather than the
        // piece that just landed, so a window still loading when one goes past
        // is put right by the next rather than left a fragment short.
        let outcome = demysto.run(&action, &parameters, |answer| {
            if let Some(showing) = SHOWING.lock().unwrap().as_ref() {
                let _ = showing.send(answer.to_owned());
            }
        });

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
