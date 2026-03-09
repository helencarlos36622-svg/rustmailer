// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::error::{RustMailerError, RustMailerResult};

#[derive(Debug)]
pub struct TaskResult {
    pub task_id: u64,
    pub last_duration_ms: usize,
    pub retry_count: usize,
    pub next_run: Option<i64>,
    pub result: RustMailerResult<()>,
}

impl TaskResult {
    /// Create a success result with task_id
    pub fn success(task_id: u64, last_duration_ms: usize) -> Self {
        Self {
            task_id,
            result: Ok(()),
            last_duration_ms,
            retry_count: Default::default(),
            next_run: None,
        }
    }

    /// Create a failure result with task_id and TaskError
    pub fn failure(task_id: u64, error: RustMailerError, last_duration_ms: usize) -> Self {
        Self {
            task_id,
            result: Err(error),
            last_duration_ms,
            retry_count: Default::default(),
            next_run: None,
        }
    }

    /// Check if the result is a success
    pub fn is_success(&self) -> bool {
        self.result.is_ok()
    }
}
