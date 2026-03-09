// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::smtp::template::entity::MessageFormat;
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
pub struct TemplateCreateRequest {
    /// A brief description of the email template (optional). Maximum length is 1024 characters.
    #[oai(validator(max_length = "1024"))]
    pub description: Option<String>,

    /// The associated account information that created or uses this template (optional). `None` indicates a public template.
    pub account_id: Option<u64>,

    /// The subject line of the email. Maximum length is 20,480 characters.
    #[oai(validator(max_length = "20480"))]
    pub subject: String,

    /// Preview text for the email (optional), displayed in email clients. Maximum length is 256 characters.
    #[oai(validator(max_length = "256"))]
    pub preview: Option<String>,

    /// The plain text content of the email template (optional). Maximum length is 8,388,608 characters (approximately 8MB).
    #[oai(validator(max_length = "8388608"))]
    pub text: Option<String>,

    /// The HTML content of the email template (optional). Maximum length is 8,388,608 characters (approximately 8MB).
    #[oai(validator(max_length = "8388608"))]
    pub html: Option<String>,

    /// Format of the HTML email content, either Markdown or HTML. Defaults to HTML if not specified.
    pub format: Option<MessageFormat>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
pub struct TemplateUpdateRequest {
    /// A brief description of the email template (optional). Maximum length is 1024 characters.
    #[oai(validator(max_length = "1024"))]
    pub description: Option<String>,

    /// The subject line of the email (optional). Maximum length is 20,480 characters.
    #[oai(validator(max_length = "20480"))]
    pub subject: Option<String>,

    /// Preview text for the email (optional), displayed in email clients. Maximum length is 256 characters.
    #[oai(validator(max_length = "256"))]
    pub preview: Option<String>,

    /// The plain text content of the email template (optional). Maximum length is 8,388,608 characters (approximately 8MB).
    #[oai(validator(max_length = "8388608"))]
    pub text: Option<String>,

    /// The HTML content of the email template (optional). Maximum length is 8,388,608 characters (approximately 8MB).
    #[oai(validator(max_length = "8388608"))]
    pub html: Option<String>,

    /// Format of the HTML email content, either Markdown or HTML. Defaults to HTML if not specified.
    pub format: Option<MessageFormat>,
}

/// Request structure for sending a test template email
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, Object)]
pub struct TemplateSentTestRequest {
    /// Account ID associated with the template and sending request
    pub account_id: u64,
    /// Email address of the recipient who will receive the test email
    ///
    /// Must be a valid email address format (e.g., user@example.com)
    #[oai(validator(custom = "crate::modules::common::validator::EmailValidator"))]
    pub recipient: String,
    /// Optional parameters to be used for template variable substitution
    ///
    /// When provided, should be a valid JSON object that matches the template's expected variables.
    /// Example: {"name": "John Doe", "order_id": 12345}
    pub template_params: Option<serde_json::Value>,
}
