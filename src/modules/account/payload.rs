// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use std::collections::BTreeSet;

use crate::modules::account::entity::{ImapConfig, MailerType, SmtpConfig};
use crate::modules::account::migration::AccountModel;
use crate::modules::account::since::DateSince;
use crate::modules::error::code::ErrorCode;
use crate::modules::error::RustMailerResult;
use crate::modules::token::AccountInfo;
use crate::{raise_error, validate_email};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
pub struct AccountCreateRequest {
    /// Email address associated with this account
    ///
    /// This field represents the user's email address and is required for account creation.
    /// Once set during account creation, the email address cannot be modified.
    #[oai(validator(custom = "crate::modules::common::validator::EmailValidator"))]
    pub email: String,
    /// Display name for the account (optional)
    pub name: Option<String>,
    /// IMAP server configuration
    pub imap: Option<ImapConfig>,
    /// SMTP server configuration
    pub smtp: Option<SmtpConfig>,
    /// Represents the account activation status.
    ///
    /// If this value is `false`, all account-related resources will be unavailable
    /// and any attempts to access them should return an error indicating the account
    /// is inactive.
    pub enabled: bool,
    /// Method used to access and manage emails.
    pub mailer_type: MailerType,
    /// Controls initial synchronization time range
    ///
    /// When dealing with large mailboxes, this restricts scanning to:
    /// - Messages after specified starting point
    /// - Or within sliding window
    ///
    /// ### Use Cases
    /// - Event-driven systems (only sync recent actionable emails)
    /// - First-time sync optimization for large accounts
    /// - Reducing server load during resyncs
    pub date_since: Option<DateSince>,
    /// Max emails to sync for this folder.  
    /// If not set, sync all emails.  
    /// otherwise sync up to `n` most recent emails (min 10).
    #[oai(validator(minimum(value = "100")))]
    pub folder_limit: Option<u32>,
    /// Minimal sync mode flag
    ///
    /// When enabled (`true`), only the most essential metadata will be synchronized:
    /// Recommended for:
    /// - Extremely resource-constrained environments
    /// - Accounts where only new message notification is needed
    pub minimal_sync: Option<bool>,
    /// Full sync interval (minutes), default 30m
    #[oai(validator(minimum(value = "10"), maximum(value = "10080")))]
    pub full_sync_interval_min: Option<i64>,
    /// Incremental sync interval (seconds), default 60s
    #[oai(validator(minimum(value = "10"), maximum(value = "3600")))]
    pub incremental_sync_interval_sec: i64,
    /// Optional proxy ID for establishing the connection to external APIs (e.g., Gmail, Outlook).
    /// - If `None` or not provided, the client will connect directly to the API server.
    /// - If `Some(proxy_id)`, the client will use the pre-configured proxy with the given ID for API requests.
    pub use_proxy: Option<u64>,
}

impl AccountCreateRequest {
    pub fn create_entity(self) -> RustMailerResult<AccountModel> {
        if let Some(date_since) = self.date_since.as_ref() {
            date_since.validate()?;
        }
        if matches!(self.mailer_type, MailerType::ImapSmtp) {
            if self.imap.is_none() || self.smtp.is_none() {
                return Err(raise_error!(
                    "Invalid input: Both 'imap' and 'smtp' must be provided.".into(),
                    ErrorCode::InvalidParameter
                ));
            }
            Self::validate_request(
                &self.imap.clone().unwrap(),
                &self.smtp.clone().unwrap(),
                &self.email,
            )?;

            if self.full_sync_interval_min.is_none() {
                return Err(raise_error!(
                    "Invalid input: 'full_sync_interval_min' must be provided.".into(),
                    ErrorCode::InvalidParameter
                ));
            }
        }
        Ok(AccountModel::create(self)?)
    }

    fn validate_request(imap: &ImapConfig, smtp: &SmtpConfig, email: &str) -> RustMailerResult<()> {
        imap.auth
            .validate()
            .map_err(|e| raise_error!(e.to_owned(), ErrorCode::InvalidParameter))?;
        smtp.auth
            .validate()
            .map_err(|e| raise_error!(e.to_owned(), ErrorCode::InvalidParameter))?;

        validate_email!(email)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
pub struct AccountUpdateRequest {
    /// Represents the account activation status.
    ///
    /// If this value is `false`, all account-related resources will be unavailable
    /// and any attempts to access them should return an error indicating the account
    /// is inactive.
    pub enabled: Option<bool>,
    /// Display name for the account (optional)
    pub name: Option<String>,
    /// IMAP server configuration
    pub imap: Option<ImapConfig>,
    /// SMTP server configuration
    pub smtp: Option<SmtpConfig>,
    /// Controls initial synchronization time range
    ///
    /// When dealing with large mailboxes, this restricts scanning to:
    /// - Messages after specified starting point
    /// - Or within sliding window
    ///
    /// ### Use Cases
    /// - Event-driven systems (only sync recent actionable emails)
    /// - First-time sync optimization for large accounts
    /// - Reducing server load during resyncs
    pub date_since: Option<DateSince>,
    /// Max emails to sync for this folder.  
    /// If not set, sync all emails.  
    /// otherwise sync up to `n` most recent emails (min 10).
    #[oai(validator(minimum(value = "100")))]
    pub folder_limit: Option<u32>,
    /// Configuration for selective folder (mailbox/label) synchronization
    ///
    /// - For IMAP/SMTP accounts:
    ///   Stores the mailbox names, since IMAP mailboxes do not have stable IDs.
    ///   Synchronization is keyed by the folder name.
    ///
    /// - For Gmail API accounts:
    ///   A Gmail label is treated as a mailbox (model mapping).
    ///   Since label names can be easily changed, the stable `labelId` is recorded here
    ///   instead of the label name.
    ///
    /// Defaults to standard folders (`INBOX`, `Sent`) if empty.
    /// Modified folders will be automatically synced on the next update.
    pub sync_folders: Option<Vec<String>>,
    /// Full sync interval (minutes)
    #[oai(validator(minimum(value = "10"), maximum(value = "10080")))]
    pub full_sync_interval_min: Option<i64>,
    /// Incremental sync interval (seconds)
    #[oai(validator(minimum(value = "10"), maximum(value = "3600")))]
    pub incremental_sync_interval_sec: Option<i64>,
    /// Optional proxy ID for establishing the connection to external APIs (e.g., Gmail, Outlook).
    /// - If `None` or not provided, the client will connect directly to the API server.
    /// - If `Some(proxy_id)`, the client will use the pre-configured proxy with the given ID for API requests.
    pub use_proxy: Option<u64>,
}

impl AccountUpdateRequest {
    pub fn validate_update_request(&self) -> RustMailerResult<()> {
        if let Some(date_since) = self.date_since.as_ref() {
            date_since.validate()?;
        }

        if let Some(mailboxes) = self.sync_folders.as_ref() {
            if mailboxes.is_empty() {
                return Err(raise_error!(
                    "Invalid configuration: 'sync_folders' cannot be empty. \
                     If you are modifying the subscription list, please provide at least one mailbox to subscribe to.".into(), ErrorCode::InvalidParameter
                ));
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]

pub struct MinimalAccount {
    pub id: u64,
    pub email: String,
    pub mailer_type: MailerType,
}

pub fn filter_accessible_accounts<'a>(
    all_accounts: &'a [MinimalAccount],
    allowed: &BTreeSet<AccountInfo>,
) -> Vec<MinimalAccount> {
    all_accounts
        .iter()
        .filter(|acct| allowed.iter().any(|a| a.id == acct.id))
        .cloned()
        .collect()
}
