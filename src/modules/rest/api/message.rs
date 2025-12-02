// Copyright © 2025 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::current_datetime;
use crate::modules::cache::model::Envelope;
use crate::modules::common::auth::ClientContext;
use crate::modules::message::append::{AppendReplyToDraftRequest, ReplyDraft};
use crate::modules::message::attachment::{retrieve_email_attachment, AttachmentRequest};
use crate::modules::message::content::{
    retrieve_email_content, FullMessageContent, MessageContentRequest,
};
use crate::modules::message::delete::{move_to_trash, MessageDeleteRequest};
use crate::modules::message::flag::{modify_flags, FlagMessageRequest};
use crate::modules::message::full::retrieve_raw_email;
use crate::modules::message::list::{
    get_thread_messages, list_messages_in_mailbox, list_threads_in_mailbox,
};
use crate::modules::message::search::payload::{MessageSearchRequest, UnifiedSearchRequest};
use crate::modules::message::tags::tag_messages_impl;
use crate::modules::message::tags::BatchTagRequest;
use crate::modules::message::transfer::{
    transfer_messages, MailboxTransferRequest, MessageTransfer,
};
use crate::modules::rest::api::ApiTags;
use crate::modules::rest::response::{CursorDataPage, DataPage};
use crate::modules::rest::ApiResult;
use poem::web::Path;
use poem::Body;
use poem_openapi::param::Query;
use poem_openapi::payload::{Attachment, AttachmentType, Json};
use poem_openapi::OpenApi;

pub struct MessageApi;

#[OpenApi(prefix_path = "/api/v1", tag = "ApiTags::Message")]
impl MessageApi {
    /// Moves messages from one mailbox to another for the specified account.
    #[oai(
        path = "/move-messages/:account_id",
        method = "post",
        operation_id = "move_messages"
    )]
    async fn move_messages(
        &self,
        /// The ID of the account owning the mailboxes.
        account_id: Path<u64>,
        /// specifying the source and destination mailboxes and messages
        payload: Json<MailboxTransferRequest>,
        context: ClientContext,
    ) -> ApiResult<()> {
        let account_id = account_id.0;
        context.require_account_access(account_id)?;
        Ok(transfer_messages(account_id, &payload.0, MessageTransfer::Move).await?)
    }

    /// Copies messages from one mailbox to another for the specified account.
    #[oai(
        path = "/copy-messages/:account_id",
        method = "post",
        operation_id = "copy_messages"
    )]
    async fn copy_messages(
        &self,
        /// The ID of the account owning the mailboxes.
        account_id: Path<u64>,
        /// specifying the source and destination mailboxes and messages.
        payload: Json<MailboxTransferRequest>,
        context: ClientContext,
    ) -> ApiResult<()> {
        let account_id = account_id.0;
        context.require_account_access(account_id)?;
        Ok(transfer_messages(account_id, &payload.0, MessageTransfer::Copy).await?)
    }

    /// Deletes messages from a mailbox or moves them to the trash for the specified account.
    #[oai(
        path = "/delete-messages/:account_id",
        method = "post",
        operation_id = "delete_messages"
    )]
    async fn delete_messages(
        &self,
        /// The ID of the account owning the mailboxes.
        account_id: Path<u64>,
        /// specifying the mailbox and messages to delete.
        payload: Json<MessageDeleteRequest>,
        context: ClientContext,
    ) -> ApiResult<()> {
        let account_id = account_id.0;
        context.require_account_access(account_id)?;
        Ok(move_to_trash(account_id, &payload.0).await?)
    }

    /// Updates flags on messages in a mailbox for the specified account.
    #[oai(
        path = "/flag-messages/:account_id",
        method = "post",
        operation_id = "update_message_flags"
    )]
    async fn update_message_flags(
        &self,
        /// The ID of the account owning the mailbox.
        account_id: Path<u64>,
        /// specifying the mailbox, messages, and flags to modify.
        payload: Json<FlagMessageRequest>,
        context: ClientContext,
    ) -> ApiResult<()> {
        let account_id = account_id.0;
        context.require_account_access(account_id)?;
        Ok(modify_flags(account_id, payload.0).await?)
    }

    /// Batch modifies the custom tags, categories, or keywords on messages.
    /// This interface is dedicated to operating on **user-defined labels** and is separate
    /// from standard system status flags (like Read/Unread).
    /// It unifies tagging across different email services:
    /// - **Gmail/Graph API:** Operates on user-defined Label IDs or Category Names.
    /// - **IMAP/SMTP:** Operates on custom IMAP Keywords (Custom Flags).
    /// **Note:** This is a high-level operation designed for user tag management.
    #[oai(
        path = "/tag-messages/:account_id",
        method = "post",
        operation_id = "tag_messages"
    )]
    async fn tag_messages(
        &self,
        /// The ID of the account owning the mailbox.
        account_id: Path<u64>,
        /// specifying the mailbox, messages, and flags to modify.
        payload: Json<BatchTagRequest>,
        context: ClientContext,
    ) -> ApiResult<()> {
        let account_id = account_id.0;
        context.require_account_access(account_id)?;
        Ok(tag_messages_impl(account_id, payload.0).await?)
    }

    /// Lists messages in a specified mailbox for the given account.
    #[oai(
        path = "/list-messages/:account_id",
        method = "get",
        operation_id = "list_messages"
    )]
    async fn list_messages(
        &self,
        /// The ID of the account owning the mailbox.
        account_id: Path<u64>,
        /// The decoded, human-readable name of the mailbox containing the email (e.g., "INBOX").
        /// This name is presented as it appears to users, with any encoding (e.g., UTF-7) automatically handled by the system,
        /// so no manual decoding is required.
        mailbox: Query<String>,
        /// fetches messages from the IMAP server; otherwise, uses local data.
        remote: Query<Option<bool>>,
        /// The token for fetching the next page of results in pagination.
        /// - If `None`, this indicates that the first page should be returned.
        /// - If `Some(token)`, the page corresponding to this token will be fetched.
        next_page_token: Query<Option<String>>,
        /// The number of messages per page.
        page_size: Query<u64>,
        /// lists messages in descending order; otherwise, ascending. internal date
        desc: Query<Option<bool>>,
        context: ClientContext,
    ) -> ApiResult<Json<CursorDataPage<Envelope>>> {
        let remote = remote.0.unwrap_or(false);
        let desc = desc.0.unwrap_or(false);
        let account_id = account_id.0;
        context.require_account_access(account_id)?;

        Ok(Json(
            list_messages_in_mailbox(
                account_id,
                mailbox.0.trim(),
                next_page_token.0.as_deref(),
                page_size.0,
                remote,
                desc,
            )
            .await?,
        ))
    }

    /// Lists threads in a specified mailbox for the given account.
    #[oai(
        path = "/list-threads/:account_id",
        method = "get",
        operation_id = "list_threads"
    )]
    async fn list_threads(
        &self,
        /// The ID of the account owning the mailbox.
        account_id: Path<u64>,
        /// The decoded, human-readable name of the mailbox containing the email (e.g., "INBOX").
        /// This name is presented as it appears to users, with any encoding (e.g., UTF-7) automatically handled by the system,
        /// so no manual decoding is required.
        mailbox: Query<String>,
        /// The token for fetching the next page of results in pagination.
        /// - If `None`, this indicates that the first page should be returned.
        /// - If `Some(token)`, the page corresponding to this token will be fetched.
        next_page_token: Query<Option<String>>,
        /// The number of messages per page.
        page_size: Query<u64>,
        /// lists messages in descending order; otherwise, ascending. internal date
        desc: Query<Option<bool>>,
        /// fetches threads from the gmail/outlook server; otherwise, uses local data.
        remote: Query<Option<bool>>,
        context: ClientContext,
    ) -> ApiResult<Json<CursorDataPage<Envelope>>> {
        let desc = desc.0.unwrap_or(false);
        let account_id = account_id.0;
        context.require_account_access(account_id)?;
        let remote = remote.0.unwrap_or(false);
        Ok(Json(
            list_threads_in_mailbox(
                account_id,
                mailbox.0.trim(),
                next_page_token.0.as_deref(),
                page_size.0,
                remote,
                desc,
            )
            .await?,
        ))
    }

    /// Get thread's envelopes in a specified mailbox for the given account.
    #[oai(
        path = "/get-thread-messages/:account_id",
        method = "get",
        operation_id = "get_thread_messages"
    )]
    async fn get_thread_messages(
        &self,
        /// The ID of the account owning the mailbox.
        account_id: Path<u64>,
        // Thread ID
        thread_id: Query<String>,
        /// fetches threads from the gmail/outlook server; otherwise, uses local data.
        remote: Query<Option<bool>>,
        context: ClientContext,
    ) -> ApiResult<Json<Vec<Envelope>>> {
        let account_id = account_id.0;
        context.require_account_access(account_id)?;
        let remote = remote.0.unwrap_or(false);
        Ok(Json(get_thread_messages(account_id, thread_id.0, remote).await?))
    }

    /// Fetches the content of a specific email for the given account.
    #[oai(
        path = "/message-content/:account_id",
        method = "post",
        operation_id = "fetch_message_content"
    )]
    async fn fetch_message_content(
        &self,
        /// The ID of the account owning the mailbox.
        account_id: Path<u64>,
        /// specifying the mailbox and message to fetch.
        payload: Json<MessageContentRequest>,
        context: ClientContext,
    ) -> ApiResult<Json<FullMessageContent>> {
        let account_id = account_id.0;
        context.require_account_access(account_id)?;
        Ok(Json(
            retrieve_email_content(account_id, payload.0, true).await?,
        ))
    }

    /// Fetches an attachment from a specific email for the given account.
    #[oai(
        path = "/message-attachment/:account_id",
        method = "post",
        operation_id = "fetch_message_attachment"
    )]
    async fn fetch_message_attachment(
        &self,
        /// The ID of the account owning the mailbox.
        account_id: Path<u64>,
        /// specifying the mailbox, message, and attachment to fetch.
        payload: Json<AttachmentRequest>,
        context: ClientContext,
    ) -> ApiResult<Attachment<Body>> {
        let request = payload.0;
        let account_id = account_id.0;
        context.require_account_access(account_id)?;
        let (reader, filename) = retrieve_email_attachment(account_id, request).await?;
        let body = Body::from_async_read(reader);
        let mut attachment = Attachment::new(body).attachment_type(AttachmentType::Attachment);
        if let Some(filename) = filename {
            attachment = attachment.filename(filename);
        }
        Ok(attachment)
    }

    /// Fetches the full content of a specific email for the given account.
    #[oai(
        path = "/raw-message/:account_id",
        method = "get",
        operation_id = "fetch_raw_message"
    )]
    async fn fetch_raw_message(
        &self,
        /// The ID of the account owning the mailbox.
        account_id: Path<u64>,
        /// The decoded, human-readable name of the mailbox containing the email (e.g., "INBOX").
        /// This name is presented as it appears to users, with any encoding (e.g., UTF-7) automatically handled by the system,
        /// so no manual decoding is required.
        mailbox: Query<Option<String>>,
        /// The unique ID of the message, either IMAP UID or Gmail API MID.
        /// - For IMAP accounts, this is the UID converted to a string. It must be a valid numeric string
        ///   that can be parsed back to a `u32`.
        /// - For Gmail API accounts, this is the message ID (`mid`) returned by the API.
        id: Query<String>,
        /// An optional filename for the attachment (defaults to a timestamped `.elm` file).
        filename: Query<Option<String>>,
        context: ClientContext,
    ) -> ApiResult<Attachment<Body>> {
        let account_id = account_id.0;
        context.require_account_access(account_id)?;
        let filename = filename.0.unwrap_or(format!("{}.elm", current_datetime!()));
        let mailbox_opt = mailbox.0.as_ref().map(|m| m.trim().to_owned());
        let id = id.0.trim();

        let reader = retrieve_raw_email(account_id, mailbox_opt.as_deref(), id).await?;
        let body = Body::from_async_read(reader);
        let attachment = Attachment::new(body)
            .attachment_type(AttachmentType::Attachment)
            .filename(filename);
        Ok(attachment)
    }

    /// Searches for messages in mailboxes for the specified account. performs the search on the IMAP server;
    #[oai(
        path = "/search-message/:account_id",
        method = "post",
        operation_id = "search_messages"
    )]
    async fn search_messages(
        &self,
        /// The ID of the account owning the mailboxes.
        account_id: Path<u64>,
        /// The token for fetching the next page of results in pagination.
        /// - If `None`, this indicates that the first page should be returned.
        /// - If `Some(token)`, the page corresponding to this token will be fetched.
        next_page_token: Query<Option<String>>,
        /// The number of messages per page.
        page_size: Query<u64>,
        /// If `true`, lists results in descending order; otherwise, ascending. imap account only
        desc: Query<Option<bool>>,
        /// specifying the search criteria (e.g., keywords, flags).
        payload: Json<MessageSearchRequest>,
        context: ClientContext,
    ) -> ApiResult<Json<CursorDataPage<Envelope>>> {
        let request = payload.0;
        let desc = desc.0.unwrap_or(false);
        let account_id = account_id.0;
        context.require_account_access(account_id)?;
        Ok(Json(
            request
                .search_impl(account_id, next_page_token.0.as_deref(), page_size.0, desc)
                .await?,
        ))
    }

    /// Searches for messages from local cache for the specified account(s).
    /// This performs a unified search across mailboxes based on indexed data,
    /// without querying the remote IMAP server.
    #[oai(
        path = "/unified-search",
        method = "post",
        operation_id = "unified_search"
    )]
    async fn unified_search(
        &self,
        /// The page number for pagination (1-based).
        page: Query<u64>,
        /// The number of messages per page.
        page_size: Query<u64>,
        /// If `true`, lists results in descending order; otherwise, ascending.
        desc: Query<Option<bool>>,
        /// The unified search criteria (email, time range, accounts, etc.).
        payload: Json<UnifiedSearchRequest>,
        /// Request context (includes authentication and permissions).
        context: ClientContext,
    ) -> ApiResult<Json<DataPage<Envelope>>> {
        let mut request = payload.0;
        let desc = desc.0.unwrap_or(false);

        // Ensure account access control
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
        Ok(Json(request.search(page.0, page_size.0, desc).await?))
    }

    /// Creates a reply draft email for the specified account.
    /// The server internally constructs the reply email, automatically linking it to the
    /// original email thread by applying appropriate headers such as `References` and `In-Reply-To`.
    /// The newly created draft is appended into the specified draft mailbox.
    #[oai(
        path = "/append-reply-to-draft/:account_id",
        method = "post",
        operation_id = "append_reply_to_draft"
    )]
    async fn append_reply_to_draft(
        &self,
        /// The ID of the email account for which the draft is created.
        account_id: Path<u64>,
        /// Request body containing original message location and reply content.
        payload: Json<AppendReplyToDraftRequest>,
        /// Request context (authentication, authorization).
        context: ClientContext,
    ) -> ApiResult<Json<ReplyDraft>> {
        let account_id = account_id.0;
        // Verify that the client has permission to access the account.
        context.require_account_access(account_id)?;
        // Perform the draft creation and append operation.
        Ok(Json(payload.0.append_reply_to_draft(account_id).await?))
    }
}
