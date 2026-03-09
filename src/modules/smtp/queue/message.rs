// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::{
    modules::{
        error::{code::ErrorCode, RustMailerError, RustMailerResult},
        scheduler::{model::TaskStatus, nativedb::TaskMetaEntity},
        smtp::request::{task::SmtpTask, DSNConfig, MailEnvelope},
    },
    raise_error,
};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
pub struct SendEmailTask {
    /// A unique identifier for the email sending task.
    pub id: u64,
    /// The Unix timestamp (milliseconds since epoch) when the task was created.
    pub created_at: i64,
    /// The current status of the task (e.g., Scheduled, Running, Success, Failed).
    pub status: TaskStatus,
    /// An optional reason why the task was stopped (e.g., manual cancellation).
    pub stopped_reason: Option<String>,
    /// An optional error message if the task failed.
    pub error: Option<String>,
    /// The duration (in milliseconds) of the last sending attempt, if applicable.
    pub last_duration_ms: Option<usize>,
    /// The number of retry attempts made, if applicable.
    pub retry_count: Option<usize>,
    /// The Unix timestamp (milliseconds since epoch) when the task is scheduled to execute.
    pub scheduled_at: i64,
    /// The ID of the account associated with the email sending task.
    pub account_id: u64,
    /// The email address of the account sending the email.
    pub account_email: String,
    /// The optional subject line of the email.
    pub subject: Option<String>,
    /// The message ID of the email (e.g., for threading or reference).
    pub message_id: String,
    /// The sender's email address (used in the email's "From" header).
    pub from: String,
    /// A list of primary recipient email addresses (used in the email's "To" header).
    pub to: Vec<String>,
    /// An optional list of CC (carbon copy) recipient email addresses.
    pub cc: Option<Vec<String>>,
    /// An optional list of BCC (blind carbon copy) recipient email addresses.
    pub bcc: Option<Vec<String>>,
    /// The number of attachments included in the email.
    pub attachment_count: usize,
    /// A unique key for caching the email content.
    pub cache_key: String,
    /// The optional email envelope containing sender and recipient addresses for SMTP.
    /// If `None`, the SMTP client may derive addresses from `from`, `to`, `cc`, and `bcc`.
    pub envelope: Option<MailEnvelope>,
    /// Whether to save a copy of the email to the sent folder after successful delivery.
    pub save_to_sent: bool,
    /// The optional name of the folder where the email should be saved if `save_to_sent` is true.
    /// If `None` and `save_to_sent` is true, a default folder (e.g., "Sent") may be used.
    pub sent_folder: Option<String>,
    /// An optional Unix timestamp (milliseconds since epoch) specifying when to send the email.
    /// If `None`, the email is sent immediately.
    pub send_at: Option<i64>,
    /// The optional name of the Mail Transfer Agent (MTA) to use for sending the email.
    /// If `None`, the SMTP client uses its default MTA.
    pub mta: Option<u64>,
    /// The optional configuration for Delivery Status Notifications (DSN) to track delivery status.
    /// If `None`, no DSNs are requested.
    pub dsn: Option<DSNConfig>,
    /// An optional flag indicating whether the email is a reply or forward.
    /// - `true`: The email is a reply to an existing email.
    /// - `false`: The email is a forward of an existing email.
    /// - `null`: The email is neither a reply nor a forward (a new email).
    pub reply: Option<bool>,
    /// The optional mailbox name (e.g., "INBOX") of the original email in reply or forward scenarios.
    /// Used when `reply` is `Some(true)` (reply) or `Some(false)` (forward) to reference the original email's mailbox.
    pub mailbox: Option<String>,
    /// The optional unique ID (e.g., IMAP UID) of the original email in reply or forward scenarios.
    /// Used when `reply` is `Some(true)` (reply) or `Some(false)` (forward) to reference the original email.
    pub uid: Option<u32>,
}

impl TryFrom<&TaskMetaEntity> for SendEmailTask {
    type Error = RustMailerError;

    fn try_from(task: &TaskMetaEntity) -> RustMailerResult<Self> {
        // Deserialize the task_params string into SmtpTask
        let smtp_task: SmtpTask = serde_json::from_str(&task.task_params)
            .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?;

        Ok(SendEmailTask {
            id: task.id,
            created_at: task.created_at,
            status: task.status.clone(),
            stopped_reason: task.stopped_reason.clone(),
            error: task.last_error.clone(),
            last_duration_ms: task.last_duration_ms,
            retry_count: task.retry_count,
            scheduled_at: task.next_run,
            account_id: smtp_task.account_id,
            account_email: smtp_task.account_email,
            subject: smtp_task.subject,
            message_id: smtp_task.message_id,
            from: smtp_task.from,
            to: smtp_task.to,
            cc: smtp_task.cc,
            bcc: smtp_task.bcc,
            attachment_count: smtp_task.attachment_count,
            cache_key: smtp_task.cache_key,
            envelope: smtp_task.control.as_ref().and_then(|c| c.envelope.clone()),
            save_to_sent: smtp_task
                .control
                .as_ref()
                .and_then(|c| c.save_to_sent)
                .unwrap_or(false),
            sent_folder: smtp_task
                .control
                .as_ref()
                .and_then(|c| c.sent_folder.clone()),
            send_at: smtp_task.control.as_ref().and_then(|c| c.send_at),
            mta: smtp_task.control.as_ref().and_then(|c| c.mta),
            dsn: smtp_task.control.as_ref().and_then(|c| c.dsn.clone()),
            reply: smtp_task.answer_email.as_ref().map(|a| a.reply),
            mailbox: smtp_task.answer_email.as_ref().map(|a| a.mailbox.clone()),
            uid: smtp_task.answer_email.as_ref().map(|a| a.uid),
        })
    }
}
