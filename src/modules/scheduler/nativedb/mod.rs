// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use std::sync::LazyLock;

use crate::modules::database::ModelsAdapter;
use crate::modules::scheduler::model::{Retry, TaskMeta, TaskStatus};
use native_db::*;
use native_model::native_model;
use native_model::Model;
use serde::{Deserialize, Serialize};

pub mod meta;

pub static TASK_MODELS: LazyLock<Models> = LazyLock::new(|| {
    let mut adapter = ModelsAdapter::new();
    adapter.register_model::<TaskMetaEntity>();
    adapter.models
});

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[native_model(id = 1, version = 1)]
#[native_db(
    primary_key(pk -> String),
    secondary_key(typed_status -> String),
    secondary_key(status -> u32)
)]
pub struct TaskMetaEntity {
    #[secondary_key(unique)]
    pub id: u64,
    #[secondary_key]
    pub task_key: String,
    pub task_params: String,
    #[secondary_key]
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

impl TaskMetaEntity {
    fn pk(&self) -> String {
        format!("{}_{}", self.created_at, self.id)
    }

    pub fn status(&self) -> u32 {
        self.status.code()
    }

    pub fn typed_status(&self) -> String {
        format!("{}_{}", &self.task_key, self.status.code())
    }

    pub fn status_filter_key(task_key: &str, status: TaskStatus) -> String {
        format!("{}_{}", task_key, status.code())
    }
}

impl From<TaskMetaEntity> for TaskMeta {
    fn from(entity: TaskMetaEntity) -> Self {
        TaskMeta {
            id: entity.id,
            task_key: entity.task_key,
            task_params: entity.task_params,
            queue_name: entity.queue_name,
            updated_at: entity.updated_at,
            created_at: entity.created_at,
            status: entity.status,
            stopped_reason: entity.stopped_reason,
            last_error: entity.last_error,
            last_duration_ms: entity.last_duration_ms,
            retry_count: entity.retry_count,
            next_run: entity.next_run,
            retry_strategy: entity.retry_strategy,
            retry_interval: entity.retry_interval,
            base_interval: entity.base_interval,
            delay_seconds: entity.delay_seconds,
            max_retries: entity.max_retries,
            heartbeat_at: entity.heartbeat_at,
        }
    }
}

impl From<TaskMeta> for TaskMetaEntity {
    fn from(entity: TaskMeta) -> Self {
        TaskMetaEntity {
            id: entity.id,
            task_key: entity.task_key,
            task_params: entity.task_params,
            queue_name: entity.queue_name,
            updated_at: entity.updated_at,
            created_at: entity.created_at,
            status: entity.status,
            stopped_reason: entity.stopped_reason,
            last_error: entity.last_error,
            last_duration_ms: entity.last_duration_ms,
            retry_count: entity.retry_count,
            next_run: entity.next_run,
            retry_strategy: entity.retry_strategy,
            retry_interval: entity.retry_interval,
            base_interval: entity.base_interval,
            delay_seconds: entity.delay_seconds,
            max_retries: entity.max_retries,
            heartbeat_at: entity.heartbeat_at,
        }
    }
}
