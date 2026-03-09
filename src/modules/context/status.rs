// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::context::executors::RUST_MAIL_CONTEXT;
use chrono::Local;
use poem_openapi::Object;
use serde::Deserialize;
use serde::Serialize;
use std::time::Duration;
use timeago::Formatter;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object)]
pub struct RustMailerStatus {
    /// The service uptime in milliseconds since it started.
    pub uptime_ms: i64,
    /// A human-readable string indicating the time elapsed since the service started (e.g., "2 hours ago").
    pub timeago: String,
    /// The timezone in which the service is operating (e.g., "UTC" or "Asia/Tokyo").
    pub timezone: String,
    /// The version of the RustMailer service currently running.
    pub version: String,
}

impl RustMailerStatus {
    pub fn get() -> Self {
        Self {
            uptime_ms: RUST_MAIL_CONTEXT.uptime_ms(),
            timeago: Formatter::new()
                .convert(Duration::from_millis(RUST_MAIL_CONTEXT.uptime_ms() as u64)),
            timezone: Local::now().offset().to_string(),
            version: env!("CARGO_PKG_VERSION").into(),
        }
    }
}
