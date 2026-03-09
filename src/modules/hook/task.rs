// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use std::collections::HashMap;
use std::time::Instant;

use crate::modules::common::http::HttpClient;
use crate::modules::error::code::ErrorCode;
use crate::modules::error::RustMailerError;
use crate::modules::hook::entity::EventHooks;
use crate::modules::hook::vrl::payload::VrlScriptTestRequest;
use crate::modules::hook::vrl::resolve_vrl_input;
use crate::modules::metrics::{
    FAILURE, RUSTMAILER_EVENT_DISPATCH_DURATION_SECONDS_BY_TYPE_STATUS_AND_DESTINATION,
    RUSTMAILER_EVENT_DISPATCH_TOTAL_BY_TYPE_STATUS_AND_DESTINATION, SUCCESS,
};
use crate::modules::scheduler::model::TaskStatus;
use crate::modules::scheduler::nativedb::TaskMetaEntity;
use crate::modules::tasks::queue::RustMailerTaskQueue;
use crate::utc_now;
use crate::{
    modules::{
        error::RustMailerResult,
        hook::{entity::HookType, nats::executor::NATS_EXECUTORS},
        scheduler::{
            retry::{RetryPolicy, RetryStrategy},
            task::{Task, TaskFuture},
        },
    },
    raise_error,
};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::modules::hook::events::EventType;

use super::payload::InternalEventHookUpdateRequest;

pub const EVENTHOOK_QUEUE: &str = "event_hook";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EventHookTask {
    pub event_hook_id: u64,
    pub account_id: u64,
    pub account_email: String,
    pub event_type: EventType,
    pub event: serde_json::Value,
}

impl EventHookTask {
    async fn event_watched(account_id: u64, event_type: EventType) -> RustMailerResult<bool> {
        let account_hook = EventHooks::get_by_account_id(account_id)
            .await?
            .map_or(false, |hook| {
                hook.enabled && hook.watched_events.contains(&event_type)
            });
        let global_hook = EventHooks::global_hooks()
            .await?
            .iter()
            .any(|hook| hook.enabled && hook.watched_events.contains(&event_type));

        Ok(account_hook || global_hook)
    }

    pub async fn is_watching_email_add_event(account_id: u64) -> RustMailerResult<bool> {
        EventHookTask::event_watched(account_id, EventType::EmailAddedToFolder).await
    }

    pub async fn is_watching_email_flags_changed(account_id: u64) -> RustMailerResult<bool> {
        EventHookTask::event_watched(account_id, EventType::EmailFlagsChanged).await
    }

    pub async fn is_watching_email_sent_success(account_id: u64) -> RustMailerResult<bool> {
        EventHookTask::event_watched(account_id, EventType::EmailSentSuccess).await
    }

    pub async fn is_watching_email_sending_error(account_id: u64) -> RustMailerResult<bool> {
        EventHookTask::event_watched(account_id, EventType::EmailSendingError).await
    }

    pub async fn is_watching_uid_validity_change(account_id: u64) -> RustMailerResult<bool> {
        EventHookTask::event_watched(account_id, EventType::UIDValidityChange).await
    }

    pub async fn is_watching_mailbox_deletion(account_id: u64) -> RustMailerResult<bool> {
        EventHookTask::event_watched(account_id, EventType::MailboxDeletion).await
    }

    pub async fn is_watching_mailbox_creation(account_id: u64) -> RustMailerResult<bool> {
        EventHookTask::event_watched(account_id, EventType::MailboxCreation).await
    }

    pub async fn is_watching_account_first_sync_completed(
        account_id: u64,
    ) -> RustMailerResult<bool> {
        EventHookTask::event_watched(account_id, EventType::AccountFirstSyncCompleted).await
    }

    pub async fn is_watching_email_bounce(account_id: u64) -> RustMailerResult<bool> {
        EventHookTask::event_watched(account_id, EventType::EmailBounce).await
    }

    pub async fn is_watching_email_feedback_report(account_id: u64) -> RustMailerResult<bool> {
        EventHookTask::event_watched(account_id, EventType::EmailFeedBackReport).await
    }

    pub async fn is_watching_email_opened(account_id: u64) -> RustMailerResult<bool> {
        EventHookTask::event_watched(account_id, EventType::EmailOpened).await
    }

    pub async fn is_watching_email_link_clicked(account_id: u64) -> RustMailerResult<bool> {
        EventHookTask::event_watched(account_id, EventType::EmailLinkClicked).await
    }

    pub async fn bounce_watched(account_id: u64) -> RustMailerResult<bool> {
        let target_events = &[EventType::EmailBounce, EventType::EmailFeedBackReport];

        let account_hook = EventHooks::get_by_account_id(account_id)
            .await?
            .map_or(false, |hook| {
                hook.enabled
                    && target_events
                        .iter()
                        .any(|e| hook.watched_events.contains(e))
            });
        let global_hook = EventHooks::global_hooks().await?.iter().any(|hook| {
            hook.enabled
                && target_events
                    .iter()
                    .any(|e| hook.watched_events.contains(e))
        });

        Ok(account_hook || global_hook)
    }

    pub async fn get_matching_hooks(
        account_id: u64,
        event_type: &EventType,
    ) -> RustMailerResult<Vec<EventHooks>> {
        let account_hook = EventHooks::get_by_account_id(account_id)
            .await?
            .filter(|hook| hook.enabled && hook.watched_events.contains(event_type));

        let global_hooks = EventHooks::global_hooks()
            .await?
            .into_iter()
            .filter(|hook| hook.enabled && hook.watched_events.contains(event_type))
            .collect::<Vec<_>>();

        let mut result = Vec::new();
        if let Some(hook) = account_hook {
            result.push(hook);
        }
        result.extend(global_hooks);

        Ok(result)
    }
}

impl Task for EventHookTask {
    const TASK_KEY: &'static str = "event_hook";
    const TASK_QUEUE: &'static str = EVENTHOOK_QUEUE;

    fn delay_seconds(&self) -> u32 {
        0
    }

    fn retry_policy(&self) -> RetryPolicy {
        RetryPolicy {
            strategy: RetryStrategy::Exponential { base: 2 },
            max_retries: Some(10),
        }
    }

    fn run(self, task_id: u64) -> TaskFuture {
        Box::pin(async move {
            // Increment call count
            let event_hook = match EventHooks::get_by_id(self.event_hook_id).await {
                Ok(Some(hook)) => hook,
                Ok(None) => {
                    info!(
                        "Event hook no longer exists or No event hook configured, event hook id: '{}'",
                        self.event_hook_id
                    );

                    //now stop this task
                    let send_queue = RustMailerTaskQueue::get().unwrap();
                    send_queue
                        .stop_task(
                            task_id,
                            Some(
                                "Event hook no longer exists or No event hook configured, aborting task execution"
                                    .into(),
                            ),
                        )
                        .await?;

                    return Err(raise_error!(
                        "Event hook no longer exists or No event hook configured.".into(),
                        ErrorCode::ResourceNotFound
                    ));
                }
                Err(e) => {
                    return Err(raise_error!(
                        format!("Failed to get event hook: {}", e),
                        ErrorCode::ResourceNotFound
                    ));
                }
            };
            let destination = event_hook.hook_type.as_str();
            EventHooks::internal_update(
                self.event_hook_id,
                InternalEventHookUpdateRequest {
                    increase_call_count: Some(true),
                    ..Default::default()
                },
            )
            .await?;
            let start = Instant::now();

            match send_event(task_id, self.event, self.event_type, event_hook).await {
                Ok(()) => {
                    let update = InternalEventHookUpdateRequest {
                        increase_success_count: Some(true),
                        ..Default::default()
                    };
                    RUSTMAILER_EVENT_DISPATCH_TOTAL_BY_TYPE_STATUS_AND_DESTINATION
                        .with_label_values(&[SUCCESS, destination])
                        .inc();
                    let elapsed = start.elapsed();
                    RUSTMAILER_EVENT_DISPATCH_DURATION_SECONDS_BY_TYPE_STATUS_AND_DESTINATION
                        .with_label_values(&[SUCCESS, destination])
                        .observe(elapsed.as_secs_f64());
                    EventHooks::internal_update(self.event_hook_id, update).await?;
                    Ok(())
                }
                Err(err) => {
                    let error_msg = err.to_string();
                    RUSTMAILER_EVENT_DISPATCH_TOTAL_BY_TYPE_STATUS_AND_DESTINATION
                        .with_label_values(&[FAILURE, destination])
                        .inc();
                    let elapsed = start.elapsed();
                    RUSTMAILER_EVENT_DISPATCH_DURATION_SECONDS_BY_TYPE_STATUS_AND_DESTINATION
                        .with_label_values(&[FAILURE, destination])
                        .observe(elapsed.as_secs_f64());
                    let update = InternalEventHookUpdateRequest {
                        increase_failure_count: Some(true),
                        last_error: Some(error_msg.clone()),
                        ..Default::default()
                    };

                    EventHooks::internal_update(self.event_hook_id, update).await?;
                    Err(err)
                }
            }
        })
    }
}

async fn process_payload(
    event: serde_json::Value,
    vrl_script: Option<String>,
) -> RustMailerResult<serde_json::Value> {
    match vrl_script {
        Some(script) => {
            let json_str = event.to_string();
            let request = VrlScriptTestRequest {
                program: script,
                event: Some(json_str),
            };
            let result = resolve_vrl_input(request).await?;
            result.result.ok_or_else(|| {
                raise_error!(
                    format!("VRL script error: {:#?}", result.error),
                    ErrorCode::InternalError
                )
            })
        }
        None => Ok(event),
    }
}

async fn send_event(
    task_id: u64,
    event: serde_json::Value,
    event_type: EventType,
    event_hook: EventHooks,
) -> RustMailerResult<()> {
    let task = RustMailerTaskQueue::get()?
        .get_hook_task(task_id)
        .await?
        .map(|t| t.headers());
    match event_hook.hook_type {
        HookType::Http => {
            let http_config = event_hook.http.ok_or_else(|| {
                raise_error!(
                    "Missing HTTP config in event hook".into(),
                    ErrorCode::MissingConfiguration
                )
            })?;

            let headers = (!http_config.custom_headers.is_empty())
                .then(|| http_config.custom_headers.into_iter().collect());
            let payload = process_payload(event, event_hook.vrl_script).await?;

            if payload != serde_json::Value::Null {
                let client = HttpClient::new(event_hook.use_proxy).await?;
                let response = client
                    .send_json_request(
                        task,
                        http_config.http_method,
                        &http_config.target_url,
                        &payload,
                        headers,
                    )
                    .await?;

                handle_response(response).await?;
            }
            Ok(())
        }
        HookType::Nats => {
            let nats_config = event_hook.nats.ok_or_else(|| {
                raise_error!(
                    "Missing NATS config in event hook".into(),
                    ErrorCode::MissingConfiguration
                )
            })?;

            let executor = NATS_EXECUTORS.get(&nats_config).await?;
            let payload = process_payload(event, event_hook.vrl_script).await?;

            if payload != serde_json::Value::Null {
                executor.publish(task, event_type, payload).await?;
            }
            Ok(())
        }
    }
}

async fn handle_response(response: reqwest::Response) -> RustMailerResult<()> {
    if !response.status().is_success() {
        let status = response.status();
        let url = response.url().clone();
        let headers = response.headers().clone();

        let body = match response.text().await {
            Ok(text) => text,
            Err(e) => format!("<failed to read body: {}>", e),
        };

        // Log detailed error information
        tracing::error!(
            status = %status,
            url = %url,
            headers = ?headers,
            body = %body,
            "HTTP request failed with error response"
        );

        return Err(raise_error!(
            format!("Error response: {} - {}", status, body),
            ErrorCode::HttpResponseError
        ));
    }

    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, Object)]
pub struct SendEventHookTask {
    pub id: u64,
    pub created_at: i64,
    pub status: TaskStatus,
    pub stopped_reason: Option<String>,
    pub error: Option<String>,
    pub last_duration_ms: Option<usize>,
    pub retry_count: Option<usize>,
    pub scheduled_at: i64,
    pub account_id: u64,
    pub account_email: String,
    pub event: serde_json::Value,
    pub event_type: EventType,
}

impl SendEventHookTask {
    pub fn headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        // Basic task identification
        headers.insert("X-Task-Id".into(), self.id.to_string());
        headers.insert(
            "X-Task-Delay-MS".into(),
            (utc_now!() - self.created_at).to_string(),
        );
        if let Some(retry_count) = self.retry_count {
            headers.insert("X-Task-Retry-Count".into(), retry_count.to_string());
        }

        headers
    }
}

impl TryFrom<&TaskMetaEntity> for SendEventHookTask {
    type Error = RustMailerError;

    fn try_from(task: &TaskMetaEntity) -> RustMailerResult<Self> {
        // Deserialize the task_params string into EventHookTask
        let event_hook_task: EventHookTask = serde_json::from_str(&task.task_params)
            .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?;

        Ok(SendEventHookTask {
            id: task.id,
            created_at: task.created_at,
            status: task.status.clone(),
            stopped_reason: task.stopped_reason.clone(),
            error: task.last_error.clone(),
            last_duration_ms: task.last_duration_ms,
            retry_count: task.retry_count,
            scheduled_at: task.next_run,
            account_id: event_hook_task.account_id,
            account_email: event_hook_task.account_email,
            event: event_hook_task.event,
            event_type: event_hook_task.event_type,
        })
    }
}
