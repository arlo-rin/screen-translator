//! Update-status wording, kept only so existing unit tests still compile.
//!
//! The running app no longer checks for, announces, or installs releases.

use std::path::PathBuf;

use crate::features::updater::UpdateInfo;
use crate::shared::i18n::t;

#[derive(Debug, Clone)]
pub(super) enum UpdateState {
    Checking,
    UpToDate,
    Available(UpdateInfo),
    Downloading {
        received: u64,
        total: u64,
    },
    /// Installed and waiting to be restarted. The restart happens on the UI
    /// thread so the settings are saved and the tray icon removed first.
    Installed(PathBuf),
    Error(String),
}

/// What the About page shows for the current state.
pub(super) struct UpdateRow {
    pub status: String,
    pub can_check: bool,
    pub can_install: bool,
    /// How far the download has got, when one is running with a known size.
    pub progress: Option<f32>,
    /// Where the build to install lives.
    pub url: Option<String>,
}

fn megabytes(bytes: u64) -> String {
    format!("{:.1}", bytes as f64 / (1024.0 * 1024.0))
}

pub(super) fn describe(state: &UpdateState) -> UpdateRow {
    let row = |status: String, can_check: bool| UpdateRow {
        status,
        can_check,
        can_install: false,
        progress: None,
        url: None,
    };

    match state {
        UpdateState::Checking => row(t("Checking for updates…").to_string(), false),
        UpdateState::UpToDate => row(
            t("You are on the latest version ({version})")
                .replace("{version}", env!("CARGO_PKG_VERSION")),
            true,
        ),
        UpdateState::Available(info) => UpdateRow {
            status: if info.size > 0 {
                t("Version {version} is available · {size} MB")
                    .replace("{version}", &info.version)
                    .replace("{size}", &megabytes(info.size))
            } else {
                t("Version {version} is available").replace("{version}", &info.version)
            },
            can_check: true,
            can_install: true,
            progress: None,
            url: Some(info.url.clone()),
        },
        UpdateState::Downloading { received, total } => UpdateRow {
            // A ten-megabyte download with no sign of movement looks like a
            // hang. Bytes when the server gives a length, and a running total
            // when it does not.
            status: if *total > 0 {
                t("Downloading the update… {done} of {total} MB")
                    .replace("{done}", &megabytes(*received))
                    .replace("{total}", &megabytes(*total))
            } else {
                t("Downloading the update… {done} MB").replace("{done}", &megabytes(*received))
            },
            can_check: false,
            can_install: false,
            progress: (*total > 0).then(|| *received as f32 / *total as f32),
            url: None,
        },
        UpdateState::Installed(_) => row(t("Update installed — restarting…").to_string(), false),
        // Already a sentence written for the user; wrapping it in "Error:" only
        // makes it look like a crash report.
        UpdateState::Error(message) => row(message.clone(), true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_download_with_a_known_size_reports_how_far_it_has_got() {
        let row = describe(&UpdateState::Downloading {
            received: 5 * 1024 * 1024,
            total: 10 * 1024 * 1024,
        });
        assert_eq!(row.progress, Some(0.5));
        assert!(!row.can_check && !row.can_install);
    }

    #[test]
    fn a_download_of_unknown_size_has_no_bar_to_draw() {
        let row = describe(&UpdateState::Downloading {
            received: 1024,
            total: 0,
        });
        assert_eq!(row.progress, None);
    }

    #[test]
    fn only_an_available_release_can_be_installed() {
        let available = describe(&UpdateState::Available(UpdateInfo {
            version: "9.9.9".into(),
            url: "https://example.test/build".into(),
            size: 0,
        }));
        assert!(available.can_install);
        assert_eq!(available.url.as_deref(), Some("https://example.test/build"));

        for state in [
            UpdateState::Checking,
            UpdateState::UpToDate,
            UpdateState::Error("nope".into()),
        ] {
            assert!(!describe(&state).can_install, "{state:?}");
        }
    }

    #[test]
    fn a_failed_check_can_be_tried_again() {
        assert!(describe(&UpdateState::Error("offline".into())).can_check);
    }
}
