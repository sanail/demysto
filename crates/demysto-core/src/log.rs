//! The local log: what Demysto did, and never what the user was looking at.
//!
//! It exists for one purpose — that a bug report can carry something (user
//! story 63) — and it is written under the constraint the rest of the
//! application is written under: a tool holding somebody's key and somebody's
//! screen contents earns that by sending neither anywhere (user story 61), and
//! by not leaving them on disk after the session that held them (user story
//! 62). A log of prompts and answers would be exactly the history the
//! Conversation store deliberately does not keep, written to the one place
//! nothing clears. So what is recorded is the shape of what happened: which
//! Action, which Model, how many messages, how long an answer was, what went
//! wrong in the words the user was already shown. ADR-0010 records the decision.
//!
//! Nothing here reports a failure of its own. There is nowhere for a logger to
//! report to, and a log that can take the application down is worse than no log.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::files;
use crate::model::Resolved;
use crate::run::{RunError, RunOutcome};

/// The directory the logs live in, inside the configuration directory.
pub(crate) const DIR_NAME: &str = "logs";

/// What the log being written is called.
const FILE_NAME: &str = "demysto.log";

/// How large one file may get before it is rolled over.
///
/// Small enough that the whole set can be attached to a bug report without
/// anybody thinking about it, and large enough to hold more than the session
/// the report is about.
const ROTATE_AT: u64 = 512 * 1024;

/// How many rolled-over files are kept behind the one being written.
///
/// Three, so that a fault noticed a day late is still in the folder, and a
/// machine left running for a month is not storing a year of this.
const KEPT: usize = 3;

/// Where Demysto writes its log, and the one writer that does.
pub(crate) struct Log {
    dir: PathBuf,
    /// Held across the read of the file's size, the roll-over, and the write —
    /// which have to be one step, or two threads finishing a Turn at once roll
    /// the same file over twice and lose what was in it.
    writing: Mutex<()>,
}

impl Log {
    pub(crate) fn new(config_dir: &Path) -> Self {
        Self {
            dir: config_dir.join(DIR_NAME),
            writing: Mutex::new(()),
        }
    }

    /// The folder the log files are in, for the button in Settings that opens
    /// it. Answered whether or not anything has been written there yet: a user
    /// looking for logs is entitled to be shown where they would be.
    pub(crate) fn dir(&self) -> &Path {
        &self.dir
    }

    /// Records that this build started, which is what dates everything under it.
    pub(crate) fn started(&self, version: &str, config_dir: &Path) {
        self.say(&format!(
            "Demysto {version} started, configured from {}",
            config_dir.display()
        ));
    }

    /// Records a Turn going out: where to and how much of the Conversation went
    /// with it, and nothing about what any of it says.
    pub(crate) fn asking(&self, resolved: &Resolved, messages: usize) {
        self.say(&format!(
            "asking {} at {} with {messages} message(s)",
            resolved.model, resolved.endpoint.base_url
        ));
    }

    /// Records what a Turn produced: how much of an answer, or what went wrong.
    ///
    /// Through `RunError::logged` rather than the sentence the user was shown,
    /// which for one kind of failure quotes back what the Provider sent — and
    /// what it sent is the Model's own words.
    pub(crate) fn answered(&self, outcome: &RunOutcome) {
        self.say(&match outcome {
            RunOutcome::Answered(text) => format!("answered, {} characters", text.chars().count()),
            RunOutcome::Stopped(text) => {
                format!("stopped by the user, {} characters", text.chars().count())
            }
            RunOutcome::Interrupted { text, error } => format!(
                "interrupted after {} characters: {}",
                text.chars().count(),
                error.logged()
            ),
            RunOutcome::Failed(error) => format!("failed: {}", error.logged()),
        });
    }

    /// Records a Turn that never reached a Provider.
    pub(crate) fn failed(&self, error: &RunError) {
        self.say(&format!(
            "failed before anything was sent: {}",
            error.logged()
        ));
    }

    /// Records something the interface would otherwise only have said on a
    /// window that may never be opened — a Hotkey nobody could claim, an Action
    /// file nobody could read.
    pub(crate) fn said(&self, line: &str) {
        self.say(line);
    }

    /// Appends one line, rolling the file over first when it has grown enough.
    ///
    /// Opened and closed per line rather than held open: a few lines are written
    /// per Run, and a handle kept open is a file that cannot be rolled over on
    /// Windows and a folder the user cannot delete while Demysto runs.
    fn say(&self, line: &str) {
        let _writing = self.writing.lock().unwrap_or_else(|held| held.into_inner());

        // Owner-only, like everything else Demysto writes: a log naming which
        // Models somebody uses is not a secret, and it is not the neighbours'
        // business either.
        if files::create_dir(&self.dir).is_err() {
            return;
        }

        let path = self.dir.join(FILE_NAME);
        self.rotate(&path);

        let Ok(mut file) = files::options().create(true).append(true).open(&path) else {
            return;
        };

        let _ = writeln!(file, "{} {line}", stamped(SystemTime::now()));
    }

    /// Moves the file being written aside when it has grown past [`ROTATE_AT`],
    /// and drops the oldest one kept.
    fn rotate(&self, path: &Path) {
        let large = fs::metadata(path).is_ok_and(|held| held.len() >= ROTATE_AT);
        if !large {
            return;
        }

        let rolled = |at: usize| self.dir.join(format!("demysto.{at}.log"));

        let _ = fs::remove_file(rolled(KEPT));

        for at in (1..KEPT).rev() {
            let _ = fs::rename(rolled(at), rolled(at + 1));
        }

        let _ = fs::rename(path, rolled(1));
    }
}

/// One instant, written the way a log is read: UTC, to the second, widest unit
/// first, so that sorting the lines is sorting the times.
///
/// Composed here rather than taken from a date library because this is the
/// whole of Demysto's interest in the calendar, and a dependency is a thing to
/// keep up to date.
fn stamped(at: SystemTime) -> String {
    let seconds = at
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs());

    let (days, rest) = (seconds / 86_400, seconds % 86_400);
    let (year, month, day) = civil(days as i64);

    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rest / 3600,
        (rest % 3600) / 60,
        rest % 60
    )
}

/// The year, month and day a count of days since 1970-01-01 lands on.
///
/// Howard Hinnant's `civil_from_days`, which is the standard way to do this
/// without a table: the era is 400 years, the length the Gregorian calendar
/// repeats over, and the arithmetic inside one runs from March so that the leap
/// day falls at the end of a year rather than inside it.
fn civil(days: i64) -> (i64, u32, u32) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);

    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);

    let march = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * march + 2) / 5 + 1) as u32;
    let month = match march < 10 {
        true => march + 3,
        false => march - 9,
    } as u32;

    let year = year_of_era + era * 400 + i64::from(month <= 2);

    (year, month, day)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn at(seconds: u64) -> String {
        stamped(UNIX_EPOCH + Duration::from_secs(seconds))
    }

    #[test]
    fn the_epoch_is_written_as_the_day_it_is() {
        assert_eq!(at(0), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn the_time_of_day_is_the_time_of_day() {
        assert_eq!(at(86_399), "1970-01-01T23:59:59Z");
        assert_eq!(at(86_400), "1970-01-02T00:00:00Z");
    }

    #[test]
    fn a_leap_day_is_a_day_of_its_own() {
        // 2024-02-29T12:00:00Z.
        assert_eq!(at(1_709_208_000), "2024-02-29T12:00:00Z");
    }

    #[test]
    fn a_century_that_is_not_a_leap_year_is_not_treated_as_one() {
        // 1900 is divisible by four and is not a leap year; 2000 is and is.
        // 2000-02-29T00:00:00Z.
        assert_eq!(at(951_782_400), "2000-02-29T00:00:00Z");
        // 2100-03-01T00:00:00Z, the day after a February that has 28 days.
        assert_eq!(at(4_107_542_400), "2100-03-01T00:00:00Z");
    }

    #[test]
    fn a_log_that_outgrows_its_file_keeps_what_was_in_it() {
        let root = tempfile::tempdir().unwrap();
        let log = Log::new(root.path());

        // Past the roll-over on its own, so that the next line moves it aside.
        log.said(&"x".repeat(ROTATE_AT as usize));
        log.said("after the roll-over");

        let written = fs::read_to_string(log.dir().join(FILE_NAME)).unwrap();
        let rolled = fs::read_to_string(log.dir().join("demysto.1.log")).unwrap();

        assert!(written.contains("after the roll-over"), "{written}");
        assert!(
            !written.contains("xxx"),
            "the file was moved aside, not kept"
        );
        assert!(
            rolled.contains("xxx"),
            "what was in it is in the rolled file"
        );
    }

    #[test]
    fn only_the_files_worth_keeping_are_kept() {
        let root = tempfile::tempdir().unwrap();
        let log = Log::new(root.path());

        for _ in 0..KEPT + 2 {
            log.said(&"x".repeat(ROTATE_AT as usize));
        }

        let held: Vec<String> = fs::read_dir(log.dir())
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();

        assert_eq!(held.len(), KEPT + 1, "{held:?}");
    }
}
