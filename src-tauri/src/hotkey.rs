//! The global Hotkeys: the one that opens the Palette, and the one an Action
//! may own.
//!
//! An Action's Hotkey is the path with no Palette in it at all (user story 6):
//! select, press, read. Which is why the whole set is claimed here rather than
//! the Palette's alone — the Hotkeys Demysto answers to are one set, and
//! whether a combination is free is a question only the set can answer.

use std::sync::Mutex;

use demysto_core::{say, DefinedAction, Demysto, Words};
use tauri::plugin::TauriPlugin;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcut, GlobalShortcutExt, Shortcut, ShortcutState};

/// The Hotkey that opens the Palette where the settings state none.
///
/// `Cmd+Shift+Space` on macOS and `Ctrl+Shift+Space` elsewhere: a key away from
/// the one Spotlight and its equivalents already own, and taken by nothing on a
/// stock system.
///
/// One string, which is the value, the sentence and what the window shows.
/// Written the way the user reads it and parsed on the way to being claimed —
/// the parser is not fussy about case and takes `Cmd` and `Ctrl` — because a
/// `Shortcut`'s own `Display` is the parser's dialect, and `shift+super+Space`
/// is not a Hotkey to show anybody.
pub(crate) const PALETTE: &str = if cfg!(target_os = "macos") {
    "Cmd+Shift+Space"
} else {
    "Ctrl+Shift+Space"
};

/// The Palette's Hotkey where the settings state none.
fn built_in_palette() -> Shortcut {
    PALETTE
        .parse()
        .expect("the Palette's own Hotkey should be one Demysto can parse")
}

/// Whether a Hotkey stating one key and no modifier is one to claim.
///
/// A Hotkey is claimed from the whole operating system, so one key on its own
/// answers to that key everywhere the user types. The core holds the few keys
/// that costs nothing — see `demysto_core`'s `hotkey` module for which, and for
/// why the neighbours are not among them.
fn claimable_alone<R: Runtime>(app: &AppHandle<R>, hotkey: &Shortcut) -> bool {
    !hotkey.mods.is_empty()
        || app
            .state::<Demysto>()
            .needs_no_modifier(&hotkey.key.to_string())
}

/// What a Hotkey Demysto has claimed does when it is pressed.
#[derive(Clone)]
enum Bound {
    /// Opens the Palette, or closes it when it is already open.
    Palette,
    /// Runs one Action straight away, by the identifier it is asked for by.
    ///
    /// The identifier rather than the Action: the catalogue is read off the
    /// disk at every Run, and a copy of an Action held here would be the one
    /// thing in Demysto that could go stale.
    Action(String),
}

/// One Hotkey Demysto has claimed, and what holds it.
struct Claim {
    hotkey: Shortcut,
    bound: Bound,
    /// What the report calls whatever holds it, where something else asks for
    /// the same combination.
    holder: String,
}

/// Every Hotkey Demysto currently answers to.
///
/// A `static` for the reason `palette::OPENING` is one: single instance is
/// enforced, so there is one set of Hotkeys per machine. A `Vec` rather than a
/// map because it is a handful of entries and this is walked on a keypress, not
/// in a loop.
static CLAIMED: Mutex<Vec<Claim>> = Mutex::new(Vec::new());

/// Held for the whole of [`claim`], so that two of them cannot interleave.
///
/// Separate from [`CLAIMED`], and never taken by the handler: claiming waits on
/// the thread the windows are drawn on, which is the thread a Hotkey arrives on,
/// so a lock a keypress can take must not be one this holds across that wait.
static CLAIMING: Mutex<()> = Mutex::new(());

pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, shortcut, event| {
            // On release rather than press: the Capture that follows sends a
            // copy keystroke, and it must not land while the modifiers of this
            // very Hotkey are still held down.
            if event.state() != ShortcutState::Released {
                return;
            }

            // Both paths take themselves off the thread that draws every window
            // Demysto has, which is what a Capture waiting on another
            // application needs and what this handler cannot do for them.
            match bound(shortcut) {
                Some(Bound::Palette) => crate::palette::toggle(app),
                Some(Bound::Action(id)) => crate::result::straight_to(app, id),
                None => {}
            }
        })
        .build()
}

/// What the Hotkey just pressed is bound to, or `None` for one Demysto has
/// given up since it was claimed.
fn bound(pressed: &Shortcut) -> Option<Bound> {
    CLAIMED
        .lock()
        .unwrap()
        .iter()
        .find(|claim| &claim.hotkey == pressed)
        .map(|claim| claim.bound.clone())
}

/// Claims every Hotkey Demysto answers to, and answers with the ones it could
/// not, in whole sentences.
///
/// The whole set every time rather than the difference: it is a few
/// registrations, and a Hotkey an Action has just given up has to stop
/// answering as surely as a new one has to start.
///
/// Claiming is also what decides whether a combination is free, rather than a
/// check made when the Action is saved. Two Actions can come to hold the same
/// Hotkey without anything being saved at all — an Action arrives in that
/// directory as a file somebody sent, which is the whole point of one file each
/// — and the operating system is the only authority on whether another
/// application got there first.
///
/// `palette` is the Hotkey the settings state for the Palette, `None` for the
/// one Demysto comes with. It is claimed first, which means changing it can take
/// a Hotkey away from an Action stating the same combination — the Action is
/// told so in the report, and the window shows the report the moment the
/// settings are saved.
pub fn claim<R: Runtime>(
    app: &AppHandle<R>,
    palette: Option<&str>,
    actions: &[DefinedAction],
) -> Vec<String> {
    // Wayland lets no application claim a Hotkey from the display server, so
    // there the whole set is asked of the GlobalShortcuts portal instead and
    // none of what follows applies — the desktop, not Demysto, decides which
    // combination each one answers to (ADR-0003).
    #[cfg(target_os = "linux")]
    if demysto_core::wayland_session() {
        return through_the_portal(app, palette, actions);
    }

    // One at a time: two of these interleaving would each give up what the
    // other had just taken, and leave the set Demysto believes it holds
    // describing neither.
    let _claiming = CLAIMING.lock().unwrap_or_else(|held| held.into_inner());

    let hotkeys = app.global_shortcut();
    let demysto = app.state::<Demysto>();
    let words = demysto.words();
    let mut claimed = Vec::new();
    let mut unclaimed = Vec::new();

    // Everything is given up first, and the new set is put together beside
    // [`CLAIMED`] rather than in it. Registering waits on the thread the windows
    // are drawn on, and that is the thread a Hotkey arrives on: holding the lock
    // across the wait would let one press take a lock this is waiting to
    // release. Nothing answers to anything in between, so the set nobody can
    // reach is never wrong.
    let _ = hotkeys.unregister_all();

    // The Palette's first, so that an Action stating it finds it taken rather
    // than taking it: the Hotkey the whole tool opens with is not something a
    // stray Action file gets to quietly take.
    unclaimed.append(&mut palettes(app, hotkeys, &mut claimed, palette, &words));

    for action in actions {
        let Some(stated) = action.hotkey.as_deref() else {
            continue;
        };

        if let Err(said) = claiming(app, hotkeys, &mut claimed, action, stated, &words) {
            unclaimed.push(said);
        }
    }

    *CLAIMED.lock().unwrap() = claimed;

    unclaimed
}

/// Claims the Palette's Hotkey: the one the settings state, or the one Demysto
/// comes with where they state none — or where the one they state cannot be had.
///
/// Every way the stated one can fail falls back rather than leaving the Palette
/// with no Hotkey at all, and every sentence it produces names what is answering
/// instead. A Palette somebody cannot open is the tool not starting, and being
/// told which key opens it is the difference between a setting that went wrong
/// and a tool that appears to be broken.
fn palettes<R: Runtime>(
    app: &AppHandle<R>,
    hotkeys: &GlobalShortcut<R>,
    claimed: &mut Vec<Claim>,
    stated: Option<&str>,
    words: &Words,
) -> Vec<String> {
    let mut said = Vec::new();

    // The Hotkey that was registered, not the one that was asked for: on the
    // way through a fallback the two differ, and a Claim holding the one nobody
    // registered would never match the key actually pressed — the Palette would
    // stop opening while the report said everything was well.
    let claim = |claimed: &mut Vec<Claim>, hotkey| {
        claimed.push(Claim {
            hotkey,
            bound: Bound::Palette,
            holder: words.text("hotkey-palette-holder"),
        })
    };

    if let Some(stated) = stated {
        match wanted(app, hotkeys, stated, words) {
            Ok(hotkey) => {
                claim(claimed, hotkey);
                return said;
            }
            Err(why) => said.push(say!(
                words,
                "hotkey-palette-fell-back",
                "why" = why,
                "hotkey" = PALETTE
            )),
        }
    }

    match hotkeys.register(built_in_palette()) {
        Ok(()) => claim(claimed, built_in_palette()),
        Err(error) => said.push(say!(
            words,
            "hotkey-palette-unclaimable",
            "hotkey" = PALETTE,
            "detail" = error.to_string()
        )),
    }

    said
}

/// Claims the Hotkey the settings state for the Palette, or says why not.
fn wanted<R: Runtime>(
    app: &AppHandle<R>,
    hotkeys: &GlobalShortcut<R>,
    stated: &str,
    words: &Words,
) -> Result<Shortcut, String> {
    let Ok(hotkey) = stated.parse::<Shortcut>() else {
        return Err(say!(
            words,
            "hotkey-palette-not-a-combination",
            "hotkey" = stated.to_owned()
        ));
    };

    if !claimable_alone(app, &hotkey) {
        return Err(say!(
            words,
            "hotkey-palette-types-something",
            "hotkey" = stated.to_owned()
        ));
    }

    hotkeys.register(hotkey).map(|()| hotkey).map_err(|error| {
        say!(
            words,
            "hotkey-palette-refused",
            "hotkey" = stated.to_owned(),
            "detail" = error.to_string()
        )
    })
}

/// Claims one Action's Hotkey, or says in a whole sentence why it could not be.
fn claiming<R: Runtime>(
    app: &AppHandle<R>,
    hotkeys: &GlobalShortcut<R>,
    claimed: &mut Vec<Claim>,
    action: &DefinedAction,
    stated: &str,
    words: &Words,
) -> Result<(), String> {
    let name = &action.name;

    let Ok(hotkey) = stated.parse::<Shortcut>() else {
        return Err(say!(
            words,
            "hotkey-action-not-a-combination",
            "action" = name.clone(),
            "hotkey" = stated.to_owned()
        ));
    };

    if !claimable_alone(app, &hotkey) {
        return Err(say!(
            words,
            "hotkey-action-types-something",
            "action" = name.clone(),
            "hotkey" = stated.to_owned()
        ));
    }

    // Asked here rather than left to the registration, which refuses a second
    // claim on the same combination without being able to say what already has
    // it — and what already has it is the only part of this worth telling
    // somebody.
    if let Some(held) = claimed.iter().find(|claim| claim.hotkey == hotkey) {
        return Err(say!(
            words,
            "hotkey-action-already-held",
            "action" = name.clone(),
            "hotkey" = stated.to_owned(),
            "holder" = held.holder.clone()
        ));
    }

    if let Err(error) = hotkeys.register(hotkey) {
        return Err(say!(
            words,
            "hotkey-action-refused",
            "action" = name.clone(),
            "hotkey" = stated.to_owned(),
            "detail" = error.to_string()
        ));
    }

    claimed.push(Claim {
        hotkey,
        bound: Bound::Action(action.id.clone()),
        holder: name.clone(),
    });

    Ok(())
}

/// Asks the portal for the same set the display server is asked for anywhere
/// else, and answers with what could not be had, in the same whole sentences.
///
/// The Hotkeys stated in the settings and by the Actions travel as preferences
/// rather than as claims: the portal shows the user what Demysto asked for and
/// the desktop assigns the combination. Which is why nothing here refuses a
/// combination the way [`claiming`] does — there is no set to collide within,
/// and a Hotkey that types something is the desktop's business to allow or not.
#[cfg(target_os = "linux")]
fn through_the_portal<R: Runtime>(
    app: &AppHandle<R>,
    palette: Option<&str>,
    actions: &[DefinedAction],
) -> Vec<String> {
    use crate::portal::Wanted;

    let demysto = app.state::<Demysto>();
    let words = demysto.words();

    let mut wanted = vec![Wanted {
        id: crate::portal::OPENS_THE_PALETTE.to_owned(),
        description: words.text("portal-palette-description"),
        trigger: crate::portal::trigger(palette.unwrap_or(PALETTE)),
    }];

    wanted.extend(actions.iter().filter_map(|action| {
        Some(Wanted {
            id: crate::portal::for_action(&action.id),
            description: say!(
                &words,
                "portal-action-description",
                "action" = action.name.clone()
            ),
            trigger: crate::portal::trigger(action.hotkey.as_deref()?),
        })
    }));

    // Off the guard already held rather than asked of the facade again: a
    // second `read` on the same lock, taken while the first is still held,
    // deadlocks against a `write` that queued between them — and saving the
    // settings is exactly that write.
    let interface = words.interface();

    let answering = app.clone();
    let noting = app.clone();

    crate::portal::claim(
        // The identifier comes from the application's own configuration rather
        // than being written out here: the portal finds the installed desktop
        // entry by it, to show the user a name and an icon, and those two names
        // must not drift apart.
        app.config().identifier.clone(),
        wanted,
        interface,
        move |pressed| {
            // The same two paths the handler above takes, off the thread that
            // draws every window for the same reason — except that this one
            // arrives on a task of the portal's rather than on a keypress, and
            // taking a Capture off it matters just as much.
            match crate::portal::action_of(pressed) {
                None => crate::palette::toggle(&answering),
                Some(action) => crate::result::straight_to(&answering, action.to_owned()),
            }
        },
        move |said| noting.state::<Demysto>().note(said),
    )
}

#[cfg(test)]
mod tests {
    //! The one thing in this module worth testing without a desktop: that the
    //! two halves of the list agree.
    //!
    //! `demysto_core` decides which keys a Hotkey may be on its own, and names
    //! them the way the W3C does — which is what a browser reports a keypress as.
    //! This module claims them through a parser that has its own idea of what a
    //! key is called. Nothing but this catches a name the core states and the
    //! parser has never heard of, and what it would cost is a Hotkey the window
    //! offers to record and the claim then quietly refuses.

    use super::*;

    #[test]
    fn every_key_the_core_allows_alone_is_one_the_parser_knows() {
        for key in demysto_core::keys_that_need_no_modifier() {
            let hotkey: Shortcut = key
                .parse()
                .unwrap_or_else(|_| panic!("{key} should be a Hotkey the parser reads"));

            assert!(hotkey.mods.is_empty(), "{key}");
            assert_eq!(hotkey.key.to_string(), key);
        }
    }

    #[test]
    fn the_palettes_own_hotkey_is_one_the_parser_knows() {
        // `built_in_palette` unwraps this, and it is the Hotkey the whole tool
        // opens with.
        let hotkey = built_in_palette();

        assert!(!hotkey.mods.is_empty());
    }
}
