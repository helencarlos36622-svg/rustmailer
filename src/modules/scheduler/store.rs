// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::{error::RustMailerResult, scheduler::model::TaskMeta};
use std::future::Future;

pub trait TaskStore {
    fn store_task(&self, task: TaskMeta) -> impl Future<Output = RustMailerResult<()>> + Send;

    fn store_tasks(
        &self,
        tasks: Vec<TaskMeta>,
    ) -> impl Future<Output = RustMailerResult<()>> + Send;

    fn fetch_pending_tasks(&self) -> impl Future<Output = RustMailerResult<Vec<TaskMeta>>> + Send;

    fn update_task_execution_status(
        &self,
        task_id: u64,
        is_success: bool,
        last_error: Option<String>,
        last_duration_ms: Option<usize>,
        retry_count: Option<usize>,
        next_run: Option<i64>,
    ) -> impl Future<Output = RustMailerResult<()>> + Send;

    fn heartbeat(&self, task_id: u64) -> impl Future<Output = RustMailerResult<()>> + Send;

    fn set_task_stopped(
        &self,
        task_id: u64,
        reason: Option<String>,
    ) -> impl Future<Output = RustMailerResult<()>> + Send;

    // fn set_task_removed(&self, task_id: &str) -> impl Future<Output = RustMailerResult<()>> + Send;

    fn cleanup(&self) -> impl Future<Output = RustMailerResult<()>> + Send;
}
