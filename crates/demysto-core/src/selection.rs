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

impl Selection {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }
}
