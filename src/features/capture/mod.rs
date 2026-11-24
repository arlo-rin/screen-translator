pub mod frame;
/// Only macOS needs the overlay window pushed past what the window server
/// would otherwise allow.
#[cfg(target_os = "macos")]
pub mod mac_window;
pub mod native;
pub mod overlay;
pub mod screenshot;
/// Windows 11 rounds a window's corners itself, given the word.
#[cfg(windows)]
pub mod win_window;
pub mod window_pick;

pub use frame::Frame;
pub use overlay::{Geometry, OverlayState};
pub use screenshot::{
    capture_desktop, capture_frame, cursor_position, foreground_window_bounds, monitor_for,
    too_small, virtual_desktop, Bounds, Monitor,
};
