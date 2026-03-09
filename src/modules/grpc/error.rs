// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::error::{code::ErrorCode, RustMailerError};
use poem_grpc::{Code, Metadata, Status};
use tracing::error;

impl From<RustMailerError> for Status {
    fn from(error: RustMailerError) -> Self {
        let (code, message, location) = match &error {
            RustMailerError::Generic {
                message,
                code,
                location,
            } => (*code, message.clone(), location),
        };

        error!(
            "gRPC Error occurred: {} (code: {:?}) at {}:{}",
            message, code, location.file, location.line
        );
        let grpc_code = match code {
            ErrorCode::InvalidParameter
            | ErrorCode::VRLScriptSyntaxError
            | ErrorCode::MissingConfiguration
            | ErrorCode::Incompatible
            | ErrorCode::ExceedsLimitation
            | ErrorCode::EmlFileParseError
            | ErrorCode::MissingContentLength => Code::InvalidArgument,
            ErrorCode::PermissionDenied => Code::PermissionDenied,
            ErrorCode::AccountDisabled
            | ErrorCode::LicenseAccountLimitReached
            | ErrorCode::LicenseExpired
            | ErrorCode::InvalidLicense
            | ErrorCode::OAuth2ItemDisabled => Code::PermissionDenied,
            ErrorCode::ResourceNotFound => Code::NotFound,
            ErrorCode::RequestTimeout => Code::DeadlineExceeded,
            ErrorCode::AlreadyExists => Code::AlreadyExists,
            ErrorCode::PayloadTooLarge => Code::ResourceExhausted,
            ErrorCode::TooManyRequest => Code::ResourceExhausted,
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
            | ErrorCode::SmtpConnectionFailed => Code::Internal,
            ErrorCode::MethodNotAllowed => Code::Unimplemented,
        };

        let mut metadata = Metadata::new();
        metadata.insert("rustmailer-error-code", (code as u32).to_string());
        Status::new(grpc_code)
            .with_message(message)
            .with_metadata(metadata)
    }
}
