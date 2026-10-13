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

/// Waits until the window is gone: only then has Windows finished handing the
/// foreground on, and only then may the next press build one by the same label.
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

/// Lets the rest be compiled and tested where the overlay is not used.
#[cfg(not(target_os = "windows"))]
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
