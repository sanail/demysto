//! The global Hotkeys: the one that opens the Palette, and the one an Action
//! may own.
//!
//! An Action's Hotkey is the path with no Palette in it at all (user story 6):
//! select, press, read. Which is why the whole set is claimed here rather than
//! the Palette's alone — the Hotkeys Demysto answers to are one set, and
//! whether a combination is free is a question only the set can answer.

use std::sync::Mutex;

use demysto_core::DefinedAction;
use tauri::plugin::TauriPlugin;
use tauri::{AppHandle, Runtime};
use tauri_plugin_global_shortcut::{
    Code, GlobalShortcut, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
};

/// The Palette's Hotkey as the user reads it, which is how the report below
/// names it. Written out rather than composed from [`for_palette`], whose own
/// `Display` is the one the parser speaks — `shift+super+Space` is not a
/// sentence to show anybody. The two are one line apart so that they stay one
/// Hotkey.
const PALETTE: &str = if cfg!(target_os = "macos") {
    "Cmd+Shift+Space"
} else {
    "Ctrl+Shift+Space"
};

/// The Hotkey that opens the Palette.
///
/// `Cmd+Shift+Space` on macOS and `Ctrl+Shift+Space` elsewhere: a key away from
/// the one Spotlight and its equivalents already own, and taken by nothing on a
/// stock system.
fn for_palette() -> Shortcut {
    let platform = if cfg!(target_os = "macos") {
        Modifiers::SUPER
    } else {
        Modifiers::CONTROL
    };

    Shortcut::new(Some(platform | Modifiers::SHIFT), Code::Space)
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
pub fn claim<R: Runtime>(app: &AppHandle<R>, actions: &[DefinedAction]) -> Vec<String> {
    // One at a time: two of these interleaving would each give up what the
    // other had just taken, and leave the set Demysto believes it holds
    // describing neither.
    let _claiming = CLAIMING.lock().unwrap_or_else(|held| held.into_inner());

    let hotkeys = app.global_shortcut();
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
    match hotkeys.register(for_palette()) {
        Ok(()) => claimed.push(Claim {
            hotkey: for_palette(),
            bound: Bound::Palette,
            holder: "the Palette".to_owned(),
        }),
        Err(error) => unclaimed.push(format!(
            "Demysto could not claim {PALETTE}, the Hotkey that opens the Palette: {error}. \
             Another application may already have it. The tray menu reaches everything the \
             Hotkey does."
        )),
    }

    for action in actions {
        let Some(stated) = action.hotkey.as_deref() else {
            continue;
        };

        if let Err(said) = claiming(hotkeys, &mut claimed, action, stated) {
            unclaimed.push(said);
        }
    }

    *CLAIMED.lock().unwrap() = claimed;

    unclaimed
}

/// Claims one Action's Hotkey, or says in a whole sentence why it could not be.
fn claiming<R: Runtime>(
    hotkeys: &GlobalShortcut<R>,
    claimed: &mut Vec<Claim>,
    action: &DefinedAction,
    stated: &str,
) -> Result<(), String> {
    let name = &action.name;

    let Ok(hotkey) = stated.parse::<Shortcut>() else {
        return Err(format!(
            "{name} states the Hotkey \"{stated}\", which is not a combination Demysto \
             understands. A Hotkey is its modifiers and then one key, written like \
             \"Ctrl+Shift+E\"."
        ));
    };

    // A Hotkey is global, so one with no modifier answers to that key
    // everywhere the user types — which is a way to lose the letter R rather
    // than to bind an Action. The window that records one will not offer this;
    // a file written by hand can still ask for it.
    if hotkey.mods.is_empty() {
        return Err(format!(
            "{name} states the Hotkey \"{stated}\", which is one key on its own. A Hotkey is \
             claimed everywhere, so it needs at least one modifier — otherwise that key would \
             stop reaching whatever you were typing into."
        ));
    }

    // Asked here rather than left to the registration, which refuses a second
    // claim on the same combination without being able to say what already has
    // it — and what already has it is the only part of this worth telling
    // somebody.
    if let Some(held) = claimed.iter().find(|claim| claim.hotkey == hotkey) {
        return Err(format!(
            "{name} states the Hotkey \"{stated}\", and {} already has it. Only {} answers \
             to it; give {name} another.",
            held.holder, held.holder
        ));
    }

    if let Err(error) = hotkeys.register(hotkey) {
        return Err(format!(
            "{name} states the Hotkey \"{stated}\", and Demysto could not claim it: {error}. \
             Another application may already have it."
        ));
    }

    claimed.push(Claim {
        hotkey,
        bound: Bound::Action(action.id.clone()),
        holder: name.clone(),
    });

    Ok(())
}
