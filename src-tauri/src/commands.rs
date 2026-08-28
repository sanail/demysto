//! The bridge between the frontend and [`demysto_core`].
//!
//! Every command here is a thin adapter: it borrows the facade out of Tauri's
//! managed state, calls one method on it, and maps the result. Logic that is
//! worth testing belongs in `demysto-core`, behind the single test seam, not here.

use std::collections::BTreeMap;

use demysto_core::{
    Action, CaptureOutcome, ConfigError, Conversation, Demysto, Edit, Preset, ProviderEdit,
    RunError, Settings, Status, Summary,
};
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, Runtime, State, WebviewWindow};

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
