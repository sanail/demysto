//! Demysto's own area selection, where the system has no tool for it
//! (ADR-0019): a frozen frame of the monitor under the pointer, dimmed, with the
//! area being dragged shown undimmed.
//!
//! Only [`system`] knows which OS this is: it freezes the frame and gives the
//! foreground back. The window and the selection are the same everywhere.

use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use demysto_core::{CaptureError, Screen, SMALLEST_AREA};
use serde::Deserialize;
use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder};

pub const LABEL: &str = "overlay";

/// How long the page has to draw the frame and show itself before the overlay
/// counts as failed: a hidden window would hold every Hotkey until then.
const SHOWING: Duration = Duration::from_secs(10);

/// Set once the application is running: the core's `Screen` is handed over
/// before there is an `AppHandle` to give it.
static APP: OnceLock<AppHandle> = OnceLock::new();

/// The overlay on screen, `None` between uses.
static SHOWN: Mutex<Option<Shown>> = Mutex::new(None);

struct Shown {
    frame: Frame,
    chosen: Sender<Option<Area>>,
}

/// A still picture of one monitor in its physical pixels, BGRA from the top
/// row down, opaque.
struct Frame {
    width: u32,
    height: u32,
    bgra: Vec<u8>,
}

/// A rectangle in the frame's pixels, as the page measured the drag.
#[derive(Clone, Copy, Deserialize)]
pub struct Area {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

pub fn reach(app: &AppHandle) {
    let _ = APP.set(app.clone());
}

pub struct Overlay;

impl Screen for Overlay {
    /// Blocks until the overlay closes, so it must not be called from the
    /// thread the windows are drawn on.
    fn area(&self) -> Result<Option<Vec<u8>>, CaptureError> {
        let failed = |error: String| CaptureError::Screen(error);
        let app = APP
            .get()
            .ok_or_else(|| failed("Demysto is not running".to_owned()))?;

        let monitor = app
            .cursor_position()
            .and_then(|cursor| crate::palette::screen_holding(app, cursor))
            .map_err(|error| failed(error.to_string()))?
            .ok_or_else(|| failed("there is no monitor".to_owned()))?;
        let (at, size) = (*monitor.position(), *monitor.size());

        // First, so that the picture is what was on screen at the press.
        let frame = system::freeze(at, size).map_err(failed)?;
        let front = system::Front::now();

        let (chosen, choosing) = mpsc::channel();
        *SHOWN.lock().unwrap() = Some(Shown { frame, chosen });

        // Built hidden and placed in physical pixels; the page shows it once
        // the frame is drawn.
        let built = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("overlay.html".into()))
            .title("Demysto")
            .decorations(false)
            .resizable(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .shadow(false)
            .visible(false)
            .build()
            .and_then(|window| {
                window.set_position(at)?;
                window.set_size(size)?;
                // Otherwise the window manager keeps it clear of the panels.
                #[cfg(target_os = "linux")]
                window.set_fullscreen(true)?;
                Ok(window)
            });

        // A sender dropped without a word is a cancel.
        let answer = match &built {
            Ok(window) => match choosing.recv_timeout(SHOWING) {
                Err(RecvTimeoutError::Timeout) if !window.is_visible().unwrap_or(false) => {
                    Err("the overlay did not appear".to_owned())
                }
                Err(RecvTimeoutError::Timeout) => Ok(choosing.recv().ok().flatten()),
                Ok(area) => Ok(area),
                Err(RecvTimeoutError::Disconnected) => Ok(None),
            },
            Err(error) => Err(error.to_string()),
        };
        let shown = SHOWN.lock().unwrap().take();

        if let Ok(window) = &built {
            let _ = window.destroy();
            gone(app);
        }
        let area = answer.map_err(failed)?;

        let picture = area
            .filter(|area| area.width.min(area.height) >= SMALLEST_AREA)
            .zip(shown)
            .and_then(|(area, shown)| png(&shown.frame, area));

        if picture.is_none() {
            front.give_back();
        }

        Ok(picture)
    }
}

/// Waits until the window is gone: only then has the system finished handing
/// the foreground on, and only then may the next press build one by the same label.
fn gone(app: &AppHandle) {
    use tauri::Manager;

    let since = Instant::now();
    while app.get_webview_window(LABEL).is_some() && since.elapsed() < Duration::from_secs(1) {
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Closes the overlay as cancelled, answering whether there was one to close.
pub fn dismiss() -> bool {
    choose(None)
}

/// What the page measured, or `None` for Esc and the focus going elsewhere.
pub fn choose(area: Option<Area>) -> bool {
    match SHOWN.lock().unwrap().as_ref() {
        Some(shown) => {
            let _ = shown.chosen.send(area);
            true
        }
        None => false,
    }
}

/// The frame as a BMP file, which the page can show as it is: BMP's own pixel
/// order is the frame's, so this is a header and one copy.
pub fn frame_as_bmp() -> Option<Vec<u8>> {
    let shown = SHOWN.lock().unwrap();
    let frame = &shown.as_ref()?.frame;
    Some(bmp(frame))
}

fn bmp(frame: &Frame) -> Vec<u8> {
    const HEADERS: u32 = 14 + 40;
    let mut file = Vec::with_capacity(HEADERS as usize + frame.bgra.len());

    file.extend(b"BM");
    file.extend((HEADERS + frame.bgra.len() as u32).to_le_bytes());
    file.extend(0u32.to_le_bytes());
    file.extend(HEADERS.to_le_bytes());

    file.extend(40u32.to_le_bytes());
    file.extend((frame.width as i32).to_le_bytes());
    // Negative: the rows run from the top down.
    file.extend((-(frame.height as i32)).to_le_bytes());
    file.extend(1u16.to_le_bytes());
    file.extend(32u16.to_le_bytes());
    file.extend([0u8; 24]);

    file.extend(&frame.bgra);
    file
}

/// The area cut out of the frame as a PNG, `None` where nothing of it lies
/// within the frame.
fn png(frame: &Frame, area: Area) -> Option<Vec<u8>> {
    let x = area.x.min(frame.width);
    let y = area.y.min(frame.height);
    let width = area.width.min(frame.width - x);
    let height = area.height.min(frame.height - y);
    if width == 0 || height == 0 {
        return None;
    }

    let row = |line: u32| {
        let start = ((y + line) * frame.width + x) as usize * 4;
        &frame.bgra[start..start + width as usize * 4]
    };
    let mut rgba: Vec<u8> = (0..height).flat_map(row).copied().collect();
    for pixel in rgba.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }

    let image = image::RgbaImage::from_raw(width, height, rgba)?;
    let mut file = std::io::Cursor::new(Vec::new());
    image.write_to(&mut file, image::ImageFormat::Png).ok()?;

    Some(file.into_inner())
}

#[cfg(target_os = "windows")]
mod system {
    use std::mem::size_of;
    use std::ptr::null_mut;

    use tauri::{PhysicalPosition, PhysicalSize};
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::Graphics::Gdi::{
        BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC,
        GetDIBits, ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CAPTUREBLT,
        DIB_RGB_COLORS, SRCCOPY,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowThreadProcessId, SetForegroundWindow,
    };

    use super::Frame;

    /// Copies the screen through GDI. Demysto is per-monitor DPI aware, so
    /// these coordinates are physical pixels on every monitor.
    pub fn freeze(at: PhysicalPosition<i32>, size: PhysicalSize<u32>) -> Result<Frame, String> {
        let (width, height) = (size.width as i32, size.height as i32);
        let mut bgra = vec![0u8; size.width as usize * size.height as usize * 4];

        let lines = unsafe {
            let screen = GetDC(null_mut());
            let memory = CreateCompatibleDC(screen);
            let bitmap = CreateCompatibleBitmap(screen, width, height);
            let previous = SelectObject(memory, bitmap);

            // CAPTUREBLT takes in layered windows, which is what tooltips and
            // menus are.
            BitBlt(
                memory,
                0,
                0,
                width,
                height,
                screen,
                at.x,
                at.y,
                SRCCOPY | CAPTUREBLT,
            );
            SelectObject(memory, previous);

            let mut info: BITMAPINFO = std::mem::zeroed();
            info.bmiHeader = BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                ..std::mem::zeroed()
            };
            let lines = GetDIBits(
                memory,
                bitmap,
                0,
                size.height,
                bgra.as_mut_ptr().cast(),
                &mut info,
                DIB_RGB_COLORS,
            );

            DeleteObject(bitmap);
            DeleteDC(memory);
            ReleaseDC(null_mut(), screen);
            lines
        };

        if lines != height {
            return Err("the screen could not be copied".to_owned());
        }

        // GDI leaves the alpha byte at zero.
        for pixel in bgra.chunks_exact_mut(4) {
            pixel[3] = u8::MAX;
        }

        Ok(Frame {
            width: size.width,
            height: size.height,
            bgra,
        })
    }

    /// The window in front before the overlay took the foreground.
    pub struct Front(usize);

    impl Front {
        pub fn now() -> Self {
            Self(unsafe { GetForegroundWindow() } as usize)
        }

        /// Only while Demysto still has the foreground: a cancel by clicking
        /// another application leaves that one in front.
        pub fn give_back(self) {
            unsafe {
                let mut pid = 0u32;
                GetWindowThreadProcessId(GetForegroundWindow(), &mut pid);
                if pid == std::process::id() {
                    SetForegroundWindow(self.0 as HWND);
                }
            }
        }
    }
}

/// X11 lets any client read the screen. Reached through a connection of its
/// own, since GTK's belongs to the thread that draws the windows.
#[cfg(target_os = "linux")]
mod system {
    use std::cell::Cell;
    use std::os::raw::{c_int, c_uchar, c_ulong};
    use std::ptr::{null, null_mut};
    use std::sync::OnceLock;

    use tauri::{PhysicalPosition, PhysicalSize};
    use x11::xlib;

    use super::Frame;

    type Handler = unsafe extern "C" fn(*mut xlib::Display, *mut xlib::XErrorEvent) -> c_int;

    static PREVIOUS: OnceLock<Option<Handler>> = OnceLock::new();

    thread_local! {
        static OURS: Cell<*mut xlib::Display> = const { Cell::new(null_mut()) };
    }

    /// An error on this connection, such as a window gone between two calls,
    /// only fails the call; the handler installed before, GDK's or Xlib's own,
    /// would end the process.
    unsafe extern "C" fn on_error(
        display: *mut xlib::Display,
        error: *mut xlib::XErrorEvent,
    ) -> c_int {
        if OURS.get() == display {
            return 0;
        }
        match PREVIOUS.get().copied().flatten() {
            Some(previous) => previous(display, error),
            None => 0,
        }
    }

    struct Display(*mut xlib::Display);

    impl Display {
        fn open() -> Option<Self> {
            PREVIOUS.get_or_init(|| unsafe { xlib::XSetErrorHandler(Some(on_error)) });
            let display = unsafe { xlib::XOpenDisplay(null()) };
            OURS.set(display);
            (!display.is_null()).then_some(Self(display))
        }

        fn root(&self) -> xlib::Window {
            unsafe { xlib::XDefaultRootWindow(self.0) }
        }

        fn atom(&self, name: &std::ffi::CStr) -> xlib::Atom {
            unsafe { xlib::XInternAtom(self.0, name.as_ptr(), xlib::False) }
        }

        /// The first value of a 32-bit property, which Xlib hands over as a long.
        fn property(&self, window: xlib::Window, name: &std::ffi::CStr) -> Option<c_ulong> {
            let (mut kind, mut format, mut count, mut after) = (0, 0, 0, 0);
            let mut data: *mut c_uchar = null_mut();
            unsafe {
                xlib::XGetWindowProperty(
                    self.0,
                    window,
                    self.atom(name),
                    0,
                    1,
                    xlib::False,
                    xlib::AnyPropertyType as c_ulong,
                    &mut kind,
                    &mut format,
                    &mut count,
                    &mut after,
                    &mut data,
                );
                if data.is_null() {
                    return None;
                }
                let value = (format == 32 && count == 1).then(|| *data.cast::<c_ulong>());
                xlib::XFree(data.cast());
                value
            }
        }

        fn active(&self) -> xlib::Window {
            self.property(self.root(), c"_NET_ACTIVE_WINDOW")
                .unwrap_or(0)
        }
    }

    impl Drop for Display {
        fn drop(&mut self) {
            unsafe { xlib::XCloseDisplay(self.0) };
            OURS.set(null_mut());
        }
    }

    /// Copies the root window, whose pixels are the physical ones Tauri gives
    /// monitors in, whatever GDK's scale.
    pub fn freeze(at: PhysicalPosition<i32>, size: PhysicalSize<u32>) -> Result<Frame, String> {
        let display = Display::open().ok_or("the X display could not be opened")?;

        let bgra = unsafe {
            let image = xlib::XGetImage(
                display.0,
                display.root(),
                at.x,
                at.y,
                size.width,
                size.height,
                !0,
                xlib::ZPixmap,
            );
            if image.is_null() {
                return Err("the screen could not be copied".to_owned());
            }
            let copied = &*image;

            // BGRX in memory, which is what a depth-24 screen has everywhere.
            let bgrx = copied.bits_per_pixel == 32
                && copied.byte_order == xlib::LSBFirst
                && copied.red_mask == 0xff0000
                && copied.blue_mask == 0xff;
            let bgra = bgrx.then(|| {
                let row = size.width as usize * 4;
                (0..size.height as usize)
                    .flat_map(|line| {
                        let start = copied.data.add(line * copied.bytes_per_line as usize);
                        std::slice::from_raw_parts(start.cast::<u8>(), row)
                    })
                    .copied()
                    .collect::<Vec<u8>>()
            });
            xlib::XDestroyImage(image);
            bgra
        };

        let mut bgra = bgra.ok_or("the screen's pixel format is not BGRX")?;
        for pixel in bgra.chunks_exact_mut(4) {
            pixel[3] = u8::MAX;
        }

        Ok(Frame {
            width: size.width,
            height: size.height,
            bgra,
        })
    }

    /// The window the window manager names active before the overlay.
    pub struct Front(xlib::Window);

    impl Front {
        pub fn now() -> Self {
            Self(Display::open().map_or(0, |display| display.active()))
        }

        /// Only while no other application has taken the focus: the window
        /// manager is asked, as a pager would ask it, to activate the window
        /// again.
        pub fn give_back(self) {
            let Some(display) = Display::open() else {
                return;
            };
            let active = display.active();
            let ours = active == 0
                || display.property(active, c"_NET_WM_PID") == Some(std::process::id().into());
            if self.0 == 0 || !ours {
                return;
            }

            let mut message = xlib::XClientMessageEvent {
                type_: xlib::ClientMessage,
                serial: 0,
                send_event: xlib::True,
                display: display.0,
                window: self.0,
                message_type: display.atom(c"_NET_ACTIVE_WINDOW"),
                format: 32,
                data: xlib::ClientMessageData::new(),
            };
            message.data.set_long(0, 2);
            unsafe {
                xlib::XSendEvent(
                    display.0,
                    display.root(),
                    xlib::False,
                    xlib::SubstructureRedirectMask | xlib::SubstructureNotifyMask,
                    &mut xlib::XEvent::from(message),
                );
                xlib::XFlush(display.0);
            }
        }
    }
}

/// Lets the rest be compiled and tested where the overlay is not used.
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
mod system {
    use tauri::{PhysicalPosition, PhysicalSize};

    use super::Frame;

    pub fn freeze(_: PhysicalPosition<i32>, _: PhysicalSize<u32>) -> Result<Frame, String> {
        Err("there is no overlay on this platform".to_owned())
    }

    pub struct Front;

    impl Front {
        pub fn now() -> Self {
            Self
        }

        pub fn give_back(self) {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A frame whose every pixel says where it is: blue is x, green is y.
    fn frame(width: u32, height: u32) -> Frame {
        let bgra = (0..height)
            .flat_map(|y| (0..width).flat_map(move |x| [x as u8, y as u8, 0, 255]))
            .collect();
        Frame {
            width,
            height,
            bgra,
        }
    }

    fn decoded(file: Vec<u8>) -> image::RgbaImage {
        image::load_from_memory(&file).unwrap().into_rgba8()
    }

    #[test]
    fn the_picture_is_the_area_dragged_pixel_for_pixel() {
        let area = Area {
            x: 3,
            y: 5,
            width: 4,
            height: 2,
        };
        let picture = decoded(png(&frame(10, 10), area).unwrap());

        assert_eq!(picture.dimensions(), (4, 2));
        assert_eq!(picture.get_pixel(0, 0).0, [0, 5, 3, 255]);
        assert_eq!(picture.get_pixel(3, 1).0, [0, 6, 6, 255]);
    }

    #[test]
    fn an_area_past_the_edge_is_cut_at_the_edge() {
        let area = Area {
            x: 8,
            y: 8,
            width: 5,
            height: 5,
        };

        assert_eq!(
            decoded(png(&frame(10, 10), area).unwrap()).dimensions(),
            (2, 2)
        );
        assert!(png(&frame(10, 10), Area { x: 10, ..area }).is_none());
    }

    #[test]
    fn the_frame_reads_as_the_bmp_the_page_is_given() {
        let file = bmp(&frame(3, 2));

        assert_eq!(&file[..2], b"BM");
        assert_eq!(i32::from_le_bytes(file[22..26].try_into().unwrap()), -2);
        assert_eq!(file.len(), 54 + 3 * 2 * 4);
    }
}
