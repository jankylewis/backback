//! The Dock icon call (BB-FEAT-080): the one `unsafe` block of backback.

use objc2::rc::Retained;
use objc2::{AnyThread, MainThreadMarker};
use objc2_app_kit::{NSApplication, NSImage};
use objc2_foundation::NSData;

/// Why the Dock icon was not set.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum DockError {
    /// Called off the main thread; `AppKit` runs only on it.
    #[error("dock icon: not on the main thread")]
    NotMainThread,
    /// `AppKit` could not read the bytes as an image.
    #[error("dock icon: not an image")]
    NotAnImage,
}

/// Show `png` as the app's Dock icon, as it is (no tile).
///
/// # Errors
/// [`DockError::NotMainThread`] off the main thread;
/// [`DockError::NotAnImage`] when `png` is not an image.
pub fn set_icon(png: &[u8]) -> Result<(), DockError> {
    let mtm: MainThreadMarker = MainThreadMarker::new().ok_or(DockError::NotMainThread)?;
    let image: Retained<NSImage> = image(png)?;
    let app: Retained<NSApplication> = NSApplication::sharedApplication(mtm);
    // SAFETY: `mtm` proves this runs on the main thread, as AppKit requires;
    // `image` is a live, retained NSImage for the whole call (AppKit keeps its
    // own reference); the argument is `Some`, so the binding's "might not
    // allow None" caveat does not apply.
    #[allow(unsafe_code, reason = "Peaker 2026-11-16: the Dock icon call only")]
    unsafe {
        app.setApplicationIconImage(Some(&image));
    }
    Ok(())
}

/// `png` as an `NSImage`; any thread.
pub(crate) fn image(png: &[u8]) -> Result<Retained<NSImage>, DockError> {
    let data: Retained<NSData> = NSData::with_bytes(png);
    NSImage::initWithData(NSImage::alloc(), &data).ok_or(DockError::NotAnImage)
}
