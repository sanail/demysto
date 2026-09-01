//! Whether Demysto starts when the user logs in.
//!
//! A resident tool that has to be launched by hand is a tool that stops working
//! at the first reboot, and one that puts itself into the login items without
//! asking is a tool nobody invited. So it is offered once, as a question with
//! two answers, in the first-run flow (user story 52) — and answered again in
//! the operating system's own settings, by anybody who changes their mind.
//!
//! What is registered per platform — a launch agent on macOS, a registry entry
//! on Windows, a desktop file on Linux — is `tauri-plugin-autostart`'s, which
//! is why this module is a question and an answer rather than three
//! implementations.

use demysto_core::{say, Words};
use tauri::{AppHandle, Runtime};
use tauri_plugin_autostart::ManagerExt;

/// Whether Demysto is in the login items now.
///
/// Asked of the system rather than remembered in the settings: the login items
/// are the operating system's list, and somebody who took Demysto out of it
/// there has said so more plainly than a file of ours could record.
///
/// A system that will not say answers no, which is what the window then offers
/// to change — the alternative is a checkbox that refuses to be drawn because
/// a registry key could not be read.
pub fn enabled<R: Runtime>(app: &AppHandle<R>) -> bool {
    app.autolaunch().is_enabled().unwrap_or(false)
}

/// Puts Demysto into the login items, or takes it out.
///
/// Answers with what went wrong, in a whole sentence, so that the window that
/// offered the choice is where the failure is reported — the same bargain
/// `accessibility::reveal` and `folder::open` make.
pub fn set<R: Runtime>(app: &AppHandle<R>, wanted: bool, words: &Words) -> Result<(), String> {
    let autostart = app.autolaunch();

    match wanted {
        true => autostart.enable(),
        false => autostart.disable(),
    }
    .map_err(|error| say!(words, "autostart-refused", "detail" = error.to_string()))
}
