//! What a Run operates on.

use crate::picture::Picture;

/// The input a Run operates on, captured at invocation time.
///
/// The kind names what a Model is handed rather than where Demysto found it
/// (`CONTEXT.md`): a picture read from the clipboard and a picture opened from
/// a path reach a Model the same way, so both are image Selections. Where it
/// came from is the Capture's business, and is [`crate::Captured`]'s to say.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Selection {
    Text {
        text: String,
    },
    /// A picture, held both fitted and original — ADR-0017. What crosses to a
    /// window is how large it is; the picture itself is asked for.
    Image {
        picture: Picture,
    },
}

/// Which of those an Action will accept.
///
/// Separate from [`Selection`] itself because an Action declares what it accepts
/// long before there is anything to run it on. The Palette is never handed one —
/// it is given the Actions that already accept what was captured — but an
/// Action file states which kinds its Action takes, so the window that writes
/// that file carries them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Text,
    Image,
}

impl Selection {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }

    pub(crate) fn picture(picture: Picture) -> Self {
        Self::Image { picture }
    }

    /// What a Run operates on, as the text a prompt is assembled around.
    ///
    /// Empty for a picture, which is not a refusal but the whole of the rule:
    /// an Action declaring `accepts = ["text", "image"]` is legitimate, and its
    /// template has to name `{{selection}}` for the text half. The picture is
    /// never substituted into the text of a prompt — it travels as a content
    /// part of its own, and nothing in prompt assembly may learn otherwise.
    pub fn as_text(&self) -> &str {
        match self {
            Self::Text { text } => text,
            Self::Image { .. } => "",
        }
    }

    /// The picture this Selection is about, `None` for one made of words.
    pub(crate) fn as_picture(&self) -> Option<&Picture> {
        match self {
            Self::Image { picture } => Some(picture),
            Self::Text { .. } => None,
        }
    }

    /// The picture as a window shows one, `None` for a Selection made of
    /// words. The one place that walk is written down, because two callers ask
    /// it: the Conversation on screen, and the Capture the Palette is showing.
    pub(crate) fn picture_url(&self) -> Option<String> {
        self.as_picture().map(|picture| picture.fitted().url())
    }

    /// How large the picture is, for the windows that quote a Selection and
    /// have no words to quote — the list of Conversations, and the line above
    /// the first Turn. `None` where there is text to show instead.
    pub(crate) fn dimensions(&self) -> Option<String> {
        self.as_picture().map(Picture::dimensions)
    }

    /// What this Selection is, so that the Palette can leave out the Actions
    /// that cannot run on it.
    pub(crate) fn kind(&self) -> Kind {
        match self {
            Self::Text { .. } => Kind::Text,
            Self::Image { .. } => Kind::Image,
        }
    }
}
