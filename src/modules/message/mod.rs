// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::context::executors::RUST_MAIL_CONTEXT;
use crate::modules::envelope::extractor::extract_minimal_envelope_meta;
use crate::modules::error::code::ErrorCode;
use crate::{encode_mailbox_name, raise_error};

use crate::modules::{envelope::MinimalEnvelopeMeta, error::RustMailerResult};

pub mod append;
pub mod attachment;
pub mod content;
pub mod delete;
pub mod flag;
pub mod full;
pub mod list;
pub mod search;
pub mod tags;
pub mod transfer;

pub async fn get_minimal_meta(
    account_id: u64,
    mailbox_name: &str,
    uid: u32,
) -> RustMailerResult<MinimalEnvelopeMeta> {
    let executor = RUST_MAIL_CONTEXT.imap(account_id).await?;
    let encoded_mailbox = encode_mailbox_name!(mailbox_name);
    let body_structures = executor
        .uid_fetch_body_structure(&uid.to_string(), &encoded_mailbox)
        .await?;

    let target = body_structures
        .into_iter()
        .find(|f| f.uid == Some(uid))
        .ok_or_else(|| {
            raise_error!(
                format!(
                    "No body structure found for UID {} in mailbox {}",
                    uid, mailbox_name
                ),
                ErrorCode::ImapUnexpectedResult
            )
        })?;

    let meta = extract_minimal_envelope_meta(&target)?;
    Ok(meta)
}
