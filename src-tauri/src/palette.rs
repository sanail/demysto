//! The Palette: the window the Hotkey opens, over whatever the user is reading.

use std::sync::atomic::AtomicBool;

use demysto_core::Demysto;
use tauri::{
    AppHandle, Emitter, Manager, Monitor, PhysicalPosition, Runtime, WebviewWindow, Window,
};

use crate::underway::Underway;

/// The window label, fixed in `tauri.conf.json`.
pub const LABEL: &str = "palette";

/// Emitted to the Palette every time a Capture completes.
const CAPTURED_EVENT: &str = "palette://captured";

/// Emitted to the Palette when a Capture begins, so that it stops showing the
/// one before it.
const CAPTURING_EVENT: &str = "palette://capturing";

/// How far from the cursor the Palette's corner sits, in logical pixels, so
/// that the window does not open underneath the pointer itself.
const CURSOR_OFFSET: f64 = 12.0;

/// How close the Palette may come to the edge of the screen, in logical pixels.
const SCREEN_MARGIN: f64 = 8.0;

/// Whether the Palette is in the middle of opening. Held through
/// [`Underway`] rather than touched directly; see that module for why the
/// detached thread [`off_thread`] spawns makes it a guard.
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
    // Held until the Palette is up and holds the focus. Any shorter and a
    // Hotkey pressed twice in a hurry sends a second copy keystroke, by then
    // aimed at the Palette itself.
    let Some(_opening) = Underway::claim(&OPENING) else {
        return;
    };

    // Hiding the Palette does not unload it, so it is still showing the last
    // Capture. Told first that another is under way, it goes back to saying it
    // is reading — which is what the user should see if this one turns out to
    // have nothing to show, or never reaches the window at all.
    let _ = window.emit(CAPTURING_EVENT, ());

    // Before the window is shown: the copy keystroke has to reach the
    // application the user is reading, and that is only the foreground
    // application until the Palette takes the focus away from it.
    let outcome = app.state::<Demysto>().capture();

    let _ = position_at_cursor(app, window);

    // Also before the window is shown, so that what it comes up showing is this
    // Capture rather than the one before it. A window that has never loaded
    // hears neither event, and asks for the Capture itself when it mounts.
    let _ = window.emit(CAPTURED_EVENT, &outcome);

    show(app, window);
}

/// Puts the Palette next to the pointer, kept whole on the screen it is on.
fn position_at_cursor<R: Runtime>(
    app: &AppHandle<R>,
    window: &WebviewWindow<R>,
) -> tauri::Result<()> {
    let cursor = app.cursor_position()?;
    let size = window.outer_size()?;
    let screen = screen_holding(app, cursor)?;

    // The scale of the screen the cursor is on, not of the one the Palette was
    // last shown on. Everything else here is in physical pixels, and only
    // CURSOR_OFFSET and SCREEN_MARGIN are stated in logical ones, so this is
    // what carries those two across — and on a mixed-DPI desktop the screens
    // disagree about it, which is what turns a 12-pixel gap into 24, or 6.
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

/// The screen the pointer is on, or the primary one when it is on none.
///
/// The monitors are walked here rather than asked for through
/// `monitor_from_point`, which answers in a different coordinate space from the
/// one the pointer arrives in: `cursor_position` is physical pixels, and that
/// lookup compares against `CGDisplayBounds`, which is logical points. On a
/// screen at 2x the two agree only for a pointer in the top-left quarter, and
/// past that the screen was simply lost — which used to mean the Palette was
/// not kept on it at all, exactly where it most needed to be. A monitor's own
/// position and size are physical, so walking them keeps everything here in one
/// space and needs no conversion on any platform.
///
/// The primary screen stands in when the pointer is on none, which is a gap
/// between two of them: somewhere to clamp against is better than nowhere, and
/// nowhere is how this went wrong.
fn screen_holding<R: Runtime>(
    app: &AppHandle<R>,
    cursor: PhysicalPosition<f64>,
) -> tauri::Result<Option<Monitor>> {
    if let Some(screen) = app
        .available_monitors()?
        .into_iter()
        .find(|screen| holds(screen, cursor))
    {
        return Ok(Some(screen));
    }

    app.primary_monitor()
}

/// Whether a point is on a screen, in the physical pixels both are given in.
fn holds(screen: &Monitor, point: PhysicalPosition<f64>) -> bool {
    let origin = screen.position();
    let size = screen.size();

    let within = |start: i32, extent: u32, at: f64| {
        let start = f64::from(start);

        at >= start && at < start + f64::from(extent)
    };

    within(origin.x, size.width, point.x) && within(origin.y, size.height, point.y)
}

/// Brings the Palette up in front of whatever the user is reading.
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn show<R: Runtime>(_app: &AppHandle<R>, window: &WebviewWindow<R>) {
    let _ = window.show();
    let _ = window.set_focus();
}

/// Brings the Palette up, and on X11 carries the one thing an activation there
/// is judged by, which `set_focus` does not.
///
/// Asking is not enough. KWin grants an activation only when the moment it
/// carries is no older than the moment the focused window last heard from the
/// user; a request carrying nothing loses that comparison against any
/// application somebody has just been typing in. The Palette got away with it
/// exactly once per run — a window the compositor has never managed is
/// activated as it is mapped — and every Hotkey after that put the Palette on
/// screen deaf, with the letters going on into the document underneath
/// (ticket 20).
///
/// The moment is asked of the X server here rather than carried from the
/// keypress, and that is the one surprise in this. A Hotkey has no timestamp
/// anywhere in it to carry — the handler is given a shortcut and its state and
/// nothing else — but even a moment recorded when the key was pressed would
/// lose, because between the press and this line Demysto sends the Capture's
/// own copy chord to the very window it is about to be compared against, and
/// that chord is the newer user input. The honest moment is this one: the
/// Palette is being put on screen now, because of a key the user has just
/// pressed, and no rule is being suspended to allow it.
///
/// On Wayland there is no such window and nothing to carry: the compositor
/// decides activation and Demysto is a guest in that decision (ADR-0003), so
/// the plain request stands.
#[cfg(target_os = "linux")]
fn show<R: Runtime>(app: &AppHandle<R>, window: &WebviewWindow<R>) {
    let _ = window.show();

    let window = window.clone();

    on_main_thread(app, move |_| {
        if present_with_server_time(&window).is_none() {
            let _ = window.set_focus();
        }
    });
}

/// Presents the Palette carrying the X server's own idea of now, or `None`
/// where this is not an X11 session and there is no such thing to carry.
///
/// GTK is what puts the moment on the window: `present_with_time` writes it to
/// `_NET_WM_USER_TIME` and sends it with the activation. Left to itself GTK
/// fills that property from the last input event the window received, and the
/// Palette receives none — the Hotkey is claimed from the whole display, so it
/// never arrives as an event of this window's at all, and the property stays
/// at whatever it was the first time the window was drawn.
#[cfg(target_os = "linux")]
fn present_with_server_time<R: Runtime>(window: &WebviewWindow<R>) -> Option<()> {
    use gtk::prelude::GtkWindowExt;

    let frame = window.gtk_window().ok()?;
    let drawn = on_the_display(&frame)?;

    frame.present_with_time(gdkx11::functions::x11_get_server_time(&drawn));

    Some(())
}

/// Puts the Palette away, the focus having gone elsewhere — unless on X11 it
/// has not.
///
/// A Hotkey is a keyboard grab, and the X server announces a grab to the window
/// that had the keyboard as a focus-out, exactly like the user clicking on
/// something else. GTK passes both on as the same event, so the Palette's own
/// Hotkey, pressed to put it away, arrives as "you have lost the focus" a
/// moment before it arrives as a Hotkey: the Palette hides itself, and the
/// press that follows finds nothing open and opens it again. The Hotkey stops
/// closing the Palette — which nobody noticed while the Palette was not getting
/// the keyboard in the first place (ticket 20).
///
/// The display is asked rather than GTK, because a grab does not move the input
/// focus; it only redirects what is typed. The window X names is still this
/// one, and that is the whole difference between a Hotkey and somebody clicking
/// away.
pub fn lost_the_keyboard<R: Runtime>(window: &Window<R>) {
    #[cfg(target_os = "linux")]
    if still_has_the_keyboard(window) {
        return;
    }

    let _ = window.hide();
}

/// Whether X still names the Palette's window as the one with the keyboard.
///
/// `false` for anything that is not an X11 session, and for anything that
/// cannot be asked: nowhere else does a focus-out arrive that the Palette
/// should sit through, and treating silence as "still focused" would leave a
/// Palette on screen that the user has walked away from.
#[cfg(target_os = "linux")]
fn still_has_the_keyboard<R: Runtime>(window: &Window<R>) -> bool {
    use gtk::glib::prelude::Cast;
    use gtk::glib::translate::ToGlibPtr;

    let Some(drawn) = window.gtk_window().ok().as_ref().and_then(on_the_display) else {
        return false;
    };
    let on = drawn.upcast_ref::<gtk::gdk::Window>().display();
    let Ok(display) = on.downcast::<gdkx11::X11Display>() else {
        return false;
    };

    let mut has_it = 0;
    let mut reverts_to = 0;

    // Xlib, because neither GDK nor GTK offers this: everything they say about
    // the focus is said through the very events being told apart here.
    unsafe {
        x11::xlib::XGetInputFocus(
            gdkx11::ffi::gdk_x11_display_get_xdisplay(display.to_glib_none().0),
            &mut has_it,
            &mut reverts_to,
        );
    }

    // The window X names is almost never the one asked about, and comparing the
    // two directly answers `false` to a Palette that plainly has the keyboard:
    // GTK keeps an unmapped child window inside every toplevel purely to hold
    // the focus, and that child is what X names. So the answer is taken from
    // whichever toplevel the named window lives inside.
    let Some(named) = gdkx11::X11Window::lookup_for_display(&display, has_it) else {
        return false;
    };

    named.upcast_ref::<gtk::gdk::Window>().toplevel() == *drawn.upcast_ref::<gtk::gdk::Window>()
}

/// The Palette's window as X11 knows it, or `None` in a session that is not
/// X11 — on Wayland this is a `GdkWaylandWindow` and none of it applies.
#[cfg(target_os = "linux")]
fn on_the_display(frame: &gtk::ApplicationWindow) -> Option<gdkx11::X11Window> {
    use gtk::glib::prelude::Cast;
    use gtk::prelude::WidgetExt;

    frame.window()?.downcast::<gdkx11::X11Window>().ok()
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

/// Runs something on the thread the windows are drawn on, and waits for it.
///
/// AppKit insists on it, and so does GTK. The caller is always a Capture's own
/// thread, never the main one, so the wait cannot deadlock — and it is what
/// lets the guard in [`open`] stay closed until the Palette is genuinely up.
#[cfg(any(target_os = "macos", target_os = "linux"))]
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
