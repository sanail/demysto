//! Obtaining a Selection from the foreground application or the clipboard.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;

use crate::i18n::{say, Words};
use crate::picture::Picture;
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
    /// The clipboard held a picture Demysto could not make anything of: one
    /// whose size and buffer disagree, or one nothing could encode.
    ///
    /// Held apart from [`Self::Clipboard`] because the clipboard answered
    /// perfectly well. Saying it was unavailable would be untrue, and would
    /// send somebody to look at the wrong thing.
    Picture,
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
            Self::Picture => say!(words, "capture-picture-unreadable"),
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

    /// The clipboard's picture, or `None` when it holds none.
    ///
    /// Raw pixels rather than encoded bytes, because that is what a clipboard
    /// hands over on every platform: encoding is Demysto's, and `picture` is
    /// where it happens.
    fn clipboard_picture(&self) -> Result<Option<Pixels>, CaptureError>;

    /// Puts a picture back on the clipboard, so that the guarantee v1 gave for
    /// text covers pictures too (user story 70).
    fn set_clipboard_picture(&self, picture: &Pixels) -> Result<(), CaptureError>;

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

/// A picture as a clipboard hands one over: raw RGBA8, with the size to read
/// it by.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Pixels {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) bytes: Vec<u8>,
}

/// What the clipboard held when a Capture began, so that it can be put back
/// afterwards and so that what arrives can be told apart from it.
struct Held {
    /// Whatever text was on it, verbatim: whitespace somebody copied is still
    /// what they copied, and it is what a restore writes back.
    text: Option<String>,
    /// The picture, read only where the text is nothing anybody meant to act
    /// on — see [`held`].
    picture: Option<Pixels>,
    /// What that picture is recognised by afterwards. Taken once, here, so that
    /// a Capture never walks the same megabytes twice.
    signature: Option<u64>,
}

/// What a picture is known by: its size, and a hash of its pixels.
///
/// A signature rather than the buffer itself, because the question asked of it
/// is only ever whether this is the picture that was there a moment ago — and a
/// Hotkey press must not walk megabytes to answer it.
fn signature(picture: &Pixels) -> u64 {
    let mut hasher = DefaultHasher::new();

    picture.width.hash(&mut hasher);
    picture.height.hash(&mut hasher);
    picture.bytes.hash(&mut hasher);

    hasher.finish()
}

/// What the clipboard holds, which is now text or a picture.
///
/// The picture is read only where the text is nothing anybody meant to act on,
/// which is where v1 returned [`Captured::Nothing`]. Text wins: the ambiguous
/// sources — a spreadsheet cell, a fragment of a page — almost always mean the
/// text and carry a picture as a secondary flavour, while the unambiguous ones
/// put no text on the clipboard at all, so the picture loses nothing by
/// yielding (user story 71). It also keeps megabytes out of the common Capture,
/// which is somebody pressing a Hotkey over a paragraph.
///
/// What it costs is one case: a clipboard holding both, whose picture is
/// therefore never read and so cannot be put back if the copy displaces it.
/// That is the price of not walking megabytes on every Hotkey press over a
/// paragraph, and it is paid where the picture was the secondary flavour of
/// something the user copied as text.
fn held<D: Desktop>(desktop: &D) -> Result<Held, CaptureError> {
    // A clipboard holding a picture and no text does not answer the same way on
    // every platform. macOS and Wayland say there is nothing to read; X11
    // answers with a failed conversion, because asking the owner for words
    // where it has none is a request it cannot satisfy — which is not a
    // clipboard that is broken, and reporting it as one made every picture on
    // X11 a Capture that failed. So the failure is held rather than returned,
    // and it is reported only where the picture turns out not to be there
    // either.
    let (text, refused) = match desktop.clipboard_text() {
        Ok(text) => (text, None),
        Err(error) => (None, Some(error)),
    };

    let picture = match meaningful(&text) {
        Some(_) => None,
        None => match desktop.clipboard_picture() {
            Ok(picture) => picture,
            // Neither half could be read. The one about words is the one worth
            // showing: it is the answer to what the user was almost certainly
            // asking for.
            Err(error) => return Err(refused.unwrap_or(error)),
        },
    };

    match (&picture, refused) {
        (None, Some(error)) => Err(error),
        _ => Ok(Held {
            signature: picture.as_ref().map(signature),
            text,
            picture,
        }),
    }
}

/// A picture as a Selection, where it is one Demysto can make anything of.
fn taken(picture: &Pixels) -> Result<Selection, CaptureError> {
    Picture::taken(picture.width, picture.height, &picture.bytes)
        .map(Selection::picture)
        .ok_or(CaptureError::Picture)
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

        let before = held(&self.desktop)?;

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
            let _ = self.restore(&before);
        }

        outcome
    }
}

impl<D: Desktop> DesktopCapture<D> {
    /// Waits for the copy to land and says what it brought, along with whether
    /// the clipboard was left holding something other than what it started on.
    fn settle_for_the_copy(&self, before: &Held) -> (Result<Captured, CaptureError>, bool) {
        let mut disturbed = false;
        let mut last = before.text.clone();
        let mut refused = None;

        for _ in 0..self.settle.attempts {
            std::thread::sleep(self.settle.interval);

            let after = match self.desktop.clipboard_text() {
                Ok(after) => after,
                // Held rather than returned, for the reason [`held`] holds one:
                // the copy that has just landed a picture is a clipboard that
                // answers a request for words with a failure, and that is not
                // a Capture that failed. What the clipboard holds is unknown
                // either way, which is reason enough to put back what the user
                // had.
                Err(error) => {
                    disturbed = true;
                    refused = Some(error);
                    None
                }
            };

            // Tracked across the whole window rather than returned on: an
            // application may leave the clipboard empty for a poll or two on
            // its way to writing the text, and that is not the end of it.
            disturbed |= after != before.text;
            last = after;

            if let Some(text) = changed(&before.text, &last) {
                return (Ok(Captured::Selection(Selection::text(text))), true);
            }
        }

        // No text arrived, which in v1 was the end of it. It is now where a
        // picture is looked for — and only here, because text wins wherever the
        // clipboard holds both.
        if let Some(text) = meaningful(&last) {
            return (Ok(Captured::Clipboard(Selection::text(text))), disturbed);
        }

        self.settle_for_a_picture(before, disturbed, refused)
    }

    /// What the clipboard holds now that the copy has brought no words: a
    /// picture the copy landed, a picture that was already there, or nothing
    /// this Capture can read.
    ///
    /// Read once, after the window rather than through it: a picture is
    /// megabytes, and polling for one would put that behind every Hotkey press
    /// over a paragraph.
    fn settle_for_a_picture(
        &self,
        before: &Held,
        disturbed: bool,
        refused: Option<CaptureError>,
    ) -> (Result<Captured, CaptureError>, bool) {
        let picture = match self.desktop.clipboard_picture() {
            Ok(picture) => picture,
            Err(error) => return (Err(refused.unwrap_or(error)), true),
        };

        let Some(picture) = picture else {
            // Neither half could be read, so the failure the words gave is
            // reported after all — this is where it turns out to have been a
            // clipboard nobody can read rather than one holding a picture.
            if let Some(error) = refused {
                return (Err(error), true);
            }

            // Nothing worth showing arrived within the window, so this is a
            // Selection Demysto cannot read: either nothing was selected, or
            // what the copy brought is neither text nor a picture. An
            // application slower than the whole window still lands its copy
            // afterwards, and that one Demysto cannot put back — the write
            // happens after the last look at it.
            return (Ok(fallback(before)), disturbed);
        };

        // A picture that was not there when the Capture began is one the copy
        // landed, which is a Selection rather than the clipboard's own
        // contents — and the clipboard now holds it in place of whatever the
        // user had.
        let landed = Some(signature(&picture)) != before.signature;

        let captured = match taken(&picture) {
            Ok(selection) => selection,
            Err(error) => return (Err(error), disturbed || landed),
        };

        let outcome = match landed {
            true => Captured::Selection(captured),
            false => Captured::Clipboard(captured),
        };

        (Ok(outcome), disturbed || landed)
    }

    /// Puts back what the clipboard held: the picture where it held one, and
    /// the text — verbatim, whitespace and all — where it did not.
    fn restore(&self, before: &Held) -> Result<(), CaptureError> {
        match &before.picture {
            Some(picture) => self.desktop.set_clipboard_picture(picture),
            None => self.desktop.set_clipboard_text(before.text.as_deref()),
        }
    }
}

/// What is left when nothing was selected: whatever the user put on the
/// clipboard themselves, or an explicit nothing.
fn fallback(clipboard: &Held) -> Captured {
    if let Some(text) = meaningful(&clipboard.text) {
        return Captured::Clipboard(Selection::text(text));
    }

    // A picture that could not be encoded is nothing this Capture can hand
    // over. It is reported as nothing rather than as a failure: this is the
    // path where the clipboard was never disturbed, and there is a sentence
    // for having found nothing to act on.
    match clipboard.picture.as_ref().map(taken) {
        Some(Ok(picture)) => Captured::Clipboard(picture),
        Some(Err(_)) | None => Captured::Nothing,
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
        Ok(fallback(&held(&self.desktop)?))
    }
}

#[cfg(test)]
pub(crate) mod fake {
    //! A desktop for the test suite: the outside world, substituted at the edge
    //! the spec's *Testing Decisions* names.

    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use super::{
        Capture, CaptureError, Capturing, ClipboardCapture, Desktop, DesktopCapture, Pixels, Settle,
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
        /// The picture on the clipboard, which a real one holds alongside the
        /// text rather than instead of it.
        picture: Mutex<Option<Pixels>>,
        selection: Option<String>,
        /// The picture the copy lands, for the desktop where what the user is
        /// looking at is one.
        selected_picture: Option<Pixels>,
        lands_after: u32,
        reads_since_copy: Mutex<Option<u32>>,
        refuses_writes: bool,
        /// A clipboard that answers a request for words with a failure, which
        /// is what X11 does when its owner holds a picture and nothing else.
        refuses_text: bool,
        refuses_permission: bool,
        /// How many times the picture has been read, so that a test can see a
        /// Capture over text does not walk megabytes to find that out.
        picture_reads: Mutex<u32>,
        /// How many times the permission has been asked about, so that a test
        /// can see it is asked at every Capture rather than once.
        permission_checks: Mutex<u32>,
    }

    /// A picture of that size, filled with one colour — enough to be a picture,
    /// and different from another of a different size or shade.
    pub(crate) fn pixels(width: u32, height: u32, shade: u8) -> Pixels {
        Pixels {
            width,
            height,
            bytes: vec![shade; (width as usize) * (height as usize) * 4],
        }
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

        /// A clipboard already holding this picture, whatever text is on it
        /// beside it.
        pub(crate) fn holding_picture(mut self, picture: Pixels) -> Self {
            self.picture = Mutex::new(Some(picture));
            self
        }

        /// A foreground application whose Selection is a picture, so that the
        /// copy lands one.
        ///
        /// Landed at the keystroke rather than after a poll or two, and landed
        /// over the text: a pasteboard written with a picture is a pasteboard
        /// written, and what was on it is gone. The waiting is text's to
        /// exercise — see [`FakeDesktop::landing_after`].
        pub(crate) fn selecting_picture(mut self, picture: Pixels) -> Self {
            self.selected_picture = Some(picture);
            self
        }

        pub(crate) fn picture_reads(&self) -> u32 {
            *self.picture_reads.lock().unwrap()
        }

        pub(crate) fn picture_now(&self) -> Option<Pixels> {
            self.picture.lock().unwrap().clone()
        }

        /// A clipboard that can be read but not written, which is what an X11
        /// session looks like when its owner changes under Demysto.
        pub(crate) fn refusing_to_restore(mut self) -> Self {
            self.refuses_writes = true;
            self
        }

        /// A clipboard that refuses to answer a request for words at all.
        ///
        /// Which is not a broken clipboard: on X11 it is what a clipboard
        /// holding a picture and nothing else answers, because the owner
        /// cannot convert what it has into what was asked for.
        pub(crate) fn refusing_text(mut self) -> Self {
            self.refuses_text = true;
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
            if self.refuses_text {
                return Err(CaptureError::Clipboard(
                    "incorrect type received from clipboard".to_owned(),
                ));
            }

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

        fn clipboard_picture(&self) -> Result<Option<Pixels>, CaptureError> {
            *self.picture_reads.lock().unwrap() += 1;

            Ok(self.picture.lock().unwrap().clone())
        }

        fn set_clipboard_picture(&self, picture: &Pixels) -> Result<(), CaptureError> {
            if self.refuses_writes {
                return Err(CaptureError::Clipboard("no owner".to_owned()));
            }

            *self.picture.lock().unwrap() = Some(picture.clone());
            Ok(())
        }

        fn send_copy(&self) -> Result<(), CaptureError> {
            *self.reads_since_copy.lock().unwrap() = Some(0);

            if let Some(picture) = &self.selected_picture {
                *self.picture.lock().unwrap() = Some(picture.clone());
                *self.clipboard.lock().unwrap() = None;
            }

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

        fn clipboard_picture(&self) -> Result<Option<Pixels>, CaptureError> {
            <FakeDesktop as Desktop>::clipboard_picture(self)
        }

        fn set_clipboard_picture(&self, picture: &Pixels) -> Result<(), CaptureError> {
            <FakeDesktop as Desktop>::set_clipboard_picture(self, picture)
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
