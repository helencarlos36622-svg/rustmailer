// Copyright © 2025 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use core::convert::Into;
use std::{collections::HashMap, fmt, sync::LazyLock};

use payload::{
    AccountChange, EmailAddedToFolder, EmailBounce, EmailFeedBackReport, EmailFlagsChanged,
    EmailSendingError, EmailSentSuccess, MailboxChange, MailboxCreation, MailboxDeletion,
};
use poem_openapi::Enum;
use serde::{Deserialize, Serialize};

use crate::{
    generate_token, id,
    modules::{
        bounce::parser::{DeliveryStatus, FeedbackReport, RawEmailHeaders},
        cache::imap::mailbox::{EmailFlag, EnvelopeFlag},
        common::Addr,
        error::{code::ErrorCode, RustMailerResult},
        hook::events::payload::{EmailLinkClicked, EmailOpened},
        message::content::{FullMessageContent, PlainText},
        settings::cli::SETTINGS,
    },
    raise_error, utc_now,
};

pub mod payload;
#[cfg(test)]
mod tests;

pub static EVENT_EXAMPLES: LazyLock<serde_json::Value> =
    LazyLock::new(RustMailerEvent::generate_event_examples);

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RustMailerEvent {
    /// Unique identifier for the event.
    pub event_id: u64,
    /// Type of event that triggered the webhook (e.g., email added, sent, or bounced).
    pub event_type: EventType,
    /// URL of the instance that generated the event.
    pub instance_url: String,
    /// Timestamp (in milliseconds) when the event occurred.
    pub timestamp: i64,
    /// Payload containing detailed data associated with the event.
    pub payload: EventPayload,
}

impl RustMailerEvent {
    pub fn new(event_type: EventType, payload: EventPayload) -> Self {
        Self {
            event_id: id!(96),
            event_type,
            instance_url: SETTINGS.rustmailer_public_url.clone(),
            timestamp: utc_now!(),
            payload,
        }
    }

    pub fn to_json_value(&self) -> RustMailerResult<serde_json::Value> {
        serde_json::to_value(&self)
            .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))
    }
}

#[derive(Clone, Debug, Hash, Default, Eq, PartialEq, Serialize, Deserialize, Enum)]
pub enum EventType {
    /// Default event triggered when an email is added to a folder, including new emails, appended emails, or emails moved or copied from another folder.
    #[default]
    EmailAddedToFolder,
    /// Event triggered when email flags are modified (e.g., marked as replied, read, or other custom flags), excluding the Recent flag.
    EmailFlagsChanged,
    /// Event triggered when an email is successfully sent to the SMTP server, not when it is queued for sending.
    EmailSentSuccess,
    /// Event triggered when an error occurs during email sending to the SMTP server, sent for each retry attempt that fails.
    EmailSendingError,
    /// Event triggered when the UID validity of a mailbox changes.
    UIDValidityChange,
    /// Event triggered when a mailbox is deleted, encompassing changes across all mailboxes in the email account, not limited to synchronized folder lists.
    MailboxDeletion,
    /// Event triggered when a new mailbox is created, encompassing changes across all mailboxes in the email account, not limited to synchronized folder lists.
    MailboxCreation,
    /// Event triggered when an account completes its first synchronization.
    AccountFirstSyncCompleted,
    /// Event triggered when an email bounces.
    EmailBounce,
    /// Event triggered when a feedback report is received for an email.
    EmailFeedBackReport,
    /// Event triggered when an email is opened by the recipient.
    EmailOpened,
    /// Event triggered when a link in an email is clicked by the recipient.
    EmailLinkClicked,
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventType::EmailAddedToFolder => write!(f, "EmailAddedToFolder"),
            EventType::EmailFlagsChanged => write!(f, "EmailFlagsChanged"),
            EventType::EmailSentSuccess => write!(f, "EmailSentSuccess"),
            EventType::EmailSendingError => write!(f, "EmailSendingError"),
            EventType::UIDValidityChange => write!(f, "UIDValidityChange"),
            EventType::MailboxDeletion => write!(f, "MailboxDeletion"),
            EventType::MailboxCreation => write!(f, "MailboxCreation"),
            EventType::AccountFirstSyncCompleted => write!(f, "AccountFirstSyncCompleted"),
            EventType::EmailBounce => write!(f, "EmailBounce"),
            EventType::EmailFeedBackReport => write!(f, "EmailFeedBackReport"),
            EventType::EmailOpened => write!(f, "EmailOpened"),
            EventType::EmailLinkClicked => write!(f, "EmailLinkClicked"),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum EventPayload {
    EmailAddedToFolder(EmailAddedToFolder),
    EmailFlagsChanged(EmailFlagsChanged),
    EmailSentSuccess(EmailSentSuccess),
    EmailSendingError(EmailSendingError),
    UIDValidityChange(MailboxChange),
    MailboxDeletion(MailboxDeletion),
    MailboxCreation(MailboxCreation),
    AccountFirstSyncCompleted(AccountChange),
    EmailBounce(EmailBounce),
    EmailFeedBackReport(EmailFeedBackReport),
    EmailOpened(EmailOpened),
    EmailLinkClicked(EmailLinkClicked),
}

impl RustMailerEvent {
    pub fn generate_event_examples() -> serde_json::Value {
        let account_email = "user@example.com".to_string();
        let timestamp = utc_now!();
        let instance_url = "https://mailer.example.com".to_string();

        // Helper function to create Addr
        let addr = |email: &str| Addr {
            name: Some("Test User".to_string()),
            address: Some(email.to_string()),
        };

        let mut map = HashMap::new();

        macro_rules! insert_event {
            ($variant:ident, $payload:expr) => {
                map.insert(
                    EventType::$variant,
                    RustMailerEvent {
                        event_id: id!(96),
                        event_type: EventType::$variant,
                        instance_url: instance_url.clone(),
                        timestamp,
                        payload: EventPayload::$variant($payload),
                    },
                );
            };
        }

        insert_event!(
            EmailAddedToFolder,
            EmailAddedToFolder {
                account_id: id!(64),
                account_email: account_email.clone(),
                mailbox_name: "INBOX".into(),
                id: "1001".to_string(),
                internal_date: Some(timestamp),
                date: Some(timestamp),
                size: 2048,
                flags: vec![EnvelopeFlag::new(EmailFlag::Seen, None).to_string()],
                cc: Some(vec![addr("cc@example.com")]),
                bcc: None,
                from: Some(addr("sender@example.com")),
                in_reply_to: Some("<msg123@server.com>".into()),
                sender: Some(addr("sender@example.com")),
                message_id: Some("<msg456@server.com>".into()),
                subject: Some("Meeting Notes".into()),
                message: FullMessageContent {
                    plain: Some(PlainText {
                        content: String::from("Welcome to use rustmailer!"),
                        truncated: false,
                    }),
                    html: Some(String::from("<p>Welcome to use rustmailer!</p>")),
                    attachments: None
                },
                thread_name: Some("Meeting Thread".into()),
                thread_id: id!(64).to_string(),
                reply_to: Some(vec![addr("reply@example.com")]),
                to: Some(vec![addr("recipient@example.com")]),
                labels: vec![]
            }
        );

        insert_event!(
            EmailFlagsChanged,
            EmailFlagsChanged {
                account_id: id!(64),
                account_email: account_email.clone(),
                mailbox_name: "INBOX".into(),
                uid: Some(1003),
                from: Some(addr("sender@example.com")),
                to: Some(vec![addr("recipient@example.com")]),
                message_id: Some("<msg101@server.com>".into()),
                subject: Some("Updated Email".into()),
                internal_date: Some(timestamp),
                date: Some(timestamp),
                flags_added: vec![EnvelopeFlag::new(EmailFlag::Seen, None).to_string()],
                flags_removed: vec![EnvelopeFlag::new(EmailFlag::Flagged, None).to_string()],
                mid: None
            }
        );

        insert_event!(
            EmailSentSuccess,
            EmailSentSuccess {
                account_id: id!(64),
                account_email: account_email.clone(),
                from: "user@example.com".into(),
                to: vec!["recipient@example.com".into()],
                subject: Some("Confirmation Email".into()),
                message_id: "<msg202@server.com>".into(),
            }
        );

        insert_event!(
            EmailSendingError,
            EmailSendingError {
                account_id: id!(64),
                account_email: account_email.clone(),
                from: "user@example.com".into(),
                to: vec!["recipient@example.com".into()],
                subject: Some("Failed Email".into()),
                message_id: "<msg303@server.com>".into(),
                error_msg: Some("SMTP timeout".into()),
                retry_count: Some(2),
                scheduled_at: Some(timestamp),
                task_id: id!(96),
                max_retries: Some(5),
            }
        );

        insert_event!(
            UIDValidityChange,
            MailboxChange {
                account_id: id!(64),
                account_email: account_email.clone(),
                mailbox_name: "INBOX".into(),
            }
        );

        insert_event!(
            MailboxDeletion,
            MailboxDeletion {
                account_id: id!(64),
                account_email: account_email.clone(),
                mailbox_names: vec!["Trash".into()],
            }
        );

        insert_event!(
            MailboxCreation,
            MailboxCreation {
                account_id: id!(64),
                account_email: account_email.clone(),
                mailbox_names: vec!["Projects".into()],
            }
        );

        insert_event!(
            AccountFirstSyncCompleted,
            AccountChange {
                account_id: id!(64),
                account_email: account_email.clone(),
            }
        );

        insert_event!(
            EmailBounce,
            EmailBounce {
                account_id: id!(64),
                account_email: account_email.clone(),
                mailbox_name: "INBOX".into(),
                uid: 1004,
                internal_date: Some(timestamp),
                date: Some(timestamp),
                from: Some(addr("sender@example.com")),
                subject: Some("Undeliverable Email".into()),
                to: Some(vec![addr("recipient@example.com")]),
                original_headers: Some(RawEmailHeaders {
                    message_id: Some(generate_token!(96)),
                    subject: Some("Meeting Notes".into()),
                    from: Some("sender@example.com".into()),
                    to: Some(vec![
                        "recipient1@example.com".into(),
                        "recipient2@example.com".into(),
                    ]),
                    date: Some(1697059200000),
                }),
                delivery_status: Some(DeliveryStatus {
                    recipient: Some("user@example.com".into()),
                    action: Some("failed".into()),
                    status: Some("5.0.0".into()),
                    error_source: Some("remote-mta".into()),
                    diagnostic_code: Some("smtp; 550 5.1.1 User unknown".into()),
                    remote_mta: Some("mail.example.com".into()),
                    reporting_mta: Some("mx1.example.org".into()),
                    received_from_mta: Some("localhost".into()),
                    postfix_queue_id: Some("ABCDEF12345".into()),
                    arrival_date: Some("2023-05-15T14:32:18Z".into()),
                    original_message_id: Some("<123456789@example.org>".into()),
                    postfix_sender: Some("sender@example.org".into()),
                })
            }
        );

        insert_event!(
            EmailFeedBackReport,
            EmailFeedBackReport {
                account_id: id!(64),
                account_email: account_email.clone(),
                mailbox_name: "INBOX".into(),
                uid: 1004,
                internal_date: Some(timestamp),
                date: Some(timestamp),
                from: Some(addr("sender@example.com")),
                subject: Some("Undeliverable Email".into()),
                to: Some(vec![addr("recipient@example.com")]),
                original_headers: Some(RawEmailHeaders {
                    message_id: Some(generate_token!(96)),
                    subject: Some("Meeting Notes".into()),
                    from: Some("sender@example.com".into()),
                    to: Some(vec![
                        "recipient1@example.com".into(),
                        "recipient2@example.com".into(),
                    ]),
                    date: Some(1697059200000),
                }),
                feedback_report: Some(FeedbackReport {
                    feedback_type: Some("abuse".into()),
                    version: Some("1.0".into()),
                    user_agent: None,
                    original_mail_from: Some("sender@example.com".into()),
                    original_rcpt_to: Some("recipient@example.com".into()),
                    original_envelope_id: None,
                    received_date: Some("2023-10-12".into()),
                    reported_domain: Some("example.com".into()),
                    reported_uri: None,
                    reporting_mta: Some("mta.example.com".into()),
                    source_ip: Some("192.168.1.1".into()),
                    source_port: None,
                    spf_dns: None,
                    delivery_result: Some("failed".into()),
                    authentication_results: None,
                    auth_failure: None,
                    arrival_date: Some("2023-10-12T10:00:00Z".into()),
                }),
            }
        );

        insert_event!(
            EmailOpened,
            EmailOpened {
                campaign_id: "camp_67890".to_string(),
                recipient: "jane.doe@company.org".to_string(),
                message_id: "msg_4567".to_string(),
                user_agent: "Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X) Mobile/15E148"
                    .to_string(),
                remote_ip: Some("203.0.113.10".to_string()),
            }
        );

        insert_event!(
            EmailLinkClicked,
            EmailLinkClicked {
                campaign_id: "camp_67890".to_string(),
                recipient: "jane.doe@company.org".to_string(),
                message_id: "msg_4567".to_string(),
                url: "https://example.com/unsubscribe".to_string(),
                remote_ip: Some("203.0.113.10".to_string()),
                user_agent: "Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X) Mobile/15E148"
                    .to_string(),
            }
        );

        serde_json::to_value(map).unwrap()
    }
}
