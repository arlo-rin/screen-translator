#![allow(unused_imports)]
//! Looking up and applying a newer release.
//!
//! This custom build keeps the helpers for leftover-file cleanup and for unit
//! tests, but it does not check for or install upstream binaries: that would
//! overwrite the local modifications.

pub mod checker;
pub mod installer;

pub use checker::{check_for_update, UpdateInfo};
pub use installer::{cleanup_previous_version, download_and_apply};

/// Startup checks, in-app install, and update notifications are off so this
/// build cannot be replaced by an upstream release.
pub const ENABLED: bool = false;

#[cfg(test)]
mod tests {
    #[test]
    fn automatic_updates_are_disabled() {
        assert!(
            !super::ENABLED,
            "this custom build must not fetch or install upstream releases"
        );
    }

    #[test]
    fn the_app_does_not_wire_the_updater() {
        let src = include_str!("../../app/mod.rs");
        for needle in [
            "spawn_check",
            "announce_update",
            "check_for_update",
            "install_update",
            "restart_if_updated",
            "update_row",
        ] {
            assert!(
                !src.contains(needle),
                "app still wires the updater via {needle}"
            );
        }
    }
}
