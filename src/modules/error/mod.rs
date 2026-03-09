// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use std::{fmt::Formatter, u32};

use crate::raise_error;
use bb8::RunError;
use code::ErrorCode;
use poem::http::StatusCode;
use poem_openapi::{payload::Json, ApiResponse, Object};
use snafu::{Location, Snafu};

pub mod code;
pub mod handler;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum RustMailerError {
    #[snafu(display("{message}"))]
    Generic {
        message: String,
        #[snafu(implicit)]
        location: Location,
        code: ErrorCode,
    },
}

pub type RustMailerResult<T, E = RustMailerError> = std::result::Result<T, E>;

impl From<RunError<RustMailerError>> for RustMailerError {
    fn from(e: RunError<RustMailerError>) -> Self {
        match e {
            RunError::User(e) => e,
            RunError::TimedOut => raise_error!(
                "Timed out while attempting to acquire a connection from the pool".into(),
                ErrorCode::ConnectionPoolTimeout
            ),
        }
    }
}
#[derive(Debug, Clone, Object)]
pub struct ApiError {
    pub message: String,
    pub code: u32,
}

impl From<RustMailerError> for ApiErrorResponse {
    fn from(error: RustMailerError) -> Self {
        match error {
            RustMailerError::Generic {
                message,
                location,
                code,
            } => {
                tracing::error!(
                    "API error occurred: [{:#?}] {} at {:?}",
                    code,
                    message,
                    location
                );
                let api_error = ApiError {
                    message,
                    code: code as u32,
                };
                ApiErrorResponse::Generic(code.status(), Json(api_error))
            }
        }
    }
}

impl ApiError {
    pub fn new(message: String, code: u32) -> Self {
        Self { message, code }
    }

    pub fn new_with_error_code<ErrorType: std::fmt::Display>(
        error: ErrorType,
        code: u32,
    ) -> ApiError {
        Self::new(format!("{:#}", error), code)
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error({}): {}", self.code, self.message)
    }
}

impl std::error::Error for ApiError {}

#[derive(Debug, Clone, ApiResponse)]
pub enum ApiErrorResponse {
    Generic(StatusCode, Json<ApiError>),
}
