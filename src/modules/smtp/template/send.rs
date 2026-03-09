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
        account::migration::AccountModel,
        context::executors::RUST_MAIL_CONTEXT,
        error::{code::ErrorCode, RustMailerResult},
        smtp::{
            template::{
                entity::EmailTemplate, payload::TemplateSentTestRequest, render::Templates,
            },
            util::generate_message_id,
        },
    },
    raise_error,
};

pub async fn send_template_test_email(
    template_id: u64,
    reqwest: TemplateSentTestRequest,
) -> RustMailerResult<()> {
    let TemplateSentTestRequest {
        account_id,
        recipient,
        template_params,
    } = reqwest;

    let template = EmailTemplate::get(template_id).await?;
    let account = AccountModel::get(account_id).await?;

    let (subject, text, html) = Templates::render(&template, &template_params)?;

    let from = Address::new_address(None::<&str>, Cow::Owned(account.email));
    let to = Address::new_address(None::<&str>, Cow::Owned(recipient));
    let mut builder = MessageBuilder::new()
        .from(from)
        .to(to)
        .subject(subject)
        .message_id(generate_message_id());
    if let Some(text) = text {
        builder = builder.text_body(text);
    }
    if let Some(html) = html {
        builder = builder.html_body(html);
    }
    let message = builder.into_message().map_err(|e| {
        raise_error!(
            format!("Failed to build message: {}", e),
            ErrorCode::InternalError
        )
    })?;
    let executor = RUST_MAIL_CONTEXT.smtp(account_id).await?;
    executor.send_email(message).await
}
