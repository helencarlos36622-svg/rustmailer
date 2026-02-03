// Copyright © 2025 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::account::entity::MailerType;
use crate::modules::cache::model::Envelope;
use crate::modules::error::code::ErrorCode;
use crate::modules::smtp::request::builder::EmailBuilder;
use crate::modules::smtp::request::headers::HeaderValue;
use crate::modules::smtp::request::task::AnswerEmail;
use crate::modules::smtp::request::EmailHandler;
use crate::modules::smtp::request::SendControl;
use crate::modules::smtp::util::generate_message_id;
use crate::validate_email;
use crate::{
    modules::{
        account::migration::AccountModel,
        error::RustMailerResult,
        smtp::{
            composer::BodyComposer,
            request::{EmailAddress, MailAttachment},
        },
    },
    raise_error,
};
use mail_send::mail_builder::{headers::address::Address, MessageBuilder};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use time_tz::timezones;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, Object)]
pub struct ForwardEmailRequest {
    /// The name of the mailbox containing the original message.
    ///
    /// This is used to locate the source message that is being forwarded.
    pub mailbox_name: String,
    /// The unique ID of the message, either IMAP UID or Gmail API MID.
    ///
    /// - For IMAP accounts, this is the UID converted to a string. It must be a valid numeric string
    ///   that can be parsed back to a `u32`.
    /// - For Gmail API accounts, this is the message ID (`mid`) returned by the API.
    pub id: String,
    /// The list of primary recipients to forward the email to.
    ///
    /// At least one recipient must be specified.
    pub to: Vec<EmailAddress>,

    /// A list of Cc recipients to include in the forwarded email.
    ///
    /// This optional field allows explicitly specifying Cc recipients.
    pub cc: Option<Vec<EmailAddress>>,

    /// A list of Bcc recipients to include in the forwarded email.
    ///
    /// This optional field allows explicitly specifying Bcc recipients.
    pub bcc: Option<Vec<EmailAddress>>,

    /// The plain text body of the forwarded email.
    ///
    /// This field is optional and can be used to provide additional comments or content.
    pub text: Option<String>,

    /// The HTML body of the forwarded email.
    ///
    /// This field is optional and can be used to provide HTML-formatted content.
    pub html: Option<String>,

    /// A preview text for the forwarded email.
    ///
    /// This optional field provides a short summary or preview of the email content.
    pub preview: Option<String>,

    /// Custom email headers to include in the forwarded email.
    ///
    /// This optional field allows specifying additional headers as key-value pairs.
    pub headers: Option<HashMap<String, HeaderValue>>,

    /// The sender's timezone (e.g., "Asia/Shanghai").
    ///
    /// This optional field may be used for formatting date/time in the forwarded content.
    pub timezone: Option<String>,

    /// A list of new attachments to include in the forwarded email.
    ///
    /// This optional field allows adding additional attachments beyond the original ones.
    pub attachments: Option<Vec<MailAttachment>>,

    /// Whether to include the original message in the forwarded email body.
    ///
    /// If true, the full original message content will be included in the body.
    pub include_original: Option<bool>,

    /// Whether to include all original attachments in the forwarded email.
    ///
    /// If true, all attachments from the original message will be forwarded as well.
    pub include_all_attachments: Option<bool>,

    /// Configuration options for controlling the email sending process.
    ///
    /// This required field specifies settings such as scheduling or retry policies for sending the forwarded email.
    pub send_control: Option<SendControl>,
}

impl EmailBuilder for ForwardEmailRequest {
    async fn validate(&self) -> RustMailerResult<()> {
        let mut errors = Vec::new();

        if let Some(cc) = &self.cc {
            for email in cc {
                if validate_email!(&email.address).is_err() {
                    errors.push("Invalid 'cc' email address".into());
                }
            }
        }

        if let Some(bcc) = &self.bcc {
            for email in bcc {
                if validate_email!(&email.address).is_err() {
                    errors.push("Invalid 'bcc' email address".into());
                }
            }
        }
        if self.to.is_empty() {
            errors.push("At least one 'to' recipient is required for forwarding".into());
        }

        for email in &self.to {
            if validate_email!(&email.address).is_err() {
                errors.push("Invalid 'to' email address".into());
            }
        }
        if let Some(send_control) = &self.send_control {
            if let Err(mut send_control_error) = send_control.validate() {
                errors.append(&mut send_control_error);
            }
        }

        if let Some(timezone) = &self.timezone {
            if timezones::get_by_name(timezone).is_none() {
                errors.push(format!("Invalid timezone: {}", timezone));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(raise_error!(
                format!("{:#?}", errors),
                ErrorCode::InvalidParameter
            ))
        }
    }

    async fn build(&self, account_id: u64) -> RustMailerResult<()> {
        self.validate().await?;
        let account = &AccountModel::get(account_id).await?;

        let (envelope, answer_email): (Envelope, Option<AnswerEmail>) = match account.mailer_type {
            MailerType::ImapSmtp => {
                let uid = self.id.parse::<u32>().ok().ok_or_else(|| {
                    raise_error!(
                        "Invalid IMAP UID: `id` must be a numeric string".into(),
                        ErrorCode::InvalidParameter
                    )
                })?;
                let envelope = EmailHandler::get_envelope(account, &self.mailbox_name, uid).await?;
                (
                    envelope.into(),
                    Some(AnswerEmail {
                        reply: true,
                        mailbox: self.mailbox_name.clone(),
                        uid,
                    }),
                )
            }
            MailerType::GmailApi => {
                let envelope =
                    EmailHandler::get_gmail_envelope(account, &self.mailbox_name, &self.id).await?;
                (envelope.into(), None)
            }
            MailerType::GraphApi => {
                let envelope =
                    EmailHandler::get_outlook_envelope(account, &self.mailbox_name, &self.id)
                        .await?;
                (envelope.into(), None)
            }
        };
        let from = Address::new_address(
            account.name.as_ref().map(|n| Cow::Owned(n.to_string())),
            Cow::Owned(account.email.clone()),
        );
        let subject = format!("Fwd: {}", envelope.subject.as_deref().unwrap_or(""));
        let mut builder = MessageBuilder::new().from(from).subject(subject.clone());
        let message_id = generate_message_id();
        builder = self.apply_recipient_headers(builder, &message_id)?;
        builder = self.apply_custom_headers(builder)?;
        builder = self.apply_references(builder, &envelope)?;
        builder = self.apply_content(builder, &envelope, account).await?;
        builder = self.apply_attachments(builder, account).await?;

        if let Some(send_control) = &self.send_control {
            let send_at = send_control.send_at;
            if let Some(send_at) = send_at {
                builder = builder.date(send_at / 1000)
            }
        }

        EmailHandler::schedule_task(
            account,
            Some(subject.clone()),
            message_id,
            self.cc.clone(),
            self.bcc.clone(),
            self.attachments.as_ref().map_or(0, |v| v.len()),
            builder,
            self.send_control.clone(),
            self.send_control.as_ref().and_then(|c| c.send_at),
            answer_email,
        )
        .await?;
        Ok(())
    }
}

impl ForwardEmailRequest {
    fn apply_recipient_headers(
        &self,
        mut builder: MessageBuilder<'static>,
        message_id: &str,
    ) -> RustMailerResult<MessageBuilder<'static>> {
        builder = builder.to(EmailHandler::to_address(&self.to)?);

        if let Some(cc) = &self.cc {
            builder = builder.cc(EmailHandler::to_address(cc)?);
        }
        if let Some(bcc) = &self.bcc {
            builder = builder.bcc(EmailHandler::to_address(bcc)?);
        }
        builder = builder.message_id(message_id.to_string());

        Ok(builder)
    }

    fn apply_custom_headers(
        &self,
        builder: MessageBuilder<'static>,
    ) -> RustMailerResult<MessageBuilder<'static>> {
        Ok(match &self.headers {
            Some(headers) => headers.iter().fold(builder, |b, (k, v)| {
                b.header(k.clone(), v.clone().to_header_type())
            }),
            None => builder,
        })
    }

    fn apply_references(
        &self,
        builder: MessageBuilder<'static>,
        envelope: &Envelope,
    ) -> RustMailerResult<MessageBuilder<'static>> {
        let mut references = envelope.references.clone().unwrap_or_default();
        if let Some(message_id) = &envelope.message_id {
            if !references.contains(message_id) {
                references.push(message_id.clone());
            }
        }
        Ok(builder.references(references))
    }

    async fn apply_content(
        &self,
        mut builder: MessageBuilder<'static>,
        envelope: &Envelope,
        account: &AccountModel,
    ) -> RustMailerResult<MessageBuilder<'static>> {
        let timezone = self.timezone.as_deref().unwrap_or("UTC");

        if self.include_original.unwrap_or_default() {
            if let Some(content) = EmailHandler::retrieve_message_content(account, envelope).await?
            {
                if let Some(original_html) = content.html() {
                    let html = BodyComposer::generate_html(
                        original_html,
                        &self
                            .html
                            .clone()
                            .unwrap_or_else(|| self.text.clone().unwrap_or_default()),
                        envelope,
                        timezone,
                        false,
                    );
                    let html = EmailHandler::insert_preview(&self.preview, html);
                    builder = builder.html_body(html);
                } else if let Some(html) = &self.html {
                    let html = EmailHandler::insert_preview(&self.preview, html.clone());
                    builder = builder.html_body(html);
                }

                if let Some(original_text) = content.plain() {
                    let text = BodyComposer::generate_text(
                        original_text,
                        &self.text.clone().unwrap_or_default(),
                        envelope,
                        timezone,
                        false,
                    );
                    builder = builder.text_body(text);
                } else if let Some(text) = &self.text {
                    builder = builder.text_body(text.clone());
                }

                //include_all_attachments
                if self.include_all_attachments.unwrap_or_default() {
                    builder = EmailHandler::add_attachment(
                        builder,
                        envelope.attachments.as_deref(),
                        content.attachments.as_deref(),
                        envelope,
                        account,
                    )
                    .await?;
                }
            } else {
                builder = self.apply_fallback_content(builder)?;
            }

            // if let Some(attachments) = envelope.attachments.as_ref() {
            //     if self.include_all_attachments {
            //         for attachment in attachments.iter().filter(|att| !att.inline) {
            //             builder = EmailHandler::add_attachment(
            //                 builder, attachment, envelope, false, account,
            //             )
            //             .await?;
            //         }
            //     }
            // }
        } else {
            builder = self.apply_fallback_content(builder)?;
        }

        Ok(builder)
    }

    fn apply_fallback_content(
        &self,
        mut builder: MessageBuilder<'static>,
    ) -> RustMailerResult<MessageBuilder<'static>> {
        if let Some(html) = &self.html {
            let html = EmailHandler::insert_preview(&self.preview, html.clone());
            builder = builder.html_body(html);
        }
        if let Some(text) = &self.text {
            builder = builder.text_body(text.clone());
        }
        Ok(builder)
    }

    async fn apply_attachments(
        &self,
        mut builder: MessageBuilder<'static>,
        account: &AccountModel,
    ) -> RustMailerResult<MessageBuilder<'static>> {
        if let Some(attachments) = &self.attachments {
            for attachment in attachments {
                let content = attachment.get_content(account).await?;
                let mime = attachment.mime_type.clone();

                builder = if attachment.inline {
                    builder.inline(
                        mime,
                        attachment.content_id.clone().ok_or_else(|| {
                            raise_error!(
                                "Missing content_id for inline attachment".into(),
                                ErrorCode::MissingConfiguration
                            )
                        })?,
                        content,
                    )
                } else {
                    builder.attachment(
                        mime,
                        attachment.file_name.clone().ok_or_else(|| {
                            raise_error!(
                                "Missing file_name for attachment".into(),
                                ErrorCode::MissingConfiguration
                            )
                        })?,
                        content,
                    )
                };
            }
        }
        Ok(builder)
    }
}
