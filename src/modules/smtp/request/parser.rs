// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::{
    modules::error::{code::ErrorCode, RustMailerResult},
    raise_error,
};
use mail_parser::{
    decoders::{base64::base64_decode_stream, quoted_printable::quoted_printable_decode},
    MessageParser, MimeHeaders,
};
use mail_send::mail_builder::mime::BodyPart;
use std::borrow::Cow;

#[derive(Debug)]
pub struct EmlData {
    pub subject: Option<String>,
    pub html: Option<String>,
    pub text: Option<String>,
    pub attachments: Option<Vec<AttachmentFromEml>>,
}

#[derive(Debug)]
pub struct AttachmentFromEml {
    pub content: BodyPart<'static>,
    pub mime_type: String,
    pub inline: bool,
    pub file_name: Option<String>,
    pub content_id: Option<String>,
}

impl EmlData {
    fn build_mime_type(
        c_type: Option<Cow<'_, str>>,
        c_subtype: Option<Option<Cow<'_, str>>>,
    ) -> RustMailerResult<String> {
        c_type
            .zip(c_subtype.flatten())
            .map(|(t, s)| format!("{}/{}", t, s))
            .ok_or_else(|| {
                raise_error!(
                    "Failed to build MIME type: missing or invalid content type/subtype".into(),
                    ErrorCode::EmlFileParseError
                )
            })
    }

    pub fn parse(input: &str) -> RustMailerResult<EmlData> {
        let message = MessageParser::new().parse(input).ok_or_else(|| {
            raise_error!(
                "Invalid EML format: failed to parse email content (RFC 5322 compliance required)"
                    .into(),
                ErrorCode::EmlFileParseError
            )
        })?;

        let attachments = if message.attachment_count() > 0 {
            Some(
                message
                    .attachments()
                    .filter(|a| a.is_text() || a.is_binary())
                    .map(|attachment| Self::process_attachment(attachment))
                    .collect::<RustMailerResult<Vec<_>>>()?,
            )
        } else {
            None
        };

        Ok(EmlData {
            subject: message.subject().map(String::from),
            html: message.body_html(0).map(String::from),
            text: message.body_text(0).map(String::from),
            attachments,
        })
    }

    fn process_attachment(
        attachment: &mail_parser::MessagePart<'_>,
    ) -> RustMailerResult<AttachmentFromEml> {
        let encoded_content = attachment.contents();
        let decoded_content = match attachment.encoding {
            mail_parser::Encoding::None => encoded_content.to_vec(),
            mail_parser::Encoding::QuotedPrintable => quoted_printable_decode(encoded_content)
                .ok_or_else(|| {
                    raise_error!(
                        "Failed to decode Quoted-Printable content".into(),
                        ErrorCode::EmlFileParseError
                    )
                })?,
            mail_parser::Encoding::Base64 => {
                base64_decode_stream(encoded_content.iter(), encoded_content.len(), u8::MAX)
                    .ok_or_else(|| {
                        raise_error!(
                            "Failed to decode Base64 content".into(),
                            ErrorCode::EmlFileParseError
                        )
                    })?
            }
        };

        let content = if attachment.is_text() {
            BodyPart::Text(Cow::Owned(String::from_utf8(decoded_content).map_err(
                |e| {
                    raise_error!(
                        format!("Invalid UTF-8 in text content: {}", e),
                        ErrorCode::EmlFileParseError
                    )
                },
            )?))
        } else {
            BodyPart::Binary(Cow::Owned(decoded_content))
        };

        let content_type = attachment.content_type();
        let mime_type = Self::build_mime_type(
            content_type.map(|c| c.c_type.clone()),
            content_type.map(|c| c.c_subtype.clone()),
        )?;

        Ok(AttachmentFromEml {
            content,
            mime_type,
            inline: content_type.is_some_and(|c| c.is_inline()),
            file_name: attachment.attachment_name().map(String::from),
            content_id: attachment.content_id().map(String::from),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_parse_eml_from_file() {
        let file_path = Path::new("E:\\wow.eml");

        // Skip test if file doesn't exist instead of panicking
        if !file_path.exists() {
            eprintln!("Skipping test: EML file not found at {:?}", file_path);
            return;
        }

        let eml_content = fs::read_to_string(file_path).expect("Failed to read EML file");

        let result = EmlData::parse(&eml_content);
        assert!(
            result.is_ok(),
            "Failed to parse EML file: {:?}",
            result.err()
        );

        if let Ok(eml_data) = result {
            println!("Parsed EML data: {:?}", eml_data);
        }
    }
}
