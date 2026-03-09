// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use crate::{
    encode_mailbox_name,
    modules::{
        account::{entity::MailerType, migration::AccountModel},
        cache::vendor::{gmail::sync::client::GmailClient, outlook::sync::client::OutlookClient},
        context::executors::RUST_MAIL_CONTEXT,
        error::RustMailerResult,
    },
};

/// Represents the color settings for a mailbox/label in RustMailer.
///
/// Only used by Gmail API accounts.  
/// `text_color` and `background_color` are optional.  
///
/// `text_color` allowed values include:
/// "#000000", "#434343", "#666666", "#999999", "#cccccc", "#efefef", "#f3f3f3", "#ffffff",
/// "#fb4c2f", "#ffad47", "#fad165", "#16a766", "#43d692", "#4a86e8", "#a479e2", "#f691b3",
/// "#f6c5be", "#ffe6c7", "#fef1d1", "#b9e4d0", "#c6f3de", "#c9daf8", "#e4d7f5", "#fcdee8",
/// "#efa093", "#ffd6a2", "#fce8b3", "#89d3b2", "#a0eac9", "#a4c2f4", "#d0bcf1", "#fbc8d9",
/// "#e66550", "#ffbc6b", "#fcda83", "#44b984", "#68dfa9", "#6d9eeb", "#b694e8", "#f7a7c0",
/// "#cc3a21", "#eaa041", "#f2c960", "#149e60", "#3dc789", "#3c78d8", "#8e63ce", "#e07798",
/// "#ac2b16", "#cf8933", "#d5ae49", "#0b804b", "#2a9c68", "#285bac", "#653e9b", "#b65775",
/// "#822111", "#a46a21", "#aa8831", "#076239", "#1a764d", "#1c4587", "#41236d", "#83334c",
/// "#464646", "#e7e7e7", "#0d3472", "#b6cff5", "#0d3b44", "#98d7e4", "#3d188e", "#e3d7ff",
/// "#711a36", "#fbd3e0", "#8a1c0a", "#f2b2a8", "#7a2e0b", "#ffc8af", "#7a4706", "#ffdeb5",
/// "#594c05", "#fbe983", "#684e07", "#fdedc1", "#0b4f30", "#b3efd3", "#04502e", "#a2dcc1",
/// "#c2c2c2", "#4986e7", "#2da2bb", "#b99aff", "#994a64", "#f691b2", "#ff7537", "#ffad46",
/// "#662e37", "#ebdbde", "#cca6ac", "#094228", "#42d692", "#16a765"
///
/// `background_color` allowed values include:
/// "#000000", "#434343", "#666666", "#999999", "#cccccc", "#efefef", "#f3f3f3", "#ffffff",
/// "#fb4c2f", "#ffad47", "#fad165", "#16a766", "#43d692", "#4a86e8", "#a479e2", "#f691b3",
/// "#f6c5be", "#ffe6c7", "#fef1d1", "#b9e4d0", "#c6f3de", "#c9daf8", "#e4d7f5", "#fcdee8",
/// "#efa093", "#ffd6a2", "#fce8b3", "#89d3b2", "#a0eac9", "#a4c2f4", "#d0bcf1", "#fbc8d9",
/// "#e66550", "#ffbc6b", "#fcda83", "#44b984", "#68dfa9", "#6d9eeb", "#b694e8", "#f7a7c0",
/// "#cc3a21", "#eaa041", "#f2c960", "#149e60", "#3dc789", "#3c78d8", "#8e63ce", "#e07798",
/// "#ac2b16", "#cf8933", "#d5ae49", "#0b804b", "#2a9c68", "#285bac", "#653e9b", "#b65775",
/// "#822111", "#a46a21", "#aa8831", "#076239", "#1a764d", "#1c4587", "#41236d", "#83334c",
/// "#464646", "#e7e7e7", "#0d3472", "#b6cff5", "#0d3b44", "#98d7e4", "#3d188e", "#e3d7ff",
/// "#711a36", "#fbd3e0", "#8a1c0a", "#f2b2a8", "#7a2e0b", "#ffc8af", "#7a4706", "#ffdeb5",
/// "#594c05", "#fbe983", "#684e07", "#fdedc1", "#0b4f30", "#b3efd3", "#04502e", "#a2dcc1",
/// "#c2c2c2", "#4986e7", "#2da2bb", "#b99aff", "#994a64", "#f691b2", "#ff7537", "#ffad46",
/// "#662e37", "#ebdbde", "#cca6ac", "#094228", "#42d692", "#16a765"
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, Object)]
pub struct LabelColor {
    /// Text color of the label.
    pub text_color: String,
    /// Background color of the label.
    pub background_color: String,
}

/// Request structure for creating a mailbox (IMAP) or a label (Gmail API).
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, Object)]
pub struct CreateMailboxRequest {
    /// Name of the mailbox or label.
    ///
    /// - For IMAP accounts, this is the name of the mailbox to create.  
    ///   Supports hierarchical paths using `/` as a separator.  
    ///   For example, `"a/b"` creates mailbox `b` under parent mailbox `a`.  
    ///   **Note:** The parent mailbox (`a`) must already exist.
    /// - For Gmail API accounts, this corresponds to the label's name.  
    ///   Gmail labels do not require the parent to exist beforehand; nested labels are created automatically.
    pub mailbox_name: String,
    /// Parent mailbox ID.
    ///
    /// Only applicable for **Graph API** accounts.  
    /// For IMAP or Gmail API accounts, this field is always `None`.  
    /// The name can be retrieved via the **`/list-mailboxes?remote=true`** endpoint.
    pub parent_name: Option<String>,
    /// Optional color settings for the label (Gmail API only).
    ///
    /// Only applicable to Gmail API accounts. See [`LabelColor`] for the allowed
    /// `text_color` and `background_color` values.
    pub label_color: Option<LabelColor>,
}

pub async fn create_mailbox(
    account_id: u64,
    request: &CreateMailboxRequest,
) -> RustMailerResult<()> {
    let account = AccountModel::check_account_active(account_id, false).await?;

    match account.mailer_type {
        MailerType::ImapSmtp => {
            let executor = RUST_MAIL_CONTEXT.imap(account_id).await?;
            executor
                .create_mailbox(encode_mailbox_name!(&request.mailbox_name).as_str())
                .await
        }
        MailerType::GmailApi => {
            GmailClient::create_label(account_id, account.use_proxy, request).await?;
            Ok(())
        }
        MailerType::GraphApi => {
            OutlookClient::create_folder(
                account_id,
                account.use_proxy,
                request.parent_name.clone(),
                &request.mailbox_name,
            )
            .await
        }
    }
}
