//! The route to the permission macOS gates a Capture behind.
//!
//! The core reports a Capture the system refused as a permission problem and
//! writes the sentence that names the pane; this is the button beside it (user
//! story 55). Opening a settings pane is nobody's business but the shell's —
//! it is a URL the desktop resolves, and `demysto-core` is not allowed to know
//! that such a thing exists.

use std::process::Command;

/// The Accessibility list inside Privacy & Security, as macOS addresses it.
///
/// The pane itself rather than the top of System Settings: what the user has to
/// do is find Demysto in one list and turn it on, and every step between them
/// and that list is a step they can get lost on.
#[cfg(target_os = "macos")]
const PANE: &str = "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility";

/// Opens the settings pane where the Accessibility permission is granted.
///
/// Answers with what went wrong, in a whole sentence, so that the window that
/// offered the button is where the failure is reported — the same bargain
/// `folder::open` makes.
#[cfg(target_os = "macos")]
pub fn reveal() -> Result<(), String> {
    // Spawned rather than waited on, for the reason a file manager is: System
    // Settings runs for as long as the user keeps it open.
    Command::new("open")
        .arg(PANE)
        .spawn()
        .map(|_| ())
        .map_err(|error| {
            format!(
                "Demysto could not open System Settings: {error}. The permission is in Privacy \
                 & Security → Accessibility."
            )
        })
}

/// No other platform gates a Capture behind a permission, so no other platform
/// has a pane to be walked to; see `demysto_core`'s `desktop`.
#[cfg(not(target_os = "macos"))]
pub fn reveal() -> Result<(), String> {
    Err("Only macOS asks for a permission before Demysto can read what you selected.".to_owned())
}
