// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use crate::{
    calculate_hash, id,
    modules::{
        cache::imap::{
            envelope::Received,
            mailbox::{EmailFlag, EnvelopeFlag},
            migration::EmailEnvelopeV3,
        },
        common::Addr,
        imap::section::{EmailBodyPart, ImapAttachment},
    },
};

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
pub struct Envelope {
    /// The unique identifier of the email message across different systems.
    ///
    /// This field maps to the primary message ID used by the respective API or protocol:
    ///
    /// - **For IMAP accounts:** This is the **UID** (Unique Identifier) converted to a string.
    /// - **For Gmail API accounts:** This is the **Message ID (MID)** returned by the Gmail API.
    /// - **For Microsoft Graph API accounts:** This is the **ID** property (e.g., the base64-encoded EWS ID) of the message object.
    pub id: String,
    /// The ID of the account owning the email.
    pub account_id: u64,
    /// The unique identifier of the mailbox where the email is stored (e.g., `MailBox::id`).
    /// Used for indexing to avoid updating indexes when mailboxes are renamed.
    pub mailbox_id: u64,
    /// The decoded, human-readable name of the mailbox (e.g., "INBOX", "Sent").
    pub mailbox_name: String,
    /// The date and time the email was received by the server, as a Unix timestamp in milliseconds.
    /// If `None`, the internal date is unavailable.
    pub internal_date: Option<i64>,
    /// The size of the email in bytes.
    pub size: u32,
    /// The flags associated with the email (e.g., `\Seen`, `\Answered`, `\Flagged`).
    /// Represented as a list of `EnvelopeFlag` for standard or custom flags.
    ///
    /// **Note:** Available only for IMAP accounts.
    pub flags: Option<Vec<EnvelopeFlag>>,
    /// A hash of the email's flags for efficient comparison or indexing.
    ///
    /// **Note:** Available only for IMAP accounts.
    pub flags_hash: Option<u64>,
    /// The blind carbon copy (BCC) recipient(s) of the email, if any.
    pub bcc: Option<Vec<Addr>>,
    /// The carbon copy (CC) recipient(s) of the email, if any.
    pub cc: Option<Vec<Addr>>,
    /// The date the email was sent, as a Unix timestamp in milliseconds, if available.
    pub date: Option<i64>,
    /// The sender's address, including name and email, if available.
    pub from: Option<Addr>,
    /// The message ID of the email to which this email is a reply, if applicable.
    pub in_reply_to: Option<String>,
    /// The actual sender's address, if different from the `from` field.
    pub sender: Option<Addr>,
    /// The return address for undeliverable emails, if specified.
    pub return_address: Option<String>,
    /// The unique message ID of the email, typically used for threading.
    pub message_id: Option<String>,
    /// The subject of the email, if available.
    pub subject: Option<String>,
    /// The name of the thread this email belongs to, if applicable.
    pub thread_name: Option<String>,
    /// The identifier of the thread this email belongs to.
    /// This is computed based on `in_reply_to` / `references` / `message_id`.
    pub thread_id: String,
    /// The MIME version of the email (e.g., "1.0"), if specified.
    pub mime_version: Option<String>,
    /// A list of message IDs referenced by this email, used for threading.
    pub references: Option<Vec<String>>,
    /// The address(es) to which replies should be sent, if specified.
    pub reply_to: Option<Vec<Addr>>,
    /// The primary recipient(s) of the email, if any.
    pub to: Option<Vec<Addr>>,
    /// A list of attachments included in the email, if any.
    ///
    /// Each `ImapAttachment` item contains metadata including the part ID and MIME type,
    /// which indicates the exact location of the attachment in the raw message structure.
    /// This allows the backend to directly fetch specific attachments without retrieving
    /// the entire message content.
    ///
    /// This is particularly useful for accounts configured with minimal sync, where full
    /// message bodies are not cached locally. By including this data in the API response,
    /// the client can request to download only the required attachment via a follow-up
    /// API call, improving both efficiency and user experience.
    ///
    /// Developers do not need to understand the internal IMAP part structure — this
    /// metadata provides a clean abstraction for fetching specific attachments.
    /// **Note:** Available only for IMAP accounts.
    pub attachments: Option<Vec<ImapAttachment>>,
    /// Metadata for the email's body parts (e.g., plain text, HTML), if available.
    ///
    /// Each `EmailBodyPart` contains detailed metadata (such as part ID, content type,
    /// and charset) describing a portion of the email body. This enables precise access
    /// to body content, such as plain text or HTML sections, without downloading the full
    /// raw message from the server.
    ///
    /// This is especially helpful for lightweight clients or minimized-sync accounts that
    /// do not cache full email content. The frontend can pass this metadata back to the
    /// server to retrieve only the desired portion of the message (e.g., the HTML body),
    /// which significantly reduces bandwidth and latency.
    ///
    /// By abstracting the complexity of MIME part navigation, developers can efficiently
    /// retrieve specific parts of an email without handling the low-level IMAP structure.
    /// **Note:** Available only for IMAP accounts.
    pub body_meta: Option<Vec<EmailBodyPart>>,
    /// Details about how the email was received, if available.
    /// **Note:** Available only for IMAP accounts.
    pub received: Option<Received>,
    /// A list of labels applied to the message.
    ///
    /// Each element is a string representing a Gmail label name (e.g., "INBOX", "UNREAD").
    /// This field reflects the current labels associated with the email.
    ///
    /// **Note:** This field is populated only for Gmail API accounts. For other account types, it will be empty.
    pub labels: Vec<String>,

    pub is_read: bool,
}

impl Envelope {
    pub fn compute_thread_id(&self) -> u64 {
        if self.in_reply_to.is_some() && self.references.as_ref().map_or(false, |r| !r.is_empty()) {
            return calculate_hash!(&self.references.as_ref().unwrap()[0]);
        }
        if let Some(message_id) = self.message_id.as_ref() {
            return calculate_hash!(message_id);
        }
        id!(128)
    }
}

impl From<EmailEnvelopeV3> for Envelope {
    fn from(value: EmailEnvelopeV3) -> Self {
        Self {
            id: value.uid.to_string(),
            account_id: value.account_id,
            mailbox_id: value.mailbox_id,
            mailbox_name: value.mailbox_name,
            internal_date: value.internal_date,
            size: value.size,
            is_read: value
                .flags
                .iter()
                .any(|f| matches!(f.flag, EmailFlag::Seen)),
            flags: Some(value.flags),
            flags_hash: Some(value.flags_hash),
            bcc: value.bcc,
            cc: value.cc,
            date: value.date,
            from: value.from,
            in_reply_to: value.in_reply_to,
            sender: value.sender,
            return_address: value.return_address,
            message_id: value.message_id,
            subject: value.subject,
            thread_name: value.thread_name,
            thread_id: value.thread_id.to_string(),
            mime_version: value.mime_version,
            references: value.references,
            reply_to: value.reply_to,
            to: value.to,
            attachments: value.attachments,
            body_meta: value.body_meta,
            received: value.received,
            labels: value.labels,
        }
    }
}
