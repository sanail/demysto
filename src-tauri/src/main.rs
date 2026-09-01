// Prevents an additional console window on Windows in release. Do not remove.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    software_rendering_where_it_is_needed();

    demysto_lib::run()
}

/// Turns WebKitGTK's accelerated compositing off, unless the user has said
/// otherwise.
///
/// Here, before anything else runs, because WebKitGTK reads it as it starts.
///
/// Watched on a live Linux desktop with no GPU acceleration: the first-run
/// window came up as a correct, complete, white rectangle. The page was there —
/// the accessibility tree read every line of it back, and the elements'
/// on-screen extents were where they belonged — and not one pixel was drawn.
/// A resize did not bring it back, nor did showing another window first. The
/// Palette and the Conversation, which the suite had watched all along, drew
/// perfectly in that same session, so nothing about it announced itself until a
/// fourth window existed to be met by. With this variable set, it draws.
///
/// The cost is that the webview composites in software on Linux. For an
/// application that renders text that is not a trade worth measuring, and it is
/// the trade every other Tauri application on that platform makes for the same
/// reason. Anybody who would rather have the acceleration exports the variable
/// themselves, which is why this only fills in a value nobody stated.
#[cfg(target_os = "linux")]
fn software_rendering_where_it_is_needed() {
    const COMPOSITING: &str = "WEBKIT_DISABLE_COMPOSITING_MODE";

    // Written before any thread of this process exists — `main`'s first line —
    // which is what makes writing to the environment sound at all.
    if std::env::var_os(COMPOSITING).is_none() {
        std::env::set_var(COMPOSITING, "1");
    }
}

/// No other platform draws its windows through WebKitGTK.
#[cfg(not(target_os = "linux"))]
fn software_rendering_where_it_is_needed() {}
