//! System notifications, and the one thing Demysto uses them for.
//!
//! A Run launched from an Action's own Hotkey is the path with no window in it:
//! select, press, read. When such a Run fails and the Conversation window is
//! not in front of the user — because showing it did not take, or because they
//! put it away while the Model was reasoning — the failure has nowhere to appear.
//! A notification is what stands in, so that a Hotkey never silently does
//! nothing (user story 47).
//!
//! Only there. Every other failure has a Conversation to appear in, and the
//! spec is explicit that errors are entries rather than dialogs.

use demysto_core::{Demysto, RunOutcome};
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_notification::NotificationExt;

/// Tells the user a Run failed, when nothing on screen is going to.
///
/// Does nothing when the Run produced an answer, or when the Conversation
/// window is in front of them — where it is, the entry in the Conversation is
/// the report, and a notification beside it would be the same news twice.
pub fn a_failure_nobody_can_see<R: Runtime>(app: &AppHandle<R>, outcome: &RunOutcome) {
    let Some(error) = outcome.error() else {
        return;
    };

    let unseen = app
        .get_webview_window(crate::result::LABEL)
        .is_none_or(|window| !window.is_visible().unwrap_or(false));

    if !unseen {
        return;
    }

    // An answer that broke off part-way is not an answer that never came, and
    // the window it is waiting in has the rest of it on offer. Said differently
    // because the two ask for different things of whoever reads them.
    let demysto = app.state::<Demysto>();
    let words = demysto.words();
    let title = match outcome.text() {
        Some(_) => words.text("notification-stopped-part-way"),
        None => words.text("notification-could-not-answer"),
    };

    // The Provider's own sentence, which is the one worth reading — the same
    // one the Conversation would have shown. Notifications are truncated by
    // every desktop that shows them, and a message trimmed by the system is
    // still better than one Demysto trimmed on its behalf.
    let _ = app
        .notification()
        .builder()
        .title(title)
        .body(error.message())
        .show();
}
