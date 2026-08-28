//! The Conversation window: where the answer to a Run appears, and where the
//! user keeps asking about the same Selection.
//!
//! An ordinary window rather than the Palette's panel — it is somewhere the
//! user reads and copies from, not something that floats over their work.

use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use demysto_core::Demysto;
use tauri::ipc::Channel;
use tauri::{AppHandle, Emitter, Manager, Runtime, WebviewWindow};

use crate::underway::Underway;

/// The window label, fixed in `tauri.conf.json`.
pub const LABEL: &str = "result";

/// Emitted when a Turn begins, so that the window stops showing what the last
/// one streamed and asks for the Conversation the new one is in.
const RUNNING_EVENT: &str = "result://running";

/// Emitted when a Turn ends, however it ended.
///
/// Neither event carries what changed: the window asks the core for the
/// Conversation as it now stands, which is the one answer that is right whether
/// or not the window was loaded for the events before it.
const ANSWERED_EVENT: &str = "result://answered";

/// Whether a Turn is under way. Held through [`Underway`]; see that module.
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

/// Runs one Action over the last Capture, in a Conversation of its own.
pub fn run<R: Runtime>(app: &AppHandle<R>, action: String, parameters: BTreeMap<String, String>) {
    off_thread(app, move |app, window| {
        // The Palette has done its part. Hidden here rather than by the window
        // that is about to take the focus from it, so that it goes even if
        // showing the Conversation window turns out to fail.
        if let Some(palette) = app.get_webview_window(crate::palette::LABEL) {
            let _ = palette.hide();
        }

        // Declared before the window is shown rather than when the Run begins:
        // a window loading for this Run asks the core what it is looking at,
        // and the question before this one is what it must not come up holding
        // — neither that question's answer nor its name.
        let demysto = app.state::<Demysto>();
        demysto.about_to_run(&action);

        // Shown before the answer exists, and told that one is on its way: the
        // user should see something immediately rather than after however long
        // the Model takes.
        let _ = window.emit(RUNNING_EVENT, ());
        reveal(window);

        demysto.run(&action, &parameters, streaming);
    });
}

/// Asks a follow-up in the Conversation the window is showing.
///
/// The window is already up and holding the focus — the user just typed into it
/// — so nothing is revealed and nothing is hidden. Only the Turn is new.
pub fn follow_up<R: Runtime>(app: &AppHandle<R>, question: String) {
    off_thread(app, move |app, window| {
        let demysto = app.state::<Demysto>();

        // Added before the window is told, so that the question is on screen
        // for as long as the Model is answering it rather than only after.
        //
        // There being no Conversation to add it to cannot happen from a window
        // that is showing one: the store forgets the oldest, and every Run that
        // pushes one off the end is also the Run that puts itself on screen.
        // Nothing is shown for it because there would be nowhere to show it.
        if !demysto.about_to_follow_up(&question) {
            return;
        }

        let _ = window.emit(RUNNING_EVENT, ());

        demysto.follow_up(&question, streaming);
    });
}

/// Runs one Turn away from the thread that draws every window Demysto has,
/// which is far shorter than a Provider across the network can be made to wait.
///
/// The two keys that start a Turn — a Hotkey and an Enter — are both ones the
/// user can press twice in a hurry, and a second Turn is not a free mistake: it
/// is another request, paid for, whose answer would race the first one into the
/// same window. The window still comes up, saying what it is already doing.
fn off_thread<R: Runtime>(
    app: &AppHandle<R>,
    ask: impl FnOnce(&AppHandle<R>, &WebviewWindow<R>) + Send + 'static,
) {
    let app = app.clone();

    std::thread::spawn(move || {
        let Some(window) = app.get_webview_window(LABEL) else {
            return;
        };

        let Some(_running) = Underway::claim(&RUNNING) else {
            reveal(&window);
            return;
        };

        ask(&app, &window);

        // A window that has never loaded hears neither event, and asks the core
        // for the Conversation as it mounts — which is why the core keeps it.
        let _ = window.emit(ANSWERED_EVENT, ());
    });
}

/// Hands the window the whole answer so far, on every hand-over rather than the
/// piece that just landed: a window still loading when one goes past is put
/// right by the next rather than left a fragment short.
fn streaming(answer: &str) {
    if let Some(showing) = SHOWING.lock().unwrap().as_ref() {
        let _ = showing.send(answer.to_owned());
    }
}

/// Brings the Conversation window in front of the user, wherever it was.
fn reveal<R: Runtime>(window: &WebviewWindow<R>) {
    let _ = window.show();
    let _ = window.set_focus();
}
