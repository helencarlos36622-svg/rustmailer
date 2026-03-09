// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use poem::http::StatusCode;
use poem_openapi::Enum;

#[derive(Copy, Clone, Debug, Enum, Eq, PartialEq)]
#[repr(u32)]
pub enum ErrorCode {
    // Client-side errors (10000–10999)
    InvalidParameter = 10000,
    VRLScriptSyntaxError = 10010,
    MissingConfiguration = 10020,
    Incompatible = 10030,
    ExceedsLimitation = 10040,
    EmlFileParseError = 10050,
    MissingContentLength = 10060,
    PayloadTooLarge = 10070,
    RequestTimeout = 10080,
    MethodNotAllowed = 10090,

    // Authentication and authorization errors (20000–20999)
    PermissionDenied = 20000,
    AccountDisabled = 20010,
    LicenseAccountLimitReached = 20020,
    LicenseExpired = 20030,
    InvalidLicense = 20040,
    OAuth2ItemDisabled = 20050,
    MissingRefreshToken = 20060,

    // Resource errors (30000–30999)
    ResourceNotFound = 30000,
    AlreadyExists = 30010,
    TooManyRequest = 30020,

    // Network connection errors (40000–40999)
    NetworkError = 40000,
    ConnectionTimeout = 40010,
    ConnectionPoolTimeout = 40020,
    HttpResponseError = 40030,

    // Mail service errors (50000–50999)
    ImapCommandFailed = 50000,
    ImapAuthenticationFailed = 50010,
    ImapUnexpectedResult = 50020,
    SmtpCommandFailed = 50030,
    SmtpConnectionFailed = 50040,
    MailBoxNotCached = 50050,
    AutoconfigFetchFailed = 50060,
    ApiCallFailed = 50070,

    // Message queue errors (60000–60999)
    NatsRequestFailed = 60000,
    NatsConnectionFailed = 60010,
    NatsCreateStreamFailed = 60020,

    // Internal system errors (70000–70999)
    InternalError = 70000,
    UnhandledPoemError = 70010,
}

impl ErrorCode {
    pub fn status(&self) -> StatusCode {
        match self {
            ErrorCode::InvalidParameter
            | ErrorCode::VRLScriptSyntaxError
            | ErrorCode::MissingConfiguration
            | ErrorCode::Incompatible
            | ErrorCode::ExceedsLimitation
            | ErrorCode::EmlFileParseError => StatusCode::BAD_REQUEST,
            ErrorCode::PermissionDenied => StatusCode::UNAUTHORIZED,
            ErrorCode::AccountDisabled
            | ErrorCode::LicenseAccountLimitReached
            | ErrorCode::LicenseExpired
            | ErrorCode::InvalidLicense
            | ErrorCode::OAuth2ItemDisabled => StatusCode::FORBIDDEN,
            ErrorCode::ResourceNotFound => StatusCode::NOT_FOUND,
            ErrorCode::RequestTimeout => StatusCode::REQUEST_TIMEOUT,
            ErrorCode::AlreadyExists => StatusCode::CONFLICT,
            ErrorCode::MissingContentLength => StatusCode::LENGTH_REQUIRED,
            ErrorCode::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            ErrorCode::TooManyRequest => StatusCode::TOO_MANY_REQUESTS,
            ErrorCode::InternalError
            | ErrorCode::AutoconfigFetchFailed
            | ErrorCode::ImapCommandFailed
            | ErrorCode::ApiCallFailed
            | ErrorCode::ImapUnexpectedResult
            | ErrorCode::HttpResponseError
            | ErrorCode::NatsRequestFailed
            | ErrorCode::NatsCreateStreamFailed
            | ErrorCode::MailBoxNotCached
            | ErrorCode::ImapAuthenticationFailed
            | ErrorCode::MissingRefreshToken
            | ErrorCode::SmtpCommandFailed
            | ErrorCode::NetworkError
            | ErrorCode::ConnectionTimeout
            | ErrorCode::ConnectionPoolTimeout
            | ErrorCode::NatsConnectionFailed
            | ErrorCode::UnhandledPoemError
            | ErrorCode::SmtpConnectionFailed => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
        }
    }
}
