// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use std::collections::BTreeSet;

use crate::modules::common::auth::ClientContext;
use crate::modules::common::paginated::paginate_vec;
use crate::modules::error::code::ErrorCode;
use crate::modules::hook::entity::EventHooks;
use crate::modules::hook::events::EVENT_EXAMPLES;
use crate::modules::hook::payload::{EventhookCreateRequest, EventhookUpdateRequest};
use crate::modules::hook::task::SendEventHookTask;
use crate::modules::hook::vrl::payload::{ResolveResult, VrlScriptTestRequest};
use crate::modules::hook::vrl::resolve_vrl_input;
use crate::modules::rest::api::ApiTags;
use crate::modules::rest::response::DataPage;
use crate::modules::rest::ApiResult;
use crate::modules::scheduler::model::TaskStatus;
use crate::modules::tasks::queue::RustMailerTaskQueue;
use crate::raise_error;
use poem::web::Path;
use poem_openapi::param::Query;
use poem_openapi::payload::Json;
use poem_openapi::OpenApi;
pub struct EventHookApi;

#[OpenApi(prefix_path = "/api/v1", tag = "ApiTags::Hook")]
impl EventHookApi {
    /// Retrieve an event hook configuration
    #[oai(
        path = "/event-hook/:id",
        method = "get",
        operation_id = "get_event_hook"
    )]
    async fn get_event_hook(
        &self,
        id: Path<u64>,
        context: ClientContext,
    ) -> ApiResult<Json<EventHooks>> {
        let id = id.0;
        let hook = EventHooks::get_by_id(id).await?.ok_or_else(|| {
            raise_error!(
                format!("Failed to retrieve webhook record. id: {id}."),
                ErrorCode::ResourceNotFound
            )
        })?;
        match hook.account_id {
            Some(account_id) => {
                context.require_account_access(account_id)?;
            }
            None => {
                context.require_root()?;
            }
        }
        Ok(Json(hook))
    }

    /// Delete an event hook configuration
    #[oai(
        path = "/event-hook/:id",
        method = "delete",
        operation_id = "remove_event_hook"
    )]
    async fn remove_event_hook(
        &self,
        ///The email account identifier
        id: Path<u64>,
        context: ClientContext,
    ) -> ApiResult<()> {
        let id = id.0;
        let hook = EventHooks::get_by_id(id).await?.ok_or_else(|| {
            raise_error!(
                format!("Failed to retrieve webhook record. id: {id}."),
                ErrorCode::ResourceNotFound
            )
        })?;
        match hook.account_id {
            Some(account_id) => {
                context.require_account_access(account_id)?;
            }
            None => {
                context.require_root()?;
            }
        }

        Ok(EventHooks::delete(id).await?)
    }

    /// Create a new event hook
    #[oai(
        path = "/event-hook",
        method = "post",
        operation_id = "create_event_hook"
    )]
    async fn create_event_hook(
        &self,
        ///Request Body
        payload: Json<EventhookCreateRequest>,
        context: ClientContext,
    ) -> ApiResult<Json<EventHooks>> {
        let payload = payload.0;
        match payload.account_id {
            Some(account_id) => {
                context.require_account_access(account_id)?;
            }
            None => {
                context.require_root()?;
            }
        }

        let entity = EventHooks::new(payload).await?;
        entity.clone().save().await?;
        Ok(Json(entity))
    }

    /// Update an event hook
    #[oai(
        path = "/event-hook/:id",
        method = "post",
        operation_id = "update_event_hook"
    )]
    async fn update_event_hook(
        &self,
        ///Request Body
        payload: Json<EventhookUpdateRequest>,
        ///The email account identifier
        id: Path<u64>,
        context: ClientContext,
    ) -> ApiResult<()> {
        let id = id.0;
        let hook = EventHooks::get_by_id(id).await?.ok_or_else(|| {
            raise_error!(
                format!("Failed to retrieve webhook record. id: {id}."),
                ErrorCode::ResourceNotFound
            )
        })?;
        match hook.account_id {
            Some(account_id) => {
                context.require_account_access(account_id)?;
            }
            None => {
                context.require_root()?;
            }
        }
        Ok(EventHooks::update(id, payload.0).await?)
    }

    /// List event hooks (root)
    /// Requires root privileges.
    #[oai(
        path = "/event-hook-list",
        method = "get",
        operation_id = "list_event_hook"
    )]
    async fn list_event_hook(
        &self,
        /// Optional. The page number to retrieve (starting from 1).
        page: Query<Option<u64>>,
        /// Optional. The number of items per page.
        page_size: Query<Option<u64>>,
        /// Optional. Whether to sort the list in descending order.
        desc: Query<Option<bool>>,
        context: ClientContext,
    ) -> ApiResult<Json<DataPage<EventHooks>>> {
        context.require_root()?;
        Ok(Json(
            EventHooks::paginate_list(page.0, page_size.0, desc.0).await?,
        ))
    }

    /// Get event examples
    #[oai(
        path = "/event-examples",
        method = "get",
        operation_id = "event_examples"
    )]
    async fn event_examples(&self) -> ApiResult<Json<serde_json::Value>> {
        Ok(Json(EVENT_EXAMPLES.clone()))
    }

    /// Test and debug VRL transformation scripts
    #[oai(
        path = "/vrl-script-resolve",
        method = "post",
        operation_id = "vrl_script_resolve"
    )]
    async fn vrl_script_resolve(
        &self,
        /// JSON body containing the script and input context.
        request: Json<VrlScriptTestRequest>,
    ) -> ApiResult<Json<ResolveResult>> {
        Ok(Json(resolve_vrl_input(request.0).await?))
    }

    /// List hook tasks
    #[oai(path = "/hook-tasks", method = "get", operation_id = "list_hook_tasks")]
    async fn list_hook_tasks(
        &self,
        /// Optional. The page number to retrieve (starting from 1).
        page: Query<Option<u64>>,
        /// Optional. The number of items per page.
        page_size: Query<Option<u64>>,
        /// Optional. Whether to sort the list in descending order.
        desc: Query<Option<bool>>,
        ///Filter by task status (optional)
        status: Query<Option<TaskStatus>>,
        context: ClientContext,
    ) -> ApiResult<Json<DataPage<SendEventHookTask>>> {
        let send_queue = RustMailerTaskQueue::get().unwrap();
        let status = status.0;
        let sort_desc = desc.0.unwrap_or(true);

        if context.accessible_accounts()?.is_none() {
            let tasks = match status {
                Some(status) => {
                    send_queue
                        .list_paged_hook_tasks_by_status(
                            page.0,
                            page_size.0,
                            Some(sort_desc),
                            status,
                        )
                        .await?
                }
                None => {
                    send_queue
                        .list_paginated_hook_tasks(page.0, page_size.0, Some(sort_desc))
                        .await?
                }
            };

            return Ok(Json(tasks));
        }

        let accessible_accounts = context.accessible_accounts()?.unwrap();
        let allowed_ids: BTreeSet<u64> = accessible_accounts.iter().map(|a| a.id).collect();

        // Fetch all relevant tasks in one go
        let all_tasks = match status {
            Some(status) => send_queue.list_hook_tasks_by_status(status).await?,
            None => send_queue.list_all_hook_tasks().await?,
        };

        // Filter and sort
        let mut filtered_tasks: Vec<SendEventHookTask> = all_tasks
            .into_iter()
            .filter(|task| allowed_ids.contains(&task.account_id))
            .collect();

        filtered_tasks.sort_by(|a, b| {
            if sort_desc {
                b.created_at.cmp(&a.created_at)
            } else {
                a.created_at.cmp(&b.created_at)
            }
        });

        Ok(Json(
            paginate_vec(&filtered_tasks, page.0, page_size.0).map(DataPage::from)?,
        ))
    }

    /// Get hook task details
    #[oai(
        path = "/hook-task/:id",
        method = "get",
        operation_id = "get_hook_task"
    )]
    async fn get_hook_task(
        &self,
        ///The task identifier
        id: Path<u64>,
        context: ClientContext,
    ) -> ApiResult<Json<SendEventHookTask>> {
        let send_queue = RustMailerTaskQueue::get().unwrap();
        let result = send_queue
            .get_hook_task(id.0)
            .await?
            .ok_or_else(|| raise_error!("Task not found".into(), ErrorCode::ResourceNotFound))?;
        context.require_account_access(result.account_id)?;
        Ok(Json(result))
    }

    /// Mark a hook task for deletion from queue
    /// Initiates asynchronous removal of an event hook task by marking it for deletion.
    /// The task will be:
    /// 1. Immediately marked as "cancelled" in the system
    /// 2. Prevented from any further execution attempts
    /// 3. Physically removed by background cleanup processes
    #[oai(
        path = "/hook-task/:id",
        method = "delete",
        operation_id = "remove_hook_task"
    )]
    async fn remove_hook_task(
        &self,
        ///The task identifier
        id: Path<u64>,
        context: ClientContext,
    ) -> ApiResult<()> {
        let send_queue = RustMailerTaskQueue::get().unwrap();
        let id = id.0;
        let task = send_queue
            .get_hook_task(id)
            .await?
            .ok_or_else(|| raise_error!("Task not found".into(), ErrorCode::ResourceNotFound))?;
        context.require_account_access(task.account_id)?;
        Ok(send_queue.remove_task(id).await?)
    }
}
