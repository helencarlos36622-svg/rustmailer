// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use std::sync::LazyLock;

use crate::rustmailer_version;
use crate::{
    modules::{context::Initialize, error::RustMailerResult},
    utc_now,
};
use prometheus::{
    register_gauge, register_gauge_vec, register_histogram, register_histogram_vec,
    register_int_counter, register_int_counter_vec, register_int_gauge_vec, Gauge, GaugeVec,
    Histogram, HistogramVec, IntCounter, IntCounterVec, IntGaugeVec,
};

pub mod endpoint;

pub const SENT: &str = "sent";
pub const RECEIVED: &str = "received";

pub const SUCCESS: &str = "success";
pub const FAILURE: &str = "failure";

pub const EMAIL: &str = "email";
pub const HOOK: &str = "hook";

pub const HTTP: &str = "http";
pub const NATS: &str = "nats";

// Metric name constants
pub const METRIC_REQUEST_DURATION_BY_STATUS: &str = "rustmailer_request_duration_seconds_by_status";
pub const METRIC_REQUEST_DURATION_BY_METHOD_AND_OPERATION: &str =
    "rustmailer_request_duration_seconds_by_method_and_operation";
pub const METRIC_REQUEST_TOTAL_BY_METHOD_AND_OPERATION: &str =
    "rustmailer_request_total_by_method_and_operation";
pub const METRIC_IMAP_TRAFFIC_TOTAL: &str = "rustmailer_imap_traffic_total";
pub const METRIC_EMAIL_SENT_TOTAL: &str = "rustmailer_email_sent_total";
pub const METRIC_EMAIL_SENT_BYTES: &str = "rustmailer_email_sent_bytes";
pub const METRIC_EMAIL_SEND_DURATION_SECONDS: &str = "rustmailer_email_send_duration_seconds";
pub const METRIC_EVENT_DISPATCH_TOTAL_BY_TYPE_STATUS_AND_DESTINATION: &str =
    "rustmailer_event_dispatch_total_by_type_status_and_destination";
pub const METRIC_EVENT_DISPATCH_DURATION_SECONDS_BY_TYPE_STATUS_AND_DESTINATION: &str =
    "rustmailer_event_dispatch_duration_seconds_by_type_status_and_destination";
pub const METRIC_NEW_EMAIL_ARRIVAL_TOTAL: &str = "rustmailer_new_email_arrival_total";
pub const METRIC_MAIL_FLAG_CHANGE_TOTAL: &str = "rustmailer_mail_flag_change_total";
pub const METRIC_EMAIL_OPENS_TOTAL: &str = "rustmailer_email_opens_total";
pub const METRIC_EMAIL_CLICKS_TOTAL: &str = "rustmailer_email_clicks_total";
pub const METRIC_TASK_FETCH_DURATION: &str = "rustmailer_task_fetch_duration_seconds";
pub const METRIC_BUILD_INFO: &str = "rustmailer_build_info";
pub const METRIC_START_TIMESTAMP: &str = "rustmailer_start_timestamp";
pub const METRIC_TASK_QUEUE_LENGTH: &str = "rustmailer_task_queue_length";

pub static RUSTMAILER_BUILD_INFO: LazyLock<GaugeVec> = LazyLock::new(|| {
    register_gauge_vec!(
        METRIC_BUILD_INFO,
        "Build information including version and commit hash",
        &["version", "commit"]
    )
    .expect("Failed to register rustmailer_build_info")
});

pub static RUSTMAILER_START_TIMESTAMP: LazyLock<Gauge> = LazyLock::new(|| {
    register_gauge!(
        METRIC_START_TIMESTAMP,
        "Unix timestamp when RustMailer started"
    )
    .expect("Failed to register rustmailer_start_timestamp")
});

// Original metrics using the name constants
pub static RUSTMAILER_REQUEST_DURATION_BY_STATUS: LazyLock<HistogramVec> = LazyLock::new(|| {
    register_histogram_vec!(
        METRIC_REQUEST_DURATION_BY_STATUS,
        "Distribution of HTTP request durations, measured in seconds, grouped by response status code",
        &["status"]
    )
    .expect("Failed to register request_duration_seconds_by_status")
});

pub static RUSTMAILER_REQUEST_DURATION_BY_METHOD_AND_OPERATION: LazyLock<HistogramVec> =
    LazyLock::new(|| {
        register_histogram_vec!(
            METRIC_REQUEST_DURATION_BY_METHOD_AND_OPERATION,
            "Distribution of HTTP request durations, measured in seconds, grouped by method, operation ID, and status code",
            &["method", "operation_id", "status"]
        )
        .expect("Failed to register request_duration_seconds_by_method_and_operation")
    });

pub static RUSTMAILER_REQUEST_TOTAL_BY_METHOD_AND_OPERATION: LazyLock<IntCounterVec> =
    LazyLock::new(|| {
        register_int_counter_vec!(
            METRIC_REQUEST_TOTAL_BY_METHOD_AND_OPERATION,
            "Total number of HTTP requests, grouped by method, operation ID, and status code",
            &["method", "operation_id", "status"]
        )
        .expect("Failed to register request_total_by_method_and_operation")
    });

pub static RUSTMAILER_IMAP_TRAFFIC_TOTAL_BY_METRIC: LazyLock<IntCounterVec> = LazyLock::new(|| {
    register_int_counter_vec!(
        METRIC_IMAP_TRAFFIC_TOTAL,
        "Total IMAP traffic metrics, grouped by metric",
        &["metric"]
    )
    .expect("Failed to register rustmailer_imap_traffic_total")
});

pub static RUSTMAILER_EMAIL_SENT_TOTAL: LazyLock<IntCounterVec> = LazyLock::new(|| {
    register_int_counter_vec!(
        METRIC_EMAIL_SENT_TOTAL,
        "Total number of sent emails, grouped by status",
        &["status"]
    )
    .expect("Failed to register rustmailer_email_sent_total")
});

pub static RUSTMAILER_EMAIL_SENT_BYTES: LazyLock<IntCounter> = LazyLock::new(|| {
    register_int_counter!(
        METRIC_EMAIL_SENT_BYTES,
        "Total bytes of successfully sent emails"
    )
    .expect("Failed to register rustmailer_email_sent_bytes")
});

pub static RUSTMAILER_EMAIL_SEND_DURATION_SECONDS: LazyLock<HistogramVec> = LazyLock::new(|| {
    register_histogram_vec!(
        METRIC_EMAIL_SEND_DURATION_SECONDS,
        "Distribution of email sending durations, measured in seconds",
        &["status"],
        vec![0.1, 0.25, 0.5, 0.75, 1.0, 2.5, 5.0, 7.5, 10.0, 30.0, 60.0]
    )
    .expect("Failed to register email_send_duration_seconds")
});

pub static RUSTMAILER_EVENT_DISPATCH_TOTAL_BY_TYPE_STATUS_AND_DESTINATION: LazyLock<IntCounterVec> =
    LazyLock::new(|| {
        register_int_counter_vec!(
            METRIC_EVENT_DISPATCH_TOTAL_BY_TYPE_STATUS_AND_DESTINATION,
            "Total number of events dispatched, grouped by event type, status, and destination (http/nats)",
            &["status", "destination"]
        )
        .expect("Failed to register event_dispatch_total_by_type_status_and_destination")
    });

pub static RUSTMAILER_EVENT_DISPATCH_DURATION_SECONDS_BY_TYPE_STATUS_AND_DESTINATION: LazyLock<
    HistogramVec,
> = LazyLock::new(|| {
    register_histogram_vec!(
        METRIC_EVENT_DISPATCH_DURATION_SECONDS_BY_TYPE_STATUS_AND_DESTINATION,
        "Distribution of event dispatch durations (in seconds), grouped by event type, status, and destination (http/nats)",
        &["status", "destination"],
        vec![0.1, 0.25, 0.5, 0.75, 1.0, 2.5, 5.0, 7.5, 10.0, 30.0, 60.0]
    )
    .expect("Failed to register event_dispatch_duration_seconds_by_type_status_and_destination")
});

pub static RUSTMAILER_NEW_EMAIL_ARRIVAL_TOTAL: LazyLock<IntCounter> = LazyLock::new(|| {
    register_int_counter!(
        METRIC_NEW_EMAIL_ARRIVAL_TOTAL,
        "Total number of new emails received (global)"
    )
    .expect("Failed to register rustmailer_new_email_arrival_total")
});

pub static RUSTMAILER_MAIL_FLAG_CHANGE_TOTAL: LazyLock<IntCounter> = LazyLock::new(|| {
    register_int_counter!(
        METRIC_MAIL_FLAG_CHANGE_TOTAL,
        "Total number of mail flag change events (global)"
    )
    .expect("Failed to register rustmailer_mail_flag_change_total")
});

pub static RUSTMAILER_EMAIL_OPENS_TOTAL: LazyLock<IntCounter> = LazyLock::new(|| {
    register_int_counter!(
        METRIC_EMAIL_OPENS_TOTAL,
        "Total number of email opens (global)"
    )
    .expect("Failed to register email_opens_total metric")
});

pub static RUSTMAILER_EMAIL_CLICKS_TOTAL: LazyLock<IntCounter> = LazyLock::new(|| {
    register_int_counter!(
        METRIC_EMAIL_CLICKS_TOTAL,
        "Total number of email link clicks (global)"
    )
    .expect("Failed to register email_clicks_total metric")
});

/// Histogram to record the duration of fetching executable tasks
pub static RUSTMAILER_TASK_FETCH_DURATION: LazyLock<Histogram> = LazyLock::new(|| {
    register_histogram!(
        METRIC_TASK_FETCH_DURATION,
        "Duration of fetching executable tasks, measured in seconds"
    )
    .expect("Failed to register task_fetch_duration_seconds")
});

pub static RUSTMAILER_TASK_QUEUE_LENGTH: LazyLock<IntGaugeVec> = LazyLock::new(|| {
    register_int_gauge_vec!(
        METRIC_TASK_QUEUE_LENGTH,
        "Current length of task queues by type",
        &["queue"]
    )
    .expect("Failed to register rustmailer_task_queue_length")
});

pub struct MetricsService;

impl Initialize for MetricsService {
    async fn initialize() -> RustMailerResult<()> {
        let now = utc_now!();
        RUSTMAILER_START_TIMESTAMP.set(now as f64);
        let version = rustmailer_version!();
        let commit = env!("GIT_HASH");
        RUSTMAILER_BUILD_INFO
            .with_label_values(&[version, commit])
            .set(1.0);
        Ok(())
    }
}
