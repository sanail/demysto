//! The bridge between the frontend and [`demysto_core`].
//!
//! Every command here is a thin adapter: it borrows the facade out of Tauri's
//! managed state, calls one method on it, and maps the result. Logic that is
//! worth testing belongs in `demysto-core`, behind the single test seam, not here.

use std::collections::BTreeMap;

use demysto_core::{
    Action, ActionEdit, ActionError, CaptureOutcome, ConfigError, Conversation, Demysto, Edit,
    Preset, ProviderEdit, RunError, Settings, Status, Summary,
};
use tauri::ipc::Channel;
use tauri::{AppHandle, Emitter, Manager, Runtime, State, WebviewWindow};

/// The catalogue as the window that writes Actions sees it: what the core holds,
/// plus what came of claiming the Hotkeys the Actions in it state.
///
/// The two travel together because they are read together — that window is where
/// a Hotkey is bound, so it is where one that could not be claimed has to be
/// reported. Flattened, so that the window sees one shape with one more field on
/// it rather than a catalogue inside a wrapper.
#[derive(serde::Serialize)]
pub struct Catalogue {
    #[serde(flatten)]
    defined: demysto_core::Catalogue,
    /// The stated Hotkeys Demysto does not answer to, in whole sentences —
    /// alongside `unreadable`, which the catalogue carries for the same reason.
    unclaimed: Vec<String>,
}

/// Claims the Hotkeys a catalogue states, puts its Actions in the tray menu,
/// and hands the catalogue and what could not be claimed to the window.
///
/// The three commands that produce a catalogue all go through here, so that the
/// Hotkeys follow the directory rather than the last save: an Action that
/// arrived as a file somebody sent answers to its Hotkey once the window that
/// writes Actions has been opened, without a restart. The Palette's own stated
/// Hotkey is read here for the same reason, so that one edited by hand follows
/// the file exactly as an Action's does.
///
/// The Palette's own `actions` deliberately does not claim, though it reads the
/// same directory: it runs on the Hotkey path itself, and giving up every Hotkey
/// to take them again is not something to do while one is being answered. This
/// is reached only from the window that writes Actions, so the settings file it
/// reads is not read on any path a keypress takes.
fn catalogued<R: Runtime>(app: &AppHandle<R>, defined: demysto_core::Catalogue) -> Catalogue {
    let palette = app.state::<Demysto>().palette_hotkey();
    let unclaimed = crate::hotkey::claim(app, palette.as_deref(), &defined.actions);

    // The tray menu lists the Actions too, and for the same reason it is
    // brought up to date here: it is the path for somebody who has not learned
    // the Hotkey, and an Action missing from it is an Action they cannot reach.
    crate::tray::follows_the_catalogue(app, &defined.actions);

    Catalogue { defined, unclaimed }
}

/// What the window that records a Hotkey has to know and cannot work out.
#[derive(serde::Serialize)]
pub struct Hotkeys {
    /// The Palette's Hotkey where the settings state none, as the user reads it.
    palette_default: &'static str,
    /// The keys a Hotkey may be on its own, because they type nothing.
    no_modifier_needed: Vec<&'static str>,
}

/// The two things about Hotkeys the window cannot answer for itself.
#[tauri::command]
pub fn hotkeys(demysto: State<'_, Demysto>) -> Hotkeys {
    Hotkeys {
        palette_default: crate::hotkey::PALETTE,
        no_modifier_needed: demysto.keys_that_need_no_modifier(),
    }
}

#[tauri::command]
pub fn status(demysto: State<'_, Demysto>) -> Status {
    demysto.status()
}

/// Emitted with the tag of the language Demysto now speaks, so that every
/// window already on screen redraws itself in it rather than waiting to be
/// reopened (user story 59).
const LANGUAGE_EVENT: &str = "language://spoken";

/// The language the windows are to draw themselves in.
///
/// The tag alone, because the catalogue itself is on both sides: the windows
/// import the same `i18n/*.ftl` files this crate is compiled against, so what
/// crosses the channel is which of them to read rather than what it says.
/// Asked at startup by every window, and again whenever a save changes it.
#[tauri::command]
pub fn language(demysto: State<'_, Demysto>) -> &'static str {
    demysto.language().tag()
}

/// What the last Capture produced, for a Palette that mounted after it.
#[tauri::command]
pub fn last_capture(demysto: State<'_, Demysto>) -> Option<CaptureOutcome> {
    demysto.last_capture()
}

/// The Actions the Palette lists for the last Capture.
#[tauri::command]
pub fn actions(demysto: State<'_, Demysto>) -> Vec<Action> {
    demysto.actions()
}

/// Runs one Action over the last Capture, with what the Palette collected for
/// the Parameters it declares.
///
/// The one command that does not call the facade directly: which window the
/// answer appears in, and which thread the waiting happens on, are Tauri's
/// business rather than the core's. It returns as soon as the Run has somewhere
/// to happen, and the answer reaches the result window as an event.
#[tauri::command]
pub fn run<R: Runtime>(app: AppHandle<R>, action: String, parameters: BTreeMap<String, String>) {
    crate::result::run(&app, action, parameters);
}

/// Asks a follow-up in the Conversation the window is showing.
///
/// Returns for the same reason [`run`] does, and through the same path: the
/// answer reaches the window as it arrives rather than as this call's result.
#[tauri::command]
pub fn follow_up<R: Runtime>(app: AppHandle<R>, question: String) {
    crate::result::follow_up(&app, question);
}

/// Stops the Turn under way, keeping what has already arrived.
#[tauri::command]
pub fn stop(demysto: State<'_, Demysto>) {
    demysto.stop();
}

/// Asks the last Turn of the Conversation on screen again, optionally of
/// another Model — the retry and the Model switch a failed Turn is offered.
///
/// Returns for the reason [`run`] does: the answer reaches the window as it
/// arrives rather than as this call's result.
#[tauri::command]
pub fn retry<R: Runtime>(app: AppHandle<R>, model: Option<String>) {
    crate::result::retry(&app, model);
}

/// Asks the Model for the rest of an answer that broke off part-way.
#[tauri::command]
pub fn continue_answer<R: Runtime>(app: AppHandle<R>) {
    crate::result::continue_answer(&app);
}

/// Every Model configured, by the name one is switched to, so that the
/// Conversation window can offer somewhere else to ask.
#[tauri::command]
pub fn models(demysto: State<'_, Demysto>) -> Vec<String> {
    demysto.models()
}

/// Brings Settings up, at one Provider where one is named — which is how a
/// refused key is fixed from where it is reported.
#[tauri::command]
pub fn open_settings<R: Runtime>(app: AppHandle<R>, provider: Option<String>) {
    crate::settings::reveal_at(&app, provider);
}

/// Opens the settings pane where the Accessibility permission is granted, which
/// is how a Capture the system refused is fixed from where it is reported.
#[tauri::command]
pub fn open_accessibility(demysto: State<'_, Demysto>) -> Result<(), String> {
    crate::accessibility::reveal(&demysto.words())
}

/// Whether this desktop gates a Capture behind a permission at all, so that the
/// first-run flow shows the step about one only where there is one.
#[tauri::command]
pub fn accessibility_asked_for() -> bool {
    crate::accessibility::gates_the_capture()
}

/// Whether Demysto is in the login items now, which is what the flow's question
/// about them starts at.
#[tauri::command]
pub fn autostart<R: Runtime>(app: AppHandle<R>) -> bool {
    crate::autostart::enabled(&app)
}

/// Puts Demysto into the login items, or takes it out — the answer to the one
/// question the first-run flow asks about them (user story 52).
#[tauri::command]
pub fn set_autostart<R: Runtime>(app: AppHandle<R>, wanted: bool) -> Result<(), String> {
    crate::autostart::set(&app, wanted, &app.state::<Demysto>().words())
}

/// Opens the folder Demysto writes its logs in, so that a bug report can carry
/// them.
#[tauri::command]
pub fn open_logs(demysto: State<'_, Demysto>) -> Result<(), String> {
    crate::folder::open(&demysto.logs_dir(), &demysto.words())
}

/// The newer version the check on the way up found, without asking the manifest
/// again — which is what Settings shows the moment it is opened.
#[tauri::command]
pub fn update_offered<R: Runtime>(app: AppHandle<R>) -> Option<String> {
    crate::update::offered(&app)
}

/// Whether a newer Demysto exists, asked of the release manifest — the version
/// where there is one, `null` where this is already the newest.
///
/// Asked here as well as on the way up, because a user who has just read that a
/// release exists should not have to wait for the next launch to be offered it.
#[tauri::command]
pub async fn look_for_update<R: Runtime>(app: AppHandle<R>) -> Result<Option<String>, String> {
    crate::update::look(&app).await
}

/// Takes the update the last check found.
///
/// Answers only by failing: where it works, this process is replaced by the
/// version it installed and there is nobody left to answer.
#[tauri::command]
pub async fn install_update<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    crate::update::take(&app).await
}

/// The Conversation the window is showing, for one that mounted after the Turn
/// it is showing began.
#[tauri::command]
pub fn conversation(demysto: State<'_, Demysto>) -> Option<Conversation> {
    demysto.conversation()
}

/// The whole of what the Conversation on screen is about, for the window whose
/// quotation of it the user has asked to expand.
#[tauri::command]
pub fn selection(demysto: State<'_, Demysto>) -> Option<String> {
    demysto.selection()
}

/// This session's Conversations, newest first, for the list the window offers.
#[tauri::command]
pub fn conversations(demysto: State<'_, Demysto>) -> Vec<Summary> {
    demysto.conversations()
}

/// Puts an earlier Conversation on screen.
#[tauri::command]
pub fn show_conversation(demysto: State<'_, Demysto>, id: u64) -> Option<Conversation> {
    demysto.show_conversation(id)
}

/// Says where a Conversation window wants an answer sent as it arrives.
#[tauri::command]
pub fn show_answers_on(channel: Channel<crate::result::Handover>) {
    crate::result::show_answers_on(channel);
}

/// Every Action there is, with everything about it, for the window that writes
/// them — and, because reading the directory is also when the Hotkeys in it are
/// claimed, what could not be claimed.
#[tauri::command]
pub fn catalogue<R: Runtime>(app: AppHandle<R>) -> Catalogue {
    let defined = app.state::<Demysto>().catalogue();

    catalogued(&app, defined)
}

/// Writes one Action, and answers with the catalogue as the directory then
/// holds it.
#[tauri::command]
pub fn save_action<R: Runtime>(
    app: AppHandle<R>,
    edit: ActionEdit,
) -> Result<Catalogue, ActionError> {
    let defined = app.state::<Demysto>().save_action(&edit)?;

    Ok(catalogued(&app, defined))
}

/// Deletes an Action of the user's own, or removes the Override over a built-in
/// and leaves the built-in.
#[tauri::command]
pub fn delete_action<R: Runtime>(app: AppHandle<R>, id: String) -> Result<Catalogue, ActionError> {
    let defined = app.state::<Demysto>().delete_action(&id)?;

    Ok(catalogued(&app, defined))
}

/// The settings as the file now holds them, for the window that edits it.
#[tauri::command]
pub fn settings(demysto: State<'_, Demysto>) -> Result<Settings, ConfigError> {
    demysto.settings()
}

/// Writes what the window edited, and answers with the settings as the file
/// then holds them.
///
/// Off the drawing thread like the two below it: a save puts every key typed
/// into the window to its Provider before writing anything.
///
/// A Palette Hotkey saved here is not claimed here: the window asks for the
/// catalogue afterwards, and claiming is what reading the catalogue does.
#[tauri::command]
pub async fn save_settings<R: Runtime>(
    app: AppHandle<R>,
    edit: Edit,
) -> Result<Settings, ConfigError> {
    let saved = {
        let app = app.clone();
        waiting(move || app.state::<Demysto>().save_settings(&edit)).await
    }?;

    // Told to every window, not only the one that saved: the language is the
    // one setting that changes what a window says rather than what it does, and
    // a Conversation left open behind Settings would otherwise go on speaking
    // the language nobody chose any more. The tray menu is not a window and
    // redraws nowhere, so it is rebuilt where the catalogue is read — see
    // `catalogued`, which the window asks for straight after a save.
    let _ = app.emit(LANGUAGE_EVENT, app.state::<Demysto>().language().tag());

    // The two native surfaces no webview redraws, put back beside the event
    // that redraws the rest: the window's own title, and — on macOS — the menu
    // bar. The tray menu is the third, and is rebuilt where the catalogue is
    // read (see `catalogued`, which the window asks for straight after a save).
    crate::settings::names_itself(&app);

    #[cfg(target_os = "macos")]
    let _ = crate::menu::build(&app);

    Ok(saved)
}

/// The services Demysto knows the conventions of, for the window to offer.
#[tauri::command]
pub fn presets(demysto: State<'_, Demysto>) -> Vec<Preset> {
    demysto.presets()
}

/// The Models a Provider says it offers, asked of the Provider as the window
/// has it now rather than as the file holds it.
#[tauri::command]
pub async fn provider_models<R: Runtime>(
    app: AppHandle<R>,
    provider: ProviderEdit,
) -> Result<Vec<String>, RunError> {
    waiting(move || app.state::<Demysto>().models_offered_by(&provider)).await
}

/// Whether a Provider accepts a key, asked of the Provider itself.
#[tauri::command]
pub async fn verify_provider<R: Runtime>(
    app: AppHandle<R>,
    provider: ProviderEdit,
    model: String,
) -> Result<(), RunError> {
    waiting(move || app.state::<Demysto>().verify(&provider, &model)).await
}

/// Runs something that waits on a Provider away from the thread that draws
/// every window Demysto has — which is the same reason a Run is spawned rather
/// than awaited, and the same distance across the network.
async fn waiting<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> T {
    tauri::async_runtime::spawn_blocking(work)
        .await
        .expect("asking a Provider should not have panicked")
}

/// Hides the window this was invoked from, which is what Escape asks for in
/// all of them.
///
/// The dock is told for the reason the close handler tells it: Escape out of
/// the last Conversation is as ordinary a way to put a window away as the close
/// button, and either one leaving Demysto in the dock would leave it there with
/// nothing to switch back to.
#[tauri::command]
pub fn dismiss<R: Runtime>(window: WebviewWindow<R>) {
    let _ = window.hide();

    crate::dock::follows_the_windows(
        window.app_handle(),
        crate::dock::Change::Hiding(window.label()),
    );

    // Escape out of the first-run flow is as final an answer to it as the
    // button at the end: `welcome` says why, and answers to nothing else.
    crate::welcome::gone(window.app_handle(), window.label());
}
