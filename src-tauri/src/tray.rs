//! The tray icon, which is the whole of Demysto's presence while it waits.

use std::error::Error;

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    App, Manager, Runtime,
};

/// Menu item ids. Matched in the event handler below.
const SHOW: &str = "show";
const QUIT: &str = "quit";

pub fn build<R: Runtime>(app: &App<R>) -> Result<(), Box<dyn Error>> {
    let show = MenuItem::with_id(app, SHOW, "Open Demysto", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, QUIT, "Quit Demysto", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    let icon = app
        .default_window_icon()
        .ok_or("no default window icon is embedded in this build")?
        .clone();

    TrayIconBuilder::with_id("main")
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id.as_ref() {
            SHOW => reveal_main_window(app),
            QUIT => app.exit(0),
            _ => {}
        })
        .build(app)?;

    Ok(())
}

/// Brings the main window back, whether it was hidden or merely behind something.
pub fn reveal_main_window<R: Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
