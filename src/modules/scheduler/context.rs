// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::scheduler::cleaner::TaskCleaner;
use crate::modules::scheduler::flow::TaskFlow;
use crate::modules::scheduler::handlers::TaskHandlers;
use crate::modules::scheduler::store::TaskStore;
use crate::modules::scheduler::task::Task;
use crate::modules::scheduler::updater::TaskStatusUpdater;
use crate::utc_now;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Duration;

pub struct TaskContext<S>
where
    S: TaskStore + Send + Sync + Clone + 'static, // Ensures that S is a type that implements the TaskStore trait
{
    queue_concurrency: DashMap<String, usize>, // Stores the concurrency level for each task queue
    handlers: TaskHandlers, // Collection of task handlers to process different task types
    store: Arc<S>, // Arc wrapper around the task store, allowing shared ownership across threads
}

impl<S> TaskContext<S>
where
    S: TaskStore + Send + Sync + Clone + 'static, // S must implement TaskStore, and be Sync and Send
{
    // /// Creates a new TaskContext with the provided store.
    // pub fn new(store: S) -> Self {
    //     let store = Arc::new(store);
    //     Self {
    //         queue_concurrency: AHashMap::new(), // Initialize concurrency map as empty
    //         handlers: TaskHandlers::new(),      // Create a new TaskHandlers instance
    //         store: store.clone(),               // Wrap the store in an Arc for shared ownership
    //     }
    // }

    /// Creates a new TaskContext with the provided Arc-wrapped store.
    pub fn with_arc_store(store: Arc<S>) -> Self {
        Self {
            queue_concurrency: DashMap::new(), // Initialize concurrency map as empty
            handlers: TaskHandlers::new(),      // Create a new TaskHandlers instance
            store,                              // Use the provided Arc directly
        }
    }

    /// Registers a new task type in the context.
    pub fn register<T>(mut self) -> Self
    where
        T: Task, // T must implement the Task trait
    {
        self.handlers.register::<T>(); // Register the task handler
        self.queue_concurrency.insert(T::TASK_QUEUE.to_owned(), 4); // Set default concurrency for the task queue
        self
    }

    /// Sets the concurrency level for a specified queue.
    pub fn set_concurrency(self, queue: &str, count: usize) -> Self {
        self.queue_concurrency.insert(queue.to_owned(), count); // Update the concurrency level for the queue
        self
    }

    /// Starts the task cleaner to periodically clean up tasks.
    fn start_task_cleaner(&self) {
        let cleaner = TaskCleaner::new(self.store.clone());
        cleaner.start(Duration::from_secs(60 * 10)); // Start the cleaner to run every 10 minutes
    }

    /// Starts worker threads for processing tasks in each queue.
    async fn start_flow(&self) {
        let status_updater = Arc::new(TaskStatusUpdater::new(
            self.store.clone(),
            self.queue_concurrency.len(),
        ));

        let flow = Arc::new(TaskFlow::new(
            self.store.clone(),
            &self.queue_concurrency,
            Arc::new(self.handlers.clone()),
            status_updater,
        ));

        flow.start().await;
    }

    /// Starts the task context with workers, leaving task cleanup to be handled manually by the user.
    // pub async fn start(self) -> Self {
    //     self.start_flow().await; // Start task workers
    //     self
    // }

    /// Runs the task context, enabling workers and the task cleaner.
    pub async fn start_with_cleaner(self) -> Self {
        self.start_flow().await; // Start task workers
        self.start_task_cleaner(); // Start the task cleaner
        self
    }

    /// Adds a new task to the context for execution.
    pub async fn add_task<T>(&self, task: T, delay_seconds: Option<u32>) -> Result<(), String>
    where
        T: Task + Send + Sync + 'static, // T must implement the Task trait and be thread-safe
    {
        let mut task_meta = task.new_meta(); // Create metadata for the new task
        let delay_seconds = delay_seconds.unwrap_or(task_meta.delay_seconds) * 1000;
        let next_run = utc_now!() + delay_seconds as i64;
        task_meta.next_run = next_run;
        self.store
            .store_task(task_meta) // Store the task metadata in the task store
            .await
            .map_err(|e| format!("{:#?}", e)) // Handle any errors during the store operation
    }

    pub async fn add_tasks<T>(&self, tasks: &[T], delay_seconds: Option<u32>) -> Result<(), String>
    where
        T: Task + Send + Sync + 'static,
    {
        let task_metas = tasks
            .iter()
            .map(|t| {
                let mut task_meta = t.new_meta();
                let delay_ms = delay_seconds.unwrap_or(task_meta.delay_seconds) * 1000;
                task_meta.next_run = utc_now!() + delay_ms as i64;
                task_meta
            })
            .collect::<Vec<_>>();

        self.store
            .store_tasks(task_metas)
            .await
            .map_err(|e| format!("Failed to store tasks: {e:#?}"))
    }

    /// stop a task
    pub async fn stop_task(
        &self,
        task_id: u64,
        stop_reason: Option<String>,
    ) -> Result<(), String> {
        self.store
            .set_task_stopped(task_id, stop_reason)
            .await
            .map_err(|e| format!("{:#?}", e)) // Handle any errors during the store operation
    }
}
