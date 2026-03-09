// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::{
    modules::{
        account::{entity::MailerType, migration::AccountModel},
        cache::{model::Envelope, vendor::gmail::sync::envelope::GmailEnvelope},
        error::{code::ErrorCode, RustMailerResult},
        smtp::{
            composer::BodyComposer,
            request::{
                builder::EmailBuilder, headers::HeaderValue, task::AnswerEmail, EmailAddress,
                EmailHandler, MailAttachment, SendControl,
            },
            util::generate_message_id,
        },
    },
    raise_error, validate_email,
};

use mail_send::mail_builder::{headers::address::Address, MessageBuilder};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use time_tz::timezones;

use std::{borrow::Cow, collections::HashMap};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, Object)]
pub struct ReplyEmailRequest {
    /// The name of the mailbox containing the original message.
    ///
    /// This is used to locate the source message that is being replied to.
    pub mailbox_name: String,
    /// The unique ID of the message, either IMAP UID or Gmail API MID.
    ///
    /// - For IMAP accounts, this is the UID converted to a string. It must be a valid numeric string
    ///   that can be parsed back to a `u32`.
    /// - For Gmail/Graph API accounts, this is the message ID (`mid`) returned by the API.
    pub id: String,
    /// The plain text body of the reply email.
    ///
    /// This field is optional and can be used to provide plain text content.
    pub text: Option<String>,
    /// The HTML body of the reply email.
    ///
    /// This field is optional and can be used to provide HTML content.
    pub html: Option<String>,
    /// A preview text for the reply email.
    ///
    /// This optional field provides a short summary or preview of the reply content.
    pub preview: Option<String>,
    /// Custom email headers to include in the reply email.
    ///
    /// This optional field allows specifying additional headers as key-value pairs.
    pub headers: Option<HashMap<String, HeaderValue>>,
    /// Whether to reply to all original recipients (Reply-All).
    ///
    /// If true, the reply will be sent to all original recipients, including Cc.
    pub reply_all: Option<bool>,
    /// A list of attachments to include in the reply.
    ///
    /// This optional field allows adding new file attachments to the reply email.
    pub attachments: Option<Vec<MailAttachment>>,
    /// A list of Cc recipients to include in the reply.
    ///
    /// This optional field allows explicitly specifying Cc recipients.
    pub cc: Option<Vec<EmailAddress>>,
    /// A list of Bcc recipients to include in the reply.
    ///
    /// This optional field allows explicitly specifying Bcc recipients.
    pub bcc: Option<Vec<EmailAddress>>,
    /// Whether to include the original message in the reply body.
    ///
    /// If true, the original message content will be quoted and included in the reply.
    pub include_original: Option<bool>,
    /// Whether to include all original attachments in the reply.
    ///
    /// If true, all attachments from the original message will be included in the reply.
    pub include_all_attachments: Option<bool>,
    /// The sender's timezone (e.g., "Asia/Shanghai").
    ///
    /// This optional field may be used for formatting date/time in the reply body.
    pub timezone: Option<String>,
    /// Configuration options for controlling the email sending process.
    ///
    /// This required field specifies settings such as scheduling, or retry policies for sending the reply.
    pub send_control: Option<SendControl>,
}

impl EmailBuilder for ReplyEmailRequest {
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
        let account = &AccountModel::get(account_id).await?;
        self.validate().await?;

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

        let to = match &envelope.reply_to {
            Some(reply_to) if !reply_to.is_empty() => reply_to.clone(),
            _ => envelope
                .from
                .clone()
                .map(|from| vec![from])
                .ok_or_else(|| {
                    raise_error!(
                        "Invalid email envelope: missing both 'reply_to' and 'from'".into(),
                        ErrorCode::InvalidParameter
                    )
                })?,
        };
        let subject = format!("Re: {}", envelope.subject.as_deref().unwrap_or(""));
        let mut builder = MessageBuilder::new()
            .from(from)
            .to(Address::from(to.clone()))
            .subject(subject.clone());
        let message_id = generate_message_id();
        builder = self.apply_recipient_headers(builder, &envelope, &message_id)?;
        builder = self.apply_custom_headers(builder)?;
        builder = apply_references(builder, &envelope)?;
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

impl ReplyEmailRequest {
    fn apply_recipient_headers(
        &self,
        mut builder: MessageBuilder<'static>,
        envelope: &Envelope,
        message_id: &str,
    ) -> RustMailerResult<MessageBuilder<'static>> {
        if self.reply_all.unwrap_or_default() {
            if let Some(cc) = &envelope.cc {
                builder = builder.cc(Address::from(cc.clone()));
            }
            if let Some(bcc) = &envelope.bcc {
                builder = builder.bcc(Address::from(bcc.clone()));
            }
        } else {
            if let Some(cc) = &self.cc {
                builder = builder.cc(EmailHandler::to_address(cc)?);
            }
            if let Some(bcc) = &self.bcc {
                builder = builder.bcc(EmailHandler::to_address(bcc)?);
            }
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
                        true,
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
                        true,
                    );
                    builder = builder.text_body(text);
                } else if let Some(text) = &self.text {
                    builder = builder.text_body(text.clone());
                }

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
                                ErrorCode::InvalidParameter
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
                                ErrorCode::InvalidParameter
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

pub fn apply_references(
    builder: MessageBuilder<'static>,
    envelope: &Envelope,
) -> RustMailerResult<MessageBuilder<'static>> {
    let builder = if let Some(message_id) = &envelope.message_id {
        builder.in_reply_to(message_id.clone())
    } else {
        builder
    };

    let mut references = envelope.references.clone().unwrap_or_default();
    if let Some(message_id) = &envelope.message_id {
        if !references.contains(message_id) {
            references.push(message_id.clone());
        }
    }
    Ok(builder.references(references))
}

pub fn apply_references2(
    builder: MessageBuilder<'static>,
    envelope: &GmailEnvelope,
) -> RustMailerResult<MessageBuilder<'static>> {
    let builder = if let Some(message_id) = &envelope.message_id {
        builder.in_reply_to(message_id.clone())
    } else {
        builder
    };

    let mut references = envelope.references.clone().unwrap_or_default();
    if let Some(message_id) = &envelope.message_id {
        if !references.contains(message_id) {
            references.push(message_id.clone());
        }
    }
    Ok(builder.references(references))
}
