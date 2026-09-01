//! Demysto's Tauri shell.
//!
//! This crate owns windows, the tray, and the command bridge. The product logic
//! lives in `demysto-core`, which knows nothing about Tauri; see ADR-0001.

mod accessibility;
mod autostart;
mod commands;
mod dock;
mod folder;
mod hotkey;
/// The menu bar, which exists on macOS alone and only for the key equivalents —
/// the module says why.
#[cfg(target_os = "macos")]
mod menu;
mod notify;
mod palette;
/// The Hotkey on Wayland.
///
/// Compiled everywhere, though only Linux reaches the half that talks to the
/// portal: the other half translates a Hotkey into the portal's own syntax, and
/// is plain string work that every platform's suite is welcome to check.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
mod portal;
mod result;
mod settings;
mod tray;
mod underway;
mod welcome;

use demysto_core::Demysto;
use tauri::{Manager, RunEvent, WindowEvent};

pub fn run() {
    let config_dir = match demysto_core::config_dir() {
        Ok(config_dir) => config_dir,
        Err(error) => {
            // The one failure with nowhere to be reported: there is no
            // configuration directory, so there is no log folder inside it
            // either, and a windowed build has no stderr for a user launching
            // from Finder or Explorer. What can be done is done — the message
            // names the variable that fixes it — and the process stops.
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
            .plugin(hotkey::plugin())
            .plugin(tauri_plugin_notification::init())
            // A launch agent on macOS rather than a login item added through
            // System Events: the second is an Automation permission to ask for
            // on top of the one Demysto already needs, for a list the user can
            // edit either way. Nothing is registered by loading the plugin —
            // the first-run flow asks, and `autostart` is the only caller.
            .plugin(tauri_plugin_autostart::init(
                tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                None,
            ));
    }

    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_nspanel::init());
    }

    let app = builder
        .manage(Demysto::new(config_dir, env!("CARGO_PKG_VERSION")))
        .setup(|app| {
            // Accessory to begin with, because at startup the Palette is all
            // there is. From here the policy follows the windows; see `dock`.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            tray::build(app)?;

            // After the tray and before everything else: with no menu item
            // carrying `Cmd+C` there is nowhere to take a selection in the
            // Conversation window (user story 15).
            #[cfg(target_os = "macos")]
            menu::build(app.handle())?;

            // Claimed from the catalogue, so that an Action already carrying a
            // Hotkey answers to it from the first keypress rather than from the
            // first save. What could not be claimed goes to the log, which is
            // where a report with no window to appear on belongs: the Settings
            // window shows these same sentences whenever it is opened, because
            // it claims the set again as it reads the catalogue.
            let demysto = app.state::<Demysto>();
            let palette = demysto.palette_hotkey();

            for said in hotkey::claim(
                app.handle(),
                palette.as_deref(),
                &demysto.catalogue().actions,
            ) {
                demysto.note(&said);
            }

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

                dock::follows_the_windows(
                    window.app_handle(),
                    dock::Change::Hiding(window.label()),
                );

                welcome::gone(window.app_handle(), window.label());
            }
            // The Palette is not a window anybody should have to manage: losing
            // the focus is the same instruction as pressing Escape. Which of
            // the two kinds of losing it this was is the Palette's own
            // question — see `palette::lost_the_keyboard`.
            WindowEvent::Focused(false) if window.label() == palette::LABEL => {
                palette::lost_the_keyboard(window);
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::status,
            commands::language,
            commands::last_capture,
            commands::actions,
            commands::run,
            commands::follow_up,
            commands::stop,
            commands::retry,
            commands::continue_answer,
            commands::models,
            commands::open_logs,
            commands::open_accessibility,
            commands::accessibility_asked_for,
            commands::autostart,
            commands::set_autostart,
            commands::open_settings,
            commands::conversation,
            commands::selection,
            commands::conversations,
            commands::show_conversation,
            commands::show_answers_on,
            commands::catalogue,
            commands::hotkeys,
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

    app.run(|app, event| match event {
        // A fresh installation is met by the flow rather than by a tray icon it
        // has to work out for itself (user story 57).
        //
        // Here rather than in `setup`, which is where every other window is
        // prepared: on WebKitGTK a window shown before the event loop is
        // running never paints. It comes up as a correct, complete, white
        // rectangle — the page is there, the accessibility tree reads it back
        // in full, and nothing at all is on screen. macOS and Windows draw it
        // either way, so this is the one ordering the three platforms disagree
        // about. By this event the Hotkey has been claimed, which the last step
        // invites a press of.
        RunEvent::Ready => {
            if !app.state::<Demysto>().welcomed() {
                // ВРЕМЕННО, ДЛЯ ОПЫТА: показать через три секунды.
                welcome::reveal(app);
            }
        }
        // The last window closing is not a request to quit; an explicit exit,
        // which carries a code, is.
        RunEvent::ExitRequested {
            code: None, api, ..
        } => api.prevent_exit(),
        _ => {}
    });
}
