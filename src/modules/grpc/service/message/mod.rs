// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use std::sync::Arc;

use crate::modules::common::auth::ClientContext;
use crate::modules::error::code::ErrorCode;
use crate::modules::grpc::auth::require_account_access;
use crate::modules::grpc::service::rustmailer_grpc::{
    AppendReplyToDraftRequest, BatchTagRequest, ByteResponse, CursorDataPage, EmailEnvelopeList,
    GetThreadMessagesRequest, ListThreadsRequest, MessageContentResponse, PagedMessages,
    UnifiedSearchRequest,
};
use crate::modules::grpc::service::rustmailer_grpc::{
    Empty, FetchMessageAttachmentRequest, FetchMessageContentRequest, FetchRawMessageRequest,
    FlagMessageRequest, ListMessagesRequest, MailboxTransferRequest, MessageDeleteRequest,
    MessageSearchRequest, MessageService,
};
use crate::modules::message::append::AppendReplyToDraftRequest as RustMailerAppendReplyToDraftRequest;
use crate::modules::message::attachment::retrieve_email_attachment;
use crate::modules::message::content::retrieve_email_content;
use crate::modules::message::delete::move_to_trash;
use crate::modules::message::flag::modify_flags;
use crate::modules::message::flag::FlagMessageRequest as RustMailerFlagMessageRequest;
use crate::modules::message::full::retrieve_raw_email;
use crate::modules::message::list::{
    get_thread_messages, list_messages_in_mailbox, list_threads_in_mailbox,
};
use crate::modules::message::search::payload::MessageSearchRequest as RustMailerMessageSearchRequest;
use crate::modules::message::search::payload::UnifiedSearchRequest as RustMailerUnifiedSearchRequest;
use crate::modules::message::tags::{
    tag_messages_impl, BatchTagRequest as RustMailerBatchTagRequest,
};
use crate::modules::message::transfer::{transfer_messages, MessageTransfer};
use crate::raise_error;
use poem_grpc::{Request, Response, Status};
use tokio::io::AsyncReadExt;

pub mod from;

#[derive(Default)]
pub struct RustMailerMessageService;

impl MessageService for RustMailerMessageService {
    async fn move_messages(
        &self,
        request: Request<MailboxTransferRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = require_account_access(request, |r| r.account_id)?;
        transfer_messages(req.account_id, &req.into(), MessageTransfer::Move).await?;
        Ok(Response::new(Empty::default()))
    }

    async fn copy_messages(
        &self,
        request: Request<MailboxTransferRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = require_account_access(request, |r| r.account_id)?;
        transfer_messages(req.account_id, &req.into(), MessageTransfer::Copy).await?;
        Ok(Response::new(Empty::default()))
    }

    async fn delete_messages(
        &self,
        request: Request<MessageDeleteRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = require_account_access(request, |r| r.account_id)?;
        move_to_trash(req.account_id, &req.into()).await?;
        Ok(Response::new(Empty::default()))
    }

    async fn update_message_flags(
        &self,
        request: Request<FlagMessageRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = require_account_access(request, |r| r.account_id)?;
        modify_flags(
            req.account_id,
            RustMailerFlagMessageRequest::try_from(req).map_err(|e: &'static str| {
                raise_error!(e.to_string(), ErrorCode::InvalidParameter)
            })?,
        )
        .await?;
        Ok(Response::new(Empty::default()))
    }

    async fn list_messages(
        &self,
        request: Request<ListMessagesRequest>,
    ) -> Result<Response<CursorDataPage>, Status> {
        let req = require_account_access(request, |r| r.account_id)?;

        let result = list_messages_in_mailbox(
            req.account_id,
            &req.mailbox_name,
            req.next_page_token.as_deref(),
            req.page_size,
            req.remote,
            req.desc,
        )
        .await?;

        Ok(Response::new(result.into()))
    }

    async fn fetch_message_content(
        &self,
        request: Request<FetchMessageContentRequest>,
    ) -> Result<Response<MessageContentResponse>, Status> {
        let req = require_account_access(request, |r| r.account_id)?;
        let result = retrieve_email_content(
            req.account_id,
            req.try_into().map_err(|e: &'static str| {
                raise_error!(e.to_string(), ErrorCode::InvalidParameter)
            })?,
            false,
        )
        .await?;
        Ok(Response::new(result.into()))
    }

    async fn fetch_message_attachment(
        &self,
        request: Request<FetchMessageAttachmentRequest>,
    ) -> Result<Response<ByteResponse>, Status> {
        let req = require_account_access(request, |r| r.account_id)?;
        let (mut reader, _) = retrieve_email_attachment(
            req.account_id,
            req.try_into().map_err(|e: &'static str| {
                raise_error!(e.to_string(), ErrorCode::InvalidParameter)
            })?,
        )
        .await?;
        let mut buffer = Vec::new();
        reader
            .read_to_end(&mut buffer)
            .await
            .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?;

        Ok(Response::new(ByteResponse { data: buffer }))
    }

    async fn fetch_raw_message(
        &self,
        request: Request<FetchRawMessageRequest>,
    ) -> Result<Response<ByteResponse>, Status> {
        let req = require_account_access(request, |r| r.account_id)?;
        let mut reader =
            retrieve_raw_email(req.account_id, req.mailbox_name.as_deref(), &req.id).await?;

        let mut buffer = Vec::new();
        reader
            .read_to_end(&mut buffer)
            .await
            .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?;

        Ok(Response::new(ByteResponse { data: buffer }))
    }

    async fn message_search(
        &self,
        request: Request<MessageSearchRequest>,
    ) -> Result<Response<CursorDataPage>, Status> {
        let req = require_account_access(request, |r| r.account_id)?;
        let account_id = req.account_id;
        let next_page_token = req.next_page_token.clone();
        let page_size = req.page_size;
        let desc = req.desc;
        let request: RustMailerMessageSearchRequest = req
            .try_into()
            .map_err(|e: &'static str| raise_error!(e.to_string(), ErrorCode::InvalidParameter))?;

        let result = request
            .search_impl(
                account_id,
                next_page_token.as_deref(),
                page_size,
                desc.unwrap_or(true),
            )
            .await?;
        Ok(Response::new(result.into()))
    }

    async fn unified_search(
        &self,
        request: Request<UnifiedSearchRequest>,
    ) -> Result<Response<PagedMessages>, Status> {
        let extensions = request.extensions().clone();
        let req = request.into_inner();

        // Get ClientContext from cloned extensions
        let context = extensions.get::<Arc<ClientContext>>().ok_or_else(|| {
            raise_error!("Missing ClientContext".into(), ErrorCode::InternalError)
        })?;
        let page = req.page;
        let page_size = req.page_size;
        let desc = req.desc;

        let mut request: RustMailerUnifiedSearchRequest = req.into();

        match &mut request.accounts {
            Some(accounts) => {
                for &account_id in accounts.iter() {
                    context.require_account_access(account_id)?;
                }
            }
            None => {
                if !context.is_root {
                    // Inject accessible accounts if not specified
                    if let Some(accessible) = context.accessible_accounts()? {
                        let account_ids = accessible.iter().map(|a| a.id).collect::<Vec<u64>>();
                        request.accounts = Some(account_ids);
                    }
                }
            }
        }
        let result = request.search(page, page_size, desc).await?;
        Ok(Response::new(result.into()))
    }

    async fn list_threads(
        &self,
        request: Request<ListThreadsRequest>,
    ) -> Result<Response<CursorDataPage>, Status> {
        let req = require_account_access(request, |r| r.account_id)?;
        let result = list_threads_in_mailbox(
            req.account_id,
            &req.mailbox_name,
            req.next_page_token.as_deref(),
            req.page_size,
            req.remote,
            req.desc,
        )
        .await?;

        Ok(Response::new(result.into()))
    }

    async fn get_thread_messages(
        &self,
        request: Request<GetThreadMessagesRequest>,
    ) -> Result<Response<EmailEnvelopeList>, Status> {
        let req = require_account_access(request, |r| r.account_id)?;
        let envelopes =
            get_thread_messages(req.account_id, req.thread_id, req.remote.unwrap_or(false)).await?;

        Ok(Response::new(EmailEnvelopeList {
            items: envelopes.into_iter().map(|e| e.into()).collect(),
        }))
    }

    async fn append_reply_to_draft(
        &self,
        request: Request<AppendReplyToDraftRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = require_account_access(request, |r| r.account_id)?;
        let account_id = req.account_id;
        let request: RustMailerAppendReplyToDraftRequest = req.into();
        request.append_reply_to_draft(account_id).await?;
        Ok(Response::new(Empty::default()))
    }

    async fn tag_messages(
        &self,
        request: Request<BatchTagRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = require_account_access(request, |r| r.account_id)?;
        let account_id = req.account_id;
        let request: RustMailerBatchTagRequest = req
            .try_into()
            .map_err(|e: &'static str| raise_error!(e.to_string(), ErrorCode::InvalidParameter))?;
        tag_messages_impl(account_id, request).await?;
        Ok(Response::new(Empty::default())) 
    }
}
