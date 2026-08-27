//! Demysto's Tauri shell.
//!
//! This crate owns windows, the tray, and the command bridge. The product logic
//! lives in `demysto-core`, which knows nothing about Tauri; see ADR-0001.

mod commands;
mod tray;

use demysto_core::Demysto;
use tauri::{RunEvent, WindowEvent};

pub fn run() {
    let config_dir = match demysto_core::config_dir() {
        Ok(config_dir) => config_dir,
        Err(error) => {
            // Ticket 11 owns making this visible: a windowed build has no stderr,
            // so a user launching from Finder or Explorer sees nothing at all.
            eprintln!("Demysto cannot start: {error}");
            std::process::exit(1);
        }
    };

    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        // A second launch belongs to the instance already running: raise its
        // window rather than starting a process that would fight it for the Hotkey.
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tray::reveal_main_window(app);
        }));
    }

    let app = builder
        .manage(Demysto::new(config_dir, env!("CARGO_PKG_VERSION")))
        .setup(|app| {
            tray::build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // Closing a window returns Demysto to the tray. Quitting is a
                // deliberate act, and the tray menu is where it lives.
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![commands::status])
        .build(tauri::generate_context!())
        .expect("Demysto failed to start");

    app.run(|_app, event| {
        // The last window closing is not a request to quit; an explicit exit,
        // which carries a code, is.
        if let RunEvent::ExitRequested {
            code: None, api, ..
        } = event
        {
            api.prevent_exit();
        }
    });
}
