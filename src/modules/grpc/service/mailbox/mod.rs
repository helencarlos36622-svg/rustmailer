// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::grpc::auth::require_account_access;
use crate::modules::grpc::service::rustmailer_grpc::{
    CreateMailboxRequest, DeleteMailboxRequest, Empty, ListMailboxesRequest, ListMailboxesResponse,
    ListSubscribedRequest, MailboxService, MailboxUpdateRequest, SubscribeRequest,
    UnsubscribeRequest,
};
use crate::modules::mailbox::{
    create::create_mailbox,
    delete::delete_mailbox,
    list::{get_account_mailboxes, list_subscribed_mailboxes},
    rename::update_mailbox_impl,
    subscribe::{subscribe_mailbox, unsubscribe_mailbox},
};
use poem_grpc::{Request, Response, Status};

pub mod from;

#[derive(Default)]
pub struct RustMailerMailboxService;

impl MailboxService for RustMailerMailboxService {
    async fn list_mailboxes(
        &self,
        request: Request<ListMailboxesRequest>,
    ) -> Result<Response<ListMailboxesResponse>, Status> {
        let req = require_account_access(request, |r| r.account_id)?;
        let result = get_account_mailboxes(req.account_id, req.remote).await?;
        Ok(Response::new(ListMailboxesResponse {
            mailboxes: result.into_iter().map(Into::into).collect(),
        }))
    }

    async fn list_subscribed_mailboxes(
        &self,
        request: Request<ListSubscribedRequest>,
    ) -> Result<Response<ListMailboxesResponse>, Status> {
        let req = require_account_access(request, |r| r.account_id)?;
        let result = list_subscribed_mailboxes(req.account_id).await?;
        Ok(Response::new(ListMailboxesResponse {
            mailboxes: result.into_iter().map(Into::into).collect(),
        }))
    }

    async fn subscribe_mailbox(
        &self,
        request: Request<SubscribeRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = require_account_access(request, |r| r.account_id)?;
        subscribe_mailbox(req.account_id, &req.mailbox_name).await?;
        Ok(Response::new(Empty::default()))
    }

    async fn unsubscribe_mailbox(
        &self,
        request: Request<UnsubscribeRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = require_account_access(request, |r| r.account_id)?;
        unsubscribe_mailbox(req.account_id, &req.mailbox_name).await?;
        Ok(Response::new(Empty::default()))
    }

    async fn create_mailbox(
        &self,
        request: Request<CreateMailboxRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = require_account_access(request, |r| r.account_id)?;
        create_mailbox(req.account_id, &req.into()).await?;
        Ok(Response::new(Empty::default()))
    }

    async fn remove_mailbox(
        &self,
        request: Request<DeleteMailboxRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = require_account_access(request, |r| r.account_id)?;
        delete_mailbox(req.account_id, &req.mailbox_name).await?;
        Ok(Response::new(Empty::default()))
    }

    async fn update_mailbox(
        &self,
        request: Request<MailboxUpdateRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = require_account_access(request, |r| r.account_id)?;
        update_mailbox_impl(req.account_id, req.into()).await?;
        Ok(Response::new(Empty::default()))
    }
}
