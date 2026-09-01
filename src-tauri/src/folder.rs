//! Handing a folder to the desktop's own file manager.
//!
//! One use: the button in Settings that opens the log folder, so that a bug
//! report can carry the logs (user story 63). The platform commands rather than
//! a plugin, because this is the whole of Demysto's interest in the file
//! manager and each platform's is one word.

use std::path::Path;
use std::process::Command;

use demysto_core::{say, Words};

/// Opens `path` in the desktop's file manager, creating it first where it is
/// not there yet.
///
/// Created rather than refused: the log folder exists from the first line
/// written, and somebody who has just installed Demysto and gone looking is
/// better served by an empty folder than by a button that does nothing.
///
/// Answers with what went wrong, in a whole sentence, so that the window that
/// offered the button is where the failure is reported.
pub fn open(path: &Path, words: &Words) -> Result<(), String> {
    std::fs::create_dir_all(path).map_err(|error| {
        say!(
            words,
            "folder-uncreatable",
            "path" = path.display().to_string(),
            "detail" = error.to_string()
        )
    })?;

    let opener = match () {
        _ if cfg!(target_os = "macos") => "open",
        _ if cfg!(target_os = "windows") => "explorer",
        _ => "xdg-open",
    };

    // Spawned rather than waited on: a file manager runs for as long as the
    // user keeps it open, and `explorer` reports a non-zero status even when it
    // did exactly what was asked.
    Command::new(opener)
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| {
            say!(
                words,
                "folder-no-file-manager",
                "detail" = error.to_string(),
                "path" = path.display().to_string()
            )
        })
}
