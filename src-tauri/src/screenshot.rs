//! An area of the screen on Linux, chosen in the desktop's own screenshot UI
//! through `org.freedesktop.portal.Screenshot`, or on X11 without one in
//! Demysto's overlay.
//!
//! Asked on Tauri's runtime rather than on one of the core's own: ashpd keeps
//! one D-Bus connection for the whole process, and the GlobalShortcuts portal
//! already holds it there.

use std::time::Duration;

use ashpd::desktop::screenshot::{Screenshot, ScreenshotProxy};
use ashpd::desktop::ResponseError;
use demysto_core::{CaptureError, Screen};

/// How long the portal is given to say whether it offers Screenshot before the
/// answer is taken to be yes: a slow portal is not a missing one.
const ASKING: Duration = Duration::from_secs(5);

/// The portal where it offers Screenshot or this is Wayland, the overlay
/// otherwise. Asked at every press: the portal may start or stop between them.
pub struct PortalOrOverlay;

impl Screen for PortalOrOverlay {
    fn area(&self) -> Result<Option<Vec<u8>>, CaptureError> {
        let overlay = !demysto_core::wayland_session()
            && tauri::async_runtime::block_on(offers()) == Some(false);

        if overlay {
            crate::overlay::Overlay.area()
        } else {
            Portal.area()
        }
    }
}

struct Portal;

impl Screen for Portal {
    /// A cancelled request is the cancel; the file the portal wrote is removed
    /// once read. Blocks on Tauri's runtime, so it must not be called from it.
    fn area(&self) -> Result<Option<Vec<u8>>, CaptureError> {
        let answered = tauri::async_runtime::block_on(async {
            Screenshot::request()
                .interactive(true)
                .send()
                .await?
                .response()
        });

        let uri = match answered {
            Ok(screenshot) => screenshot.uri().as_str().to_owned(),
            Err(ashpd::Error::Response(ResponseError::Cancelled)) => return Ok(None),
            // GNOME answers Esc in its screenshot UI with "other" and nothing
            // more; a portal failing that way is taken for a cancel too.
            Err(ashpd::Error::Response(ResponseError::Other)) => return Ok(None),
            Err(error) => return Err(CaptureError::Screen(error.to_string())),
        };

        let Some(file) = tauri::Url::parse(&uri)
            .ok()
            .and_then(|url| url.to_file_path().ok())
        else {
            return Err(CaptureError::Screen(uri));
        };

        let read = std::fs::read(&file);
        let _ = std::fs::remove_file(&file);

        read.map(Some)
            .map_err(|error| CaptureError::Screen(error.to_string()))
    }
}

/// Whether this is Wayland and no portal there offers Screenshot, so that no
/// area of the screen can be had at all.
pub async fn missing() -> bool {
    demysto_core::wayland_session() && offers().await == Some(false)
}

/// Whether a portal offers Screenshot, `None` where it did not say in time.
async fn offers() -> Option<bool> {
    tokio::time::timeout(ASKING, offered()).await.ok()
}

/// Asked of the property itself: ashpd takes a portal service that is not
/// running at all for one at version 1.
async fn offered() -> bool {
    match ScreenshotProxy::new().await {
        Ok(proxy) => proxy.get_property::<u32>("version").await.is_ok(),
        Err(_) => false,
    }
}
