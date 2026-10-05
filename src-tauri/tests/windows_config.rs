//! `tauri.windows.conf.json` repeats every window of `tauri.conf.json`, because
//! a platform file replaces the `windows` array rather than merging into it.
//! It differs only in `"focus": false`: on Windows a window created hidden
//! still takes the foreground, and the keys a user types while Demysto starts
//! went to a window nobody could see. Linux gets no such setting, because there
//! it stops the window taking keys at all.

use serde_json::Value;

fn windows(file: &str) -> Vec<Value> {
    let path = format!("{}/{file}", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path}: {error}"));
    let config: Value = serde_json::from_str(&text).unwrap_or_else(|error| panic!("{path}: {error}"));
    config["app"]["windows"].as_array().cloned().unwrap_or_default()
}

#[test]
fn the_windows_file_repeats_every_window_and_only_adds_focus_false() {
    let shared = windows("tauri.conf.json");
    let on_windows = windows("tauri.windows.conf.json");
    assert_eq!(on_windows.len(), shared.len(), "a window was added or removed in one file only");
    for (mine, theirs) in on_windows.iter().zip(&shared) {
        let mut without_focus = mine.clone();
        assert_eq!(without_focus["focus"], Value::Bool(false), "{} takes focus on Windows", mine["label"]);
        without_focus.as_object_mut().unwrap().remove("focus");
        assert_eq!(&without_focus, theirs, "{} differs between the two files", mine["label"]);
    }
}
