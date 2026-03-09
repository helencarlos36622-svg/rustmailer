// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::{
    id,
    modules::scheduler::retry::{RetryPolicy, RetryStrategy},
    utc_now,
};
use poem_openapi::Enum;
use serde::{Deserialize, Serialize};
use std::fmt;

type LinearInterval = u32;
type ExponentialBase = u32;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaskMeta {
    pub id: u64,
    pub task_key: String,
    pub task_params: String,
    pub queue_name: String,
    pub updated_at: i64,
    pub status: TaskStatus,
    pub stopped_reason: Option<String>,
    pub last_error: Option<String>,
    pub last_duration_ms: Option<usize>,
    pub retry_count: Option<usize>,
    pub next_run: i64,
    pub retry_strategy: Retry,
    pub retry_interval: u32,
    pub base_interval: u32,
    pub delay_seconds: u32,
    pub max_retries: Option<u32>,
    pub heartbeat_at: i64,
    pub created_at: i64,
}

/// Represents the status of a task in the system.
///
/// This enum defines the lifecycle states that a task can be in,
/// from initial scheduling to completion or removal.
#[derive(Clone, Debug, Eq, Default, PartialEq, Serialize, Deserialize, Hash, Enum)]
pub enum TaskStatus {
    /// Task has been scheduled but has not started executing yet.
    #[default]
    Scheduled,

    /// Task is currently running.
    Running,

    /// Task has completed successfully.
    Success,

    /// Task has failed.
    Failed,

    /// Task has been marked for removal and will be cleaned up by a dedicated thread.
    Removed,

    /// Task has been stopped.
    Stopped,
}

impl TaskStatus {
    pub fn code(&self) -> u32 {
        match &self {
            TaskStatus::Scheduled => 1,
            TaskStatus::Running => 2,
            TaskStatus::Success => 3,
            TaskStatus::Failed => 4,
            TaskStatus::Removed => 5,
            TaskStatus::Stopped => 6,
        }
    }
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status_str = match self {
            TaskStatus::Scheduled => "Scheduled",
            TaskStatus::Running => "Running",
            TaskStatus::Success => "Success",
            TaskStatus::Failed => "Failed",
            TaskStatus::Removed => "Removed",
            TaskStatus::Stopped => "Stopped",
        };
        write!(f, "{}", status_str)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum Retry {
    #[default]
    Linear,
    Exponential,
}

impl fmt::Display for Retry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Retry::Linear => write!(f, "Linear"),
            Retry::Exponential => write!(f, "Exponential"),
        }
    }
}

fn to_retry(retry_policy: RetryPolicy) -> (Retry, LinearInterval, ExponentialBase) {
    match retry_policy.strategy {
        RetryStrategy::Linear { interval } => (Retry::Linear, interval, Default::default()),
        RetryStrategy::Exponential { base } => (Retry::Exponential, Default::default(), base),
    }
}

impl TaskMeta {
    pub fn new(
        task_key: String,
        task_params: String,
        queue_name: String,
        retry_policy: RetryPolicy,
        delay_seconds: u32,
    ) -> Self {
        // Extract retry strategy and intervals from the given retry policy.
        let (retry_strategy, retry_interval, base_interval) = to_retry(retry_policy);
        Self {
            id: id!(96),
            task_key,
            task_params,
            queue_name,
            updated_at: utc_now!(),
            status: TaskStatus::Scheduled,
            last_error: Default::default(),
            last_duration_ms: Default::default(),
            retry_count: Default::default(),
            next_run: Default::default(),
            stopped_reason: Default::default(),
            retry_strategy,
            retry_interval,
            base_interval,
            max_retries: retry_policy.max_retries,
            heartbeat_at: Default::default(),
            delay_seconds,
            created_at: utc_now!(),
        }
    }

    pub fn retry_policy(&self) -> RetryPolicy {
        let strategy = match self.retry_strategy {
            Retry::Linear => RetryStrategy::Linear {
                interval: self.retry_interval,
            },
            Retry::Exponential => RetryStrategy::Exponential {
                base: self.base_interval,
            },
        };

        RetryPolicy {
            strategy,
            max_retries: self.max_retries,
        }
    }
}
