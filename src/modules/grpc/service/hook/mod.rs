// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use std::{collections::BTreeSet, sync::Arc};

use poem_grpc::{Request, Response, Status};

use crate::{
    modules::{
        common::{auth::ClientContext, paginated::paginate_vec},
        error::code::ErrorCode,
        grpc::service::rustmailer_grpc::{
            CreateEventHookRequest, Empty, EventHookTask, EventHooks, EventHooksService,
            GetEventHookRequest, GetTaskRequest, ListEventHookRequest, ListTasksRequest,
            PagedEventHookTask, PagedEventHooks, RemoveEventHookRequest, RemoveTaskRequest,
            ResolveResult, UpdateEventhookRequest, VrlScriptTestRequest,
        },
        hook::{events::EVENT_EXAMPLES, vrl::resolve_vrl_input},
        rest::response::DataPage,
        scheduler::model::TaskStatus,
        tasks::queue::RustMailerTaskQueue,
        utils::json_value_to_prost_value,
    },
    raise_error,
};

use crate::modules::hook::entity::EventHooks as RustMailerEventHooks;
use crate::modules::hook::task::SendEventHookTask as RustMailerQueuedEventHookTask;

mod from;

#[derive(Default)]
pub struct RustMailerEventHooksService;

impl EventHooksService for RustMailerEventHooksService {
    async fn get_event_hook(
        &self,
        request: Request<GetEventHookRequest>,
    ) -> Result<Response<EventHooks>, Status> {
        let extensions = request.extensions().clone();
        let req = request.into_inner();

        // Get ClientContext from cloned extensions
        let context = extensions.get::<Arc<ClientContext>>().ok_or_else(|| {
            raise_error!("Missing ClientContext".into(), ErrorCode::InternalError)
        })?;

        let result = RustMailerEventHooks::get_by_id(req.id)
            .await?
            .ok_or_else(|| {
                raise_error!("event hook not found".into(), ErrorCode::ResourceNotFound)
            })?;

        match result.account_id {
            Some(account_id) => {
                context.require_account_access(account_id)?;
            }
            None => {
                context.require_root()?;
            }
        }

        Ok(Response::new(result.into()))
    }

    async fn remove_event_hook(
        &self,
        request: Request<RemoveEventHookRequest>,
    ) -> Result<Response<Empty>, Status> {
        let extensions = request.extensions().clone();
        let req = request.into_inner();

        // Get ClientContext from cloned extensions
        let context = extensions.get::<Arc<ClientContext>>().ok_or_else(|| {
            raise_error!("Missing ClientContext".into(), ErrorCode::InternalError)
        })?;

        let result = RustMailerEventHooks::get_by_id(req.id)
            .await?
            .ok_or_else(|| {
                raise_error!("event hook not found".into(), ErrorCode::ResourceNotFound)
            })?;

        match result.account_id {
            Some(account_id) => {
                context.require_account_access(account_id)?;
            }
            None => {
                context.require_root()?;
            }
        }
        RustMailerEventHooks::delete(result.id).await?;
        Ok(Response::new(Empty::default()))
    }

    async fn create_event_hook(
        &self,
        request: Request<CreateEventHookRequest>,
    ) -> Result<Response<EventHooks>, Status> {
        let extensions = request.extensions().clone();
        let req = request.into_inner();

        // Get ClientContext from cloned extensions
        let context = extensions.get::<Arc<ClientContext>>().ok_or_else(|| {
            raise_error!("Missing ClientContext".into(), ErrorCode::InternalError)
        })?;
        match req.account_id {
            Some(account_id) => {
                context.require_account_access(account_id)?;
            }
            None => {
                context.require_root()?;
            }
        }
        let entity =
            RustMailerEventHooks::new(req.try_into().map_err(|e: &'static str| {
                raise_error!(e.to_string(), ErrorCode::InvalidParameter)
            })?)
            .await?;
        entity.clone().save().await?;
        Ok(Response::new(entity.into()))
    }

    async fn update_event_hook(
        &self,
        request: Request<UpdateEventhookRequest>,
    ) -> Result<Response<Empty>, Status> {
        let extensions = request.extensions().clone();
        let req = request.into_inner();

        // Get ClientContext from cloned extensions
        let context = extensions.get::<Arc<ClientContext>>().ok_or_else(|| {
            raise_error!("Missing ClientContext".into(), ErrorCode::InternalError)
        })?;

        let hook = RustMailerEventHooks::get_by_id(req.id)
            .await?
            .ok_or_else(|| {
                raise_error!(
                    "the event hook you want to modify does not exist".into(),
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
        RustMailerEventHooks::update(
            hook.id,
            req.try_into().map_err(|e: &'static str| {
                raise_error!(e.to_string(), ErrorCode::InvalidParameter)
            })?,
        )
        .await?;
        Ok(Response::new(Empty::default()))
    }

    async fn list_event_hook(
        &self,
        request: Request<ListEventHookRequest>,
    ) -> Result<Response<PagedEventHooks>, Status> {
        let extensions = request.extensions().clone();
        let req = request.into_inner();

        // Get ClientContext from cloned extensions
        let context = extensions.get::<Arc<ClientContext>>().ok_or_else(|| {
            raise_error!("Missing ClientContext".into(), ErrorCode::InternalError)
        })?;

        context.require_root()?;
        let result = RustMailerEventHooks::paginate_list(req.page, req.page_size, req.desc).await?;

        Ok(Response::new(result.into()))
    }

    async fn event_examples(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<::prost_types::Value>, Status> {
        Ok(Response::new(json_value_to_prost_value(
            EVENT_EXAMPLES.clone(),
        )))
    }

    async fn vrl_script_resolve(
        &self,
        request: Request<VrlScriptTestRequest>,
    ) -> Result<Response<ResolveResult>, Status> {
        let req = request.into_inner();
        let result = resolve_vrl_input(req.into()).await?;
        Ok(Response::new(result.into()))
    }

    async fn list_event_hook_tasks(
        &self,
        request: Request<ListTasksRequest>,
    ) -> Result<Response<PagedEventHookTask>, Status> {
        let extensions = request.extensions().clone();
        let req = request.into_inner();

        // Get ClientContext from cloned extensions
        let context = extensions.get::<Arc<ClientContext>>().ok_or_else(|| {
            raise_error!("Missing ClientContext".into(), ErrorCode::InternalError)
        })?;

        let accessible_accounts = context.accessible_accounts()?;

        let send_queue = RustMailerTaskQueue::get().unwrap();

        let status = req
            .status
            .map(|i| TaskStatus::try_from(i))
            .transpose()
            .map_err(|e| raise_error!(e.to_string(), ErrorCode::InvalidParameter))?;

        if accessible_accounts.is_none() {
            let result = match status {
                Some(status) => {
                    send_queue
                        .list_paged_hook_tasks_by_status(req.page, req.page_size, req.desc, status)
                        .await?
                }
                None => {
                    send_queue
                        .list_paginated_hook_tasks(req.page, req.page_size, req.desc)
                        .await?
                }
            };
            return Ok(Response::new(result.into()));
        }

        let all_tasks = match status {
            Some(status) => send_queue.list_hook_tasks_by_status(status).await?,
            None => send_queue.list_all_hook_tasks().await?,
        };

        let allowed_ids: BTreeSet<u64> =
            accessible_accounts.unwrap().iter().map(|a| a.id).collect();

        let mut filtered_tasks: Vec<RustMailerQueuedEventHookTask> = all_tasks
            .into_iter()
            .filter(|acct| allowed_ids.contains(&acct.account_id))
            .collect();

        let sort_desc = req.desc.unwrap_or(true);
        filtered_tasks.sort_by(|a, b| {
            if sort_desc {
                b.created_at.cmp(&a.created_at)
            } else {
                a.created_at.cmp(&b.created_at)
            }
        });
        let page_data =
            paginate_vec(&filtered_tasks, req.page, req.page_size).map(DataPage::from)?;

        Ok(Response::new(page_data.into()))
    }

    async fn get_event_hook_task(
        &self,
        request: Request<GetTaskRequest>,
    ) -> Result<Response<EventHookTask>, Status> {
        let extensions = request.extensions().clone();
        let req = request.into_inner();

        // Get ClientContext from cloned extensions
        let context = extensions.get::<Arc<ClientContext>>().ok_or_else(|| {
            raise_error!("Missing ClientContext".into(), ErrorCode::InternalError)
        })?;
        let send_queue = RustMailerTaskQueue::get().unwrap();
        let result = send_queue
            .get_hook_task(req.id)
            .await?
            .ok_or_else(|| raise_error!("task not found".into(), ErrorCode::ResourceNotFound))?;

        // Check account access
        context.require_account_access(result.account_id)?;
        Ok(Response::new(result.into()))
    }

    async fn remove_event_hook_task(
        &self,
        request: Request<RemoveTaskRequest>,
    ) -> Result<Response<Empty>, Status> {
        let extensions = request.extensions().clone();
        let req = request.into_inner();

        // Get ClientContext from cloned extensions
        let context = extensions.get::<Arc<ClientContext>>().ok_or_else(|| {
            raise_error!("Missing ClientContext".into(), ErrorCode::InternalError)
        })?;
        let send_queue = RustMailerTaskQueue::get().unwrap();

        let result = send_queue
            .get_hook_task(req.id)
            .await?
            .ok_or_else(|| raise_error!("task not found".into(), ErrorCode::ResourceNotFound))?;

        // Check account access
        context.require_account_access(result.account_id)?;
        send_queue.remove_task(req.id).await?;
        Ok(Response::new(Empty::default()))
    }
}
