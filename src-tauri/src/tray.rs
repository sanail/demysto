//! The tray icon, which is the whole of Demysto's presence while it waits.
//!
//! Everything the Hotkey reaches is reachable from here as well: the Palette,
//! and through it every Action, and Settings. That is not a convenience — it is
//! the path for somebody who has not learned the Hotkey yet, and the path that
//! still works when another application has taken it (user story 51). The dock
//! cannot be relied on for any of it, because Demysto is not in the dock while
//! it is only waiting; see `dock`.
//!
//! It also reaches what the Hotkey does not: this session's Conversations, which
//! nothing else brings back once their window is closed (user story 88).
//!
//! There is no list of Actions here: a menu built before any Capture cannot
//! leave out the ones the Selection would be refused by, and the Palette can.

use std::error::Error;

use demysto_core::{say, Demysto};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::{App, AppHandle, Manager, Runtime};

/// Menu item ids. Matched in the event handler below.
const SHOW: &str = "show";
const CHATS: &str = "chats";
const SETTINGS: &str = "settings";
const UPDATE: &str = "update";
const QUIT: &str = "quit";

/// The tray icon's id, so that the menu can be replaced when what it says
/// changes under it.
const TRAY: &str = "main";

pub fn build<R: Runtime>(app: &App<R>) -> Result<(), Box<dyn Error>> {
    // The icon the tray is given is not the one the application is bundled
    // under, and neither platform wants the same picture.
    //
    // macOS draws the menu bar in whichever appearance the desktop is in and
    // recolours a template icon to match it, so the glyph is handed over flat
    // and black. Both `.icon` and `.icon_as_template` below: without the second
    // the first is drawn as it was written, and a black glyph on a dark menu
    // bar is an icon nobody can find.
    #[cfg(target_os = "macos")]
    let icon = tauri::include_image!("icons/tray-macos-template.png");
    // Windows and Linux draw it as given, which is why they are given a plate
    // and not the bare mark: a panel may be light or dark, and half the mark is
    // nearly white. Rounded, because a square tile sits among flat panel glyphs
    // as a block — measured on KDE, where the difference is plain.
    #[cfg(not(target_os = "macos"))]
    let icon = tauri::include_image!("icons/tray-plate.png");

    let demysto = app.state::<Demysto>();

    tauri::tray::TrayIconBuilder::with_id(TRAY)
        .icon(icon)
        .icon_as_template(cfg!(target_os = "macos"))
        // What the icon is called. Windows shows it as a tooltip and, more to
        // the point, hands it to a screen reader — without one the icon is
        // nameless, and the mouse-only path user story 51 is about starts at an
        // icon nobody can identify. macOS and the Linux indicators ignore it.
        .tooltip(demysto.words().text("app-name"))
        .menu(&menu(app, &demysto)?)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id.as_ref() {
            SHOW => {
                give_back_the_foreground();
                crate::palette::reveal(app)
            }
            // No Capture and no foreground handed back: the Conversations are
            // about Selections already taken, and nothing here takes another.
            CHATS => crate::result::reopen(app),
            // Settings rather than the installation itself: taking an update
            // ends this process and starts another, and that is not something
            // to set off from a menu with nothing said first. What is said is
            // in the window this opens.
            SETTINGS | UPDATE => crate::settings::reveal(app),
            QUIT => app.exit(0),
            _ => {}
        })
        .build(app)?;

    Ok(())
}

/// Hands the foreground back to the application the user was reading, for the
/// Capture the Palette starts.
///
/// On macOS a status item's menu leaves the frontmost application where it
/// was. On Windows the click on the icon puts the taskbar in front, and showing
/// the menu puts Demysto's own window there; a copy keystroke sent then reaches
/// neither the text nor the user, and Capture falls back on a clipboard they
/// never meant.
fn give_back_the_foreground() {
    #[cfg(target_os = "windows")]
    crate::foreground::give_back();
}

/// Rebuilds the menu, for the two things it says that change while Demysto
/// runs: the language, and whether an update is on offer. A failure is
/// swallowed — the menu keeps the words it had, which is a menu out of date
/// rather than a save that reported an error about a menu.
pub fn rebuild<R: Runtime>(app: &AppHandle<R>) {
    let Some(tray) = app.tray_by_id(TRAY) else {
        return;
    };

    if let Ok(menu) = menu(app, &app.state::<Demysto>()) {
        let _ = tray.set_menu(Some(menu));
    }
}

/// The menu in whatever language Demysto is speaking now.
fn menu<R: Runtime, M: Manager<R>>(manager: &M, demysto: &Demysto) -> tauri::Result<Menu<R>> {
    let words = demysto.words();

    let show = MenuItem::with_id(manager, SHOW, words.text("tray-open"), true, None::<&str>)?;
    // Always enabled: with no Conversation yet the window says why it is empty, which
    // a greyed-out item cannot, and an item that changes state would be one
    // more reason to rebuild the menu.
    let chats = MenuItem::with_id(manager, CHATS, words.text("tray-chats"), true, None::<&str>)?;
    let settings = MenuItem::with_id(
        manager,
        SETTINGS,
        words.text("tray-settings"),
        true,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(manager, QUIT, words.text("tray-quit"), true, None::<&str>)?;

    // Only where there is one. The tray is Demysto's whole presence while it
    // waits, so it is where news about Demysto itself belongs — and an item
    // that appears when there is something to say costs nobody a glance on
    // every other day.
    let update = crate::update::offered(manager)
        .map(|version| {
            MenuItem::with_id(
                manager,
                UPDATE,
                say!(&words, "tray-update", "version" = version),
                true,
                None::<&str>,
            )
        })
        .transpose()?;

    let separator = PredefinedMenuItem::separator(manager)?;
    let mut top: Vec<&dyn tauri::menu::IsMenuItem<R>> = vec![&show, &chats, &separator];

    if let Some(update) = &update {
        top.push(update);
    }

    top.push(&settings);
    top.push(&quit);

    Menu::with_items(manager, &top)
}
