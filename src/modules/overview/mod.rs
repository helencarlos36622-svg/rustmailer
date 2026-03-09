// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use crate::modules::{
    account::migration::AccountModel,
    context::executors::RUST_MAIL_CONTEXT,
    error::RustMailerResult,
    metrics::{
        EMAIL, FAILURE, HOOK, HTTP, METRIC_EMAIL_CLICKS_TOTAL, METRIC_EMAIL_OPENS_TOTAL,
        METRIC_EMAIL_SENT_BYTES, METRIC_EMAIL_SENT_TOTAL,
        METRIC_EVENT_DISPATCH_TOTAL_BY_TYPE_STATUS_AND_DESTINATION, METRIC_IMAP_TRAFFIC_TOTAL,
        METRIC_MAIL_FLAG_CHANGE_TOTAL, METRIC_NEW_EMAIL_ARRIVAL_TOTAL, METRIC_TASK_QUEUE_LENGTH,
        NATS, RECEIVED, SENT, SUCCESS,
    },
    overview::metrics::DailyMetrics,
    scheduler::model::TaskStatus,
    tasks::queue::RustMailerTaskQueue,
};

pub mod clean;
pub mod metrics;
pub mod saver;

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
struct TimeSeriesPoint {
    timestamp: i64,
    value: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
pub struct Overview {
    pub pending_email_task_num: usize,
    pub pending_hook_task_num: usize,
    pub account_num: usize,
    pub uptime: i64,
    pub rustmailer_version: String,
    pub time_series: MetricsTimeSeries,
}

impl Overview {
    pub async fn get() -> RustMailerResult<Self> {
        let uptime = RUST_MAIL_CONTEXT.uptime_ms();
        let send_queue = RustMailerTaskQueue::get().unwrap();
        let pending_email_tasks = send_queue
            .list_email_tasks_by_status(TaskStatus::Scheduled)
            .await?;
        let pending_hook_tasks = send_queue
            .list_hook_tasks_by_status(TaskStatus::Scheduled)
            .await?;
        let account_num = AccountModel::count().await?;
        let mut time_series = MetricsTimeSeries::get().await?;
        time_series.sort_by_timestamp();

        Ok(Self {
            pending_email_task_num: pending_email_tasks.len(),
            pending_hook_task_num: pending_hook_tasks.len(),
            account_num,
            uptime,
            rustmailer_version: env!("CARGO_PKG_VERSION").into(),
            time_series,
        })
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
pub struct MetricsTimeSeries {
    imap_traffic_sent: Vec<TimeSeriesPoint>,
    imap_traffic_received: Vec<TimeSeriesPoint>,
    email_sent_success: Vec<TimeSeriesPoint>,
    email_sent_failure: Vec<TimeSeriesPoint>,
    email_sent_bytes: Vec<TimeSeriesPoint>,
    new_email_arrival: Vec<TimeSeriesPoint>,
    mail_flag_change: Vec<TimeSeriesPoint>,
    email_opens: Vec<TimeSeriesPoint>,
    email_clicks: Vec<TimeSeriesPoint>,
    event_dispatch_success_http: Vec<TimeSeriesPoint>,
    event_dispatch_success_nats: Vec<TimeSeriesPoint>,
    event_dispatch_failure_http: Vec<TimeSeriesPoint>,
    event_dispatch_failure_nats: Vec<TimeSeriesPoint>,
    email_task_queue_length: Vec<TimeSeriesPoint>,
    hook_task_queue_length: Vec<TimeSeriesPoint>,
}

impl MetricsTimeSeries {
    fn new() -> Self {
        MetricsTimeSeries {
            imap_traffic_sent: Vec::new(),
            imap_traffic_received: Vec::new(),
            email_sent_success: Vec::new(),
            email_sent_failure: Vec::new(),
            email_sent_bytes: Vec::new(),
            new_email_arrival: Vec::new(),
            mail_flag_change: Vec::new(),
            email_opens: Vec::new(),
            email_clicks: Vec::new(),
            event_dispatch_success_http: Vec::new(),
            event_dispatch_success_nats: Vec::new(),
            event_dispatch_failure_http: Vec::new(),
            event_dispatch_failure_nats: Vec::new(),
            email_task_queue_length: Vec::new(),
            hook_task_queue_length: Vec::new(),
        }
    }

    pub async fn get() -> RustMailerResult<Self> {
        let mut result = Self::new();
        let all = DailyMetrics::list_all().await?;

        for record in all {
            let point = TimeSeriesPoint {
                timestamp: record.created_at,
                value: record.value,
            };
            if record.metric == METRIC_IMAP_TRAFFIC_TOTAL && record.label == SENT {
                result.imap_traffic_sent.push(point);
            } else if record.metric == METRIC_IMAP_TRAFFIC_TOTAL && record.label == RECEIVED {
                result.imap_traffic_received.push(point);
            } else if record.metric == METRIC_EMAIL_SENT_TOTAL && record.label == SUCCESS {
                result.email_sent_success.push(point);
            } else if record.metric == METRIC_EMAIL_SENT_TOTAL && record.label == FAILURE {
                result.email_sent_failure.push(point);
            } else if record.metric == METRIC_EMAIL_SENT_BYTES {
                result.email_sent_bytes.push(point);
            } else if record.metric == METRIC_NEW_EMAIL_ARRIVAL_TOTAL {
                result.new_email_arrival.push(point);
            } else if record.metric == METRIC_MAIL_FLAG_CHANGE_TOTAL {
                result.mail_flag_change.push(point);
            } else if record.metric == METRIC_EMAIL_OPENS_TOTAL {
                result.email_opens.push(point)
            } else if record.metric == METRIC_EMAIL_CLICKS_TOTAL {
                result.email_clicks.push(point);
            } else if record.metric == METRIC_EVENT_DISPATCH_TOTAL_BY_TYPE_STATUS_AND_DESTINATION
                && record.label == format!("{}_{}", SUCCESS, HTTP)
            {
                result.event_dispatch_success_http.push(point);
            } else if record.metric == METRIC_EVENT_DISPATCH_TOTAL_BY_TYPE_STATUS_AND_DESTINATION
                && record.label == format!("{}_{}", SUCCESS, NATS)
            {
                result.event_dispatch_success_nats.push(point);
            } else if record.metric == METRIC_EVENT_DISPATCH_TOTAL_BY_TYPE_STATUS_AND_DESTINATION
                && record.label == format!("{}_{}", FAILURE, HTTP)
            {
                result.event_dispatch_failure_http.push(point);
            } else if record.metric == METRIC_EVENT_DISPATCH_TOTAL_BY_TYPE_STATUS_AND_DESTINATION
                && record.label == format!("{}_{}", FAILURE, NATS)
            {
                result.event_dispatch_failure_nats.push(point);
            } else if record.metric == METRIC_TASK_QUEUE_LENGTH && record.label == EMAIL {
                result.email_task_queue_length.push(point);
            } else if record.metric == METRIC_TASK_QUEUE_LENGTH && record.label == HOOK {
                result.hook_task_queue_length.push(point);
            }
        }

        Ok(result)
    }

    pub fn sort_by_timestamp(&mut self) {
        self.imap_traffic_sent.sort_by_key(|point| point.timestamp);
        self.imap_traffic_received
            .sort_by_key(|point| point.timestamp);
        self.email_sent_success.sort_by_key(|point| point.timestamp);
        self.email_sent_failure.sort_by_key(|point| point.timestamp);
        self.email_sent_bytes.sort_by_key(|point| point.timestamp);
        self.new_email_arrival.sort_by_key(|point| point.timestamp);
        self.mail_flag_change.sort_by_key(|point| point.timestamp);
        self.email_opens.sort_by_key(|point| point.timestamp);
        self.email_clicks.sort_by_key(|point| point.timestamp);
        self.event_dispatch_success_http
            .sort_by_key(|point| point.timestamp);
        self.event_dispatch_success_nats
            .sort_by_key(|point| point.timestamp);
        self.event_dispatch_failure_http
            .sort_by_key(|point| point.timestamp);
        self.event_dispatch_failure_nats
            .sort_by_key(|point| point.timestamp);
    }
}
