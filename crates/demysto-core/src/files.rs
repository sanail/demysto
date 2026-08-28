//! Writing files in the configuration directory, without ever leaving a
//! half-written one behind.
//!
//! Here rather than in `config` because `catalogue` writes files under the same
//! rules: the directory is created owner-only, and a file is written beside
//! itself and renamed over rather than truncated and filled in. Both callers
//! wrap what goes wrong in an error of their own, so this layer speaks `io` and
//! composes no sentences.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// What a half-written file is called while it is being written.
const WRITING: &str = "writing";

/// Replaces `path` with `text`, owner-only.
///
/// Written beside the file and renamed over it rather than truncated and filled
/// in: the settings file holds a key, and a crash between the truncation and
/// the last byte would be a user whose credentials are simply gone. The rename
/// is one step as far as anybody reading the file is concerned, and the new
/// file carries the mode rather than inheriting whatever the old one had —
/// ADR-0002 asks for owner-only, and a file an interface wrote is no less bound
/// by it than one Demysto created.
pub(crate) fn replace(path: &Path, text: &str) -> io::Result<()> {
    use std::io::Write;

    if let Some(parent) = path.parent() {
        create_dir(parent)?;
    }

    // Beside the file, so that the rename stays within one filesystem: a
    // temporary directory elsewhere would make it a copy, which is exactly the
    // half-written state this is here to avoid.
    let beside = beside(path)?;

    // Whatever a crashed write left there is not a file to append to, and
    // `create_new` below would refuse it. Its mode is not to be trusted either.
    let _ = fs::remove_file(&beside);

    let mut file = options().create_new(true).write(true).open(&beside)?;

    if let Err(error) = file
        .write_all(text.as_bytes())
        .and_then(|()| file.sync_all())
    {
        drop(file);
        let _ = fs::remove_file(&beside);

        return Err(error);
    }

    drop(file);

    fs::rename(&beside, path).inspect_err(|_| {
        // The half-written file is not left lying next to the real one, where
        // the next write would have to distrust it and the user would have to
        // wonder what it is.
        let _ = fs::remove_file(&beside);
    })
}

/// Where a file is written before it is renamed over the real one.
///
/// The whole name plus a suffix rather than a replaced extension, so that
/// `settings.toml` becomes `settings.toml.writing`: two files whose names
/// differ only in extension must not be written to the same place, and a
/// leftover has to say what it was going to be.
fn beside(path: &Path) -> io::Result<PathBuf> {
    let name = path.file_name().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "a file to write must have a name",
        )
    })?;

    Ok(path.with_file_name(format!("{}.{WRITING}", name.to_string_lossy())))
}

/// Whether a directory entry is one of those half-written files, which nothing
/// reading a directory should mistake for the real thing.
pub(crate) fn is_half_written(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension == WRITING)
}

/// A file carrying a key is created readable by nobody else — the whole of what
/// ADR-0002 asks in exchange for keeping the key out of the keychain. Files
/// that carry none are written the same way rather than differently, because a
/// rule with an exception is a rule somebody has to remember.
#[cfg(unix)]
pub(crate) fn options() -> fs::OpenOptions {
    use std::os::unix::fs::OpenOptionsExt;

    let mut options = fs::OpenOptions::new();
    options.mode(0o600);
    options
}

#[cfg(not(unix))]
pub(crate) fn options() -> fs::OpenOptions {
    fs::OpenOptions::new()
}

/// The directory those files go in, owner-only for the same reason.
#[cfg(unix)]
pub(crate) fn create_dir(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;

    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path)
}

#[cfg(not(unix))]
pub(crate) fn create_dir(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)
}
