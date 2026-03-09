// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use std::collections::BTreeSet;

use crate::modules::error::RustMailerResult;
use crate::{encrypt, modules::account::since::DateSince};
use native_db::*;
use native_model::{native_model, Model};

use poem_openapi::{Enum, Object};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
#[native_model(id = 5, version = 1)]
#[native_db(primary_key(pk -> String))]
pub struct Account {
    /// Unique account identifier
    #[secondary_key(unique)]
    pub id: u64,
    /// IMAP server configuration
    pub imap: ImapConfig,
    /// SMTP server configuration
    pub smtp: SmtpConfig,
    /// Represents the account activation status.
    ///
    /// If this value is `false`, all account-related resources will be unavailable
    /// and any attempts to access them should return an error indicating the account
    /// is inactive.
    pub enabled: bool,
    /// Email address associated with this account
    #[oai(validator(custom = "crate::modules::common::validator::EmailValidator"))]
    pub email: String,
    /// Display name for the account (optional)
    pub name: Option<String>,
    /// Minimal sync mode flag
    ///
    /// When enabled (`true`), only the most essential metadata will be synchronized:
    /// Recommended for:
    /// - Extremely resource-constrained environments
    /// - Accounts where only new message notification is needed
    pub minimal_sync: bool,
    /// IMAP Server-supported capability flags
    pub capabilities: Vec<String>,
    /// DSN (Delivery Status Notification) support flag
    pub dsn_capable: Option<bool>,
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
    /// Configuration for selective folder synchronization
    ///
    /// Defaults to standard folders (`INBOX`, `Sent`) if empty.
    /// Modified folders will be automatically synced on next update.
    pub sync_folders: Vec<String>,
    /// Full sync interval (minutes), default 30m
    pub full_sync_interval_min: i64,
    /// Incremental sync interval (seconds), default 60s
    pub incremental_sync_interval_sec: i64,
    /// Tracks known mail folders and detects changes (creations/deletions)
    pub known_folders: BTreeSet<String>,
    /// Creation timestamp (UNIX epoch milliseconds)
    pub created_at: i64,
    /// Last update timestamp (UNIX epoch milliseconds)
    pub updated_at: i64,
}

impl Account {
    fn pk(&self) -> String {
        format!("{}_{}", self.created_at, self.id)
    }
}

#[derive(Clone, Default, Debug, Eq, PartialEq, Serialize, Deserialize, Object)]
pub struct ImapConfig {
    /// IMAP server hostname or IP address
    #[oai(validator(max_length = 253, pattern = r"^[a-zA-Z0-9\-\.]+$"))]
    pub host: String,
    /// IMAP server port number
    #[oai(validator(minimum(value = "1"), maximum(value = "65535")))]
    pub port: u16,
    /// Connection encryption method
    pub encryption: Encryption,
    /// Authentication configuration
    pub auth: AuthConfig,
    /// Optional proxy ID for establishing the connection.
    /// - If `None` or not provided, the client will connect directly to the IMAP server.
    /// - If `Some(proxy_id)`, the client will use the pre-configured proxy with the given ID.
    pub use_proxy: Option<u64>,
}

impl ImapConfig {
    pub fn try_encrypt_password(self) -> RustMailerResult<Self> {
        Ok(Self {
            host: self.host,
            port: self.port,
            encryption: self.encryption,
            auth: self.auth.encrypt()?,
            use_proxy: self.use_proxy,
        })
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, Object)]
pub struct SmtpConfig {
    /// SMTP server hostname or IP address
    #[oai(validator(max_length = 253, pattern = r"^[a-zA-Z0-9\-\.]+$"))]
    pub host: String,
    /// SMTP server port number
    #[oai(validator(minimum(value = "1"), maximum(value = "65535")))]
    pub port: u16,
    /// Connection encryption method
    pub encryption: Encryption,
    /// Authentication configuration
    pub auth: AuthConfig,
    /// Optional proxy ID for establishing the connection.
    /// - If `None` or not provided, the client will connect directly to the SMTP server.
    /// - If `Some(proxy_id)`, the client will use the pre-configured proxy with the given ID.
    pub use_proxy: Option<u64>,
}

impl SmtpConfig {
    pub fn try_encrypt_password(self) -> RustMailerResult<Self> {
        Ok(Self {
            host: self.host,
            port: self.port,
            encryption: self.encryption,
            auth: self.auth.encrypt()?,
            use_proxy: self.use_proxy,
        })
    }
}

#[derive(Enum, Default, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum AuthType {
    /// Standard password authentication (PLAIN/LOGIN)
    #[default]
    Password,
    /// OAuth 2.0 authentication (SASL XOAUTH2)
    OAuth2,
}

#[derive(Object, Default, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AuthConfig {
    ///Authentication method to use
    pub auth_type: AuthType,
    /// Credential secret for Password authentication.
    ///
    /// Users should provide a plaintext password (1 to 256 characters).
    /// The server will encrypt the password using AES-256-GCM and securely store it.
    /// The plaintext password is never stored, so users must remember it for authentication.
    #[oai(validator(max_length = 256, min_length = 1))]
    pub password: Option<String>,
}

impl AuthConfig {
    pub fn encrypt(self) -> RustMailerResult<Self> {
        match self.password {
            Some(password) => Ok(Self {
                auth_type: self.auth_type,
                password: Some(encrypt!(&password)?),
            }),
            None => Ok(self),
        }
    }
}

impl AuthConfig {
    pub fn validate(&self) -> Result<(), &'static str> {
        match self.auth_type {
            AuthType::Password if self.password.is_none() => {
                Err("When auth_type is Passwd, password must not be None.")
            }
            _ => Ok(()),
        }
    }
}

#[derive(Clone, Default, Debug, Eq, PartialEq, Serialize, Deserialize, Enum)]
pub enum Encryption {
    /// SSL/TLS encrypted connection
    #[default]
    Ssl,
    /// StartTLS encryption
    StartTls,
    /// Unencrypted connection
    None,
}

impl From<bool> for Encryption {
    fn from(value: bool) -> Self {
        if value {
            Self::Ssl
        } else {
            Self::None
        }
    }
}

#[derive(Clone, Default, Debug, Eq, PartialEq, Serialize, Deserialize, Enum)]
pub enum MailerType {
    /// Use IMAP/SMTP protocol
    #[default]
    ImapSmtp,
    /// Use Gmail API
    GmailApi,
    /// Use Graph API
    GraphApi,
}
