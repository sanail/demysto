//! The Palette: the window the Hotkey opens, over whatever the user is reading.

use std::sync::atomic::{AtomicBool, Ordering};

use demysto_core::Demysto;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, Runtime, WebviewWindow};

/// The window label, fixed in `tauri.conf.json`.
pub const LABEL: &str = "palette";

/// Emitted to the Palette every time a Capture completes.
const CAPTURED_EVENT: &str = "palette://captured";

/// How far from the cursor the Palette's corner sits, in logical pixels, so
/// that the window does not open underneath the pointer itself.
const CURSOR_OFFSET: f64 = 12.0;

/// How close the Palette may come to the edge of the screen, in logical pixels.
const SCREEN_MARGIN: f64 = 8.0;

/// Whether the Palette is in the middle of opening. Single instance is
/// enforced, so one flag covers the whole application.
static OPENING: AtomicBool = AtomicBool::new(false);

/// Opens the Palette, or closes it when it is already open.
pub fn toggle<R: Runtime>(app: &AppHandle<R>) {
    off_thread(app, |app, window| {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            open(app, window);
        }
    });
}

/// Opens the Palette whether or not it is already open, which is what the tray
/// and a second launch ask for.
pub fn reveal<R: Runtime>(app: &AppHandle<R>) {
    off_thread(app, open);
}

/// Runs something against the Palette away from the thread that draws it.
///
/// Everything here begins with a Capture, and a Capture waits on another
/// application; on the interface thread that wait would freeze every window
/// Demysto has.
fn off_thread<R: Runtime>(
    app: &AppHandle<R>,
    act: impl FnOnce(&AppHandle<R>, &WebviewWindow<R>) + Send + 'static,
) {
    let app = app.clone();

    std::thread::spawn(move || {
        if let Some(window) = app.get_webview_window(LABEL) {
            act(&app, &window);
        }
    });
}

fn open<R: Runtime>(app: &AppHandle<R>, window: &WebviewWindow<R>) {
    if OPENING.swap(true, Ordering::SeqCst) {
        return;
    }

    // Before the window is shown: the copy keystroke has to reach the
    // application the user is reading, and that is only the foreground
    // application until the Palette takes the focus away from it.
    let outcome = app.state::<Demysto>().capture();

    let _ = position_at_cursor(app, window);
    show(app, window);

    // Emitted after the window is up, and asked for again by the Palette when
    // it mounts, so that neither order loses the Capture.
    let _ = window.emit(CAPTURED_EVENT, &outcome);

    // Released only once the Palette holds the focus. Any earlier and a Hotkey
    // pressed twice in a hurry sends a second copy keystroke, by then aimed at
    // the Palette itself.
    OPENING.store(false, Ordering::SeqCst);
}

/// Puts the Palette next to the pointer, kept whole on the screen it is on.
fn position_at_cursor<R: Runtime>(
    app: &AppHandle<R>,
    window: &WebviewWindow<R>,
) -> tauri::Result<()> {
    let cursor = app.cursor_position()?;
    let size = window.outer_size()?;
    let screen = app.monitor_from_point(cursor.x, cursor.y)?;

    // The scale of the screen the cursor is on, not of the one the Palette was
    // last shown on. Every distance here is in logical pixels of the screen the
    // window is about to appear on, and on a mixed-DPI desktop those two
    // screens disagree — a 12-pixel gap becomes 24, or 6.
    let scale = match &screen {
        Some(screen) => screen.scale_factor(),
        None => window.scale_factor()?,
    };

    let mut x = cursor.x + CURSOR_OFFSET * scale;
    let mut y = cursor.y + CURSOR_OFFSET * scale;

    if let Some(screen) = screen {
        let origin = screen.position();
        let bounds = screen.size();
        let margin = SCREEN_MARGIN * scale;

        let furthest_x = f64::from(origin.x + bounds.width as i32 - size.width as i32) - margin;
        let furthest_y = f64::from(origin.y + bounds.height as i32 - size.height as i32) - margin;

        // `max` after `min`, so that a window larger than the screen still has
        // its top-left corner on it rather than off the near edge.
        x = x.min(furthest_x).max(f64::from(origin.x) + margin);
        y = y.min(furthest_y).max(f64::from(origin.y) + margin);
    }

    window.set_position(PhysicalPosition::new(x, y))
}

/// Brings the Palette up in front of whatever the user is reading.
#[cfg(not(target_os = "macos"))]
fn show<R: Runtime>(_app: &AppHandle<R>, window: &WebviewWindow<R>) {
    let _ = window.show();
    let _ = window.set_focus();
}

/// Brings the Palette up as the panel it is, which is what keeps macOS from
/// carrying the user off to Demysto's own Space to show it to them.
#[cfg(target_os = "macos")]
fn show<R: Runtime>(app: &AppHandle<R>, _window: &WebviewWindow<R>) {
    use tauri_nspanel::ManagerExt;

    on_main_thread(app, |app| {
        if let Ok(panel) = app.get_webview_panel(LABEL) {
            panel.show();
        }
    });
}

/// Runs something on the thread AppKit insists on, and waits for it.
///
/// The caller is always a Capture's own thread, never the main one, so the wait
/// cannot deadlock — and it is what lets the guard in [`open`] stay closed
/// until the Palette is genuinely up.
#[cfg(target_os = "macos")]
fn on_main_thread<R: Runtime>(
    app: &AppHandle<R>,
    act: impl FnOnce(&AppHandle<R>) + Send + 'static,
) {
    let (done, wait) = std::sync::mpsc::channel();
    let handle = app.clone();

    let dispatched = app.run_on_main_thread(move || {
        act(&handle);
        let _ = done.send(());
    });

    if dispatched.is_ok() {
        let _ = wait.recv();
    }
}

/// Turns the Palette into an `NSPanel`, per the spec's *Shell and platform*.
///
/// Two things come with the class that an ordinary window cannot have: it may
/// be shown alongside a full-screen application instead of on Demysto's own
/// Space, and it takes the keyboard without taking activation away from the
/// application the user is reading.
#[cfg(target_os = "macos")]
// `tauri-nspanel` takes the collection behaviour as `cocoa`'s type, and `cocoa`
// is deprecated in favour of `objc2-app-kit`. The choice belongs to the crate
// the spec names, not to us.
#[allow(deprecated)]
pub fn into_panel<R: Runtime>(window: &WebviewWindow<R>) -> tauri::Result<()> {
    use tauri_nspanel::cocoa::appkit::NSWindowCollectionBehavior;
    use tauri_nspanel::WebviewWindowExt;

    /// `NSWindowStyleMaskNonActivatingPanel`.
    const NON_ACTIVATING_PANEL: i32 = 1 << 7;
    /// `NSFloatingWindowLevel`.
    const FLOATING: i32 = 3;

    let panel = window.to_panel()?;

    panel.set_style_mask(NON_ACTIVATING_PANEL);
    panel.set_level(FLOATING);
    panel.set_collection_behaviour(
        NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
            | NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary
            | NSWindowCollectionBehavior::NSWindowCollectionBehaviorStationary,
    );
    // Escape and a lost focus hide the Palette rather than closing it, but a
    // panel that frees itself on close would take the whole window with it.
    panel.set_released_when_closed(false);

    Ok(())
}
