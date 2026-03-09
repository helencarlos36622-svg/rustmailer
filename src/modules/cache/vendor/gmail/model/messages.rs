// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use mail_parser::MessageParser;
use serde::{Deserialize, Serialize};

use crate::{
    base64_decode_url_safe,
    modules::{
        cache::vendor::gmail::sync::envelope::GmailEnvelope,
        common::Addr,
        error::{code::ErrorCode, RustMailerError, RustMailerResult},
        message::content::{AttachmentInfo, FullMessageContent, PlainText},
    },
    raise_error,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageIndex {
    pub id: String,
    #[serde(rename = "threadId")]
    pub thread_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageList {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<MessageIndex>>,
    #[serde(rename = "nextPageToken")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
    #[serde(rename = "resultSizeEstimate")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_size_estimate: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageMeta {
    pub id: String,
    #[serde(rename = "threadId")]
    pub thread_id: String,
    #[serde(rename = "historyId")]
    pub history_id: String,
    #[serde(rename = "internalDate")]
    pub internal_date: String,
    #[serde(rename = "labelIds")]
    pub label_ids: Vec<String>,
    pub payload: Payload,
    #[serde(rename = "sizeEstimate")]
    pub size_estimate: u32,
    pub snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Payload {
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
    pub headers: Vec<Header>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Header {
    pub name: String,
    pub value: String,
}

fn parser_date(date: &str) -> RustMailerResult<i64> {
    let input = format!("Date: {date}");
    let headers = MessageParser::default()
        .parse_headers(&input)
        .ok_or_else(|| {
            raise_error!(
                format!("Failed to parse headers from input: {:?}", input),
                ErrorCode::InternalError
            )
        })?;
    let date = headers.date().ok_or_else(|| {
        raise_error!(
            format!("Date field not found or invalid in header: {:?}", input),
            ErrorCode::InternalError
        )
    })?;

    Ok(date.to_timestamp() * 1000)
}

impl TryFrom<MessageMeta> for GmailEnvelope {
    type Error = RustMailerError;

    fn try_from(value: MessageMeta) -> Result<Self, Self::Error> {
        let payload = value.payload;
        let mut envelope = Self {
            account_id: 0,
            label_id: 0,
            label_name: "".into(),
            id: value.id,
            internal_date: value.internal_date.parse().map_err(|e| {
                raise_error!(
                    format!("Failed to parse internal_date: {}", e),
                    ErrorCode::InternalError
                )
            })?,
            size: value.size_estimate,
            bcc: None,
            cc: None,
            date: None,
            from: None,
            in_reply_to: None,
            sender: None,
            message_id: None,
            subject: None,
            thread_id: 0,
            mime_version: None,
            references: None,
            reply_to: None,
            to: None,
            snippet: value.snippet,
            history_id: value.history_id,
            gmail_thread_id: value.thread_id,
            label_ids: value.label_ids,
        };

        for header in payload.headers {
            let name = header.name.to_ascii_lowercase();
            match name.as_str() {
                "date" => {
                    envelope.date = Some(parser_date(&header.value)?);
                }
                "from" => envelope.from = Some(Addr::parse(&header.value)),
                "sender" => envelope.sender = Some(Addr::parse(&header.value)),
                "reply-to" => envelope.reply_to = Some(Self::parse_addr_list(&header.value)),
                "in-reply-to" => {
                    envelope.in_reply_to = Some(Self::clean_angle_brackets(&header.value).into())
                }
                "message-id" => {
                    envelope.message_id = Some(Self::clean_angle_brackets(&header.value).into())
                }
                "mime-version" => envelope.mime_version = Some(header.value),
                "references" => {
                    envelope.references = Some(
                        header
                            .value
                            .split_whitespace()
                            .map(Self::clean_angle_brackets)
                            .filter(|id| !id.is_empty())
                            .map(|id| id.to_string())
                            .collect(),
                    )
                }
                "subject" => envelope.subject = Some(header.value),
                "to" => envelope.to = Some(Self::parse_addr_list(&header.value)),
                "bcc" => envelope.bcc = Some(Self::parse_addr_list(&header.value)),
                "cc" => envelope.cc = Some(Self::parse_addr_list(&header.value)),
                _ => {}
            }
        }

        Ok(envelope)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PartBody {
    Attachment {
        #[serde(rename = "attachmentId")]
        attachment_id: String,
        size: u32,
    },
    Body {
        data: String,
        size: u32,
    },
    Empty {
        size: u32,
    },
}

impl Default for PartBody {
    fn default() -> Self {
        PartBody::Empty { size: 0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessagePart {
    pub body: PartBody,
    pub filename: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub headers: Vec<Header>,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    #[serde(rename = "partId")]
    pub part_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parts: Vec<MessagePart>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FullMessage {
    #[serde(default, rename = "historyId")]
    pub history_id: String,
    pub id: String,
    #[serde(rename = "internalDate")]
    pub internal_date: String,
    #[serde(rename = "labelIds")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub label_ids: Vec<String>,
    pub payload: Option<MessagePart>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw: Option<String>,
    #[serde(rename = "sizeEstimate")]
    pub size_estimate: Option<i64>,
    pub snippet: Option<String>,
    #[serde(rename = "threadId")]
    pub thread_id: Option<String>,
}

impl TryFrom<FullMessage> for FullMessageContent {
    type Error = RustMailerError;

    fn try_from(value: FullMessage) -> Result<Self, Self::Error> {
        let mut message_content = FullMessageContent::default();
        let mut attachments = Vec::new();
        let payload = value
            .payload
            .ok_or_else(|| raise_error!(
                "Missing `payload` field in Gmail API response; this usually indicates an unexpected API change or abnormal response".into(), 
                ErrorCode::InternalError
            )
        )?;

        walk_part(&payload, &mut message_content, &mut attachments)?;
        message_content.attachments = Some(attachments);
        Ok(message_content)
    }
}

fn walk_part(
    part: &MessagePart,
    message_content: &mut FullMessageContent,
    attachments: &mut Vec<AttachmentInfo>,
) -> RustMailerResult<()> {
    match &part.body {
        PartBody::Body { data, .. } => match part.mime_type.as_str() {
            "text/plain" => {
                if message_content.plain.is_none() {
                    let content = decode_body(data)?;
                    message_content.plain = Some(PlainText {
                        content,
                        truncated: false,
                    });
                }
            }
            "text/html" => {
                if message_content.html.is_none() {
                    let content = decode_body(data)?;
                    message_content.html = Some(content);
                }
            }
            _ => {}
        },
        PartBody::Attachment {
            attachment_id,
            size,
        } => {
            let mut a = AttachmentInfo::default();
            a.id = attachment_id.clone();
            a.file_type = part.mime_type.clone();
            a.filename = part.filename.clone();
            a.size = *size;

            for h in &part.headers {
                let name = h.name.to_ascii_lowercase();
                let mut value = h.value.clone();
                if name == "content-disposition" && value.to_ascii_lowercase().contains("inline;") {
                    a.inline = true;
                } else if name == "content-id" {
                    if value.starts_with('<') && value.ends_with('>') {
                        value = value
                            .trim_start_matches('<')
                            .trim_end_matches('>')
                            .to_string();
                    }
                    a.content_id = Some(value);
                } else if name == "content-transfer-encoding" {
                    a.transfer_encoding = Some(value);
                }
            }
            attachments.push(a);
        }
        PartBody::Empty { .. } => {}
    }

    if part.mime_type.starts_with("multipart/") {
        for sp in &part.parts {
            walk_part(sp, message_content, attachments)?;
        }
    }

    Ok(())
}

#[inline]
fn decode_body(data: &str) -> RustMailerResult<String> {
    let decoded = base64_decode_url_safe!(data).map_err(|e| {
        raise_error!(
            format!("Failed to decode base64_content: {}", e),
            ErrorCode::InternalError
        )
    })?;
    String::from_utf8(decoded).map_err(|e| {
        raise_error!(
            format!("Invalid UTF-8 in content: {}", e),
            ErrorCode::InternalError
        )
    })
}
