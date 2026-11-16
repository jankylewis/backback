//! The Dock icon without a Dock: the shipped image reads, bad bytes and a
//! call off the main thread are refused (unit tests run off it).

use super::bb_icon::image;
use super::{set_icon, DockError};

const DOCK: &[u8] = include_bytes!("../../../desktop/bb_assets/dock.png");

#[test]
fn the_shipped_dock_icon_is_an_image() {
    assert!(image(DOCK).is_ok());
}

#[test]
fn bytes_that_are_not_an_image_are_refused() {
    assert_eq!(image(b"not a png").err(), Some(DockError::NotAnImage));
}

#[test]
fn off_the_main_thread_nothing_is_set() {
    let result: Result<(), DockError> = std::thread::spawn(|| set_icon(DOCK)).join().unwrap();
    assert_eq!(result, Err(DockError::NotMainThread));
}
