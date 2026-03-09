// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use std::{sync::LazyLock, time::Duration};

use tokio::{sync::mpsc, time::Instant};
use tracing::error;

use crate::modules::{
    error::RustMailerResult,
    hook::{events::RustMailerEvent, task::EventHookTask},
    tasks::queue::RustMailerTaskQueue,
};

pub static EVENT_CHANNEL: LazyLock<EventChannel> = LazyLock::new(EventChannel::new);

const BATCH_SIZE: usize = 50;

#[derive(Debug)]
pub struct Event {
    account_id: u64,
    account_email: String,
    event: RustMailerEvent,
}

impl Event {
    pub fn new(account_id: u64, account_email: &str, event: RustMailerEvent) -> Self {
        Self {
            account_id,
            account_email: account_email.into(),
            event,
        }
    }
}

pub struct EventChannel {
    sender: mpsc::Sender<Event>,
}

impl EventChannel {
    pub async fn queue(&self, event: Event) {
        if let Err(e) = self.sender.send(event).await {
            error!("Failed to queue event. Channel error: {:#?}", e);
        }
    }

    pub fn new() -> Self {
        let (sender, mut receiver) = mpsc::channel::<Event>(1000);
        let instance = EventChannel { sender };
        let mut buffer: Vec<Event> = Vec::with_capacity(BATCH_SIZE);
        let mut last_flush_time = Instant::now();

        tokio::spawn({
            async move {
                loop {
                    tokio::select! {
                        maybe_event = receiver.recv() => {
                            match maybe_event {
                                Some(e) => buffer.push(e),
                                None => break, // channel closed
                            }
                        }
                        _ = tokio::time::sleep(Duration::from_secs(3)) => {
                        }
                    }

                    let should_flush = buffer.len() >= BATCH_SIZE
                        || last_flush_time.elapsed() >= Duration::from_secs(3);
                    if should_flush && !buffer.is_empty() {
                        match Self::handle(&buffer).await {
                            Ok(_) => tracing::debug!(
                                "Successfully processed batch of {} messages",
                                buffer.len()
                            ),
                            Err(e) => tracing::error!("Error processing batch: {:?}", e),
                        }
                        buffer.clear();
                        last_flush_time = Instant::now();
                    }
                }

                if !buffer.is_empty() {
                    if let Err(e) = Self::handle(&buffer).await {
                        tracing::error!("Error processing final batch: {:?}", e);
                    }
                }
            }
        });

        instance
    }

    pub async fn handle(events: &[Event]) -> RustMailerResult<()> {
        let mut all_tasks = Vec::new();

        for event in events {
            let hooks =
                EventHookTask::get_matching_hooks(event.account_id, &event.event.event_type)
                    .await?;

            for h in hooks {
                all_tasks.push(EventHookTask {
                    event_hook_id: h.id,
                    account_id: event.account_id,
                    account_email: event.account_email.clone(),
                    event_type: event.event.event_type.clone(),
                    event: event.event.to_json_value().unwrap(),
                });
            }
        }

        let task_queue = RustMailerTaskQueue::get()?;
        for chunk in all_tasks.chunks(BATCH_SIZE) {
            task_queue.submit_tasks(chunk, None).await?;
        }

        Ok(())
    }
}
