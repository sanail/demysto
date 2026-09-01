//! The Hotkey on Wayland, where an application cannot claim one for itself.
//!
//! Wayland gives no client a way to hear a key it is not focused for, so the
//! Hotkey is asked of `org.freedesktop.portal.GlobalShortcuts` instead: Demysto
//! says what it would like each Hotkey to be, and the desktop decides — and can
//! be told otherwise by the user, in the desktop's own shortcut settings
//! (ADR-0003). A portal that is not there is not a failure to hide: without it
//! no Hotkey answers at all, and the user is told so in the same place every
//! other Hotkey that could not be claimed is reported.
//!
//! A desktop that is there and not yet ready is a different thing again, and
//! the one autostart on Wayland runs into: for the first minute of a session
//! the portal takes the request for a Hotkey and never answers it. So an asking
//! that came to nothing — or to no answer at all — is repeated, a handful of
//! times over a few minutes, rather than leaving the Hotkey dead until Demysto
//! is restarted.
//!
//! Only Linux reaches the half of this that talks to the portal. The half that
//! translates a Hotkey into the syntax the portal takes is plain string work,
//! and is compiled and tested everywhere so that the one platform it runs on is
//! not the only place it is checked.

use std::time::Duration;

use demysto_core::{say, Words};

/// The identifier the portal knows the Palette's Hotkey by.
///
/// Fixed rather than derived, and unlike anything [`for_action`] produces, so
/// that an Action cannot arrive with an identifier that takes the Palette's
/// place.
pub const OPENS_THE_PALETTE: &str = "palette";

/// The identifier the portal knows one Action's Hotkey by.
pub fn for_action(action: &str) -> String {
    format!("action:{action}")
}

/// The Action a portal identifier names, or `None` where it names the Palette.
pub fn action_of(id: &str) -> Option<&str> {
    id.strip_prefix("action:")
}

/// One Hotkey Demysto asks the portal for.
#[derive(Clone, PartialEq, Eq)]
pub struct Wanted {
    /// What the portal knows it by, and what comes back when it is pressed.
    pub id: String,
    /// What the desktop's own shortcut settings show beside it, so that a user
    /// looking at a list of every application's shortcuts can tell which is
    /// which.
    pub description: String,
    /// The combination Demysto would like it to be, where the settings state
    /// one the portal's syntax can carry.
    ///
    /// A preference and not a claim: the desktop assigns the combination, and
    /// is free to assign another or none at all.
    pub trigger: Option<String>,
}

/// One Hotkey the desktop is holding, as it answered when it was asked for it.
///
/// The portal's own answer reduced to the two things any of this turns on, so
/// that what became of an asking is decided and worded on every platform's
/// suite rather than only where there is a portal to reach.
struct Held<'a> {
    /// What it was asked for by, which is [`Wanted::id`].
    id: &'a str,
    /// The combination the desktop says it is held under, and `None` where it
    /// is held under none at all — which is a Hotkey nothing answers to and the
    /// user's to assign.
    under: Option<&'a str>,
}

/// The Hotkey the desktop is holding under the identifier one was asked for by,
/// or `None` where it took none.
fn holding<'a>(held: &'a [Held], wanted: &Wanted) -> Option<&'a Held<'a>> {
    held.iter().find(|held| held.id == wanted.id)
}

/// What the user is owed about the Hotkeys the portal now holds: the ones it
/// holds under no combination, and the ones it did not take at all.
///
/// Not a line each for the ones that came out well — the desktop's own shortcut
/// settings list those, and this is the window's list of what does not answer.
///
/// `asking_again` is whether Demysto is going to ask the desktop again, and it
/// changes what the user is told to do about a Hotkey that was not taken: while
/// Demysto is still asking, sending them to their desktop's settings would be
/// sending them to undo work that is still under way.
fn what_came_of(
    wanted: &[Wanted],
    held: &[Held],
    asking_again: bool,
    words: &Words,
) -> Vec<String> {
    wanted
        .iter()
        .filter_map(|wanted| {
            let named = wanted.description.clone();

            let Some(held) = holding(held, wanted) else {
                // Two messages rather than one with a clause in it: while
                // Demysto is still asking, sending somebody to their desktop's
                // settings is sending them to undo work that is still under way.
                return Some(match asking_again {
                    true => say!(words, "portal-not-taken-yet", "wanted" = named),
                    false => say!(words, "portal-not-taken", "wanted" = named),
                });
            };

            held.under
                .is_none()
                .then(|| say!(words, "portal-held-under-nothing", "wanted" = named))
        })
        .collect()
}

/// Whether the desktop took nothing at all of what it was asked for, which is
/// the answer worth asking again about.
///
/// A session and the applications it starts come up together — autostart on
/// Wayland — and for the first minute of one the desktop takes the request for
/// a Hotkey and never answers it: no error, no refusal, nothing to report, and
/// not one of the Hotkeys asked for in the answer that never came.
///
/// Nothing at all, rather than one of several: a desktop that took some of the
/// set answered and decided, and asking again means giving up what it did take
/// to ask for the lot afresh. Nor a Hotkey it is holding under no combination —
/// that one it took, and giving it keys is the user's to do in their own
/// settings, where asking again would fight them.
fn took_nothing(wanted: &[Wanted], held: &[Held]) -> bool {
    !wanted.is_empty() && !wanted.iter().any(|wanted| holding(held, wanted).is_some())
}

/// How long one asking is given to be answered before it is taken as lost.
///
/// Watched on a Plasma session coming up: the portal takes the request for the
/// Hotkeys and never answers it. Not an error, not a refusal, not an empty
/// answer — no answer at all, for the rest of the session, and a Demysto
/// waiting on one waits that long too.
///
/// A minute, because the two things this waits on are minutes apart. A portal
/// that is going to answer answers in well under a second; the other half of
/// the wait is GNOME, where the answer is the user pressing a button on the
/// consent dialog, and a minute is longer than that takes anybody who is
/// looking at it.
const ANSWER_WITHIN: Duration = Duration::from_secs(60);

/// How long to wait before asking the desktop again, or `None` where it has
/// been asked as often as it is going to be.
///
/// `asked` counts the askings that have already happened, the first of which is
/// the one at startup.
///
/// Doubling: a desktop that is merely slow is asked again a second after it
/// came to nothing rather than a minute after, and one that is never going to
/// take the Hotkey is asked six times in all and then left alone. The bound is
/// what keeps a desktop with no GlobalShortcuts portal at all — or one whose
/// user has decided Demysto is not having a Hotkey — from being asked for the
/// rest of the day.
fn before_asking_again(asked: u32) -> Option<Duration> {
    /// How many askings there are altogether, the one at startup included.
    const ALTOGETHER: u32 = 6;

    (1..ALTOGETHER)
        .contains(&asked)
        .then(|| Duration::from_secs(1 << (asked - 1)))
}

/// A Hotkey as the portal reads one: the modifiers it takes, and then the key,
/// as named by the "shortcuts" XDG specification.
///
/// `None` for a Hotkey stated in a way that specification has no name for,
/// which costs only the preference — the desktop assigns a combination either
/// way, and the user can change it.
pub fn trigger(hotkey: &str) -> Option<String> {
    let mut parts: Vec<&str> = hotkey.split('+').filter(|part| !part.is_empty()).collect();
    let key = parts.pop()?;

    let mut stated: Vec<&str> = parts
        .iter()
        .map(|part| modifier(part))
        .collect::<Option<_>>()?;
    stated.push(named(key)?);

    Some(stated.join("+"))
}

/// One modifier, as the portal names it. Demysto writes them the way the user
/// reads them, and the parser that claims a Hotkey everywhere else takes
/// several spellings of each; all of them arrive here.
fn modifier(part: &str) -> Option<&'static str> {
    match part.to_ascii_lowercase().as_str() {
        "ctrl" | "control" => Some("CTRL"),
        "shift" => Some("SHIFT"),
        "alt" | "option" => Some("ALT"),
        "cmd" | "command" | "super" | "meta" | "win" | "windows" => Some("SUPER"),
        _ => None,
    }
}

/// The key half, translated from the name a browser reports a keypress under —
/// which is what a Hotkey is recorded and stored as — into the keysym name the
/// portal takes.
fn named(key: &str) -> Option<&'static str> {
    /// The keys a Hotkey may be on its own — every one of which has to be here,
    /// which the suite below checks — and the ones anybody reaches for with a
    /// modifier. Written on the left as a browser reports the keypress, and on
    /// the right as `xkbcommon-keysyms.h` names the key, which is the naming
    /// the "shortcuts" specification takes.
    const NAMED: &[(&str, &str)] = &[
        ("space", "space"),
        ("enter", "Return"),
        ("tab", "Tab"),
        ("backspace", "BackSpace"),
        ("escape", "Escape"),
        ("pause", "Pause"),
        ("scrolllock", "Scroll_Lock"),
        ("printscreen", "Print"),
        ("insert", "Insert"),
        ("delete", "Delete"),
        ("home", "Home"),
        ("end", "End"),
        ("pageup", "Page_Up"),
        ("pagedown", "Page_Down"),
        ("arrowup", "Up"),
        ("arrowdown", "Down"),
        ("arrowleft", "Left"),
        ("arrowright", "Right"),
        ("audiovolumeup", "XF86AudioRaiseVolume"),
        ("audiovolumedown", "XF86AudioLowerVolume"),
        ("audiovolumemute", "XF86AudioMute"),
        ("mediaplay", "XF86AudioPlay"),
        ("mediapause", "XF86AudioPause"),
        // XKB has no separate play-pause: the one key on the keyboard is the
        // one this names, and a desktop that distinguishes them is not one the
        // "shortcuts" specification can be told about.
        ("mediaplaypause", "XF86AudioPlay"),
        ("mediastop", "XF86AudioStop"),
        ("mediatracknext", "XF86AudioNext"),
        ("mediatrackprevious", "XF86AudioPrev"),
    ];

    let key = key.to_ascii_lowercase();

    // A letter and a digit are the common half of this and carry their own
    // name: `KeyE` is the key that types an `e`, and the portal calls it `e`.
    if let Some(letter) = key.strip_prefix("key") {
        return single(letter);
    }

    if let Some(digit) = key.strip_prefix("digit") {
        return single(digit);
    }

    if let Some(number) = key.strip_prefix('f') {
        return function_key(number);
    }

    NAMED
        .iter()
        .find(|(reported, _)| *reported == key)
        .map(|(_, keysym)| *keysym)
}

/// The one character a `KeyX` or a `DigitN` ends in, borrowed from a table of
/// them so that it can be handed back as the `'static` every other arm answers
/// with.
fn single(rest: &str) -> Option<&'static str> {
    /// Every character either of those two can end in.
    const CHARACTERS: &str = "abcdefghijklmnopqrstuvwxyz0123456789";

    let [character] = rest.as_bytes() else {
        return None;
    };

    let at = CHARACTERS.find(char::from(*character))?;

    CHARACTERS.get(at..=at)
}

/// `F1` through `F24`, which are the Hotkeys somebody reaches for when every
/// combination they wanted is taken — and, from F13 up, the keys a Hotkey may
/// be on its own.
fn function_key(number: &str) -> Option<&'static str> {
    /// Named rather than formatted, for the reason [`single`] has a table: the
    /// portal is handed a `'static` name.
    const KEYS: &[&str] = &[
        "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12", "F13", "F14",
        "F15", "F16", "F17", "F18", "F19", "F20", "F21", "F22", "F23", "F24",
    ];

    let at = number.parse::<usize>().ok()?;

    KEYS.get(at.checked_sub(1)?).copied()
}

#[cfg(target_os = "linux")]
pub use binding::claim;

#[cfg(target_os = "linux")]
mod binding {
    //! What talks to the portal.

    use std::sync::Mutex;
    use std::time::Duration;

    use ashpd::desktop::global_shortcuts::{
        Activated, BindShortcutsOptions, GlobalShortcuts, NewShortcut, Shortcut,
    };
    use ashpd::desktop::{CreateSessionOptions, Session};
    use ashpd::AppID;
    use demysto_core::Interface;
    use futures_util::{Stream, StreamExt};
    use tauri::async_runtime::{Receiver, Sender};

    use super::{
        before_asking_again, say, took_nothing, what_came_of, Held, Wanted, Words, ANSWER_WITHIN,
    };

    /// What the Hotkeys the portal holds came to, in whole sentences, for the
    /// window that reports every Hotkey Demysto does not answer to.
    ///
    /// Held here rather than answered by [`claim`] because the portal is asked
    /// on a task of its own: binding can put a dialog in front of the user, and
    /// the thread [`claim`] is called on is the thread every window Demysto has
    /// is drawn on. So the report is what the last asking came to — the one at
    /// startup, or whichever asking again has happened since.
    static REPORT: Mutex<Vec<String>> = Mutex::new(Vec::new());

    /// What was last asked of the portal, so that asking again for the same
    /// thing can be recognised and skipped.
    ///
    /// The catalogue is read at startup and again by every window that lists
    /// Actions, and reading it is what claims the Hotkeys in it — deliberately,
    /// so that an Action which arrived as a file somebody sent answers to its
    /// Hotkey without a restart. Everywhere else that costs a re-registration
    /// nobody notices. Here it costs the user a second consent dialog stacked
    /// on the first: the Settings window is created hidden at startup, its
    /// webview asks for the catalogue about a second later, and the desktop
    /// dutifully asks again about Hotkeys it is already asking about.
    ///
    /// So the guard is here rather than in `hotkey::claim`: this is the one
    /// platform where asking twice is something the user has to answer twice.
    ///
    /// What counts as the same set includes each Hotkey's description, and that
    /// is deliberate rather than incidental: the description is what the
    /// desktop's own shortcut settings show beside the combination, so it is
    /// part of the interface and follows the interface language. The cost is
    /// that changing the language in Settings asks the desktop again — one more
    /// consent dialog, and on a desktop that drops its bindings when a session
    /// is replaced, the combinations assigned by hand with it. Renaming an
    /// Action has always cost the same, for the same reason; the language is a
    /// second way to reach it. The alternative is a shortcut list left naming
    /// Demysto's Actions in a language the user has stopped reading, which is
    /// worse in the place it is worst — the one screen where they go to fix a
    /// Hotkey that does not answer.
    static ASKED: Mutex<Option<Vec<Wanted>>> = Mutex::new(None);

    /// How the task holding the portal session open is told to let go, which is
    /// what a fresh set of Actions needs.
    ///
    /// Told rather than killed: the session has to be closed on the bus, and a
    /// task aborted mid-await never gets to do it — `ashpd`'s session has no
    /// `Drop` of its own, so one merely let go of stays open, still holding
    /// every Hotkey in it.
    static STOPPING: Mutex<Option<Sender<()>>> = Mutex::new(None);

    /// Asks the portal for every Hotkey Demysto answers to, and answers with
    /// what the last such request came to.
    ///
    /// `pressed` is handed the portal identifier of whichever Hotkey was
    /// pressed, for as long as the session lives. `noted` is handed each
    /// sentence this one comes to, once it has: the asking happens on a task,
    /// and at startup there is no window for what it finds to appear on — the
    /// log is where such a report belongs, and the caller is what has one.
    pub fn claim(
        app_id: String,
        wanted: Vec<Wanted>,
        interface: Interface,
        pressed: impl Fn(&str) + Send + Sync + 'static,
        noted: impl Fn(&str) + Send + Sync + 'static,
    ) -> Vec<String> {
        // Nothing has changed, so there is nothing to ask: the session already
        // open holds exactly these Hotkeys, and giving it up to ask for them
        // again would put a second dialog in front of the user for no gain.
        let mut asked = ASKED.lock().unwrap_or_else(|held| held.into_inner());
        if asked.as_deref() == Some(wanted.as_slice()) {
            return REPORT
                .lock()
                .unwrap_or_else(|held| held.into_inner())
                .clone();
        }
        *asked = Some(wanted.clone());
        drop(asked);

        let mut stopping = STOPPING.lock().unwrap_or_else(|held| held.into_inner());

        // Before the new one is started, so that the portal is not left holding
        // two sessions asking for the same combinations — and so that one
        // keypress is not answered twice, once per session.
        if let Some(previous) = stopping.take() {
            let _ = previous.try_send(());
        }

        let (stop, stopped) = tauri::async_runtime::channel(1);
        *stopping = Some(stop);

        tauri::async_runtime::spawn(async move {
            // Nothing said yet, so whatever this comes to is said: a sequence of
            // askings that ends in a sentence ends because of that sentence, and
            // it belongs in the log however familiar it reads.
            // The language settled here rather than borrowed from the caller:
            // this task outlives the call that started it by the whole of the
            // session, and everything it comes to has to be said in something.
            // A save that changes the language asks again, through `claim`,
            // which is where this one is stopped and the next one started.
            let words = Words::spoken(interface);

            if let Err(said) = hold(app_id, wanted, pressed, &noted, stopped, &words).await {
                report(vec![said], &noted, &mut Vec::new());
            }
        });

        REPORT
            .lock()
            .unwrap_or_else(|held| held.into_inner())
            .clone()
    }

    /// Puts what the portal came to where the window reads it, and where the
    /// log keeps it for a bug report to carry.
    ///
    /// The window is told every time, because a Hotkey the desktop has since
    /// taken has to stop being reported as one that does not answer. The log is
    /// told only what these askings have not told it already: one asking can be
    /// followed by five more coming to the same thing, and a log repeating
    /// itself is a log nobody reads to the end.
    ///
    /// `told` is what those askings have said so far, and is the caller's
    /// rather than a second `static` beside [`REPORT`], so that the quiet is
    /// only ever within the one sequence of askings. A later `claim`, about a
    /// set of Hotkeys somebody has just changed, says its piece even where it
    /// comes out in the same words.
    fn report(said: Vec<String>, noted: &impl Fn(&str), told: &mut Vec<String>) {
        if *told != said {
            for one in &said {
                noted(one);
            }

            told.clone_from(&said);
        }

        *REPORT.lock().unwrap_or_else(|held| held.into_inner()) = said;
    }

    /// Binds the Hotkeys and stays in the loop that answers them — asking again
    /// first, for as long as [`before_asking_again`] allows, where the desktop
    /// took none of them or never answered that it had.
    ///
    /// Answers `Err` with the sentence saying so for every way this ends that
    /// leaves the user without a Hotkey, and `Ok` for the one that does not:
    /// being told to let go, which happens only because another asking is
    /// taking this one's place.
    async fn hold(
        app_id: String,
        wanted: Vec<Wanted>,
        pressed: impl Fn(&str),
        noted: &impl Fn(&str),
        mut stopped: Receiver<()>,
        words: &Words,
    ) -> Result<(), String> {
        say_who_we_are(&app_id).await;

        let portal = GlobalShortcuts::new()
            .await
            .map_err(|error| unreachable(error, words))?;

        // Subscribed before anything is bound, so that a Hotkey pressed the
        // instant the desktop assigns it is not one nobody was listening for.
        // Pinned because it is waited on again and again, beside the other
        // things this waits on. Once and not once per asking: the signal is the
        // portal's rather than the session's, and a second subscription would
        // answer every keypress twice.
        let mut activated = Box::pin(
            portal
                .receive_activated()
                .await
                .map_err(|error| unreachable(error, words))?,
        );

        let hotkeys: Vec<NewShortcut> = wanted.iter().map(asked_for).collect();

        let mut session = portal
            .create_session(CreateSessionOptions::default())
            .await
            .map_err(|error| unreachable(error, words))?;

        // How many askings have happened, which is what decides whether there
        // is another and how long it waits. The asking again lives here rather
        // than going back through [`claim`]: the guard there recognises an
        // unchanged set and skips the work, and this is exactly a set that has
        // not changed.
        let mut asked = 0;

        // What the log has been told so far, so that six askings coming to the
        // same thing do not write it out six times.
        let mut told = Vec::new();

        loop {
            let answer =
                portal.bind_shortcuts(&session, &hotkeys, None, BindShortcutsOptions::default());

            // An asking that is never answered is an asking that came to
            // nothing, and is told from one that came back empty only in the
            // log. Both leave the user without a Hotkey and both are worth
            // asking again about, so everything below says the same of either —
            // because it is the same thing.
            let bound = match tokio::time::timeout(ANSWER_WITHIN, answer).await {
                Ok(answered) => Some(
                    answered
                        .map_err(|error| unreachable(error, words))?
                        .response()
                        .map_err(|error| refused(error, words))?,
                ),
                Err(_) => None,
            };
            asked += 1;

            // Asked before the report is written, because binding can sit in
            // front of the user for as long as they leave the dialog there:
            // another asking may have replaced this one in the meantime, and
            // its report is the one to keep.
            if stopped.try_recv().is_ok() {
                return closing(&session).await;
            }

            let held: Vec<Held> = bound
                .iter()
                .flat_map(|bound| bound.shortcuts())
                .map(held)
                .collect();

            // Decided before the report is written, because it is half of what
            // the report says: a Hotkey nobody has taken yet is the user's to
            // go and assign only once Demysto has stopped asking for it.
            let nothing_taken = took_nothing(&wanted, &held);
            let asking_again = nothing_taken.then(|| before_asking_again(asked)).flatten();

            report(
                what_came_of(&wanted, &held, asking_again.is_some(), words),
                noted,
                &mut told,
            );

            if !nothing_taken {
                if asked > 1 {
                    // The sentences above this one in the log are not the
                    // last word on it: a Hotkey that arrived on the second
                    // asking is a Hotkey that arrived.
                    noted(&words.text("portal-taken-in-the-end"));
                }

                break;
            }

            let Some(before) = asking_again else {
                noted(&asked_enough(asked, words));

                break;
            };

            if asked == 1 {
                // Once, on the first asking that came to nothing: the line
                // above it names the Hotkey and says Demysto is asking again,
                // and this says why and for how long.
                noted(&words.text("portal-asking-again"));
            }

            match waiting(before, &mut activated, &pressed, &mut stopped).await {
                Waited::Through => {}
                Waited::Stopped => return closing(&session).await,
                Waited::PortalGone => return Err(words.text("portal-stopped-answering")),
            }

            // Given up before the next one is created, so that the portal is
            // not left holding two sessions asking for the same combinations.
            // Nothing is lost with it: this is reached only where the desktop
            // took nothing, so the session being given up holds nothing.
            let _ = session.close().await;
            session = portal
                .create_session(CreateSessionOptions::default())
                .await
                .map_err(|error| unreachable(error, words))?;
        }

        loop {
            tokio::select! {
                activation = activated.next() => if !still_speaking(activation, &pressed) {
                    break Err(words.text("portal-stopped-answering"));
                },
                _ = stopped.recv() => break closing(&session).await,
            }
        }
    }

    /// Answers the Hotkey an activation names, and says whether the portal is
    /// still speaking at all.
    ///
    /// A stream that has ended is every Hotkey gone with it — xdg-desktop-portal
    /// restarted, or the session was closed under Demysto. Worth telling apart,
    /// because a Hotkey that has quietly stopped answering is the failure a user
    /// cannot tell from a tool that has hung.
    fn still_speaking(activation: Option<Activated>, pressed: &impl Fn(&str)) -> bool {
        let Some(activation) = activation else {
            return false;
        };

        pressed(activation.shortcut_id());

        true
    }

    /// What ended the wait between one asking and the next.
    enum Waited {
        /// The wait ran out, which is what a wait is for.
        Through,
        /// Another asking has taken this one's place.
        Stopped,
        /// The portal stopped speaking, and there is nothing left to ask.
        PortalGone,
    }

    /// Waits out the pause before the next asking, and says what ended it.
    ///
    /// Waiting rather than sleeping: being told to let go has to be heard
    /// through the pause, and so has the portal falling silent, which is the
    /// one thing that makes the next asking pointless.
    async fn waiting(
        before: Duration,
        activated: &mut (impl Stream<Item = Activated> + Unpin),
        pressed: &impl Fn(&str),
        stopped: &mut Receiver<()>,
    ) -> Waited {
        // A deadline rather than a fresh sleep each time round, so that a
        // Hotkey pressed during the wait does not put the next asking off.
        let until = tokio::time::Instant::now() + before;

        loop {
            tokio::select! {
                _ = tokio::time::sleep_until(until) => break Waited::Through,
                _ = stopped.recv() => break Waited::Stopped,
                activation = activated.next() => if !still_speaking(activation, pressed) {
                    break Waited::PortalGone;
                },
            }
        }
    }

    /// Gives the session back, which is what gives up the Hotkeys in it.
    ///
    /// Nothing is reported: this only happens because another asking has
    /// replaced this one, and that one's report is the one the window wants.
    async fn closing(session: &Session<GlobalShortcuts>) -> Result<(), String> {
        let _ = session.close().await;

        Ok(())
    }

    /// Tells the portal which application this is, before asking it for
    /// anything.
    ///
    /// Without this every GlobalShortcuts request is refused outright with
    /// `NotAllowed: An app id is required`, on GNOME and on KDE alike — watched
    /// on both. The portal keeps a Hotkey against an application identity, and
    /// a sandboxed application carries one it cannot forge; an ordinary one has
    /// to say who it is, and `org.freedesktop.host.portal.Registry` is where.
    /// Qt says it for its own applications, which is why a desktop's own
    /// components manage what Demysto could not.
    ///
    /// The identifier has to match an installed desktop entry — that is how the
    /// portal finds the name and icon it shows the user — so it comes from the
    /// application's own configuration rather than being written out here.
    ///
    /// A failure is deliberately not reported: registering twice on the one
    /// connection `ashpd` keeps is refused, and so is registering on a desktop
    /// whose portal predates the interface. Neither is worth a sentence,
    /// because the request that follows is about to produce the real one.
    async fn say_who_we_are(app_id: &str) {
        let Ok(app_id) = app_id.parse::<AppID>() else {
            return;
        };

        let _ = ashpd::register_host_app(app_id).await;
    }

    /// One Hotkey as the portal is asked for it.
    fn asked_for(wanted: &Wanted) -> NewShortcut {
        NewShortcut::new(&wanted.id, &wanted.description)
            .preferred_trigger(wanted.trigger.as_deref())
    }

    /// One Hotkey the portal answered with, as the half of this that words the
    /// answer reads one.
    fn held(shortcut: &Shortcut) -> Held<'_> {
        Held {
            id: shortcut.id(),
            // The portal says "held under no combination" with an empty string,
            // and the half that words the answer should not have to know that.
            under: Some(shortcut.trigger_description()).filter(|under| !under.is_empty()),
        }
    }

    /// What the user is told when the desktop has been asked as often as it is
    /// going to be. Which Hotkeys are still missing is the report's to say; this
    /// says only that nothing more is going to happen on its own.
    fn asked_enough(asked: u32, words: &Words) -> String {
        say!(words, "portal-asked-enough", "asked" = asked)
    }

    /// What the user is told when the desktop answered the request for the
    /// Hotkeys with something other than the Hotkeys — dismissing the dialog it
    /// puts up, most often.
    ///
    /// Held apart from [`unreachable`], which is about not reaching a portal at
    /// all: a dialog somebody closed is not a desktop to go and upgrade.
    fn refused(error: ashpd::Error, words: &Words) -> String {
        say!(words, "portal-refused", "detail" = error.to_string())
    }

    /// What the user is told when the portal is not there at all — which is
    /// every Hotkey Demysto has, gone.
    ///
    /// Names where the portal comes from, because on a desktop old enough not
    /// to have one there is nothing to turn on and the answer is an upgrade.
    fn unreachable(error: ashpd::Error, words: &Words) -> String {
        say!(words, "portal-unreachable", "detail" = error.to_string())
    }
}

#[cfg(test)]
mod tests {
    //! Everything here that is not I/O: the translation, which decides whether
    //! a Wayland user's stated Hotkey reaches the desktop as the combination
    //! they stated, and what Demysto makes of the answer — the sentences the
    //! user reads, and whether the desktop is asked again.

    use super::*;

    /// The suite reads its assertions in English, whatever the machine running
    /// it is set to — the sentences themselves are `i18n`'s to check.
    fn english() -> Words {
        use demysto_core::Interface;

        Words::spoken(Interface::English)
    }

    /// One Hotkey asked for, of the shape the Palette's and an Action's both
    /// have: the identifier is what the answer is matched against, and the
    /// description is what every sentence names.
    fn wanted(id: &str) -> Wanted {
        Wanted {
            id: id.to_owned(),
            description: format!("Demysto — {id}"),
            trigger: trigger("Ctrl+Shift+KeyE"),
        }
    }

    /// One Hotkey the desktop took and gave keys to.
    fn taken(id: &str) -> Held<'_> {
        Held {
            id,
            under: Some("Ctrl+Shift+E"),
        }
    }

    /// One Hotkey the desktop took and left under no combination.
    fn unassigned(id: &str) -> Held<'_> {
        Held { id, under: None }
    }

    #[test]
    fn a_hotkey_the_desktop_did_not_take_is_reported() {
        let said = what_came_of(&[wanted("palette")], &[], false, &english());

        assert_eq!(said.len(), 1);
        assert!(said[0].contains("did not take"), "{}", said[0]);
        assert!(said[0].contains("Demysto — palette"), "{}", said[0]);
        assert!(
            said[0].contains("keyboard shortcut settings"),
            "{}",
            said[0]
        );
    }

    /// While Demysto is still asking, the window says so rather than sending
    /// the user off to assign by hand what may yet arrive on its own.
    #[test]
    fn a_hotkey_still_being_asked_for_says_so_instead() {
        let said = what_came_of(&[wanted("palette")], &[], true, &english());

        assert_eq!(said.len(), 1);
        assert!(said[0].contains("asking again"), "{}", said[0]);
        assert!(
            !said[0].contains("keyboard shortcut settings"),
            "{}",
            said[0]
        );
    }

    #[test]
    fn a_hotkey_held_under_no_combination_is_reported() {
        let said = what_came_of(
            &[wanted("palette")],
            &[unassigned("palette")],
            false,
            &english(),
        );

        assert_eq!(said.len(), 1);
        assert!(said[0].contains("under no combination"), "{}", said[0]);
    }

    #[test]
    fn a_hotkey_the_desktop_took_is_not_reported() {
        let said = what_came_of(&[wanted("palette")], &[taken("palette")], false, &english());

        assert!(said.is_empty(), "{said:?}");
    }

    /// The Palette's Hotkey and an Action's are one set, and it is the set that
    /// is asked for again — so it is the set that decides.
    ///
    /// Nothing taken is a desktop that has not answered, and is asked again. A
    /// desktop that took some of it has answered and decided, and asking again
    /// would give up what it did take. One it took and gave no keys to counts
    /// as taken: assigning that one is the user's to do, and asking again would
    /// take it away from them and start over.
    #[test]
    fn only_a_desktop_that_took_nothing_is_asked_again() {
        let wanted = [wanted("palette"), wanted("action:explain")];

        assert!(took_nothing(&wanted, &[]));
        assert!(!took_nothing(&wanted, &[taken("palette")]));
        assert!(!took_nothing(&wanted, &[unassigned("palette")]));
        assert!(!took_nothing(
            &wanted,
            &[taken("palette"), unassigned("action:explain")]
        ));
    }

    /// A desktop that is never going to take the Hotkey is asked a handful of
    /// times over a few minutes, and then left alone.
    ///
    /// The whole of it, not the pauses alone: an asking that is never answered
    /// costs [`ANSWER_WITHIN`] before the pause after it even begins, and it is
    /// the sum that the log promises the user and that a desktop refusing every
    /// asking is going to spend.
    #[test]
    fn the_desktop_stops_being_asked() {
        let mut spent = Duration::ZERO;
        let mut asked = 1;

        while let Some(before) = before_asking_again(asked) {
            assert!(asked < 100, "the asking should not go on forever");
            spent += before + ANSWER_WITHIN;
            asked += 1;
        }

        assert!(
            (Duration::from_secs(120)..=Duration::from_secs(600)).contains(&spent),
            "asking for {spent:?} is neither the few minutes the log promises nor long enough \
             for a desktop watched to need a minute of them"
        );
    }

    /// A desktop that is merely slow is asked again while it is still coming
    /// up, rather than once the user has given up and reached for the tray.
    #[test]
    fn the_first_asking_again_comes_soon() {
        assert!(before_asking_again(1).unwrap() <= Duration::from_secs(2));
    }

    #[test]
    fn a_letter_is_the_letter_it_types() {
        assert_eq!(trigger("Ctrl+Shift+KeyE").as_deref(), Some("CTRL+SHIFT+e"));
    }

    #[test]
    fn a_modifier_and_a_key_that_types_nothing_travel_together() {
        assert_eq!(
            trigger("Ctrl+Shift+Space").as_deref(),
            Some("CTRL+SHIFT+space")
        );
    }

    /// The Hotkey the whole tool opens with. One the portal cannot be told
    /// about is one the user has to go and assign before Demysto opens at all.
    #[test]
    fn the_palettes_own_hotkey_is_one_the_portal_takes() {
        let palette = crate::hotkey::PALETTE;

        assert!(trigger(palette).is_some(), "{palette}");
    }

    #[test]
    fn every_spelling_of_a_modifier_is_one_the_portal_takes() {
        for stated in ["Cmd+KeyE", "Super+KeyE", "Meta+KeyE", "Command+KeyE"] {
            assert_eq!(trigger(stated).as_deref(), Some("SUPER+e"), "{stated}");
        }
    }

    #[test]
    fn a_digit_is_the_digit_it_types() {
        assert_eq!(trigger("Alt+Digit1").as_deref(), Some("ALT+1"));
    }

    #[test]
    fn a_function_key_keeps_its_number() {
        assert_eq!(trigger("F13").as_deref(), Some("F13"));
        assert_eq!(trigger("F24").as_deref(), Some("F24"));
    }

    /// A Hotkey with no modifier at all is one of the keys that type nothing,
    /// and those are exactly the ones the Palette offers to record alone.
    #[test]
    fn a_key_that_needs_no_modifier_travels_alone() {
        for key in demysto_core::keys_that_need_no_modifier() {
            assert!(trigger(key).is_some(), "{key}");
        }
    }

    #[test]
    fn a_key_with_no_name_in_the_specification_states_no_preference() {
        assert_eq!(trigger("Ctrl+Shift+Quux"), None);
        assert_eq!(trigger("Hyper+KeyE"), None);
        assert_eq!(trigger("F25"), None);
        assert_eq!(trigger("KeyEE"), None);
        assert_eq!(trigger(""), None);
    }

    #[test]
    fn an_action_is_never_mistaken_for_the_palette() {
        assert_eq!(action_of(OPENS_THE_PALETTE), None);
        assert_eq!(action_of(&for_action("explain")).unwrap(), "explain");
        // Even one whose own identifier is the Palette's.
        assert_eq!(
            action_of(&for_action(OPENS_THE_PALETTE)).unwrap(),
            OPENS_THE_PALETTE
        );
    }
}
