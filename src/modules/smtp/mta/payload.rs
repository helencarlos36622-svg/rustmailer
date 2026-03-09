// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::{
    modules::{
        error::{code::ErrorCode, RustMailerResult},
        smtp::mta::entity::{MTACredentials, SmtpServerConfig},
    },
    raise_error, validate_email,
};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
pub struct MTACreateRequest {
    /// Optional descriptive text about the MTA.
    pub description: Option<String>,
    /// Credentials used for authenticating with the MTA server.
    pub credentials: MTACredentials,
    /// SMTP server configuration details.
    pub server: SmtpServerConfig,
    /// Whether the MTA supports DSN (Delivery Status Notification).
    pub dsn_capable: bool,
    /// Optional proxy ID for establishing the connection.
    /// - If `None` or not provided, the client will connect directly to the MTA server.
    /// - If `Some(proxy_id)`, the client will use the pre-configured proxy with the given ID.
    pub use_proxy: Option<u64>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
pub struct MTAUpdateRequest {
    /// Optional descriptive text about the MTA.
    pub description: Option<String>,
    /// Optional updated credentials.
    pub credentials: Option<MTACredentials>,
    /// Optional updated SMTP server configuration.
    pub server: Option<SmtpServerConfig>,
    /// Optional updated DSN support flag.
    pub dsn_capable: Option<bool>,
    /// Optional proxy ID for establishing the connection.
    /// - If `None` or not provided, the client will connect directly to the MTA server.
    /// - If `Some(proxy_id)`, the client will use the pre-configured proxy with the given ID.
    pub use_proxy: Option<u64>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
pub struct SendTestEmailRequest {
    /// The email address of the sender (e.g., "no-reply@yourdomain.com").
    pub from: String,
    /// The email address of a single recipient (e.g., "user@example.com").
    pub to: String,
    /// The subject line of the email.
    #[oai(validator(max_length = 256, min_length = 1))]
    pub subject: String,
    /// The plain text body content of the email, sent as the text/plain part of the email.
    #[oai(validator(max_length = 1024, min_length = 1))]
    pub message: String,
}

impl SendTestEmailRequest {
    pub fn validate(&self) -> RustMailerResult<()> {
        let mut errors: Vec<String> = Vec::new();

        // Validate 'from' email address
        if validate_email!(&self.from).is_err() {
            errors.push("Invalid 'from' email address".into());
        }

        // Validate 'to' email address
        if validate_email!(&self.to).is_err() {
            errors.push("Invalid 'to' email address".into());
        }

        // Validate 'subject' is not empty or whitespace-only
        if self.subject.trim().is_empty() {
            errors.push("Subject cannot be empty".into());
        }

        // Validate 'message' is not empty or whitespace-only
        if self.message.trim().is_empty() {
            errors.push("Message cannot be empty".into());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(raise_error!(
                format!("Validation errors: {:#?}", errors),
                ErrorCode::InvalidParameter
            ))
        }
    }
}
