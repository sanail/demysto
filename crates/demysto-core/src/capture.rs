//! Obtaining a Selection from the foreground application or the clipboard.

use std::time::Duration;

use crate::i18n::{say, Words};
use crate::selection::Selection;

/// The act of obtaining a Selection, behind a trait so that the core can be
/// exercised without a desktop attached.
pub trait Capture: Send + Sync {
    fn capture(&self) -> Result<Captured, CaptureError>;
}

/// What a Capture on this desktop is able to read.
///
/// Answered once, from the session Demysto started in, and shown rather than
/// inferred: everywhere but Wayland a Capture reads what the user selected, and
/// on Wayland it can only read what they copied themselves. A user who is not
/// told that reasonably concludes the Hotkey is broken (user story 56).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "reads", content = "detail", rename_all = "snake_case")]
pub enum Capturing {
    /// The desktop accepts a synthetic copy, so a Capture reads the Selection
    /// out of whatever the user is looking at.
    Selection,
    /// The desktop refuses synthetic input, so a Capture reads only what the
    /// user put on the clipboard themselves.
    ///
    /// The fact and not the sentence about it. The sentence lives in the
    /// catalogue, where every other sentence lives, and is said by whichever
    /// window is on screen when it needs saying — `capture-clipboard-only`.
    ClipboardOnly,
}

/// What one Capture produced.
///
/// The origin is part of the result rather than an inference the Palette makes:
/// falling back to the clipboard is a different thing from reading a Selection,
/// and the user is told which one happened.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "origin", content = "selection", rename_all = "snake_case")]
pub enum Captured {
    /// Text that was selected in the foreground application.
    Selection(Selection),
    /// Nothing was selected, so this is what the clipboard already held.
    Clipboard(Selection),
    /// Nothing was selected and the clipboard was empty.
    Nothing,
}

/// What a Capture produced, failure included.
///
/// A failure is a state the Palette shows rather than an error that stops it:
/// the window still opens and says what went wrong. The window gives it the
/// retry and the route into Settings.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "status", content = "detail", rename_all = "snake_case")]
pub enum CaptureOutcome {
    Captured(Captured),
    Failed(CaptureError),
}

impl CaptureOutcome {
    /// The Selection this Capture produced, or `None` when it produced none:
    /// there was nothing to read, or reading it failed.
    pub fn selection(&self) -> Option<&Selection> {
        match self {
            Self::Captured(captured) => captured.selection(),
            Self::Failed(_) => None,
        }
    }
}

impl Captured {
    /// The Selection, wherever it came from. Where it came from is what the
    /// Palette shows; what a Run operates on is the same either way.
    pub fn selection(&self) -> Option<&Selection> {
        match self {
            Self::Selection(selection) | Self::Clipboard(selection) => Some(selection),
            Self::Nothing => None,
        }
    }
}

impl From<Result<Captured, CaptureError>> for CaptureOutcome {
    fn from(result: Result<Captured, CaptureError>) -> Self {
        match result {
            Ok(captured) => Self::Captured(captured),
            Err(error) => Self::Failed(error),
        }
    }
}

/// Why a Capture produced nothing.
///
/// Neither `Display` nor `Error`, deliberately: what the user is told depends
/// on the language the interface is speaking, and neither trait has anywhere to
/// be told it — [`Self::message`] takes the words instead. Nothing treats this
/// as an error in the `?` sense either; it is a state the Palette shows, as
/// [`CaptureOutcome`] says.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", content = "message", rename_all = "snake_case")]
pub enum CaptureError {
    /// The clipboard could not be read or written.
    Clipboard(String),
    /// The copy keystroke could not be delivered to the foreground application.
    Keystroke(String),
    /// The operating system is withholding the permission a Capture needs.
    ///
    /// Held apart from [`Self::Keystroke`] because what the user is owed
    /// differs: a keystroke that failed is worth trying again, and this is not
    /// worth trying again until they have granted something. Only macOS has
    /// such a permission, so the sentence that names it — and the pane it is
    /// granted in — is one message in the catalogue rather than a string this
    /// carries.
    Permission,
}

impl CaptureError {
    /// What the user is told this was, in their own language.
    ///
    /// Not a `Display`: the sentence depends on which language the interface is
    /// speaking, and a `Display` has nowhere to be told. What the platform said
    /// travels inside the error and is quoted into the sentence here.
    pub(crate) fn message(&self, words: &Words) -> String {
        match self {
            Self::Clipboard(detail) => {
                say!(
                    words,
                    "capture-clipboard-unavailable",
                    "detail" = detail.clone()
                )
            }
            Self::Keystroke(detail) => {
                say!(
                    words,
                    "capture-keystroke-refused",
                    "detail" = detail.clone()
                )
            }
            Self::Permission => say!(words, "capture-no-accessibility"),
        }
    }
}

/// The parts of the desktop a Capture touches, so that the surrounding
/// behaviour — the fallback, the restoration — is testable without one.
pub(crate) trait Desktop: Send + Sync {
    /// The clipboard's text, or `None` when it holds nothing this can read.
    fn clipboard_text(&self) -> Result<Option<String>, CaptureError>;

    /// Replaces the clipboard's text, or empties it when given `None`.
    fn set_clipboard_text(&self, text: Option<&str>) -> Result<(), CaptureError>;

    /// Sends the platform's copy keystroke to the foreground application.
    fn send_copy(&self) -> Result<(), CaptureError>;

    /// Whether this desktop is letting Demysto type into another application
    /// at all.
    ///
    /// Asked at every Capture rather than answered once at startup: macOS gates
    /// synthetic input behind the Accessibility permission and withdraws it
    /// whenever the binary's signature changes, so an answer held from startup
    /// is an answer that goes stale under a running Demysto (the spec's *Shell
    /// and platform*).
    fn permitted(&self) -> Result<(), CaptureError>;
}

/// How long a Capture waits for the copied text to reach the clipboard.
///
/// The copy is delivered to another process, so the clipboard changes some time
/// after the keystroke rather than because of it. Polling briefly beats one
/// fixed sleep: a fast application is not made to wait for a slow one's budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Settle {
    pub(crate) interval: Duration,
    pub(crate) attempts: u32,
}

impl Default for Settle {
    fn default() -> Self {
        Self {
            interval: Duration::from_millis(20),
            attempts: 15,
        }
    }
}

/// Capture as it works everywhere the desktop accepts synthetic input: send a
/// copy, read what arrived, and put back what was there before.
pub(crate) struct DesktopCapture<D> {
    desktop: D,
    settle: Settle,
}

impl<D: Desktop> DesktopCapture<D> {
    pub(crate) fn new(desktop: D) -> Self {
        Self {
            desktop,
            settle: Settle::default(),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_settle(desktop: D, settle: Settle) -> Self {
        Self { desktop, settle }
    }
}

impl<D: Desktop> Capture for DesktopCapture<D> {
    fn capture(&self) -> Result<Captured, CaptureError> {
        // Asked before the clipboard is touched, because without the permission
        // the copy never reaches the application the user is reading and what
        // follows is the fallback: the clipboard reported as though nothing had
        // been selected. Which is the wrong sentence for a Selection that is
        // sitting right there — and one no amount of pressing the Hotkey again
        // will improve.
        self.desktop.permitted()?;

        let before = self.desktop.clipboard_text()?;

        self.desktop.send_copy()?;

        let (outcome, disturbed) = self.settle_for_the_copy(&before);

        // Whether the copy brought a Selection is a separate question from
        // whether it overwrote the clipboard: one that lands an image, or a
        // blank line, overwrites it just as thoroughly and gives Demysto
        // nothing to show for it. The clipboard goes back on every path that
        // disturbed it, this one included — and before the outcome is reported,
        // so that a failed Capture still leaves the user what they had.
        //
        // Restored verbatim rather than from the meaningful reading of it:
        // whitespace the user copied is still what they copied. A restore that
        // fails is not allowed to take a Selection down with it — the text has
        // been read, and losing it as well would only make things worse.
        if disturbed {
            let _ = self.desktop.set_clipboard_text(before.as_deref());
        }

        outcome
    }
}

impl<D: Desktop> DesktopCapture<D> {
    /// Waits for the copy to land and says what it brought, along with whether
    /// the clipboard was left holding something other than what it started on.
    fn settle_for_the_copy(
        &self,
        before: &Option<String>,
    ) -> (Result<Captured, CaptureError>, bool) {
        let mut disturbed = false;

        for _ in 0..self.settle.attempts {
            std::thread::sleep(self.settle.interval);

            let after = match self.desktop.clipboard_text() {
                Ok(after) => after,
                // What the clipboard holds is now unknown, and unknown is
                // reason enough to put back what the user had.
                Err(error) => return (Err(error), true),
            };

            // Tracked across the whole window rather than returned on: an
            // application may leave the clipboard empty for a poll or two on
            // its way to writing the text, and that is not the end of it.
            disturbed |= &after != before;

            if let Some(text) = changed(before, &after) {
                return (Ok(Captured::Selection(Selection::text(text))), true);
            }
        }

        // Nothing worth showing arrived within the window, so this is a
        // Selection Demysto cannot read: either nothing was selected, or what
        // the copy brought is not text. An application slower than the whole
        // window still lands its copy afterwards, and that one Demysto cannot
        // put back — the write happens after the last look at it.
        (Ok(fallback(before)), disturbed)
    }
}

/// What is left when nothing was selected: whatever the user put on the
/// clipboard themselves, or an explicit nothing.
fn fallback(clipboard: &Option<String>) -> Captured {
    match meaningful(clipboard) {
        Some(text) => Captured::Clipboard(Selection::text(text)),
        None => Captured::Nothing,
    }
}

/// The newly copied text, when the copy landed and brought something with it.
fn changed<'a>(before: &Option<String>, after: &'a Option<String>) -> Option<&'a str> {
    let after = meaningful(after)?;
    (Some(after) != meaningful(before)).then_some(after)
}

/// Text that is only whitespace is nothing anybody meant to act on.
fn meaningful(text: &Option<String>) -> Option<&str> {
    text.as_deref().filter(|text| !text.trim().is_empty())
}

/// Capture where synthetic input is unavailable: read whatever the user copied
/// themselves and say so. On Wayland this is the whole of it — see ADR-0003.
pub(crate) struct ClipboardCapture<D> {
    desktop: D,
}

impl<D: Desktop> ClipboardCapture<D> {
    pub(crate) fn new(desktop: D) -> Self {
        Self { desktop }
    }
}

impl<D: Desktop> Capture for ClipboardCapture<D> {
    /// Nothing is asked of the desktop first: this Capture types into nothing,
    /// so there is no permission it could be missing. Reading the clipboard is
    /// something every session allows.
    fn capture(&self) -> Result<Captured, CaptureError> {
        Ok(fallback(&self.desktop.clipboard_text()?))
    }
}

#[cfg(test)]
pub(crate) mod fake {
    //! A desktop for the test suite: the outside world, substituted at the edge
    //! the spec's *Testing Decisions* names.

    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use super::{
        Capture, CaptureError, Capturing, ClipboardCapture, Desktop, DesktopCapture, Settle,
    };

    /// A desktop as the facade is handed one: the Capture it performs, and what
    /// that Capture can read.
    ///
    /// The two together because the facade takes them together — see
    /// `Demysto::with_capture` for why neither is any use without the other.
    pub(crate) type Reading = (Box<dyn Capture>, Capturing);

    /// A desktop whose foreground application holds `selection`, and which puts
    /// it on the clipboard `lands_after` reads after being sent a copy.
    #[derive(Default)]
    pub(crate) struct FakeDesktop {
        clipboard: Mutex<Option<String>>,
        selection: Option<String>,
        lands_after: u32,
        reads_since_copy: Mutex<Option<u32>>,
        refuses_writes: bool,
        refuses_permission: bool,
        /// How many times the permission has been asked about, so that a test
        /// can see it is asked at every Capture rather than once.
        permission_checks: Mutex<u32>,
    }

    impl FakeDesktop {
        pub(crate) fn new(clipboard: Option<&str>, selection: Option<&str>) -> Self {
            Self {
                clipboard: Mutex::new(clipboard.map(str::to_owned)),
                selection: selection.map(str::to_owned),
                ..Self::default()
            }
        }

        pub(crate) fn landing_after(mut self, reads: u32) -> Self {
            self.lands_after = reads;
            self
        }

        /// A clipboard that can be read but not written, which is what an X11
        /// session looks like when its owner changes under Demysto.
        pub(crate) fn refusing_to_restore(mut self) -> Self {
            self.refuses_writes = true;
            self
        }

        /// A desktop withholding the permission synthetic input needs, which is
        /// what macOS looks like with Accessibility turned off.
        pub(crate) fn refusing_permission(mut self) -> Self {
            self.refuses_permission = true;
            self
        }

        pub(crate) fn permission_checks(&self) -> u32 {
            *self.permission_checks.lock().unwrap()
        }

        pub(crate) fn clipboard_now(&self) -> Option<String> {
            self.clipboard.lock().unwrap().clone()
        }
    }

    impl Desktop for FakeDesktop {
        fn clipboard_text(&self) -> Result<Option<String>, CaptureError> {
            let mut reads = self.reads_since_copy.lock().unwrap();
            if let Some(count) = reads.as_mut() {
                if *count >= self.lands_after {
                    if let Some(selection) = &self.selection {
                        *self.clipboard.lock().unwrap() = Some(selection.clone());
                    }
                }
                *count += 1;
            }
            Ok(self.clipboard.lock().unwrap().clone())
        }

        fn set_clipboard_text(&self, text: Option<&str>) -> Result<(), CaptureError> {
            if self.refuses_writes {
                return Err(CaptureError::Clipboard("no owner".to_owned()));
            }

            *self.clipboard.lock().unwrap() = text.map(str::to_owned);
            Ok(())
        }

        fn send_copy(&self) -> Result<(), CaptureError> {
            *self.reads_since_copy.lock().unwrap() = Some(0);
            Ok(())
        }

        fn permitted(&self) -> Result<(), CaptureError> {
            *self.permission_checks.lock().unwrap() += 1;

            match self.refuses_permission {
                true => Err(CaptureError::Permission),
                false => Ok(()),
            }
        }
    }

    /// Shared so that a test can look at the clipboard the Capture it was given
    /// has been working on.
    impl Desktop for Arc<FakeDesktop> {
        fn clipboard_text(&self) -> Result<Option<String>, CaptureError> {
            <FakeDesktop as Desktop>::clipboard_text(self)
        }

        fn set_clipboard_text(&self, text: Option<&str>) -> Result<(), CaptureError> {
            <FakeDesktop as Desktop>::set_clipboard_text(self, text)
        }

        fn send_copy(&self) -> Result<(), CaptureError> {
            <FakeDesktop as Desktop>::send_copy(self)
        }

        fn permitted(&self) -> Result<(), CaptureError> {
            <FakeDesktop as Desktop>::permitted(self)
        }
    }

    /// The Capture every desktop that accepts synthetic input uses, with the
    /// waiting taken out of it.
    pub(crate) fn over(desktop: &Arc<FakeDesktop>) -> Reading {
        let capture = DesktopCapture::with_settle(
            Arc::clone(desktop),
            Settle {
                interval: Duration::ZERO,
                attempts: 5,
            },
        );

        (Box::new(capture), Capturing::Selection)
    }

    /// The Capture a Wayland session gets instead, and the sentence that goes
    /// with it.
    pub(crate) fn clipboard_only_over(desktop: &Arc<FakeDesktop>) -> Reading {
        (
            Box::new(ClipboardCapture::new(Arc::clone(desktop))),
            Capturing::ClipboardOnly,
        )
    }
}
