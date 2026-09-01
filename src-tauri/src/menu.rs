//! The application menu, which exists for one reason: key equivalents.
//!
//! macOS does not bind `Cmd+C` to a window or to a text view — it binds it to a
//! menu item, and a window whose application has no such item has no way to
//! copy at all. Demysto had none: the tray carries everything the user reaches
//! deliberately, and an application that lives in the tray does not obviously
//! need a menu bar besides. The cost of that was user story 15 — the answer
//! could be selected with the mouse and then not taken anywhere.
//!
//! macOS only. Everywhere else `Cmd+C` is `Ctrl+C`, which the web view handles
//! itself, and a menu bar attached to the windows of a tray utility would be
//! furniture nobody asked for.
//!
//! Only the items that carry a key equivalent somebody needs are here. A menu
//! is not a place to list what the application can do — the tray is that, and
//! it is the one the spec's user story 51 is about.

use std::error::Error;

use demysto_core::Demysto;
use tauri::menu::{Menu, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Manager, Runtime};

/// Builds the menu bar Demysto shows while a window of its own is on screen.
///
/// Which is exactly when it is a `Regular` application; while only the Palette
/// is up the policy is `Accessory` and macOS shows no menu bar at all, so this
/// costs nothing in the state Demysto spends most of its life in. See `dock`.
///
/// Built again whenever the language changes, for the reason the tray menu is:
/// a menu bar is not a window and nothing redraws it, so without that it would
/// be the one surface still speaking the language nobody chose.
#[cfg(target_os = "macos")]
pub fn build<R: Runtime>(app: &AppHandle<R>) -> Result<(), Box<dyn Error>> {
    let demysto = app.state::<Demysto>();
    let words = demysto.words();

    // The first submenu becomes the application menu whatever it is called, and
    // macOS puts the application's own name on it regardless of what is written
    // here. It is present for `Cmd+Q` and `Cmd+H`, which a user will reach for
    // out of habit and which no other item provides.
    let application = Submenu::with_items(
        app,
        "Demysto",
        true,
        &[
            &PredefinedMenuItem::hide(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            // Quitting stays a deliberate act, as the tray's own item has it:
            // this is the same act reached by the key people already know.
            //
            // The only item here with a name of Demysto's own. The four below
            // take macOS's, which macOS has already translated — and better
            // than a catalogue of ours would, because they are the same words
            // in every application on the machine.
            &PredefinedMenuItem::quit(app, Some(&words.text("menu-quit")))?,
        ],
    )?;

    // The reason the menu exists. Copy is what user story 15 asks for; the
    // other three are what the follow-up field needs, and leaving them out
    // would make the Conversation window the one place on the system where
    // pasting a question does not work.
    let edit = Submenu::with_items(
        app,
        words.text("menu-edit"),
        true,
        &[
            &PredefinedMenuItem::cut(app, None)?,
            &PredefinedMenuItem::copy(app, None)?,
            &PredefinedMenuItem::paste(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::select_all(app, None)?,
        ],
    )?;

    app.set_menu(Menu::with_items(app, &[&application, &edit])?)?;

    Ok(())
}
