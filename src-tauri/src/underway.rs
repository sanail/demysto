//! A flag held for as long as one thing is under way, and cleared however that
//! thing ends.
//!
//! Two of Demysto's paths begin on a detached thread and must not overlap
//! themselves: opening the Palette, which sends a copy keystroke, and running an
//! Action, which spends the user's tokens. Both are started by a key the user
//! can press twice in a hurry.
//!
//! A guard rather than a pair of stores, because the threads are detached: a
//! panic on one takes nothing else down and nobody hears about it. A flag left
//! set by one would make every later attempt return at its first line, so the
//! Hotkey, the tray, and a second launch would all quietly do nothing for the
//! rest of the process's life. Unwinding past the guard clears the flag, which
//! keeps the damage to the press that caused it.

use std::sync::atomic::{AtomicBool, Ordering};

/// Holds one flag for as long as the thing it stands for is under way.
///
/// Single instance is enforced, so one flag per path covers the application,
/// which is why the flag is a `static` the caller owns rather than state
/// threaded through it.
pub struct Underway(&'static AtomicBool);

impl Underway {
    /// Claims the flag, or answers `None` when it is already held.
    pub fn claim(flag: &'static AtomicBool) -> Option<Self> {
        (!flag.swap(true, Ordering::SeqCst)).then_some(Self(flag))
    }
}

impl Drop for Underway {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}
