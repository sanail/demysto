//! What Demysto is doing in the dock, which is not the same at every moment.
//!
//! A resident utility has no business there while it is only waiting: the
//! Palette floats over what the user is reading and is gone again in a
//! keystroke, and an icon bouncing into the dock for it would be the tool
//! announcing itself for no reason. A Conversation is the opposite — somewhere
//! the user reads, copies from, and comes back to — and coming back to it means
//! the window switcher, which on macOS lists only applications the dock lists.
//!
//! So the activation policy follows the windows rather than being settled at
//! startup (the spec's *Shell and platform*): accessory while the Palette is
//! all there is, regular while a Conversation or Settings is open, accessory
//! again when the last of them closes.
//!
//! Only macOS has such a policy. Everywhere else the taskbar answers the same
//! question from the windows themselves, which is why the Palette declares
//! `skipTaskbar` in `tauri.conf.json` and the other two do not.

use tauri::{AppHandle, Manager, Runtime};

/// The windows whose presence on screen is what puts Demysto in the dock.
///
/// The Palette is deliberately not among them: it is a panel that takes the
/// keyboard without taking activation, and nobody switches back to it — they
/// press the Hotkey. The first-run flow is, and is the first window Demysto
/// ever shows anybody: one they cannot switch back to would be a flow lost
/// behind whatever they were reading.
const SWITCHED_BACK_TO: [&str; 3] = [
    crate::result::LABEL,
    crate::settings::LABEL,
    crate::welcome::LABEL,
];

/// What is about to happen to the window this is being told about.
///
/// Named rather than asked about, because neither state reads as what it is
/// about to be: a window on its way up is still hidden when the policy has to
/// be right for it — an accessory application's window is in no switcher, so it
/// has to be shown as a regular application's — and one being put away is still
/// on screen while the event announcing it is being handled.
#[derive(Debug, Clone, Copy)]
pub enum Change<'a> {
    Showing(&'a str),
    Hiding(&'a str),
}

/// Puts Demysto in the dock while there is a window to switch back to, and
/// takes it out again when there is none.
pub fn follows_the_windows<R: Runtime>(app: &AppHandle<R>, change: Change<'_>) {
    apply(app, belongs_in_the_dock(on_screen(app), change));
}

/// Which of the windows that count are on screen right now.
fn on_screen<R: Runtime>(app: &AppHandle<R>) -> Vec<(&'static str, bool)> {
    SWITCHED_BACK_TO
        .into_iter()
        .map(|label| {
            let showing = app
                .get_webview_window(label)
                .is_some_and(|window| window.is_visible().unwrap_or(false));

            (label, showing)
        })
        .collect()
}

/// Whether Demysto belongs in the dock, given what is on screen and what is
/// about to change about it.
///
/// A window that is not one of [`SWITCHED_BACK_TO`] — the Palette — matches
/// nothing here, so showing or hiding it leaves the answer to the other two.
fn belongs_in_the_dock<'a>(
    on_screen: impl IntoIterator<Item = (&'a str, bool)>,
    change: Change<'_>,
) -> bool {
    on_screen.into_iter().any(|(label, showing)| match change {
        Change::Showing(arriving) => showing || label == arriving,
        Change::Hiding(going) => showing && label != going,
    })
}

/// Says it to macOS.
///
/// Called from wherever a window is shown or hidden, which is a Run's own
/// thread as often as the main one. `set_activation_policy` runs the change
/// straight away when it is already on the thread AppKit insists on and queues
/// it behind the caller's other window messages when it is not — which is the
/// same path `show` takes, so the two keep the order they were asked in.
#[cfg(target_os = "macos")]
fn apply<R: Runtime>(app: &AppHandle<R>, wanted: bool) {
    let policy = match wanted {
        true => tauri::ActivationPolicy::Regular,
        false => tauri::ActivationPolicy::Accessory,
    };

    let _ = app.set_activation_policy(policy);
}

/// Everywhere else this is the window manager's own business; see the module's
/// note.
#[cfg(not(target_os = "macos"))]
fn apply<R: Runtime>(_app: &AppHandle<R>, _wanted: bool) {}

#[cfg(test)]
mod tests {
    //! The one part of this worth testing without a desktop: which windows put
    //! Demysto in the dock, and which do not.

    use super::*;

    const PALETTE: &str = "palette";
    const CONVERSATION: &str = "result";
    const SETTINGS: &str = "settings";
    const WELCOME: &str = "welcome";

    /// A desktop with none of the windows that count on screen, which is
    /// Demysto waiting: the Palette and nothing else.
    const NOTHING: [(&str, bool); 3] = [(CONVERSATION, false), (SETTINGS, false), (WELCOME, false)];

    #[test]
    fn a_conversation_on_its_way_up_puts_demysto_in_the_dock() {
        // The window is still hidden at this point — it is shown next, and it
        // has to be shown as a regular application's window or it is in no
        // switcher (user story 50).
        assert!(belongs_in_the_dock(NOTHING, Change::Showing(CONVERSATION)));
    }

    #[test]
    fn settings_on_its_way_up_does_too() {
        assert!(belongs_in_the_dock(NOTHING, Change::Showing(SETTINGS)));
    }

    /// The first window a fresh installation ever sees, and the one nobody has
    /// learned the Hotkey for yet.
    #[test]
    fn the_first_run_flow_does_too() {
        assert!(belongs_in_the_dock(NOTHING, Change::Showing(WELCOME)));
    }

    #[test]
    fn a_conversation_already_on_screen_keeps_demysto_there() {
        assert!(belongs_in_the_dock(
            [(CONVERSATION, true), (SETTINGS, false)],
            Change::Showing(SETTINGS)
        ));
    }

    #[test]
    fn nothing_but_the_palette_keeps_demysto_out_of_it() {
        // The Palette is in neither list: it floats over what the user is
        // reading and nobody switches back to it.
        assert!(!belongs_in_the_dock(NOTHING, Change::Showing(PALETTE)));
        assert!(!belongs_in_the_dock(NOTHING, Change::Hiding(PALETTE)));
    }

    #[test]
    fn the_window_being_put_away_no_longer_counts() {
        // The close is handled while the window is still on screen: asking
        // whether it is visible would answer yes and leave Demysto in the dock
        // with nothing to switch back to.
        assert!(!belongs_in_the_dock(
            [(CONVERSATION, true), (SETTINGS, false), (WELCOME, false)],
            Change::Hiding(CONVERSATION)
        ));
    }

    #[test]
    fn closing_one_of_two_leaves_demysto_in_the_dock_for_the_other() {
        assert!(belongs_in_the_dock(
            [(CONVERSATION, true), (SETTINGS, true), (WELCOME, false)],
            Change::Hiding(SETTINGS)
        ));
    }
}
