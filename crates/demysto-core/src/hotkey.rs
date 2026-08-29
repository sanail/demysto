//! Which keys a Hotkey may be on its own.
//!
//! A Hotkey is claimed from the whole operating system, so a Hotkey that is one
//! key answers to that key everywhere — in the middle of an email, in a text
//! field, in somebody else's editor. For nearly every key that is a way to lose
//! the key rather than to bind an Action, which is why a Hotkey ordinarily has
//! to state a modifier.
//!
//! A few keys are not like that. Pause, ScrollLock, PrintScreen, F13 and above,
//! and the volume and media keys type nothing, move no cursor, and are what a
//! resident utility is worth binding to: taking one costs the user nothing they
//! were using. This module is the list of them.
//!
//! Here rather than in the shell that does the claiming, because "which keys
//! type nothing" is a decision about the product and not about any operating
//! system — the shell has already parsed the combination by the time it asks,
//! and all it needs back is an answer about a name. The names are the ones the
//! W3C gives keys, which is what a browser reports a keypress as and what the
//! Hotkey parser reads, so one list serves the window that records a Hotkey and
//! the claim that follows.

/// The keys a Hotkey may be on its own.
///
/// Deliberately narrow, and worth leaving narrow. The near neighbours are all
/// keys somebody is already using: `F1` to `F12` are bound by applications —
/// refresh, full screen, developer tools — and claiming one takes it from every
/// one of them; the arrows and `Home`, `End`, `PageUp` and `PageDown` move a
/// cursor; `Enter`, `Space`, `Tab`, `Escape`, `Backspace` and `Delete` are how
/// text gets written and unwritten. None of those belongs here however much a
/// Hotkey on one would suit somebody's fingers.
const TYPES_NOTHING: &[&str] = &[
    "Pause",
    "PrintScreen",
    "ScrollLock",
    "F13",
    "F14",
    "F15",
    "F16",
    "F17",
    "F18",
    "F19",
    "F20",
    "F21",
    "F22",
    "F23",
    "F24",
    "AudioVolumeUp",
    "AudioVolumeDown",
    "AudioVolumeMute",
    "MediaPlay",
    "MediaPause",
    "MediaPlayPause",
    "MediaStop",
    "MediaTrackNext",
    "MediaTrackPrevious",
];

/// Whether a Hotkey stating this key and nothing else is one Demysto will claim.
pub(crate) fn needs_no_modifier(key: &str) -> bool {
    TYPES_NOTHING.contains(&key)
}

/// The whole list, for the window that records a Hotkey: it has to know which
/// bare keypress to take and which to go on waiting through.
pub(crate) fn keys_that_need_no_modifier() -> Vec<&'static str> {
    TYPES_NOTHING.to_vec()
}
