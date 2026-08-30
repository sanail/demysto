//! The real desktop: the clipboard, and a copy keystroke aimed at whatever the
//! user is looking at.
//!
//! The clipboard and the keystroke are the outside world, so nothing that
//! touches them is covered by the test suite. They are substituted at the
//! [`Desktop`] trait, which is what leaves everything built on top of them
//! testable — see the spec's *Testing Decisions*. What is tested here are the
//! two decisions that are not I/O: which Capture this session can perform, and
//! which key the copy chord is sent as. Whether macOS has granted the
//! Accessibility permission is asked of macOS, so the suite exercises it
//! through the fake desktop instead.

use std::ffi::OsStr;
use std::sync::Mutex;
use std::time::Duration;

use enigo::{Direction, Enigo, Key, Keyboard, Settings};

use crate::capture::{Capture, CaptureError, Capturing, ClipboardCapture, Desktop, DesktopCapture};

/// The environment variable Linux session managers use to say which display
/// server is running.
///
/// The one thing outside `config` that reads the environment, and read for the
/// reason a key is not: this says which display server Demysto is talking to,
/// and that cannot change without the session it belongs to ending.
const SESSION_TYPE_ENV: &str = "XDG_SESSION_TYPE";

/// What the user is told on a session that will not let Demysto read a
/// Selection for them (user story 56).
///
/// Says what to do instead as well as what is wrong: a limitation stated
/// without the way round it is indistinguishable, to somebody who has just
/// pressed the Hotkey, from the tool being broken.
const WAYLAND_CLIPBOARD_ONLY: &str = "This is a Wayland session, and Wayland does not let one \
     application type into another. Demysto cannot read what you have selected: copy it yourself \
     with Ctrl+C first, then press the Hotkey, and Demysto reads the clipboard.";

/// How long the modifiers of the Hotkey the user just pressed are given to come
/// back up before the copy chord is sent, so that the chord is not read as the
/// user's own keys plus ours.
const MODIFIER_SETTLE: Duration = Duration::from_millis(60);

/// The Capture this session can actually perform, and what it can read.
///
/// The two together because whoever holds the first has to be able to say the
/// second: the interface tells the user what it can read, and being told the
/// wrong thing is worse than not being told.
pub(crate) fn for_platform() -> (Box<dyn Capture>, Capturing) {
    match reading(std::env::var_os(SESSION_TYPE_ENV).as_deref()) {
        Capturing::Selection => (
            Box::new(DesktopCapture::new(SystemDesktop::new())),
            Capturing::Selection,
        ),
        degraded => (
            Box::new(ClipboardCapture::new(SystemDesktop::new())),
            degraded,
        ),
    }
}

/// Whether this is a Wayland session.
///
/// Wayland lets an ordinary application neither type into another one nor claim
/// a Hotkey for itself: the first is ADR-0003's, and the second is why the
/// Hotkey there is asked of the GlobalShortcuts portal rather than of the
/// display server. One question because it is one fact, asked by the two parts
/// of the shell it decides.
pub(crate) fn wayland() -> bool {
    refuses_synthetic_input(std::env::var_os(SESSION_TYPE_ENV).as_deref())
}

/// What a Capture can read on a session of this type.
fn reading(session_type: Option<&OsStr>) -> Capturing {
    match refuses_synthetic_input(session_type) {
        true => Capturing::ClipboardOnly(WAYLAND_CLIPBOARD_ONLY.to_owned()),
        false => Capturing::Selection,
    }
}

/// Whether this session refuses to let one application type into another.
///
/// Wayland does, by design, and reaching for the RemoteDesktop portal to get
/// around it was rejected in ADR-0003. Every other session is assumed not to —
/// including one that names no type at all, which is every macOS and Windows
/// machine there is.
fn refuses_synthetic_input(session_type: Option<&OsStr>) -> bool {
    session_type.is_some_and(|kind| kind.eq_ignore_ascii_case("wayland"))
}

/// The clipboard and keyboard of the machine Demysto is running on.
///
/// The clipboard connection is held for as long as Demysto runs rather than
/// opened per call, and that is not a saving — it is the only way the writes
/// survive.
///
/// On X11 the clipboard holds no content of its own: it records which window
/// owns the selection, and that window hands the text over when somebody asks
/// for it. An owner that has gone leaves nothing behind. Opening a connection,
/// writing, and closing it therefore puts text on the clipboard for as long as
/// it takes to close — which is to say not at all. Watched doing exactly that:
/// a Capture that restored the user's clipboard left it empty instead, so
/// every Capture over a Selection destroyed whatever the user had copied.
///
/// Wayland and macOS keep the content themselves and would not have minded
/// either way; one connection for all three is simpler than one rule per
/// platform.
pub(crate) struct SystemDesktop {
    clipboard: Mutex<Option<arboard::Clipboard>>,
}

impl SystemDesktop {
    pub(crate) fn new() -> Self {
        Self {
            clipboard: Mutex::new(None),
        }
    }

    /// Runs something against the held clipboard connection, opening one on
    /// first use.
    ///
    /// A connection that has failed is dropped rather than kept: an X server
    /// that went away takes it with it, and the next Capture deserves a fresh
    /// one instead of the same error for the rest of the session.
    fn on_clipboard<T>(
        &self,
        act: impl FnOnce(&mut arboard::Clipboard) -> Result<T, arboard::Error>,
    ) -> Result<T, arboard::Error> {
        let mut held = self
            .clipboard
            .lock()
            .unwrap_or_else(|held| held.into_inner());

        if held.is_none() {
            *held = Some(arboard::Clipboard::new()?);
        }

        let outcome = act(held.as_mut().expect("a clipboard was just opened"));

        if matches!(
            outcome,
            Err(arboard::Error::ClipboardOccupied) | Err(arboard::Error::Unknown { .. })
        ) {
            *held = None;
        }

        outcome
    }
}

impl Desktop for SystemDesktop {
    fn clipboard_text(&self) -> Result<Option<String>, CaptureError> {
        match self.on_clipboard(arboard::Clipboard::get_text) {
            Ok(text) => Ok(Some(text)),
            // An empty clipboard and one holding an image both come back this
            // way; for a text-only v1 they are the same thing.
            Err(arboard::Error::ContentNotAvailable) => Ok(None),
            Err(error) => Err(CaptureError::Clipboard(error.to_string())),
        }
    }

    fn set_clipboard_text(&self, text: Option<&str>) -> Result<(), CaptureError> {
        self.on_clipboard(|clipboard| match text {
            Some(text) => clipboard.set_text(text),
            None => clipboard.clear(),
        })
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
            .and_then(|()| enigo.raw(COPY_LETTER, Direction::Click))
            .map_err(|error| keystroke(&error.to_string()))?;

        // Released outside the `?` above: leaving a modifier stuck down would
        // be a worse failure than the one that got us here.
        enigo
            .key(copy, Direction::Release)
            .map_err(|error| keystroke(&error.to_string()))
    }

    fn permitted(&self) -> Result<(), CaptureError> {
        accessibility()
    }
}

/// What the user is told when macOS is withholding the Accessibility
/// permission.
///
/// Names the pane rather than only the permission, so that the sentence and the
/// button the interface puts beside it say the same thing — and so that the
/// sentence is still followable for somebody who reads it in a notification,
/// where there is no button.
#[cfg(target_os = "macos")]
const NO_ACCESSIBILITY: &str = "macOS is not letting Demysto read what you selected: Demysto \
     needs the Accessibility permission. Open Privacy & Security → Accessibility and turn \
     Demysto on. macOS withdraws it whenever the application changes, so this can come back \
     after an update.";

/// Whether macOS is letting Demysto type into another application.
///
/// `AXIsProcessTrusted` rather than the variant that offers to ask for the
/// permission: this is called at every Capture, and a system dialog on every
/// Hotkey press would be its own kind of broken. Walking the user to the
/// permission is the first-run flow's, and is ticket 15's.
#[cfg(target_os = "macos")]
fn accessibility() -> Result<(), CaptureError> {
    // Declared here rather than taken from a crate: one function with no
    // arguments is the whole of Demysto's interest in the Accessibility API.
    // It answers a `Boolean`, which is a byte and not a Rust `bool`.
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrusted() -> std::ffi::c_uchar;
    }

    // SAFETY: no arguments, no pointers, and documented as callable from any
    // thread — which matters, because a Capture is never on the main one.
    let trusted = unsafe { AXIsProcessTrusted() } != 0;

    match trusted {
        true => Ok(()),
        false => Err(CaptureError::Permission(NO_ACCESSIBILITY.to_owned())),
    }
}

/// Nothing gates synthetic input on X11 or on Windows: what is typed there is
/// typed. Wayland refuses it outright, and that is not a permission anybody can
/// grant — it is a Capture Demysto does not attempt, and `for_platform` is
/// where that is decided (ADR-0003).
#[cfg(not(target_os = "macos"))]
fn accessibility() -> Result<(), CaptureError> {
    Ok(())
}

// The letter half of the copy chord is sent as the physical key the user's own
// fingers are on, not as the character printed on it — through enigo's `raw`,
// which takes the platform's own number for a key and asks nothing of the
// keyboard layout. Every platform numbers keys differently, so the constant is
// per platform and the way it travels is not.
//
// Asking for the character instead would send enigo to the active layout to
// find out where a `c` sits, and a layout carrying no Latin `c` has no answer:
//
//   - On macOS the lookup goes through the Text Services Manager, which asserts
//     it is being called on the main thread and aborts the process when it is
//     not — and a Capture is never on that thread; see `palette::off_thread`.
//     Finding no `c`, it would come back with keycode zero and send `Cmd+A`.
//   - On Windows the lookup is `VkKeyScanEx`, which answers -1 for a character
//     the layout does not carry; enigo then falls back to entering the
//     character as text, and a text event under a held Ctrl is not a copy.
//   - On X11 enigo does have an answer — it binds the keysym to a spare keycode
//     — so the character would have worked there. It is sent as the physical
//     key anyway, so that all three platforms send what pressing the key sends.
//
// Which is what the chord is matched against besides: `Cmd+C` and `Ctrl+C` copy
// from the same key under a Cyrillic layout as under a Latin one, because that
// is where the user's finger goes.

/// `kVK_ANSI_C`, from `Carbon/HIToolbox/Events.h`.
#[cfg(target_os = "macos")]
const COPY_LETTER: u16 = 0x08;

/// The scan code of the `C` key in set 1, which is what `SendInput` takes and
/// what Windows translates back through whatever layout is active.
#[cfg(target_os = "windows")]
const COPY_LETTER: u16 = 0x2E;

/// The X11 keycode of the `C` key: the Linux input event code for it, 46, plus
/// the 8 X11 offsets every evdev keycode by.
///
/// Only X11 reaches this. A Wayland session never sends a copy chord at all —
/// [`for_platform`] gives it the Capture that does not try (ADR-0003).
///
/// This is enigo's `x11rb` backend, which is the one its default features
/// select. Its `xdo` backend has never implemented sending a keycode at all,
/// and turning that feature on here would panic every Capture on X11.
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const COPY_LETTER: u16 = 54;

/// A keystroke enigo itself refused, which is a different thing from one macOS
/// would not have delivered — that is asked about first, in [`accessibility`],
/// and reported as the permission it is.
fn keystroke(message: &str) -> CaptureError {
    CaptureError::Keystroke(message.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn on(session_type: Option<&str>) -> Capturing {
        reading(session_type.map(OsStr::new))
    }

    /// What a session that reads only the clipboard says about itself, or the
    /// test's own failure where it reads a Selection after all.
    fn said_on(session_type: Option<&str>) -> String {
        match on(session_type) {
            Capturing::ClipboardOnly(said) => said,
            Capturing::Selection => {
                panic!("{session_type:?} should be a session that reads only the clipboard")
            }
        }
    }

    #[test]
    fn x11_reads_the_selection() {
        assert_eq!(on(Some("x11")), Capturing::Selection);
    }

    #[test]
    fn wayland_reads_only_the_clipboard() {
        assert!(matches!(on(Some("wayland")), Capturing::ClipboardOnly(_)));
    }

    #[test]
    fn the_session_type_is_matched_regardless_of_case() {
        assert!(matches!(on(Some("Wayland")), Capturing::ClipboardOnly(_)));
    }

    #[test]
    fn a_platform_that_sets_no_session_type_reads_the_selection() {
        assert_eq!(on(None), Capturing::Selection);
    }

    /// The sentence is the whole of what a Wayland user is given (user story
    /// 56), so it has to name the limitation and the way round it. A sentence
    /// that says only that something is unavailable reads as a broken tool.
    #[test]
    fn a_clipboard_only_session_says_what_to_do_instead() {
        let said = said_on(Some("wayland"));

        assert!(said.contains("Wayland"), "{said}");
        assert!(said.contains("clipboard"), "{said}");
        assert!(said.contains("Ctrl+C"), "{said}");
    }
}
