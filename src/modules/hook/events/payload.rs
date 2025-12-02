// Copyright © 2025 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::{
    bounce::parser::{DeliveryStatus, FeedbackReport, RawEmailHeaders},
    common::Addr,
    message::content::FullMessageContent,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EmailAddedToFolder {
    /// Unique identifier of the account associated with the email.
    pub account_id: u64,
    /// Email address of the account associated with the email.
    pub account_email: String,
    /// Name of the mailbox (folder) where the email was added.
    pub mailbox_name: String,
    /// The unique ID of the message, either IMAP UID or Gmail API MID.
    /// - For IMAP accounts, this is the UID converted to a string.
    /// - For Gmail API accounts, this is the message ID returned by the API.
    pub id: String,
    /// Optional internal date (in milliseconds) assigned to the email by the server.
    pub internal_date: Option<i64>,
    /// Optional date (in milliseconds) of the email, typically from the email's header.
    pub date: Option<i64>,
    /// Size of the email in bytes.
    pub size: u32,
    /// List of flags associated with the email (e.g., Seen, Flagged).
    pub flags: Vec<String>,
    /// Optional list of CC (carbon copy) recipient addresses.
    pub cc: Option<Vec<Addr>>,
    /// Optional list of BCC (blind carbon copy) recipient addresses.
    pub bcc: Option<Vec<Addr>>,
    /// Optional sender address of the email.
    pub from: Option<Addr>,
    /// Optional message ID referenced by the email (e.g., for replies).
    pub in_reply_to: Option<String>,
    /// Optional sender address, as specified in the email's header.
    pub sender: Option<Addr>,
    /// Optional unique message ID of the email.
    pub message_id: Option<String>,
    /// Optional subject line of the email.
    pub subject: Option<String>,
    /// Content of the email, including body and related metadata.
    pub message: FullMessageContent,
    /// The identifier of the thread this email belongs to.
    /// This is computed based on `in_reply_to` / `references` / `message_id`.
    pub thread_id: String,
    /// Optional name of the thread to which the email belongs.
    pub thread_name: Option<String>,
    /// Optional list of reply-to addresses for the email.
    pub reply_to: Option<Vec<Addr>>,
    /// Optional list of recipient addresses (To field) for the email.
    pub to: Option<Vec<Addr>>,
    /// A list of labels applied to the message.
    ///
    /// Each element is a string representing a Gmail label ID (e.g., "INBOX", "UNREAD").
    /// This field reflects the current labels associated with the email.
    ///
    /// Note: This field is populated only for Gmail API accounts. For other account types, it will be empty.
    pub labels: Vec<String>,
}

// #[derive(Clone, Serialize, Deserialize, Debug)]
// pub struct Attachment {
//     /// Optional filename of the attachment, as specified in the email.
//     pub filename: Option<String>,
//     /// Indicates whether the attachment is inline (e.g., embedded in the email body) or not.
//     pub inline: bool,
//     /// Size of the attachment in bytes.
//     pub size: usize,
//     /// MIME type of the attachment (e.g., "application/pdf", "image/jpeg").
//     pub file_type: String,
// }

// impl From<ImapAttachment> for Attachment {
//     fn from(value: ImapAttachment) -> Self {
//         Self {
//             filename: value.filename,
//             inline: value.inline,
//             size: value.size,
//             file_type: value.file_type,
//         }
//     }
// }

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EmailFlagsChanged {
    /// Unique identifier of the account associated with the email.
    pub account_id: u64,
    /// Email address of the account associated with the email.
    pub account_email: String,
    /// Name of the mailbox (folder) containing the email.
    pub mailbox_name: String,
    /// Unique identifier (UID) of the email within the mailbox.
    pub uid: Option<u32>,
    /// Unique identifier (UID) of the email within the mailbox.
    pub mid: Option<String>,
    /// Optional sender address of the email.
    pub from: Option<Addr>,
    /// Optional list of recipient addresses (To field) for the email.
    pub to: Option<Vec<Addr>>,
    /// Optional unique message ID of the email.
    pub message_id: Option<String>,
    /// Optional subject line of the email.
    pub subject: Option<String>,
    /// Optional internal date (in milliseconds) assigned to the email by the server.
    pub internal_date: Option<i64>,
    /// Optional date (in milliseconds) of the email, typically from the email's header.
    pub date: Option<i64>,
    /// List of flags added to the email during the flag change event.
    pub flags_added: Vec<String>,
    /// List of flags removed from the email during the flag change event.
    pub flags_removed: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EmailSentSuccess {
    /// Unique identifier of the account associated with the email.
    pub account_id: u64,
    /// Email address of the account associated with the email.
    pub account_email: String,
    /// Sender email address of the email.
    pub from: String,
    /// List of recipient email addresses (To field) for the email.
    pub to: Vec<String>,
    /// Optional subject line of the email.
    pub subject: Option<String>,
    /// Unique message ID of the email.
    pub message_id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EmailSendingError {
    /// Unique identifier of the account associated with the email.
    pub account_id: u64,
    /// Email address of the account associated with the email.
    pub account_email: String,
    /// Sender email address of the email.
    pub from: String,
    /// List of recipient email addresses (To field) for the email.
    pub to: Vec<String>,
    /// Optional subject line of the email.
    pub subject: Option<String>,
    /// Unique message ID of the email.
    pub message_id: String,
    /// Optional error message describing the reason for the sending failure.
    pub error_msg: Option<String>,
    /// Optional count of retry attempts made for sending the email.
    pub retry_count: Option<usize>,
    /// Optional timestamp (in milliseconds) when the email sending was scheduled.
    pub scheduled_at: Option<i64>,
    /// Unique identifier of the task associated with the email sending attempt.
    pub task_id: u64,
    /// Optional maximum number of retry attempts allowed for sending the email.
    pub max_retries: Option<u32>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MailboxChange {
    /// Unique identifier of the account associated with the mailbox.
    pub account_id: u64,
    /// Email address of the account associated with the mailbox.
    pub account_email: String,
    /// Name of the mailbox (folder) affected by the change.
    pub mailbox_name: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MailboxDeletion {
    /// Unique identifier of the account associated with the mailbox.
    pub account_id: u64,
    /// Email address of the account associated with the mailbox.
    pub account_email: String,
    /// List of names of the mailboxes (folders) that were deleted.
    pub mailbox_names: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MailboxCreation {
    /// Unique identifier of the account associated with the mailbox.
    pub account_id: u64,
    /// Email address of the account associated with the mailbox.
    pub account_email: String,
    /// List of names of the mailboxes (folders) that were created.
    pub mailbox_names: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AccountChange {
    /// Unique identifier of the account.
    pub account_id: u64,
    /// Email address of the account.
    pub account_email: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EmailBounce {
    /// Unique identifier of the account associated with the email.
    pub account_id: u64,
    /// Email address of the account associated with the email.
    pub account_email: String,
    /// Name of the mailbox (folder) containing the bounced email.
    pub mailbox_name: String,
    /// Unique identifier (UID) of the bounced email within the mailbox.
    pub uid: u32,
    /// Optional internal date (in milliseconds) assigned to the bounced email by the server.
    pub internal_date: Option<i64>,
    /// Optional date (in milliseconds) of the bounced email, typically from the email's header.
    pub date: Option<i64>,
    /// Optional sender address of the bounced email.
    pub from: Option<Addr>,
    /// Optional subject line of the bounced email.
    pub subject: Option<String>,
    /// Optional list of recipient addresses (To field) for the bounced email.
    pub to: Option<Vec<Addr>>,
    /// Optional raw headers of the original email that bounced.
    pub original_headers: Option<RawEmailHeaders>,
    /// Optional delivery status information for the bounced email.
    pub delivery_status: Option<DeliveryStatus>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EmailFeedBackReport {
    /// Unique identifier of the account associated with the email.
    pub account_id: u64,
    /// Email address of the account associated with the email.
    pub account_email: String,
    /// Name of the mailbox (folder) containing the email associated with the feedback report.
    pub mailbox_name: String,
    /// Unique identifier (UID) of the email within the mailbox.
    pub uid: u32,
    /// Optional internal date (in milliseconds) assigned to the email by the server.
    pub internal_date: Option<i64>,
    /// Optional date (in milliseconds) of the email, typically from the email's header.
    pub date: Option<i64>,
    /// Optional sender address of the email.
    pub from: Option<Addr>,
    /// Optional subject line of the email.
    pub subject: Option<String>,
    /// Optional list of recipient addresses (To field) for the email.
    pub to: Option<Vec<Addr>>,
    /// Optional raw headers of the original email associated with the feedback report.
    pub original_headers: Option<RawEmailHeaders>,
    /// Optional feedback report details (e.g., spam or abuse report) for the email.
    pub feedback_report: Option<FeedbackReport>,
}

/// Represents an event triggered when an email is opened by a recipient.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EmailOpened {
    /// The unique identifier of the email campaign.
    pub campaign_id: String,
    /// The email address of the recipient who opened the email.
    pub recipient: String,
    /// The unique identifier of the email message.
    pub message_id: String,
    /// The user agent string of the client used to open the email.
    pub user_agent: String,
    /// The IP address of the client that opened the email.
    pub remote_ip: Option<String>,
}

/// Represents an event triggered when a link in an email is clicked by a recipient.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EmailLinkClicked {
    /// The unique identifier of the email campaign.
    pub campaign_id: String,
    /// The email address of the recipient who clicked the link.
    pub recipient: String,
    /// The unique identifier of the email message.
    pub message_id: String,
    /// The URL that was clicked in the email.
    pub url: String,
    /// The IP address of the client that clicked the link.
    pub remote_ip: Option<String>,
    /// The user agent string of the client used to click the link.
    pub user_agent: String,
}
