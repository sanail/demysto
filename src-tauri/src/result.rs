//! The Conversation window: where the answer to a Run appears, and where the
//! user keeps asking about the same Selection.
//!
//! An ordinary window rather than the Palette's panel — it is somewhere the
//! user reads and copies from, not something that floats over their work.

use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use demysto_core::{Arriving, Demysto, RunOutcome};
use serde::Serialize;
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
static SHOWING: Mutex<Option<Channel<Handover>>> = Mutex::new(None);

/// What goes down that channel, which is [`Arriving`] in the shape the window
/// reads.
///
/// Reasoning travels here rather than as an event for the reason the answer
/// does — an event reaches every window, and the Palette has no use for either
/// — and it costs the bridge one message per Turn against the answer's
/// hundreds.
#[derive(Clone, Serialize)]
#[serde(tag = "arriving", rename_all = "snake_case")]
pub(crate) enum Handover {
    Answer { answer: String },
    Reasoning,
}

/// Takes the channel a result window wants its answers on, replacing whatever
/// window said so before it.
///
/// The window says so as it mounts, which may be after the Run it is about to
/// show has started. Nothing is replayed: every hand-over carries the whole
/// answer so far, so the first one to arrive after this puts the window right.
pub fn show_answers_on(channel: Channel<Handover>) {
    *SHOWING.lock().unwrap() = Some(channel);
}

/// Runs one Action over the last Capture, in a Conversation of its own.
pub fn run<R: Runtime>(app: &AppHandle<R>, action: String, parameters: BTreeMap<String, String>) {
    off_thread(app, move |app, window| {
        dismiss_palette(app);
        opening(app, window, &action, &parameters);
    });
}

/// Asks the last Turn of the Conversation on screen again, optionally somewhere
/// else — the retry and the Model switch a failed Turn is offered.
pub fn retry<R: Runtime>(app: &AppHandle<R>, model: Option<String>) {
    again(app, move |demysto| {
        demysto.retry(model.as_deref(), streaming)
    });
}

/// Asks the Model for the rest of an answer that broke off part-way.
pub fn continue_answer<R: Runtime>(app: &AppHandle<R>) {
    again(app, |demysto| demysto.continue_answer(streaming));
}

/// Asks a Turn the Conversation already holds, however it came to be asked
/// again. The window is up and holding the focus, so nothing is revealed and
/// nothing is hidden — only the Turn is new.
fn again<R: Runtime>(
    app: &AppHandle<R>,
    ask: impl FnOnce(&Demysto) -> RunOutcome + Send + 'static,
) {
    off_thread(app, move |app, window| {
        let demysto = app.state::<Demysto>();

        // Told before the Turn goes out, so that the window replaces the
        // failure with "asking" rather than leaving it there until an answer
        // arrives. Nothing to ask again is nothing to say, for the reason a
        // follow-up with no Conversation says nothing.
        if !demysto.about_to_retry() {
            return;
        }

        let _ = window.emit(RUNNING_EVENT, ());

        ask(&demysto);
    });
}

/// Runs one Action on a Selection captured for it, with no Palette anywhere on
/// the path.
///
/// What an Action's own Hotkey does (user story 6): select, press, read. The
/// Capture happens here because on the other path it happens in `palette::open`,
/// and this one never goes near the Palette.
///
/// Nothing is collected for the Parameters the Action declares. The Palette is
/// where they are asked for, and it is precisely what the user bound this Hotkey
/// to skip — so each Parameter takes what it offers, which is why a built-in's
/// default is chosen to be the answer somebody would have typed.
pub fn straight_to<R: Runtime>(app: &AppHandle<R>, action: String) {
    off_thread(app, move |app, window| {
        // Before the Capture, so that a copy keystroke meant for the application
        // the user is reading is not answered by a window of Demysto's own.
        dismiss_palette(app);

        // Also before the Conversation window is shown, for the reason
        // `palette::open` captures before it shows the Palette: the copy
        // keystroke only reaches what the user is reading while that is still
        // the foreground application.
        app.state::<Demysto>().capture();

        let outcome = opening(app, window, &action, &BTreeMap::new());

        // The one path that can fail with nothing on screen to say so: this
        // one puts up the Conversation window itself, and a Run that fails
        // while that window is not in front of the user would be a Hotkey that
        // silently did nothing (user story 47).
        crate::notify::a_failure_nobody_can_see(app, &outcome);
    });
}

/// Puts the Conversation window up for the Run about to begin, and runs it.
///
/// Both paths into a Run end here; what differs is where the Selection came
/// from, and that is settled before this is called.
fn opening<R: Runtime>(
    app: &AppHandle<R>,
    window: &WebviewWindow<R>,
    action: &str,
    parameters: &BTreeMap<String, String>,
) -> RunOutcome {
    // Declared before the window is shown rather than when the Run begins: a
    // window loading for this Run asks the core what it is looking at, and the
    // question before this one is what it must not come up holding — neither
    // that question's answer nor its name.
    let demysto = app.state::<Demysto>();
    demysto.about_to_run(action);

    // Shown before the answer exists, and told that one is on its way: the user
    // should see something immediately rather than after however long the Model
    // takes.
    let _ = window.emit(RUNNING_EVENT, ());
    reveal(window);

    demysto.run(action, parameters, streaming)
}

/// Takes the Palette off the screen, which every Run does before it puts a
/// Conversation there.
///
/// Here rather than in the window that is about to take the focus from it, so
/// that it goes even if showing the Conversation window turns out to fail.
fn dismiss_palette<R: Runtime>(app: &AppHandle<R>) {
    if let Some(palette) = app.get_webview_window(crate::palette::LABEL) {
        let _ = palette.hide();
    }
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
///
/// The news that the Model is reasoning goes the same way, and needs no such
/// care: a window that misses it is a window whose next hand-over is the answer
/// itself, which is what it would have replaced the reasoning with anyway.
fn streaming(arriving: Arriving) {
    let handover = match arriving {
        Arriving::Answer(answer) => Handover::Answer {
            answer: answer.to_owned(),
        },
        Arriving::Reasoning => Handover::Reasoning,
    };

    if let Some(showing) = SHOWING.lock().unwrap().as_ref() {
        let _ = showing.send(handover);
    }
}

/// Brings the Conversation window in front of the user, wherever it was.
///
/// The dock is told first: a window belonging to an accessory application is
/// not in the window switcher, and the whole point of a Conversation is that
/// the user can leave it and come back to it with the keys they use for every
/// other window (user story 50).
fn reveal<R: Runtime>(window: &WebviewWindow<R>) {
    crate::dock::follows_the_windows(
        window.app_handle(),
        crate::dock::Change::Showing(window.label()),
    );

    let _ = window.show();
    let _ = window.set_focus();
}
