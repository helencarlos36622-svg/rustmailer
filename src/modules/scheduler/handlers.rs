// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::error::code::ErrorCode;
use crate::modules::error::RustMailerError;
use crate::modules::scheduler::model::TaskMeta;
use crate::modules::scheduler::result::TaskResult;
use crate::modules::scheduler::task::Task;
use crate::{raise_error, utc_now};
use ahash::AHashMap;
use std::time::Instant;
use std::{future::Future, pin::Pin, sync::Arc};
use tracing::{error, info, warn};

// Type alias for a task handler that takes `Value` as input
// and returns a pinned future that resolves to a Result.
pub type Handler = Arc<
    dyn Fn(String, u64) -> Pin<Box<dyn Future<Output = Result<(), RustMailerError>> + Send>>
        + Send
        + Sync,
>;

#[derive(Clone)]
pub struct TaskHandlers {
    // A hashmap to store task handlers, mapping task keys to their corresponding handlers.
    handlers: AHashMap<String, Handler>,
}

impl TaskHandlers {
    /// Creates a new `TaskHandlers` container, initializing an empty handler map.
    pub fn new() -> Self {
        Self {
            handlers: AHashMap::new(),
        }
    }

    /// Registers a new task type in the handler map.
    ///
    /// # Type Parameter
    /// * `T`: The type of the task being registered, which must implement the `Task` trait.
    pub fn register<T>(&mut self)
    where
        T: Task,
    {
        self.handlers.insert(
            T::TASK_KEY.to_owned(), // Use the task key as the hashmap key.
            Arc::new(|params, task_id| process::<T>(params, task_id)), // Create a new handler for the task type.
        );
    }

    /// Executes a task based on its metadata.
    ///
    /// # Arguments
    /// * `task_meta`: Metadata of the task to be executed, encapsulated in `TaskMetaEntity`.
    ///
    /// # Returns
    /// * A `TaskResult` indicating the success or failure of the task execution.
    pub async fn execute(&self, task_meta: TaskMeta) -> TaskResult {
        let task_id = task_meta.id; // Clone the task ID for later use.
        let task_key = task_meta.task_key.clone(); // Clone the task key to retrieve the handler.
        let retry_policy = task_meta.retry_policy(); // Get the retry policy associated with the task.
        let attempts = task_meta.retry_count.unwrap_or(0); // Initialize the attempt counter.

        // Define an asynchronous block for task execution with retry logic.
        let execution_future = async move {
            // Get the handler for the specified task key.
            let handler_option = self
                .handlers
                .get(&task_key)
                .map(|handler| execute(handler.clone(), Arc::new(task_meta.clone())));

            // Attempt to execute the task using the found handler.
            let mut result = if let Some(execution_future) = handler_option {
                execution_future.await // Await the execution result.
            } else {
                error!("Unrecognized Task '{task_key}'. This error should not occur; it may indicate that the task has not been registered by the developer.");
                TaskResult::failure(
                    task_id,
                    raise_error!(
                        "Unrecognized task type for the client".into(),
                        ErrorCode::InternalError
                    ),
                    0,
                )
                // Handle unrecognized task.
            };

            if result.is_success() {
                return result;
            }
            result.retry_count = attempts + 1;

            if let Some(max) = retry_policy.max_retries {
                if result.retry_count >= (max as usize) {
                    warn!(
                        "Task {} has exceeded the maximum retry attempts of {}",
                        task_id, max
                    );
                    return result; // Return the result after exceeding max retries.
                }
            }
            // Wait for the defined backoff period before retrying.
            let wait = retry_policy.wait_time((attempts + 1) as u32);
            result.next_run = Some(utc_now!() + (wait * 1000) as i64);
            result
        };
        execution_future.await // Await the execution future.
    }
}

/// Processes the parameters for a given task type and returns a pinned future.
///
/// # Type Parameter
/// * `T`: The type of the task, which must implement the `Task` trait.
///
/// # Arguments
/// * `params`: The parameters for the task, represented as a `Value`.
///
/// # Returns
/// * A pinned future that resolves to a `Result` indicating the success or failure of the task processing.
pub fn process<T>(
    params: String,
    task_id: u64,
) -> Pin<Box<dyn Future<Output = Result<(), RustMailerError>> + Send>>
where
    T: Task,
{
    Box::pin(async move {
        // Deserialize the parameters into the specific task type `T`.
        let task = serde_json::from_str::<T>(params.as_str())
            .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?;
        // Execute the task and return any errors that occur during execution.
        task.run(task_id).await
    })
}

/// Executes a given handler with the specified task metadata.
///
/// # Arguments
/// * `handler`: The task handler to be executed.
/// * `task_meta`: The metadata for the task to be executed.
///
/// # Returns
/// * A `TaskResult` indicating the outcome of the task execution.
async fn execute(handler: Handler, task_meta: Arc<TaskMeta>) -> TaskResult {
    let task_name = task_meta.task_key.clone(); // Clone the task key for logging.
    let task_queue = task_meta.queue_name.clone(); // Clone the task queue name for logging.
    let start = Instant::now(); // Record the start time of the task execution.
    let task_params = task_meta.task_params.clone(); // Clone the task parameters for execution.
    let task_id = task_meta.id;
    // Spawn a new asynchronous task to execute the handler.
    let task_future = tokio::spawn(async move { (handler)(task_params, task_id).await });
    match task_future.await {
        Ok(Ok(_)) => {
            let duration = start.elapsed(); // Calculate the duration of the task execution.
            info!(
                "Task '{{{task_name}-{task_id}}}' in queue '{task_queue}' executed successfully, took {:?}",
                duration
            );
            TaskResult::success(task_meta.id, duration.as_millis() as usize)
            // Return success result.
        }
        Ok(Err(e)) => {
            let duration = start.elapsed();
            warn!("Task '{{{task_name}-{task_id}}}' in queue '{task_queue}' errored, {e:#?}");
            TaskResult::failure(task_meta.id, e, duration.as_millis() as usize)
            // Return failure result with the error.
        }
        Err(e) if e.is_panic() => {
            let duration = start.elapsed();
            warn!("Task '{{{task_name}-{task_id}}}' in queue '{task_queue}' panicked");
            TaskResult::failure(
                task_meta.id,
                raise_error!("Task panicked".into(), ErrorCode::InternalError),
                duration.as_millis() as usize,
            ) // Handle panic case.
        }
        Err(e) => {
            println!(
                "Task '{{{task_name}}}' in queue '{task_queue}' failed unexpectedly: {:?}",
                e
            );
            let duration = start.elapsed();
            TaskResult::failure(
                task_id,
                raise_error!(
                    format!("task failed unexpectedly: {:?}", e),
                    ErrorCode::InternalError
                ),
                duration.as_millis() as usize,
            )
            // Handle unexpected failure.
        }
    }
}
