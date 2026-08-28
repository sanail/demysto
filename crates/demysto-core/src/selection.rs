//! What a Run operates on.

/// The input a Run operates on, captured at invocation time.
///
/// v1 captures text only; the kind is modelled as an enum because the Action
/// catalogue declares the kinds it accepts, and images and files are already
/// known to be coming (see the spec's *Out of Scope*).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Selection {
    Text { text: String },
}

/// Which of those an Action will accept.
///
/// Separate from [`Selection`] itself because an Action declares what it accepts
/// long before there is anything to run it on. The Palette is never handed one —
/// it is given the Actions that already accept what was captured — but an
/// Action file states which kinds its Action takes, so the window that writes
/// that file carries them.
///
/// `Image` has no [`Selection`] to go with it in v1 — capturing one is v1.1's,
/// per the spec's *Out of Scope*. It is here because Model resolution turns on
/// it: the Default Vision Model exists to keep the cheap everyday Model from
/// being asked to look at pictures, and that rule is worth having settled and
/// tested before there is a picture to test it with.
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

    /// What a Run operates on, as the text a prompt is assembled around.
    pub fn as_text(&self) -> &str {
        match self {
            Self::Text { text } => text,
        }
    }

    /// What this Selection is, so that the Palette can leave out the Actions
    /// that cannot run on it.
    pub(crate) fn kind(&self) -> Kind {
        match self {
            Self::Text { .. } => Kind::Text,
        }
    }
}
