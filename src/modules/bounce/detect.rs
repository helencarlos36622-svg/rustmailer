// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::bounce::models::{MimePart, MimeType};
use imap_proto::{BodyContentCommon, BodyStructure};

#[derive(Clone, Debug)]
/// A struct to extract file attachments from a given body structure.
pub struct BounceMessageExtractor<'a> {
    pub structure: &'a BodyStructure<'a>,
}

impl<'a> BounceMessageExtractor<'a> {
    /// Creates a new `AttachmentExtractor` with the specified body structure.
    pub fn new(structure: &'a BodyStructure<'a>) -> Self {
        Self { structure }
    }

    pub fn get_bounce_message_parts(&self) -> Option<Vec<MimePart>> {
        Self::find_bounce_message_parts(self.structure)
    }

    fn find_bounce_message_parts(body_structure: &BodyStructure<'a>) -> Option<Vec<MimePart>> {
        match body_structure {
            BodyStructure::Multipart { bodies, .. } => {
                // Recursively parse each part and collect attachments, if any part returns None, return None
                let mut attachments = Vec::new();
                for (_, body) in bodies.iter().enumerate() {
                    if let Some(mut part_attachments) = Self::find_bounce_message_parts(body) {
                        attachments.append(&mut part_attachments);
                    }
                }
                if attachments.is_empty() {
                    None
                } else {
                    Some(attachments)
                }
            }
            BodyStructure::Basic {
                common, other: _, ..
            }
            | BodyStructure::Message {
                common, other: _, ..
            }
            | BodyStructure::Text {
                common, other: _, ..
            } => {
                // Parse the attachment, and return as a single-element vector if Some
                Self::extract_bounce_part(common).map(|a| vec![a])
            }
        }
    }

    fn extract_bounce_part(common: &BodyContentCommon<'a>) -> Option<MimePart> {
        let ty = common.ty.ty.as_ref().to_ascii_lowercase();
        let subtype = common.ty.subtype.as_ref().to_ascii_lowercase();

        if !matches!(
            subtype.as_str(),
            "delivery-status" | "rfc822-headers" | "rfc822" | "feedback-report"
        ) {
            return None;
        }

        let mime_type = match (ty.as_str(), subtype.as_str()) {
            ("text", "delivery-status") => MimeType::TextDeliveryStatus,
            ("message", "delivery-status") => MimeType::MessageDeliveryStatus,
            ("text", "rfc822-headers") => MimeType::TextRfc822Headers,
            ("message", "rfc822-headers") => MimeType::MessageRfc822Headers,
            ("text", "rfc822") => MimeType::TextRfc822,
            ("message", "rfc822") => MimeType::MessageRfc822,
            ("message", "feedback-report") => MimeType::FeedbackReport,
            _ => return None,
        };

        Some(MimePart { mime_type })
    }
}
