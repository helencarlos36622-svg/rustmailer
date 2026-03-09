// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use crate::{
    modules::{
        error::{code::ErrorCode, RustMailerResult},
        license::License,
    },
    raise_error, rustmailer_version, utc_now,
};

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct ReleaseNotification {
    /// Details of the latest release, if available. `None` if no release data is available.
    pub latest: Option<Release>,
    /// Indicates whether the latest release is newer than the current RustMailer service version.
    pub is_newer: bool,
    /// Optional error message if the release check failed (e.g., network or API issues).
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct Release {
    /// The tag name of the release (e.g., "v1.2.3").
    pub tag_name: String,
    /// The publication date of the release in string format (e.g., ISO 8601 format).
    pub published_at: String,
    /// The body of the release notes, typically in Markdown format.
    pub body: String,
    /// The URL to the release's web page (e.g., GitHub release page).
    pub html_url: String,
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct Notifications {
    pub release: ReleaseNotification,
    pub license: LicenseCheckResult,
}

pub async fn fetch_notifications() -> RustMailerResult<Notifications> {
    let current_version = rustmailer_version!();
    let release = check_new_release("rustmailer", "rustmailer", current_version).await;
    let license = check_license().await?;
    Ok(Notifications { release, license })
}

async fn check_new_release(owner: &str, repo: &str, current_version: &str) -> ReleaseNotification {
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        owner, repo
    );

    let client = reqwest::Client::new();
    let response = match client
        .get(&url)
        .header("User-Agent", "reqwest")
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            return ReleaseNotification {
                latest: None,
                is_newer: false,
                error_message: Some(format!("Failed to send request: {}", e)),
            }
        }
    };

    if response.status().is_success() {
        match response.json::<Release>().await {
            Ok(release) => {
                let is_newer = release.tag_name != current_version;
                ReleaseNotification {
                    latest: Some(release),
                    is_newer,
                    error_message: None,
                }
            }
            Err(e) => ReleaseNotification {
                latest: None,
                is_newer: false,
                error_message: Some(format!("Failed to parse response: {}", e)),
            },
        }
    } else {
        ReleaseNotification {
            latest: None,
            is_newer: false,
            error_message: Some(format!("Request failed with status: {}", response.status())),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct LicenseCheckResult {
    pub expired: bool,
    pub days: u32,
}

async fn check_license() -> RustMailerResult<LicenseCheckResult> {
    let license = License::get_current_license()
        .await?
        .ok_or_else(|| raise_error!("license not found.".into(), ErrorCode::ResourceNotFound))?;
    let expires_at = license.expires_at;
    let now = utc_now!();
    let days_diff = (expires_at - now) / (1000 * 60 * 60 * 24);

    let (days, expired) = match days_diff {
        diff if diff > 0 => (diff as u32, false),
        _ => (days_diff.unsigned_abs() as u32, true),
    };
    Ok(LicenseCheckResult { days, expired })
}

#[cfg(test)]
mod test {
    use crate::{modules::version::check_new_release, rustmailer_version};

    #[tokio::test]
    async fn test() {
        let current_version = rustmailer_version!();
        println!("current_version: {}", rustmailer_version!());
        let result = check_new_release("rustmailer", "persistent-scheduler", current_version).await;

        println!("{:#?}", result);
    }
}
