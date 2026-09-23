//! Which application the user is reading, on Windows, once Demysto's tray menu
//! has taken the foreground from it — and giving it back.
//!
//! Nothing records where the foreground was before the click on the tray icon,
//! but the Z-order still says it: the application the user was in is the
//! topmost ordinary window, below the taskbar and anything of Demysto's own.

use std::time::{Duration, Instant};

use windows_sys::core::BOOL;
use windows_sys::Win32::Foundation::{HWND, LPARAM};
use windows_sys::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetForegroundWindow, GetWindowLongW, GetWindowTextLengthW,
    GetWindowThreadProcessId, IsIconic, IsWindowVisible, SetForegroundWindow, GWL_EXSTYLE,
    WS_EX_TOOLWINDOW,
};

/// The shell's own windows, which are never what the user is reading: the
/// taskbars, the hidden icons' flyout, and the desktop behind everything.
const SHELL: [&str; 6] = [
    "Shell_TrayWnd",
    "Shell_SecondaryTrayWnd",
    "NotifyIconOverflowWindow",
    "TopLevelWindowForOverflowXamlIsland",
    "Progman",
    "WorkerW",
];

/// How long the foreground may take to arrive before the Capture goes ahead.
const SETTLE: Duration = Duration::from_millis(500);

/// Makes the application the user was reading the foreground one again, and
/// waits until it is. Does nothing when there is none — an empty desktop —
/// which leaves Capture to the clipboard, as it would have been.
pub fn give_back() {
    let Some(window) = reader() else {
        return;
    };

    // Allowed: showing the menu made Demysto the foreground process.
    unsafe { SetForegroundWindow(window) };

    let since = Instant::now();
    while unsafe { GetForegroundWindow() } != window && since.elapsed() < SETTLE {
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// The topmost window, in Z-order, that is an application's and not Demysto's.
fn reader() -> Option<HWND> {
    let mut found: Option<HWND> = None;
    unsafe {
        EnumWindows(
            Some(first_reader),
            &mut found as *mut Option<HWND> as LPARAM,
        )
    };
    found
}

unsafe extern "system" fn first_reader(window: HWND, found: LPARAM) -> BOOL {
    if !is_reader(window) {
        return 1;
    }
    *(found as *mut Option<HWND>) = Some(window);
    0
}

fn is_reader(window: HWND) -> bool {
    unsafe {
        if IsWindowVisible(window) == 0
            || IsIconic(window) != 0
            || GetWindowTextLengthW(window) == 0
        {
            return false;
        }
        if GetWindowLongW(window, GWL_EXSTYLE) as u32 & WS_EX_TOOLWINDOW != 0 {
            return false;
        }

        let mut pid = 0u32;
        GetWindowThreadProcessId(window, &mut pid);
        if pid == std::process::id() {
            return false;
        }

        // A suspended Store app keeps a visible window the compositor hides.
        let mut cloaked = 0u32;
        let asked = DwmGetWindowAttribute(
            window,
            DWMWA_CLOAKED as u32,
            &mut cloaked as *mut u32 as *mut _,
            std::mem::size_of::<u32>() as u32,
        );
        if asked == 0 && cloaked != 0 {
            return false;
        }

        let mut class = [0u16; 64];
        let length = GetClassNameW(window, class.as_mut_ptr(), class.len() as i32);
        let class = String::from_utf16_lossy(&class[..length.max(0) as usize]);
        !SHELL.contains(&class.as_str())
    }
}
