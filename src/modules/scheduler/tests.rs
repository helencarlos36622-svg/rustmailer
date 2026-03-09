// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use std::sync::Arc;

use native_db::Builder;
use serde::{Deserialize, Serialize};

use crate::{
    generate_token,
    modules::{
        error::code::ErrorCode,
        scheduler::{
            context::TaskContext,
            nativedb::{meta::NativeDbTaskStore, TASK_MODELS},
        },
    },
    raise_error,
};

use super::task::Task;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RetryTestTask;

#[tokio::test]
async fn test_retry_policy() {
    // Initialize database and task store
    let db = Builder::new().create_in_memory(&TASK_MODELS).unwrap();
    let task_store = Arc::new(NativeDbTaskStore::init(Arc::new(db)));

    // Create task context
    let task_context = TaskContext::with_arc_store(task_store.clone())
        .register::<RetryTestTask>()
        .set_concurrency("retry_test_queue", 1)
        .start_with_cleaner()
        .await;

    // Add test task
    task_context.add_task(RetryTestTask, None).await.unwrap();

    // Wait for all retries to complete (5s interval * 3 retries = 15s + buffer)
    tokio::time::sleep(std::time::Duration::from_secs(50)).await;
}

#[test]
fn test1() {
    println!("task-{}", generate_token!(48).to_lowercase())
}

impl Task for RetryTestTask {
    const TASK_KEY: &'static str = "retry_test_task_key";
    const TASK_QUEUE: &'static str = "retry_test_queue";

    fn delay_seconds(&self) -> u32 {
        0
    }

    fn retry_policy(&self) -> super::retry::RetryPolicy {
        super::retry::RetryPolicy {
            strategy: super::retry::RetryStrategy::Exponential { base: 2 },
            max_retries: Some(5),
        }
    }

    fn run(self, _task_id: u64) -> super::task::TaskFuture {
        Box::pin(async move {
            // In a real test, you would pass the metrics through some context mechanism
            // For this example, we'll just simulate the failure without recording
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            println!(
                "current time: {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
            );
            Err(raise_error!("Task failed".into(), ErrorCode::InternalError))
        })
    }
}
