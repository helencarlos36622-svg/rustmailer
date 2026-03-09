// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::account::entity::MailerType;
use crate::modules::account::migration::AccountModel;
use crate::modules::cache::imap::mailbox::{AttributeEnum, MailBox};
use crate::modules::cache::vendor::gmail::sync::client::GmailClient;
use crate::modules::cache::vendor::outlook::sync::client::OutlookClient;
use crate::modules::context::executors::RUST_MAIL_CONTEXT;
use crate::modules::error::code::ErrorCode;
use crate::modules::{envelope::generate_uid_set, error::RustMailerResult};
use crate::{encode_mailbox_name, raise_error};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
pub struct MessageDeleteRequest {
    /// A list of unique message identifiers as strings.
    ///
    /// - For IMAP accounts, each UID is converted to a numeric string (parseable back to `u32`).
    /// - For Gmail API accounts, each element is a message ID (`mid`) returned by the API.
    /// Unifying them as strings simplifies handling across different backends.
    pub ids: Vec<String>,
    /// The decoded, human-readable name of the mailbox containing the email (e.g., "INBOX").  (IMAP only)
    /// This name is presented as it appears to users, with any encoding (e.g., UTF-7) automatically handled by the system,
    /// so no manual decoding is required.
    /// In Gmail/Graph API, this field is not required and can be set to `None`.
    pub mailbox: Option<String>,
}

pub async fn move_to_trash(
    account_id: u64,
    request: &MessageDeleteRequest,
) -> RustMailerResult<()> {
    let account = AccountModel::check_account_active(account_id, false).await?;

    match account.mailer_type {
        MailerType::ImapSmtp => {
            let mailbox = request.mailbox.as_deref().ok_or_else(|| {
                raise_error!(
                    "IMAP request missing required field 'mailbox'".into(),
                    ErrorCode::InvalidParameter
                )
            })?;

            if request.ids.is_empty() {
                return Err(raise_error!(
                    "`ids` must contain at least one element".into(),
                    ErrorCode::InvalidParameter
                ));
            }

            let uids: Vec<u32> = request
                .ids
                .iter()
                .map(|id| {
                    id.parse::<u32>().map_err(|_| {
                        raise_error!(
                            format!("Invalid IMAP UID: '{}', must be a numeric string", id),
                            ErrorCode::InvalidParameter
                        )
                    })
                })
                .collect::<Result<_, _>>()?;

            move_to_trash_or_delete_messages_directly(account_id, &uids, mailbox).await
        }
        MailerType::GmailApi => gmail_move_to_trash(&account, &request.ids).await,
        MailerType::GraphApi => outlook_move_to_trash(&account, &request.ids).await,
    }
}

pub async fn gmail_move_to_trash(account: &AccountModel, mids: &[String]) -> RustMailerResult<()> {
    let account_id = account.id;
    let use_proxy = account.use_proxy;
    GmailClient::batch_delete(account_id, use_proxy, mids).await
}

pub async fn outlook_move_to_trash(
    account: &AccountModel,
    mids: &[String],
) -> RustMailerResult<()> {
    let account_id = account.id;
    let use_proxy = account.use_proxy;
    for mid in mids {
        OutlookClient::delete_message(account_id, use_proxy, mid).await?;
    }
    Ok(())
}

async fn move_to_trash_or_delete_messages_directly(
    account_id: u64,
    uids: &[u32],
    mailbox: &str,
) -> RustMailerResult<()> {
    let uid_set = generate_uid_set(uids.to_vec());
    let executor = RUST_MAIL_CONTEXT.imap(account_id).await?;

    let all_mailboxes: Vec<MailBox> = executor
        .list_all_mailboxes()
        .await?
        .iter()
        .map(|name| name.into())
        .collect();

    let trash_or_junk_mailboxes: Vec<&MailBox> = all_mailboxes
        .iter()
        .filter(|mailbox| {
            mailbox.has_attr(&AttributeEnum::Trash) || mailbox.has_attr(&AttributeEnum::Junk)
        })
        .collect();

    if trash_or_junk_mailboxes.is_empty()
        || trash_or_junk_mailboxes.iter().any(|m| m.name == mailbox)
    {
        let mailbox = encode_mailbox_name!(mailbox);
        executor
            .uid_delete_envelopes(uid_set.as_str(), mailbox.as_str())
            .await?;
        return Ok(());
    }

    let trash_first_target = all_mailboxes
        .iter()
        .find(|mailbox| mailbox.has_attr(&AttributeEnum::Trash))
        // If no trash is found, try to find the mailbox marked as junk
        .or_else(|| {
            all_mailboxes
                .iter()
                .find(|mailbox| mailbox.has_attr(&AttributeEnum::Junk))
        });

    if let Some(target_mailbox) = trash_first_target {
        let to_mailbox_name = encode_mailbox_name!(&target_mailbox.name);
        let from_mailbox_name = encode_mailbox_name!(mailbox);
        executor
            .uid_move_envelopes(&uid_set, &from_mailbox_name, &to_mailbox_name)
            .await?;
    }

    Ok(())
}
