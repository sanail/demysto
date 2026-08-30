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
//! Only Linux reaches the half of this that talks to the portal. The half that
//! translates a Hotkey into the syntax the portal takes is plain string work,
//! and is compiled and tested everywhere so that the one platform it runs on is
//! not the only place it is checked.

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

    use ashpd::desktop::global_shortcuts::{
        BindShortcutsOptions, GlobalShortcuts, NewShortcut, Shortcut,
    };
    use ashpd::desktop::CreateSessionOptions;
    use futures_util::StreamExt;
    use tauri::async_runtime::{Receiver, Sender};

    use super::Wanted;

    /// What the Hotkeys the portal holds came to, in whole sentences, for the
    /// window that reports every Hotkey Demysto does not answer to.
    ///
    /// Held here rather than answered by [`claim`] because the portal is asked
    /// on a task of its own: binding can put a dialog in front of the user, and
    /// the thread [`claim`] is called on is the thread every window Demysto has
    /// is drawn on. So the report is what the last binding came to, which is
    /// the one from startup by the time anybody opens Settings to read it.
    static REPORT: Mutex<Vec<String>> = Mutex::new(Vec::new());

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
        wanted: Vec<Wanted>,
        pressed: impl Fn(&str) + Send + Sync + 'static,
        noted: impl Fn(&str) + Send + Sync + 'static,
    ) -> Vec<String> {
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
            if let Err(said) = hold(wanted, pressed, &noted, stopped).await {
                report(vec![said], &noted);
            }
        });

        REPORT
            .lock()
            .unwrap_or_else(|held| held.into_inner())
            .clone()
    }

    /// Puts what the portal came to where the window reads it, and where the
    /// log keeps it for a bug report to carry.
    fn report(said: Vec<String>, noted: &impl Fn(&str)) {
        for one in &said {
            noted(one);
        }

        *REPORT.lock().unwrap_or_else(|held| held.into_inner()) = said;
    }

    /// Binds the Hotkeys and stays in the loop that answers them.
    ///
    /// Answers `Err` with the sentence saying so for every way this ends that
    /// leaves the user without a Hotkey, and `Ok` for the one that does not:
    /// being told to let go, which happens only because another asking is
    /// taking this one's place.
    async fn hold(
        wanted: Vec<Wanted>,
        pressed: impl Fn(&str),
        noted: &impl Fn(&str),
        mut stopped: Receiver<()>,
    ) -> Result<(), String> {
        let portal = GlobalShortcuts::new().await.map_err(unreachable)?;
        let session = portal
            .create_session(CreateSessionOptions::default())
            .await
            .map_err(unreachable)?;

        // Subscribed before anything is bound, so that a Hotkey pressed the
        // instant the desktop assigns it is not one nobody was listening for.
        // Pinned because it is waited on again and again, beside the other
        // thing this waits on.
        let mut activated = Box::pin(portal.receive_activated().await.map_err(unreachable)?);

        let asked: Vec<NewShortcut> = wanted.iter().map(asked_for).collect();

        let bound = portal
            .bind_shortcuts(&session, &asked, None, BindShortcutsOptions::default())
            .await
            .map_err(unreachable)?
            .response()
            .map_err(refused)?;

        // Asked before the report is written, because binding can sit in front
        // of the user for as long as they leave the dialog there: another
        // asking may have replaced this one in the meantime, and its report is
        // the one to keep.
        if stopped.try_recv().is_ok() {
            return closing(&session).await;
        }

        report(what_came_of(&wanted, bound.shortcuts()), noted);

        loop {
            tokio::select! {
                activation = activated.next() => match activation {
                    Some(activation) => pressed(activation.shortcut_id()),
                    // The portal stopped speaking, which takes every Hotkey
                    // with it — xdg-desktop-portal restarted, or the session
                    // was closed under Demysto. Reported rather than left as a
                    // Hotkey that has quietly stopped answering, which is the
                    // failure a user cannot tell from a tool that has hung.
                    None => break Err(
                        "The desktop's GlobalShortcuts portal stopped answering, so no Hotkey \
                         answers either. Restarting Demysto asks for them again; the tray menu \
                         reaches everything the Hotkey does in the meantime."
                            .to_owned(),
                    ),
                },
                _ = stopped.recv() => break closing(&session).await,
            }
        }
    }

    /// Gives the session back, which is what gives up the Hotkeys in it.
    ///
    /// Nothing is reported: this only happens because another asking has
    /// replaced this one, and that one's report is the one the window wants.
    async fn closing(session: &ashpd::desktop::Session<GlobalShortcuts>) -> Result<(), String> {
        let _ = session.close().await;

        Ok(())
    }

    /// One Hotkey as the portal is asked for it.
    fn asked_for(wanted: &Wanted) -> NewShortcut {
        NewShortcut::new(&wanted.id, &wanted.description)
            .preferred_trigger(wanted.trigger.as_deref())
    }

    /// What the user is owed about the Hotkeys the portal now holds: the ones
    /// it holds under no combination, and the ones it did not take at all.
    ///
    /// Not a line each for the ones that came out well — the desktop's own
    /// shortcut settings list those, and this is the window's list of what does
    /// not answer.
    fn what_came_of(wanted: &[Wanted], bound: &[Shortcut]) -> Vec<String> {
        wanted
            .iter()
            .filter_map(|wanted| {
                let Some(held) = bound.iter().find(|held| held.id() == wanted.id) else {
                    return Some(format!(
                        "The desktop did not take a Hotkey for {}, so nothing answers to it. Its \
                         keyboard shortcut settings are where Demysto's Hotkeys are assigned.",
                        wanted.description
                    ));
                };

                held.trigger_description().is_empty().then(|| {
                    format!(
                        "The desktop is holding a Hotkey for {} under no combination, so nothing \
                         answers to it yet. Give it one in the desktop's own keyboard shortcut \
                         settings.",
                        wanted.description
                    )
                })
            })
            .collect()
    }

    /// What the user is told when the desktop answered the request for the
    /// Hotkeys with something other than the Hotkeys — dismissing the dialog it
    /// puts up, most often.
    ///
    /// Held apart from [`unreachable`], which is about not reaching a portal at
    /// all: a dialog somebody closed is not a desktop to go and upgrade.
    fn refused(error: ashpd::Error) -> String {
        format!(
            "The desktop did not give Demysto the Hotkeys it asked for: {error}. Nothing answers \
             to one until it does — its keyboard shortcut settings are where they are assigned, \
             and the tray menu reaches everything the Hotkey does."
        )
    }

    /// What the user is told when the portal is not there at all — which is
    /// every Hotkey Demysto has, gone.
    ///
    /// Names where the portal comes from, because on a desktop old enough not
    /// to have one there is nothing to turn on and the answer is an upgrade.
    fn unreachable(error: ashpd::Error) -> String {
        format!(
            "This is a Wayland session, where Demysto has to ask the desktop's GlobalShortcuts \
             portal for a Hotkey — and it could not reach one: {error}. No Hotkey answers. The \
             portal arrives with xdg-desktop-portal, on KDE and on GNOME from version 48. The \
             tray menu reaches everything the Hotkey does."
        )
    }
}

#[cfg(test)]
mod tests {
    //! The translation, which is the half of this that is not I/O — and the
    //! half that decides whether a Wayland user's stated Hotkey reaches the
    //! desktop as the combination they stated.

    use super::*;

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
