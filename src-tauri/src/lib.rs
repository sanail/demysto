//! Demysto's Tauri shell.
//!
//! This crate owns windows, the tray, and the command bridge. The product logic
//! lives in `demysto-core`, which knows nothing about Tauri; see ADR-0001.

mod commands;
mod hotkey;
mod palette;
mod result;
mod settings;
mod tray;
mod underway;

use demysto_core::Demysto;
use tauri::{Manager, RunEvent, WindowEvent};

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
        // Palette rather than starting a process that would fight it for the Hotkey.
        builder = builder
            .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
                palette::reveal(app);
            }))
            .plugin(hotkey::plugin());
    }

    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_nspanel::init());
    }

    let app = builder
        .manage(Demysto::new(config_dir, env!("CARGO_PKG_VERSION")))
        .setup(|app| {
            // Accessory while the Palette is all there is: a resident utility
            // has no business in the dock. Ticket 12 makes this follow whether
            // a Conversation or Settings window is open.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            tray::build(app)?;
            hotkey::register(app.handle());

            #[cfg(target_os = "macos")]
            if let Some(window) = app.get_webview_window(palette::LABEL) {
                palette::into_panel(&window)?;
            }

            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                // Closing a window returns Demysto to the tray. Quitting is a
                // deliberate act, and the tray menu is where it lives.
                api.prevent_close();
                let _ = window.hide();
            }
            // The Palette is not a window anybody should have to manage: losing
            // the focus is the same instruction as pressing Escape.
            WindowEvent::Focused(false) if window.label() == palette::LABEL => {
                let _ = window.hide();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::status,
            commands::last_capture,
            commands::actions,
            commands::run,
            commands::follow_up,
            commands::stop,
            commands::conversation,
            commands::conversations,
            commands::show_conversation,
            commands::show_answers_on,
            commands::catalogue,
            commands::save_action,
            commands::delete_action,
            commands::settings,
            commands::save_settings,
            commands::presets,
            commands::provider_models,
            commands::verify_provider,
            commands::dismiss,
        ])
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
