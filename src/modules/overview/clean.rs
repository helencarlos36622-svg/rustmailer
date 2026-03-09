// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::{
    modules::{
        context::RustMailTask, overview::metrics::DailyMetrics, scheduler::periodic::PeriodicTask,
    },
    utc_now,
};

use std::time::Duration;

const TASK_INTERVAL: Duration = Duration::from_secs(5 * 60); // every 5 mins
const METRIC_RETENTION_MS: i64 = 24 * 60 * 60 * 1000; // 1 day

///This task cleans up expired weekly metrics entries older than 7 days.
pub struct MetricsCleanTask;

impl RustMailTask for MetricsCleanTask {
    fn start() {
        let periodic_task = PeriodicTask::new("daily-metrics-cleaner");

        let task = move |_ctx: Option<u64>| {
            Box::pin(async move {
                let now = utc_now!();
                let expire_before = now - METRIC_RETENTION_MS;
                DailyMetrics::clean(expire_before).await
            })
        };

        periodic_task.start(task, None, TASK_INTERVAL, false, false);
    }
}
