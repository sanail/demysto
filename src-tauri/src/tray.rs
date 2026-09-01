//! The tray icon, which is the whole of Demysto's presence while it waits.
//!
//! Everything the Hotkey reaches is reachable from here as well: the Palette,
//! every Action, and Settings. That is not a convenience — it is the path for
//! somebody who has not learned the Hotkey yet, and the path that still works
//! when another application has taken it (user story 51). The dock cannot be
//! relied on for any of it, because Demysto is not in the dock while it is only
//! waiting; see `dock`.

use std::error::Error;

use demysto_core::{DefinedAction, Demysto};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{App, AppHandle, Manager, Runtime};

/// Menu item ids. Matched in the event handler below.
const SHOW: &str = "show";
const SETTINGS: &str = "settings";
const QUIT: &str = "quit";

/// What an Action's own item is identified by, ahead of the identifier the
/// Action is asked for by.
///
/// Prefixed rather than used bare so that an Action can be called anything at
/// all — `settings` included — without quietly becoming one of the items above.
const ACTION: &str = "action:";

/// What the menu identifies one Action's item by.
fn item_for(action: &str) -> String {
    format!("{ACTION}{action}")
}

/// The Action an item that was chosen runs, `None` for one of the items above.
fn action_in(item: &str) -> Option<&str> {
    item.strip_prefix(ACTION)
}

/// The tray icon's id, so that the menu can be replaced when the catalogue
/// changes under it.
const TRAY: &str = "main";

pub fn build<R: Runtime>(app: &App<R>) -> Result<(), Box<dyn Error>> {
    let icon = app
        .default_window_icon()
        .ok_or("no default window icon is embedded in this build")?
        .clone();

    let demysto = app.state::<Demysto>();
    let actions = demysto.catalogue().actions;

    tauri::tray::TrayIconBuilder::with_id(TRAY)
        .icon(icon)
        .menu(&menu(app, &actions, &demysto)?)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id.as_ref() {
            SHOW => crate::palette::reveal(app),
            SETTINGS => crate::settings::reveal(app),
            QUIT => app.exit(0),
            id => {
                if let Some(action) = action_in(id) {
                    // The same path an Action's own Hotkey takes, Capture and
                    // all: what the user is looking at is still the foreground
                    // application while the tray menu is open.
                    crate::result::straight_to(app, action.to_owned());
                }
            }
        })
        .build(app)?;

    Ok(())
}

/// Puts the Actions the catalogue now holds in the menu.
///
/// Called wherever the catalogue is read, for the reason the Hotkeys in it are
/// claimed there: an Action written, deleted, or dropped into the directory as
/// a file somebody sent should be in this menu without a restart. A failure is
/// swallowed — the menu keeps the Actions it had, which is a menu one Action out
/// of date rather than a save that reported an error about a menu.
pub fn follows_the_catalogue<R: Runtime>(app: &AppHandle<R>, actions: &[DefinedAction]) {
    let Some(tray) = app.tray_by_id(TRAY) else {
        return;
    };

    // Rebuilt in whatever language Demysto is speaking now, which is also why
    // a save that only changed the language still comes through here: the tray
    // is the one part of the interface no window redraws.
    let demysto = app.state::<Demysto>();

    if let Ok(menu) = menu(app, actions, &demysto) {
        let _ = tray.set_menu(Some(menu));
    }
}

/// The menu as it stands for one catalogue.
///
/// Rebuilt whole rather than edited, because what changes is the middle of it:
/// the Actions are the only part that moves, and a menu is cheap next to the
/// bookkeeping of keeping one in step item by item.
fn menu<R: Runtime, M: Manager<R>>(
    manager: &M,
    actions: &[DefinedAction],
    demysto: &Demysto,
) -> tauri::Result<Menu<R>> {
    let words = demysto.words();

    let show = MenuItem::with_id(manager, SHOW, words.text("tray-open"), true, None::<&str>)?;
    let settings = MenuItem::with_id(
        manager,
        SETTINGS,
        words.text("tray-settings"),
        true,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(manager, QUIT, words.text("tray-quit"), true, None::<&str>)?;

    let runnable = actions
        .iter()
        .map(|action| {
            MenuItem::with_id(
                manager,
                item_for(&action.id),
                &action.name,
                true,
                None::<&str>,
            )
        })
        .collect::<tauri::Result<Vec<_>>>()?;

    // A submenu rather than a flat list: the Actions are the part of this that
    // grows, and a user with twenty of them should not have to walk past all of
    // them to reach Settings. Disabled when there are none, which is a
    // catalogue that could not be read at all.
    let items = runnable
        .iter()
        .map(|item| item as &dyn tauri::menu::IsMenuItem<R>)
        .collect::<Vec<_>>();
    let listed = Submenu::with_items(
        manager,
        words.text("tray-actions"),
        !items.is_empty(),
        &items,
    )?;

    Menu::with_items(
        manager,
        &[
            &show,
            &listed,
            &PredefinedMenuItem::separator(manager)?,
            &settings,
            &quit,
        ],
    )
}

#[cfg(test)]
mod tests {
    //! The one thing here worth testing without a desktop: that the two halves
    //! agree.
    //!
    //! The menu writes an item's id and the handler reads it back, and nothing
    //! but this catches the two parting company — what it would cost is a tray
    //! menu offering Actions that quietly do nothing when they are chosen.

    use super::*;

    #[test]
    fn the_action_a_chosen_item_runs_is_the_one_the_menu_wrote() {
        // Every shape an Action's identifier takes: a built-in's, a name the
        // catalogue found from what the user typed, and one carrying the
        // separator this prefix is written with.
        for action in ["explain", "translate-to-french", "notes:daily"] {
            assert_eq!(action_in(&item_for(action)), Some(action));
        }
    }

    #[test]
    fn an_action_may_be_called_what_the_menu_calls_its_own_items() {
        // Nothing stops somebody writing an Action with the identifier
        // `settings`: Actions are files in a directory of the user's own, and
        // an item that chose Settings instead of running one would be a fault
        // nobody could see in the file.
        for reserved in [SHOW, SETTINGS, QUIT] {
            assert_ne!(item_for(reserved), reserved);
            assert_eq!(action_in(&item_for(reserved)), Some(reserved));
            assert_eq!(action_in(reserved), None);
        }
    }
}
