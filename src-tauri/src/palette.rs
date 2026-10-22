//! The Palette: the window the Hotkey opens, over whatever the user is reading.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Mutex};
use std::time::Duration;

use demysto_core::{CaptureError, CaptureOutcome, Demysto};
use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, Monitor, PhysicalPosition, PhysicalRect,
    PhysicalSize, Runtime, WebviewWindow, Window,
};

use crate::underway::Underway;

/// The window label, fixed in `tauri.conf.json`.
pub const LABEL: &str = "palette";

/// Emitted to the Palette every time a Capture completes.
const CAPTURED_EVENT: &str = "palette://captured";

/// Emitted to the Palette when a Capture begins, so that it stops showing the
/// one before it.
const CAPTURING_EVENT: &str = "palette://capturing";

/// How close the Palette may come to the edge of the screen, in logical pixels.
const SCREEN_MARGIN: f64 = 8.0;

/// How long a Palette that has been handed a Capture is given to say it has
/// drawn it and fitted itself, before it is shown anyway.
const READY_WITHIN: Duration = Duration::from_millis(500);

/// How long a page that has never answered is given instead: one still loading
/// after launch answers up to about 600 ms after the Hotkey.
const LOADED_WITHIN: Duration = Duration::from_secs(2);

/// Whether the Palette has ever said it is ready, which is when its page has
/// loaded.
static ANSWERED: AtomicBool = AtomicBool::new(false);

/// Told when the Palette says it is ready, while a Capture waits to show it.
static WAITING: Mutex<Option<mpsc::Sender<()>>> = Mutex::new(None);

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

/// Lets the user choose an area of the screen and opens the Palette on it.
///
/// Nothing is shown and nothing is said when they cancel; a tool that failed is
/// reported in a notification, since a Palette retrying would read the
/// clipboard instead. The clipboard and the foreground application are never
/// touched.
pub fn area<R: Runtime>(app: &AppHandle<R>) {
    // Pressed again while the overlay is up, it cancels it.
    if crate::overlay::dismiss() {
        return;
    }

    off_thread(app, |app, window| {
        let Some(_opening) = Underway::claim(&OPENING) else {
            return;
        };

        // Out of the picture the user is about to take.
        let _ = window.hide();

        let outcome = match app.state::<Demysto>().capture_area() {
            None => return,
            Some(CaptureOutcome::Failed(CaptureError::Screen(detail))) => {
                return crate::notify::screen_not_taken(app, &detail);
            }
            Some(CaptureOutcome::Failed(CaptureError::ScreenRecording)) => {
                let why = app
                    .state::<Demysto>()
                    .words()
                    .text("capture-no-screen-recording");
                return crate::notify::screen_not_taken(app, &why);
            }
            Some(outcome) => outcome,
        };

        let _ = position_on_screen(app, window);

        show_with(app, window, &outcome);
    });
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

    let _ = position_on_screen(app, window);

    show_with(app, window, &outcome);
}

/// Hands the Palette a Capture and shows it once it has drawn it and fitted
/// itself, so that it does not resize in front of the user.
///
/// A window still loading hears nothing, but asks for the Capture itself when
/// it mounts and answers then; one that never answers is shown after
/// [`LOADED_WITHIN`] all the same.
fn show_with<R: Runtime>(app: &AppHandle<R>, window: &WebviewWindow<R>, outcome: &CaptureOutcome) {
    let (tell, told) = mpsc::channel();
    *WAITING.lock().unwrap() = Some(tell);

    let _ = window.emit(CAPTURED_EVENT, outcome);
    let _ = told.recv_timeout(match ANSWERED.load(Ordering::Relaxed) {
        true => READY_WITHIN,
        false => LOADED_WITHIN,
    });

    WAITING.lock().unwrap().take();
    show(app, window);
}

/// Lets a Capture waiting in [`show_with`] show the Palette.
pub fn ready() {
    ANSWERED.store(true, Ordering::Relaxed);

    if let Some(tell) = WAITING.lock().unwrap().take() {
        let _ = tell.send(());
    }
}

/// Fits the Palette to the height its content asks for, in logical pixels, as
/// far as the work area allows, and centres it again at that height.
pub fn fit<R: Runtime>(
    app: &AppHandle<R>,
    window: &WebviewWindow<R>,
    height: f64,
) -> tauri::Result<()> {
    // Wayland names no screen for a window that has never been shown, which the
    // Palette is the first time it opens.
    let screen = match window.current_monitor()? {
        Some(screen) => Some(screen),
        None => screen_holding(app, app.cursor_position().unwrap_or_default())?,
    };
    let Some(screen) = screen else {
        return Ok(());
    };

    let scale = screen.scale_factor();
    let work_area = *screen.work_area();
    // The width is the configured one rather than the drawn one: on Windows the
    // outer size takes in a frame the inner size does not, and the Palette
    // would widen by it on every fit.
    let size: PhysicalSize<u32> = LogicalSize::new(
        configured_size(app).map_or(0.0, |configured| configured.width),
        fitted_height(height, work_area.size.height, scale),
    )
    .to_physical(scale);

    window.set_size(size)?;

    let (outer, inner) = (window.outer_size()?, window.inner_size()?);
    let framed = PhysicalSize::new(
        size.width + outer.width.saturating_sub(inner.width),
        size.height + outer.height.saturating_sub(inner.height),
    );

    window.set_position(placement(work_area, framed, scale))
}

/// The height asked for, cut down to what fits between the work area's
/// margins; `work_area` is physical, the rest logical.
fn fitted_height(asked: f64, work_area: u32, scale: f64) -> f64 {
    asked.min(f64::from(work_area) / scale - 2.0 * SCREEN_MARGIN)
}

/// Puts the Palette in the middle of the screen the pointer is on.
///
/// Not beside the pointer: the pointer is often nowhere near what is being
/// read — the text was selected with the keyboard, or the pointer was left
/// parked — and a fixed place is one the eyes learn.
fn position_on_screen<R: Runtime>(
    app: &AppHandle<R>,
    window: &WebviewWindow<R>,
) -> tauri::Result<()> {
    // No screen at all, not even a primary one: the Palette opens where it was.
    let Some(screen) = screen_holding(app, app.cursor_position()?)? else {
        return Ok(());
    };

    // The scale of the screen the Palette is going to, not of the one it was
    // last shown on: on a mixed-DPI desktop the two disagree, and the window
    // takes the new one's size the moment it lands there.
    let scale = screen.scale_factor();
    let size = logical_size(app, window)?.to_physical(scale);

    let at = placement(*screen.work_area(), size, scale);

    window.set_position(at)
}

/// The Palette's size in logical pixels.
///
/// GTK knows no size for a window it has never drawn and answers 0×0, which
/// would centre the Palette's corner rather than the Palette the first time it
/// opens on Linux. The size it is configured with stands in until then.
fn logical_size<R: Runtime>(
    app: &AppHandle<R>,
    window: &WebviewWindow<R>,
) -> tauri::Result<LogicalSize<f64>> {
    let drawn = window.outer_size()?;

    if drawn.width > 0 && drawn.height > 0 {
        return Ok(drawn.to_logical(window.scale_factor()?));
    }

    Ok(configured_size(app).unwrap_or_else(|| drawn.to_logical(1.0)))
}

/// The size the Palette is configured with in `tauri.conf.json`.
fn configured_size<R: Runtime>(app: &AppHandle<R>) -> Option<LogicalSize<f64>> {
    app.config()
        .app
        .windows
        .iter()
        .find(|configured| configured.label == LABEL)
        .map(|configured| LogicalSize::new(configured.width, configured.height))
}

/// Where the Palette's top-left corner goes on a screen: centred on its work
/// area both ways, and kept off the edges by [`SCREEN_MARGIN`], which `scale`
/// turns into physical pixels.
fn placement(
    work_area: PhysicalRect<i32, u32>,
    size: PhysicalSize<u32>,
    scale: f64,
) -> PhysicalPosition<f64> {
    let (left, top) = (
        f64::from(work_area.position.x),
        f64::from(work_area.position.y),
    );
    let (width, height) = (
        f64::from(work_area.size.width),
        f64::from(work_area.size.height),
    );
    let (wide, tall) = (f64::from(size.width), f64::from(size.height));
    let margin = SCREEN_MARGIN * scale;

    let x = left + (width - wide) / 2.0;
    let y = top + (height - tall) / 2.0;

    // `max` after `min`, so that a window larger than the screen still has its
    // top-left corner on it rather than off the near edge.
    PhysicalPosition::new(
        x.min(left + width - wide - margin).max(left + margin),
        y.min(top + height - tall - margin).max(top + margin),
    )
}

/// The screen the pointer is on, or the primary one when it is on none.
///
/// The monitors are walked here rather than asked for through
/// `monitor_from_point`, which answers in a different coordinate space from the
/// one the pointer arrives in: `cursor_position` is physical pixels, and that
/// lookup compares against `CGDisplayBounds`, which is logical points. On a
/// screen at 2x the two agree only for a pointer in the top-left quarter, and
/// past that the screen was simply lost. A monitor's own
/// position and size are physical, so walking them keeps everything here in one
/// space and needs no conversion on any platform.
///
/// The primary screen stands in when the pointer is on none, which is a gap
/// between two of them.
pub(crate) fn screen_holding<R: Runtime>(
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
/// screen deaf, with the letters going on into the document underneath.
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
/// the keyboard in the first place.
///
/// The display is asked rather than GTK, because a grab does not move the input
/// focus; it only redirects what is typed. The window X names is still this
/// one, and that is the whole difference between a Hotkey and somebody clicking
/// away.
pub fn lost_the_keyboard<R: Runtime>(window: &Window<R>) {
    if !kept_the_keyboard(window) {
        let _ = window.hide();
    }
}

/// Whether a focus-out was only a Hotkey's grab, which leaves the window the
/// keyboard.
pub fn kept_the_keyboard<R: Runtime>(window: &Window<R>) -> bool {
    #[cfg(target_os = "linux")]
    return still_has_the_keyboard(window);

    #[cfg(not(target_os = "linux"))]
    {
        let _ = window;
        false
    }
}

/// Whether X still names the window as the one with the keyboard.
///
/// `false` for anything that is not an X11 session, and for anything that
/// cannot be asked: nowhere else does a focus-out arrive that a window should
/// sit through, and treating silence as "still focused" would leave a window
/// on screen that the user has walked away from.
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

/// A window as X11 knows it, or `None` in a session that is not
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

    // tao resizes and moves a window on the main dispatch queue, which the event
    // loop does not wait for: the panel would come up before its fit lands.
    dispatch2::DispatchQueue::main().exec_sync(|| {});

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

/// Turns the Palette into an `NSPanel`.
///
/// Two things come with the class that an ordinary window cannot have: it may
/// be shown alongside a full-screen application instead of on Demysto's own
/// Space, and it takes the keyboard without taking activation away from the
/// application the user is reading.
#[cfg(target_os = "macos")]
// `tauri-nspanel` takes the collection behaviour as `cocoa`'s type, and `cocoa`
// is deprecated in favour of `objc2-app-kit`; the choice is that crate's.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn area(x: i32, y: i32, width: u32, height: u32) -> PhysicalRect<i32, u32> {
        PhysicalRect {
            position: PhysicalPosition::new(x, y),
            size: PhysicalSize::new(width, height),
        }
    }

    #[test]
    fn the_palette_is_centred_on_the_work_area_both_ways() {
        // A work area that starts away from the origin, as one does beside a
        // menu bar or on a second screen to the left of the primary one.
        let at = placement(
            area(-1920, 50, 1920, 1000),
            PhysicalSize::new(600, 400),
            1.0,
        );

        assert_eq!(at, PhysicalPosition::new(-1260.0, 350.0));
    }

    #[test]
    fn a_palette_too_big_for_the_work_area_is_held_off_its_edges_at_its_scale() {
        // At 2x the logical margin is twice as many physical pixels.
        let at = placement(area(100, 60, 800, 600), PhysicalSize::new(1000, 900), 2.0);

        assert_eq!(at, PhysicalPosition::new(116.0, 76.0));
    }

    #[test]
    fn the_height_asked_for_is_kept_while_the_work_area_has_room() {
        assert_eq!(fitted_height(240.0, 1600, 2.0), 240.0);
    }

    #[test]
    fn the_height_stops_at_the_work_area_less_both_margins() {
        // 1600 physical pixels at 2x are 800 logical, less 8 above and below.
        assert_eq!(fitted_height(1000.0, 1600, 2.0), 784.0);
    }
}
