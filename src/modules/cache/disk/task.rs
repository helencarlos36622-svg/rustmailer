// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::{
    cache::disk::DISK_CACHE, context::RustMailTask, scheduler::periodic::PeriodicTask,
};
use std::time::Duration;
const TASK_INTERVAL: Duration = Duration::from_secs(10 * 60);

///This task periodically cleans up the disk cache when storage usage exceeds a specified threshold to ensure efficient use of disk space.
pub struct DiskCacheCleanTask;

impl RustMailTask for DiskCacheCleanTask {
    fn start() {
        let periodic_task = PeriodicTask::new("disk-cache-cleaner");

        let task = move |_: Option<u64>| {
            Box::pin(async move {
                DISK_CACHE.clean_cache_if_needed().await;
                Ok(())
            })
        };

        periodic_task.start(task, None, TASK_INTERVAL, false, false);
    }
}
