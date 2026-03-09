// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::{
    context::status::RustMailerStatus,
    grpc::service::rustmailer_grpc,
    version::{LicenseCheckResult, Notifications, Release, ReleaseNotification},
};

impl From<RustMailerStatus> for rustmailer_grpc::ServerStatus {
    fn from(value: RustMailerStatus) -> Self {
        Self {
            uptime_ms: value.uptime_ms,
            timeago: value.timeago,
            timezone: value.timezone,
            version: value.version,
        }
    }
}

impl From<ReleaseNotification> for rustmailer_grpc::ReleaseNotification {
    fn from(value: ReleaseNotification) -> Self {
        Self {
            latest: value.latest.map(Into::into),
            is_newer: value.is_newer,
            error_message: value.error_message,
        }
    }
}

impl From<LicenseCheckResult> for rustmailer_grpc::LicenseCheckResult {
    fn from(value: LicenseCheckResult) -> Self {
        Self {
            expired: value.expired,
            days: value.days,
        }
    }
}

impl From<Notifications> for rustmailer_grpc::Notifications {
    fn from(value: Notifications) -> Self {
        Self {
            release: Some(value.release.into()),
            license: Some(value.license.into()),
        }
    }
}

impl From<Release> for rustmailer_grpc::Release {
    fn from(value: Release) -> Self {
        Self {
            tag_name: value.tag_name,
            published_at: value.published_at,
            body: value.body,
            html_url: value.html_url,
        }
    }
}
