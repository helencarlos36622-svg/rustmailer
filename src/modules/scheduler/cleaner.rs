// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::scheduler::periodic::PeriodicTask;
use crate::modules::scheduler::store::TaskStore;
use std::{sync::Arc, time::Duration};

pub struct TaskCleaner<T>
where
    T: TaskStore,
{
    pub task_store: Arc<T>,
    periodic_task: PeriodicTask,
}

impl<T> TaskCleaner<T>
where
    T: TaskStore + Sync + Send + 'static,
{
    pub fn new(task_store: Arc<T>) -> Self {
        Self {
            task_store: task_store.clone(),
            periodic_task: PeriodicTask::new("task-cleaner"),
        }
    }

    pub fn start(self, interval: Duration) {
        let task = move |_: Option<u64>| {
            let task_store = self.task_store.clone();
            Box::pin(async move { task_store.cleanup().await })
        };
        self.periodic_task.start(task, None, interval, false, false);
    }
}
