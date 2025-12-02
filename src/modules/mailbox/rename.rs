// Copyright © 2025 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::{
    encode_mailbox_name,
    modules::{
        account::{entity::MailerType, migration::AccountModel},
        cache::vendor::{gmail::sync::client::GmailClient, outlook::sync::client::OutlookClient},
        context::executors::RUST_MAIL_CONTEXT,
        error::{code::ErrorCode, RustMailerResult},
        mailbox::create::LabelColor,
    },
    raise_error,
};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

/// Request structure for updating an existing mailbox (IMAP) or label (Gmail API).
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
pub struct MailboxUpdateRequest {
    /// Current name of the mailbox or label.
    ///
    /// - For IMAP accounts, this is the existing mailbox name.  
    /// - For Gmail API accounts, this is the existing label name.  
    /// - For Graph API accounts, this should be the full mailbox path as displayed by  
    ///   `list-mailboxes?remote=true`, where subfolders are separated by `/`.  
    ///   For example, if the folder path is `test1/test2`, you must provide the full name  
    ///   `test1/test2` instead of just `test2`.  
    ///
    /// The path format is handled internally by RustMailer to ensure consistent folder resolution.
    #[oai(validator(min_length = "1", max_length = "1024"))]
    pub current_name: String,
    /// New name for the mailbox or label (optional).
    #[oai(validator(min_length = "1", max_length = "1024"))]
    pub new_name: Option<String>,
    /// Optional color settings for the label (Gmail API only).
    ///
    /// Only applicable to Gmail API accounts. See [`LabelColor`] for allowed
    /// `text_color` and `background_color` values.
    pub label_color: Option<LabelColor>,
}

pub async fn update_mailbox_impl(
    account_id: u64,
    payload: MailboxUpdateRequest,
) -> RustMailerResult<()> {
    let account = AccountModel::check_account_active(account_id, false).await?;
    match account.mailer_type {
        MailerType::ImapSmtp => {
            if payload.new_name.is_none() {
                return Err(raise_error!(
                    "The `new_name` field is required when updating a mailbox.".into(),
                    ErrorCode::InvalidParameter
                ));
            }

            let executor = RUST_MAIL_CONTEXT.imap(account_id).await?;
            executor
                .rename_mailbox(
                    encode_mailbox_name!(&payload.current_name).as_str(),
                    encode_mailbox_name!(&payload.new_name.unwrap()).as_str(),
                )
                .await
        }
        MailerType::GmailApi => {
            if payload.new_name.is_none() && payload.label_color.is_none() {
                return Err(raise_error!(
                    "You must provide either `new_name` or `label_color` to update a mailbox."
                        .into(),
                    ErrorCode::InvalidParameter
                ));
            }

            let map = GmailClient::for_lookup_label_id(account_id, account.use_proxy, true).await?;
            let label_id = map.get(&payload.current_name).ok_or_else(|| {
                raise_error!(
                    format!(
                        "Gmail label '{}' not found for account {}",
                        &payload.current_name, account_id
                    ),
                    ErrorCode::ResourceNotFound
                )
            })?;
            GmailClient::update_label(account_id, account.use_proxy, label_id, &payload).await
        }
        MailerType::GraphApi => {
            if payload.new_name.is_none() {
                return Err(raise_error!(
                    "The `new_name` field is required when updating a mailbox.".into(),
                    ErrorCode::InvalidParameter
                ));
            }
            let mailboxes = OutlookClient::list_mailfolders(account_id, account.use_proxy).await?;
            let target_folder = mailboxes
                .iter()
                .find(|f| f.display_name == payload.current_name)
                .cloned();

            if let Some(folder) = target_folder {
                OutlookClient::rename_folder(
                    account_id,
                    account.use_proxy,
                    &folder.id,
                    &payload.new_name.unwrap(),
                )
                .await?;
                return Ok(());
            } else {
                return Err(raise_error!(
                    format!("Mailbox '{}' not found.", payload.current_name),
                    ErrorCode::ResourceNotFound
                ));
            }
        }
    }
}
