// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::cache::imap::mailbox::EnvelopeFlag;
use crate::modules::common::Addr;
use crate::modules::imap::section::{EmailBodyPart, ImapAttachment};
use crate::modules::utils::envelope_hash;
use crate::utc_now;
use mail_parser::Received as OriginalReceived;
use native_db::*;
use native_model::{native_model, Model};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
#[native_model(id = 1, version = 1)]
#[native_db(primary_key(pk -> String), secondary_key(create_envelope_id -> u64, unique))]
pub struct EmailEnvelope {
    /// The ID of the account owning the email.
    #[secondary_key]
    pub account_id: u64,
    /// The unique identifier of the mailbox where the email is stored (e.g., `MailBox::id`).
    /// Used for indexing to avoid updating indexes when mailboxes are renamed.
    #[secondary_key]
    pub mailbox_id: u64,
    /// The decoded, human-readable name of the mailbox (e.g., "INBOX", "Sent").
    pub mailbox_name: String,
    /// The unique identifier (IMAP UID) of the email within the mailbox.
    pub uid: u32,
    /// The date and time the email was received by the server, as a Unix timestamp in milliseconds.
    /// If `None`, the internal date is unavailable.
    pub internal_date: Option<i64>,
    /// The size of the email in bytes.
    pub size: u32,
    /// The flags associated with the email (e.g., `\Seen`, `\Answered`, `\Flagged`).
    /// Represented as a list of `EnvelopeFlag` for standard or custom flags.
    pub flags: Vec<EnvelopeFlag>,
    /// A hash of the email's flags for efficient comparison or indexing.
    pub flags_hash: u64,
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
    pub body_meta: Option<Vec<EmailBodyPart>>,
    /// Details about how the email was received, if available.
    pub received: Option<Received>,
}

impl EmailEnvelope {
    pub fn pk(&self) -> String {
        format!(
            "{}_{}",
            self.internal_date.unwrap_or(utc_now!()),
            envelope_hash(self.account_id, self.mailbox_id, self.uid)
        )
    }

    pub fn create_envelope_id(&self) -> u64 {
        envelope_hash(self.account_id, self.mailbox_id, self.uid)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
pub struct Received {
    /// The server or host from which the email was received, if available.
    pub from: Option<String>,
    /// The server or host that received the email, if available.
    pub by: Option<String>,
    /// The protocol used to receive the email (e.g., "SMTP", "ESMTP"), if specified.
    pub with: Option<String>,
    /// The date and time the email was received, as a Unix timestamp in milliseconds, if available.
    pub date: Option<i64>,
}

impl<'x> From<&OriginalReceived<'x>> for Received {
    fn from(value: &OriginalReceived<'x>) -> Self {
        let convert_host = |host: &mail_parser::Host<'x>| match host {
            mail_parser::Host::Name(cow) => cow.to_string(),
            mail_parser::Host::IpAddr(ip_addr) => ip_addr.to_string(),
        };

        let from = value.from.as_ref().map(convert_host);
        let by = value.by.as_ref().map(convert_host);
        let with = value.with.as_ref().map(|p| p.to_string());
        let date = value.date.map(|d| d.to_timestamp() * 1000);

        Self {
            from,
            by,
            with,
            date,
        }
    }
}
