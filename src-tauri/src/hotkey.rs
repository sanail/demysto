//! The global Hotkey that opens the Palette.
//!
//! One Hotkey for now. Ticket 10 gives Actions their own, at which point the
//! registration below becomes a set rather than a single call.

use tauri::plugin::TauriPlugin;
use tauri::{AppHandle, Runtime};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

/// The Hotkey that opens the Palette.
///
/// `Cmd+Shift+Space` on macOS and `Ctrl+Shift+Space` elsewhere: a key away from
/// the one Spotlight and its equivalents already own, and taken by nothing on a
/// stock system.
fn for_palette() -> Shortcut {
    let platform = if cfg!(target_os = "macos") {
        Modifiers::SUPER
    } else {
        Modifiers::CONTROL
    };

    Shortcut::new(Some(platform | Modifiers::SHIFT), Code::Space)
}

pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, shortcut, event| {
            // On release rather than press: the Capture that follows sends a
            // copy keystroke, and it must not land while the modifiers of this
            // very Hotkey are still held down.
            if event.state() != ShortcutState::Released || shortcut != &for_palette() {
                return;
            }

            // A Capture waits on another application. Doing that here would
            // stall the thread that draws every window Demysto has.
            let app = app.clone();
            std::thread::spawn(move || crate::palette::toggle(&app));
        })
        .build()
}

/// Claims the Palette's Hotkey, and survives failing to.
///
/// Another application may already own it, and that is a reason to say so
/// rather than to refuse to start: the tray reaches everything the Hotkey does.
/// Ticket 11 owns making this visible. Changing it belongs with ticket 10,
/// which gives Actions Hotkeys of their own: the machinery for reading a key
/// combination out of the settings and claiming it is the same machinery, and
/// building half of it here would be building it twice.
pub fn register<R: Runtime>(app: &AppHandle<R>) {
    if let Err(error) = app.global_shortcut().register(for_palette()) {
        eprintln!("Demysto could not claim its Hotkey: {error}");
    }
}
