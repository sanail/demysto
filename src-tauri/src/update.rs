//! Whether a newer Demysto exists, and putting it in place.
//!
//! A resident utility is the kind of program nobody looks at for a year, so
//! staying current cannot be something the user has to remember (user story
//! 64). Demysto asks once at startup, and what it finds waits — in the tray and
//! in Settings — until the user says to take it.
//!
//! Never taken under them, for a reason the platforms make plain: installing
//! means this process going away and coming back, and on Windows the installer
//! ends it outright. A tool that does that in the middle of a Conversation is
//! worse than one a version behind.
//!
//! An update is trusted because it is signed with a key generated for this and
//! nothing else, whose private half is a repository secret. That signature is
//! the whole of what makes taking one safe, and it is unrelated to any Apple or
//! Microsoft certificate — neither of which this project holds (ADR-0015).

use std::sync::Mutex;
use std::time::Duration;

use demysto_core::{say, Demysto};
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_updater::{Update, UpdaterExt};

// Demysto's words are taken out at each place that says something rather than
// once per function, and the identifier stays a literal at each of them. Both
// are constraints rather than habits: `words()` hands back a read guard, and a
// guard held across an `.await` is not something these futures can carry; and
// `i18n`'s suite finds a message by scanning the sources for `say!(` and
// `.text(`, so one asked for through a variable is a message it cannot see.

/// How long a copy that is left running waits before asking again.
///
/// A day, because that is the pace releases arrive at, and because the point of
/// asking twice is the copy nobody restarts for a month — not a copy that wants
/// to hear about a release the hour it appears.
const AGAIN: Duration = Duration::from_secs(60 * 60 * 24);

/// Emitted with the version now on offer, `null` where there is none, so that
/// the Settings window follows the answer instead of the moment it loaded.
///
/// It needs telling because of when the two things happen: every window is
/// created at startup and loads its page then, while the check on the way up
/// answers a moment later. A window that only asked as it loaded would show a
/// user who opens Settings an hour later the answer from before there was one.
const OFFERED_EVENT: &str = "update://offered";

/// The newer version the last check found, held between finding it and taking
/// it.
///
/// Held rather than looked for again when Install is pressed, so that what is
/// installed is the version the user read about. It is also what the tray reads
/// to decide whether to offer an update at all.
#[derive(Default)]
pub struct Offered(Mutex<Option<Update>>);

/// The version on offer, `None` where the last check found nothing — or where
/// no check has finished yet.
pub fn offered<R: Runtime, M: Manager<R>>(manager: &M) -> Option<String> {
    let held = manager.state::<Offered>();
    let held = held.0.lock().ok()?;

    held.as_ref().map(|update| update.version.clone())
}

/// Asks the manifest whether there is a newer version, and remembers the
/// answer.
///
/// Answers with the version found, `None` where this is already the newest.
/// A failure is a whole sentence: this is reached from Settings, where somebody
/// pressed a button and is owed one.
pub async fn look<R: Runtime>(app: &AppHandle<R>) -> Result<Option<String>, String> {
    let found = asked(app).await.map_err(|error| {
        let demysto = app.state::<Demysto>();
        let words = demysto.words();

        say!(&words, "update-refused", "detail" = error.to_string())
    })?;

    let version = found.as_ref().map(|update| update.version.clone());

    if let Ok(mut held) = app.state::<Offered>().0.lock() {
        *held = found;
    }

    let _ = app.emit(OFFERED_EVENT, version.clone());

    // The tray is how an update announces itself to somebody who is not sitting
    // in Settings, so the menu is rebuilt as soon as there is something to
    // announce — and rebuilt when there is not, which is what takes the item
    // away again once an update has been taken.
    //
    // On the main thread, because a menu belongs to the operating system and
    // every caller here is on a background task.
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        let actions = handle.state::<Demysto>().catalogue().actions;
        crate::tray::follows_the_catalogue(&handle, &actions);
    });

    Ok(version)
}

/// Takes the update the last check found: downloads it, verifies its signature,
/// installs it, and starts the version that was installed.
///
/// On Windows the installer ends this process itself and the restart below is
/// never reached. On macOS and Linux what is replaced is the bundle under a
/// process that goes on running, and restarting is how the user arrives at the
/// version they asked for rather than the one still in memory.
pub async fn take<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let update = {
        let demysto = app.state::<Demysto>();
        let words = demysto.words();

        app.state::<Offered>()
            .0
            .lock()
            .ok()
            .and_then(|held| held.clone())
            .ok_or_else(|| words.text("update-nothing-found"))?
    };

    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|error| {
            let demysto = app.state::<Demysto>();
            let words = demysto.words();

            say!(
                &words,
                "update-install-refused",
                "detail" = error.to_string()
            )
        })?;

    app.restart();
}

/// Asks in the background: on the way up, and once a day after that.
///
/// Twice rather than once because of what Demysto is. A tray utility is started
/// at login and left alone for weeks, and a copy that only ever asked at startup
/// would go a month without hearing of a release — which is the chore user
/// story 64 exists to remove.
///
/// Nothing is reported to the user: a check that failed because the machine is
/// on a train is not news, and at startup there is no window to report it in
/// anyway. It goes to the log, which is where a question nobody asked belongs.
///
/// A thread that sleeps rather than a timer, because sleeping is the whole of
/// what it does between two questions a day apart, and a timer would be a
/// dependency taken on for that.
pub fn keeps_looking<R: Runtime>(app: &AppHandle<R>) {
    let app = app.clone();

    std::thread::spawn(move || loop {
        let app = app.clone();

        tauri::async_runtime::spawn(async move {
            if let Err(said) = look(&app).await {
                app.state::<Demysto>().note(&said);
            }
        });

        std::thread::sleep(AGAIN);
    });
}

/// The manifest's answer, in the machinery's own terms.
///
/// Apart from the sentence a failure becomes, so that the two steps which can
/// fail — building the updater and asking it — are said the same way when
/// either does.
async fn asked<R: Runtime>(app: &AppHandle<R>) -> tauri_plugin_updater::Result<Option<Update>> {
    app.updater()?.check().await
}
