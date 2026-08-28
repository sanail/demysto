//! The real desktop: the clipboard, and a copy keystroke aimed at whatever the
//! user is looking at.
//!
//! The clipboard and the keystroke are the outside world, so nothing that
//! touches them is covered by the test suite. They are substituted at the
//! [`Desktop`] trait, which is what leaves everything built on top of them
//! testable — see the spec's *Testing Decisions*. What is tested here are the
//! two decisions that are not I/O: which Capture this session can perform, and
//! which key the copy chord is sent as.

use std::ffi::OsStr;
use std::time::Duration;

use enigo::{Direction, Enigo, Key, Keyboard, Settings};

use crate::capture::{Capture, CaptureError, ClipboardCapture, Desktop, DesktopCapture};

/// The environment variable Linux session managers use to say which display
/// server is running.
const SESSION_TYPE_ENV: &str = "XDG_SESSION_TYPE";

/// How long the modifiers of the Hotkey the user just pressed are given to come
/// back up before the copy chord is sent, so that the chord is not read as the
/// user's own keys plus ours.
const MODIFIER_SETTLE: Duration = Duration::from_millis(60);

/// The Capture this session can actually perform.
pub(crate) fn for_platform() -> Box<dyn Capture> {
    if accepts_synthetic_input(std::env::var_os(SESSION_TYPE_ENV).as_deref()) {
        Box::new(DesktopCapture::new(SystemDesktop))
    } else {
        Box::new(ClipboardCapture::new(SystemDesktop))
    }
}

/// Whether this session lets an ordinary application type into another one.
///
/// Wayland does not, by design, and reaching for the RemoteDesktop portal to
/// get around it was rejected in ADR-0003. Everything else is assumed to.
fn accepts_synthetic_input(session_type: Option<&OsStr>) -> bool {
    !session_type.is_some_and(|session_type| session_type.eq_ignore_ascii_case("wayland"))
}

/// The clipboard and keyboard of the machine Demysto is running on.
pub(crate) struct SystemDesktop;

impl Desktop for SystemDesktop {
    fn clipboard_text(&self) -> Result<Option<String>, CaptureError> {
        match clipboard()?.get_text() {
            Ok(text) => Ok(Some(text)),
            // An empty clipboard and one holding an image both come back this
            // way; for a text-only v1 they are the same thing.
            Err(arboard::Error::ContentNotAvailable) => Ok(None),
            Err(error) => Err(CaptureError::Clipboard(error.to_string())),
        }
    }

    fn set_clipboard_text(&self, text: Option<&str>) -> Result<(), CaptureError> {
        let mut clipboard = clipboard()?;

        match text {
            Some(text) => clipboard.set_text(text),
            None => clipboard.clear(),
        }
        .map_err(|error| CaptureError::Clipboard(error.to_string()))
    }

    fn send_copy(&self) -> Result<(), CaptureError> {
        let mut enigo =
            Enigo::new(&Settings::default()).map_err(|error| keystroke(&error.to_string()))?;

        // The Hotkey that got us here is itself a chord, and its modifiers may
        // still be down. Releasing them first keeps the copy from arriving as
        // something else entirely.
        for modifier in [Key::Shift, Key::Control, Key::Alt, Key::Meta] {
            let _ = enigo.key(modifier, Direction::Release);
        }
        std::thread::sleep(MODIFIER_SETTLE);

        let copy = if cfg!(target_os = "macos") {
            Key::Meta
        } else {
            Key::Control
        };

        enigo
            .key(copy, Direction::Press)
            .and_then(|()| enigo.key(copy_letter(), Direction::Click))
            .map_err(|error| keystroke(&error.to_string()))?;

        // Released outside the `?` above: leaving a modifier stuck down would
        // be a worse failure than the one that got us here.
        enigo
            .key(copy, Direction::Release)
            .map_err(|error| keystroke(&error.to_string()))
    }
}

/// The letter half of the copy chord, as the key the user's fingers are on.
///
/// Asking enigo for the character would make it consult the active keyboard
/// layout to find out where that character sits, and on macOS that lookup goes
/// through the Text Services Manager, which asserts it is being called on the
/// main thread and aborts the process when it is not — see the test below, and
/// `palette::off_thread` for why a Capture is never on that thread.
///
/// The keycode is the better answer besides. macOS matches a Command chord
/// against the physical key whatever layout is active, so `Cmd+C` copies from
/// the same key under a Cyrillic layout as under a Latin one — while the
/// character lookup, finding no `c` on that layout at all, would come back with
/// keycode zero and send `Cmd+A`.
#[cfg(target_os = "macos")]
fn copy_letter() -> Key {
    /// `kVK_ANSI_C`, from `Carbon/HIToolbox/Events.h`.
    const ANSI_C: u32 = 0x08;

    Key::Other(ANSI_C)
}

/// The letter half of the copy chord.
///
/// macOS is the platform whose character lookup insists on a thread this code
/// is never on, and the only one this has been watched failing on, so it is the
/// only one moved off the character. Whether X11 and Windows resolve a `c` that
/// a Cyrillic layout does not carry is ticket 13's to find out, on machines
/// that can be watched.
#[cfg(not(target_os = "macos"))]
fn copy_letter() -> Key {
    Key::Unicode('c')
}

fn clipboard() -> Result<arboard::Clipboard, CaptureError> {
    arboard::Clipboard::new().map_err(|error| CaptureError::Clipboard(error.to_string()))
}

/// On macOS a refused keystroke is nearly always the Accessibility permission
/// rather than anything about the key itself; ticket 12 owns saying so.
fn keystroke(message: &str) -> CaptureError {
    CaptureError::Keystroke(message.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn accepts(session_type: Option<&str>) -> bool {
        accepts_synthetic_input(session_type.map(OsStr::new))
    }

    /// The copy chord's letter must not be one enigo has to look up.
    ///
    /// `Key::Unicode` sends enigo to the Text Services Manager to ask where the
    /// character sits on the layout that is active, and that API asserts it is
    /// being called on the main thread. A Capture is never on it — it waits on
    /// another application, and the comment on `palette::off_thread` says why
    /// that wait cannot happen there — so the assertion aborts the process.
    #[test]
    #[cfg(target_os = "macos")]
    fn the_copy_letter_is_not_one_the_keyboard_layout_has_to_resolve() {
        assert!(!matches!(copy_letter(), Key::Unicode(_)));
    }

    #[test]
    fn x11_accepts_a_synthetic_copy() {
        assert!(accepts(Some("x11")));
    }

    #[test]
    fn wayland_does_not() {
        assert!(!accepts(Some("wayland")));
    }

    #[test]
    fn the_session_type_is_matched_regardless_of_case() {
        assert!(!accepts(Some("Wayland")));
    }

    #[test]
    fn a_platform_that_sets_no_session_type_is_not_wayland() {
        assert!(accepts(None));
    }
}
