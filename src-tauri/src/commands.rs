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
use tauri::{AppHandle, Manager, Runtime, State, WebviewWindow};

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

/// Claims the Hotkeys a catalogue states, and hands both to the window.
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

/// Opens the folder Demysto writes its logs in, so that a bug report can carry
/// them.
#[tauri::command]
pub fn open_logs(demysto: State<'_, Demysto>) -> Result<(), String> {
    crate::folder::open(&demysto.logs_dir())
}

/// The Conversation the window is showing, for one that mounted after the Turn
/// it is showing began.
#[tauri::command]
pub fn conversation(demysto: State<'_, Demysto>) -> Option<Conversation> {
    demysto.conversation()
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
pub fn show_answers_on(channel: Channel<String>) {
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
    waiting(move || app.state::<Demysto>().save_settings(&edit)).await
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
#[tauri::command]
pub fn dismiss<R: Runtime>(window: WebviewWindow<R>) {
    let _ = window.hide();
}
