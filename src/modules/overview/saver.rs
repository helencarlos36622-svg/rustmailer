// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use ahash::AHashMap;
use std::sync::{LazyLock, Mutex};

use crate::{
    modules::{
        context::RustMailTask,
        error::RustMailerResult,
        metrics::{
            EMAIL, FAILURE, HOOK, HTTP, METRIC_EMAIL_CLICKS_TOTAL, METRIC_EMAIL_OPENS_TOTAL,
            METRIC_EMAIL_SENT_BYTES, METRIC_EMAIL_SENT_TOTAL,
            METRIC_EVENT_DISPATCH_TOTAL_BY_TYPE_STATUS_AND_DESTINATION, METRIC_IMAP_TRAFFIC_TOTAL,
            METRIC_MAIL_FLAG_CHANGE_TOTAL, METRIC_NEW_EMAIL_ARRIVAL_TOTAL,
            METRIC_TASK_QUEUE_LENGTH, NATS, RECEIVED, RUSTMAILER_EMAIL_CLICKS_TOTAL,
            RUSTMAILER_EMAIL_OPENS_TOTAL, RUSTMAILER_EMAIL_SENT_BYTES, RUSTMAILER_EMAIL_SENT_TOTAL,
            RUSTMAILER_EVENT_DISPATCH_TOTAL_BY_TYPE_STATUS_AND_DESTINATION,
            RUSTMAILER_IMAP_TRAFFIC_TOTAL_BY_METRIC, RUSTMAILER_MAIL_FLAG_CHANGE_TOTAL,
            RUSTMAILER_NEW_EMAIL_ARRIVAL_TOTAL, RUSTMAILER_TASK_QUEUE_LENGTH, SENT, SUCCESS,
        },
        overview::metrics::DailyMetrics,
        scheduler::periodic::PeriodicTask,
    },
    utc_now,
};

use std::time::Duration;

const TASK_INTERVAL: Duration = Duration::from_secs(1 * 60); // every 1 min

static METRIC_CACHE: LazyLock<MetricCache> = LazyLock::new(|| MetricCache {
    last_values: Mutex::new(AHashMap::new()),
});

struct MetricCache {
    last_values: Mutex<AHashMap<String, u64>>,
}

impl MetricCache {
    fn calculate_delta(&self, metric_name: &str, label: &str, current_value: u64) -> u64 {
        let key = format!("{}_{}", metric_name, label);
        let mut last_values = self.last_values.lock().unwrap();

        let delta = match last_values.get(&key) {
            Some(last_value) => {
                if current_value >= *last_value {
                    current_value - *last_value
                } else {
                    current_value
                }
            }
            None => 0,
        };

        last_values.insert(key, current_value);
        delta
    }
}

///This task cleans up expired weekly metrics entries older than 7 days.
pub struct MetricsSaveTask;

impl RustMailTask for MetricsSaveTask {
    fn start() {
        let periodic_task = PeriodicTask::new("daily-metrics-saver");
        let task = move |_ctx: Option<u64>| Box::pin(async move { take_snapshot().await });
        periodic_task.start(task, None, TASK_INTERVAL, false, true);
    }
}

async fn take_snapshot() -> RustMailerResult<()> {
    let now = utc_now!();

    // IMAP sent traffic
    let current_sent = RUSTMAILER_IMAP_TRAFFIC_TOTAL_BY_METRIC
        .with_label_values(&[SENT])
        .get();
    let delta_sent = METRIC_CACHE.calculate_delta(METRIC_IMAP_TRAFFIC_TOTAL, SENT, current_sent);
    DailyMetrics::save(
        METRIC_IMAP_TRAFFIC_TOTAL.to_string(),
        delta_sent,
        SENT.to_string(),
        now,
    )
    .await?;

    // IMAP received traffic
    let current_received = RUSTMAILER_IMAP_TRAFFIC_TOTAL_BY_METRIC
        .with_label_values(&[RECEIVED])
        .get();
    let delta_received =
        METRIC_CACHE.calculate_delta(METRIC_IMAP_TRAFFIC_TOTAL, RECEIVED, current_received);
    DailyMetrics::save(
        METRIC_IMAP_TRAFFIC_TOTAL.to_string(),
        delta_received,
        RECEIVED.to_string(),
        now,
    )
    .await?;

    let email_task_queue_length = RUSTMAILER_TASK_QUEUE_LENGTH
        .with_label_values(&[EMAIL])
        .get();
    DailyMetrics::save(
        METRIC_TASK_QUEUE_LENGTH.to_string(),
        email_task_queue_length as u64,
        EMAIL.to_string(),
        now,
    )
    .await?;

    let hook_task_queue_length = RUSTMAILER_TASK_QUEUE_LENGTH
        .with_label_values(&[HOOK])
        .get();
    DailyMetrics::save(
        METRIC_TASK_QUEUE_LENGTH.to_string(),
        hook_task_queue_length as u64,
        HOOK.to_string(),
        now,
    )
    .await?;

    // Email sent success count
    let current_email_sent_success = RUSTMAILER_EMAIL_SENT_TOTAL
        .with_label_values(&[SUCCESS])
        .get();
    let delta_email_sent_success =
        METRIC_CACHE.calculate_delta(METRIC_EMAIL_SENT_TOTAL, SUCCESS, current_email_sent_success);
    DailyMetrics::save(
        METRIC_EMAIL_SENT_TOTAL.to_string(),
        delta_email_sent_success,
        SUCCESS.to_string(),
        now,
    )
    .await?;

    // Email sent failure count
    let current_email_sent_failure = RUSTMAILER_EMAIL_SENT_TOTAL
        .with_label_values(&[FAILURE])
        .get();
    let delta_email_sent_failure =
        METRIC_CACHE.calculate_delta(METRIC_EMAIL_SENT_TOTAL, FAILURE, current_email_sent_failure);
    DailyMetrics::save(
        METRIC_EMAIL_SENT_TOTAL.to_string(),
        delta_email_sent_failure,
        FAILURE.to_string(),
        now,
    )
    .await?;

    // Email sent bytes
    let current_email_sent_bytes = RUSTMAILER_EMAIL_SENT_BYTES.get();
    let delta_email_sent_bytes =
        METRIC_CACHE.calculate_delta(METRIC_EMAIL_SENT_BYTES, "", current_email_sent_bytes);
    DailyMetrics::save(
        METRIC_EMAIL_SENT_BYTES.to_string(),
        delta_email_sent_bytes,
        "".to_string(),
        now,
    )
    .await?;

    // New email arrivals
    let current_new_email_arrival = RUSTMAILER_NEW_EMAIL_ARRIVAL_TOTAL.get();
    let delta_new_email_arrival = METRIC_CACHE.calculate_delta(
        METRIC_NEW_EMAIL_ARRIVAL_TOTAL,
        "",
        current_new_email_arrival,
    );
    DailyMetrics::save(
        METRIC_NEW_EMAIL_ARRIVAL_TOTAL.to_string(),
        delta_new_email_arrival,
        "".to_string(),
        now,
    )
    .await?;

    // Mail flag changes
    let current_mail_flag_change = RUSTMAILER_MAIL_FLAG_CHANGE_TOTAL.get();
    let delta_mail_flag_change =
        METRIC_CACHE.calculate_delta(METRIC_MAIL_FLAG_CHANGE_TOTAL, "", current_mail_flag_change);
    DailyMetrics::save(
        METRIC_MAIL_FLAG_CHANGE_TOTAL.to_string(),
        delta_mail_flag_change,
        "".to_string(),
        now,
    )
    .await?;

    // Email opens
    let current_email_opens = RUSTMAILER_EMAIL_OPENS_TOTAL.get();
    let delta_email_opens =
        METRIC_CACHE.calculate_delta(METRIC_EMAIL_OPENS_TOTAL, "", current_email_opens);
    DailyMetrics::save(
        METRIC_EMAIL_OPENS_TOTAL.to_string(),
        delta_email_opens,
        "".to_string(),
        now,
    )
    .await?;

    // Email clicks
    let current_email_clicks = RUSTMAILER_EMAIL_CLICKS_TOTAL.get();
    let delta_email_clicks =
        METRIC_CACHE.calculate_delta(METRIC_EMAIL_CLICKS_TOTAL, "", current_email_clicks);
    DailyMetrics::save(
        METRIC_EMAIL_CLICKS_TOTAL.to_string(),
        delta_email_clicks,
        "".to_string(),
        now,
    )
    .await?;

    // Event dispatch success to HTTP
    let current_event_dispatch_success_http =
        RUSTMAILER_EVENT_DISPATCH_TOTAL_BY_TYPE_STATUS_AND_DESTINATION
            .with_label_values(&[SUCCESS, HTTP])
            .get();
    let delta_event_dispatch_success_http = METRIC_CACHE.calculate_delta(
        METRIC_EVENT_DISPATCH_TOTAL_BY_TYPE_STATUS_AND_DESTINATION,
        &format!("{}_{}", SUCCESS, HTTP),
        current_event_dispatch_success_http,
    );
    DailyMetrics::save(
        METRIC_EVENT_DISPATCH_TOTAL_BY_TYPE_STATUS_AND_DESTINATION.to_string(),
        delta_event_dispatch_success_http,
        format!("{}_{}", SUCCESS, HTTP),
        now,
    )
    .await?;

    // Event dispatch success to NATS
    let current_event_dispatch_success_nats =
        RUSTMAILER_EVENT_DISPATCH_TOTAL_BY_TYPE_STATUS_AND_DESTINATION
            .with_label_values(&[SUCCESS, NATS])
            .get();
    let delta_event_dispatch_success_nats = METRIC_CACHE.calculate_delta(
        METRIC_EVENT_DISPATCH_TOTAL_BY_TYPE_STATUS_AND_DESTINATION,
        &format!("{}_{}", SUCCESS, NATS),
        current_event_dispatch_success_nats,
    );
    DailyMetrics::save(
        METRIC_EVENT_DISPATCH_TOTAL_BY_TYPE_STATUS_AND_DESTINATION.to_string(),
        delta_event_dispatch_success_nats,
        format!("{}_{}", SUCCESS, NATS),
        now,
    )
    .await?;

    // Event dispatch failure to HTTP
    let current_event_dispatch_failure_http =
        RUSTMAILER_EVENT_DISPATCH_TOTAL_BY_TYPE_STATUS_AND_DESTINATION
            .with_label_values(&[FAILURE, HTTP])
            .get();
    let delta_event_dispatch_failure_http = METRIC_CACHE.calculate_delta(
        METRIC_EVENT_DISPATCH_TOTAL_BY_TYPE_STATUS_AND_DESTINATION,
        &format!("{}_{}", FAILURE, HTTP),
        current_event_dispatch_failure_http,
    );
    DailyMetrics::save(
        METRIC_EVENT_DISPATCH_TOTAL_BY_TYPE_STATUS_AND_DESTINATION.to_string(),
        delta_event_dispatch_failure_http,
        format!("{}_{}", FAILURE, HTTP),
        now,
    )
    .await?;

    // Event dispatch failure to NATS
    let current_event_dispatch_failure_nats =
        RUSTMAILER_EVENT_DISPATCH_TOTAL_BY_TYPE_STATUS_AND_DESTINATION
            .with_label_values(&[FAILURE, NATS])
            .get();
    let delta_event_dispatch_failure_nats = METRIC_CACHE.calculate_delta(
        METRIC_EVENT_DISPATCH_TOTAL_BY_TYPE_STATUS_AND_DESTINATION,
        &format!("{}_{}", FAILURE, NATS),
        current_event_dispatch_failure_nats,
    );
    DailyMetrics::save(
        METRIC_EVENT_DISPATCH_TOTAL_BY_TYPE_STATUS_AND_DESTINATION.to_string(),
        delta_event_dispatch_failure_nats,
        format!("{}_{}", FAILURE, NATS),
        now,
    )
    .await?;

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::{
        modules::{
            error::code::ErrorCode,
            metrics::{RUSTMAILER_EMAIL_SENT_BYTES, RUSTMAILER_IMAP_TRAFFIC_TOTAL_BY_METRIC, SENT},
        },
        raise_error,
    };

    #[test]
    fn test1() {
        let value = RUSTMAILER_EMAIL_SENT_BYTES.get();
        println!("{}", value);

        let value = RUSTMAILER_IMAP_TRAFFIC_TOTAL_BY_METRIC
            .get_metric_with_label_values(&[SENT])
            .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))
            .unwrap();
        println!("{}", value.get());
    }
}
