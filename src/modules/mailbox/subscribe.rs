// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::{
    encode_mailbox_name,
    modules::{
        account::migration::AccountModel, context::executors::RUST_MAIL_CONTEXT, error::RustMailerResult,
    },
};

pub async fn subscribe_mailbox(account_id: u64, mailbox_name: &str) -> RustMailerResult<()> {
    AccountModel::check_account_active(account_id, true).await?;
    let executor = RUST_MAIL_CONTEXT.imap(account_id).await?;
    executor
        .subscribe_mailbox(encode_mailbox_name!(mailbox_name).as_str())
        .await
}

pub async fn unsubscribe_mailbox(account_id: u64, mailbox_name: &str) -> RustMailerResult<()> {
    AccountModel::check_account_active(account_id, true).await?;
    let executor = RUST_MAIL_CONTEXT.imap(account_id).await?;
    executor
        .unsubscribe_mailbox(encode_mailbox_name!(mailbox_name).as_str())
        .await
}
