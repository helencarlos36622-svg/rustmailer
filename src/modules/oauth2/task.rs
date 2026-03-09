// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::{
    context::RustMailTask, oauth2::pending::OAuth2PendingEntity, scheduler::periodic::PeriodicTask,
};
use std::time::Duration;

const TASK_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);

///This task cleans up expired OAuth2 pending authorizations that haven't been completed by users in a timely manner.
pub struct OAuth2CleanTask;

impl RustMailTask for OAuth2CleanTask {
    fn start() {
        let periodic_task = PeriodicTask::new("oauth2-pending-task-cleaner");

        let task = move |_: Option<u64>| {
            Box::pin(async move {
                OAuth2PendingEntity::clean().await?;
                Ok(())
            })
        };

        periodic_task.start(task, None, TASK_INTERVAL, false, false);
    }
}
