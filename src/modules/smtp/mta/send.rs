// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use std::borrow::Cow;

use mail_send::{
    mail_builder::{headers::address::Address, MessageBuilder},
    smtp::message::IntoMessage,
};

use crate::{
    modules::{
        context::executors::RUST_MAIL_CONTEXT,
        error::{code::ErrorCode, RustMailerResult},
        smtp::{
            mta::{entity::Mta, payload::SendTestEmailRequest},
            util::generate_message_id,
        },
    },
    raise_error,
};

pub async fn send_test_email(id: u64, reqwest: SendTestEmailRequest) -> RustMailerResult<()> {
    reqwest.validate()?;
    let mta = Mta::get(id).await?.ok_or_else(|| {
        raise_error!(
            format!("Mta with id: '{}' not found.", id),
            ErrorCode::ResourceNotFound
        )
    })?;
    let SendTestEmailRequest {
        from,
        to,
        subject,
        message,
    } = reqwest;
    let from = Address::new_address(None::<&str>, Cow::Owned(from));
    let to = Address::new_address(None::<&str>, Cow::Owned(to));
    let builder = MessageBuilder::new()
        .from(from)
        .to(to)
        .subject(subject)
        .text_body(message)
        .message_id(generate_message_id());
    let message = builder.into_message().map_err(|e| {
        raise_error!(
            format!("Failed to build message: {}", e),
            ErrorCode::InternalError
        )
    })?;
    let executor = RUST_MAIL_CONTEXT.mta(mta.id).await?;
    executor.send_email(message).await
}
