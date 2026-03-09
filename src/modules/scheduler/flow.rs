// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::common::signal::SIGNAL_MANAGER;
use crate::modules::scheduler::model::TaskMeta;
use crate::modules::scheduler::processor::Package;
use crate::modules::scheduler::store::TaskStore;
use crate::modules::scheduler::{handlers, processor::TaskProcessor, updater::TaskStatusUpdater};
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

pub struct TaskFlow<T>
where
    T: TaskStore + Send + Sync + Clone + 'static,
{
    task_store: Arc<T>,
    processors: Arc<DashMap<String, TaskProcessor>>,
}

impl<T> TaskFlow<T>
where
    T: TaskStore + Send + Sync + Clone + 'static,
{
    pub fn new(
        task_store: Arc<T>,
        queue_concurrency: &DashMap<String, usize>,
        handlers: Arc<handlers::TaskHandlers>,
        status_updater: Arc<TaskStatusUpdater>,
    ) -> Self {
        let processors = DashMap::new();
        //create processor for each queue
        for entry in queue_concurrency.iter() {
            let queue = entry.key().to_string();
            let processor = TaskProcessor::new(
                queue.clone(),
                *entry.value(),
                handlers.clone(),
                status_updater.clone(),
            );
            processors.insert(queue, processor);
        }

        Self {
            task_store,
            processors: Arc::new(processors),
        }
    }

    pub async fn start(self: Arc<Self>) {
        let task_store = self.task_store.clone();
        let processors = self.processors.clone();
        let mut shutdown = SIGNAL_MANAGER.subscribe();
        let mut interval = tokio::time::interval(Duration::from_millis(200));
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        match task_store.clone().fetch_pending_tasks().await {
                            Ok(tasks) => {
                                let mut queued_tasks: HashMap<String, Vec<TaskMeta>> = HashMap::new();
                                for task in tasks {
                                    queued_tasks
                                        .entry(task.queue_name.clone())
                                        .or_default()
                                        .push(task);
                                }

                                for (queue, tasks) in queued_tasks {
                                    if let Err(e) = Self::send_tasks_to_channel(processors.clone(), &queue, tasks).await {
                                        error!(
                                            "Error sending tasks to channel for queue '{}': {:?}",
                                            queue, e
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to fetch tasks: {:?}", e);
                            }
                        }
                    }
                    _ = shutdown.recv() => {
                        info!("Stop to fetch pending tasks.");
                        Self::send_poison(processors.clone()).await;
                        break;
                    }

                }
            }
        });
    }

    async fn send_tasks_to_channel(
        processors: Arc<DashMap<String, TaskProcessor>>,
        queue_name: &str,
        tasks: Vec<TaskMeta>,
    ) -> Result<(), String> {
        let processor = processors.get(queue_name).ok_or_else(|| format!(
            "Processor for queue '{}' not found. You may have forgotten to call `.register::<MyTask>()` on the TaskContext instance.",
            queue_name
        ))?;

        for task in tasks {
            processor.accept(Package::task(task)).await;
        }

        Ok(())
    }

    async fn send_poison(processors: Arc<DashMap<String, TaskProcessor>>) {
        for entry in processors.iter() {
            entry.value().accept(Package::PoisonPill).await;
        }
    }
}
