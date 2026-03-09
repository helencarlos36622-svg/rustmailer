// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use poem::{Endpoint, Middleware, Request, Result};
use std::time::Duration;
use tracing::error;

use crate::modules::error::code::ErrorCode;

use super::create_api_error_response;

pub const TIMEOUT_HEADER: &str = "X-RustMailer-Timeout-Seconds";

pub struct Timeout;

impl<E: Endpoint> Middleware<E> for Timeout {
    type Output = TimeoutEndpoint<E>;

    fn transform(&self, ep: E) -> Self::Output {
        TimeoutEndpoint { ep }
    }
}

pub struct TimeoutEndpoint<E> {
    ep: E,
}

#[inline]
fn extract_timeout(req: &Request) -> Option<u64> {
    if let Some(v) = req.header(TIMEOUT_HEADER) {
        v.parse::<u64>().ok()
    } else {
        None
    }
}

impl<E: Endpoint> Endpoint for TimeoutEndpoint<E> {
    type Output = E::Output;

    async fn call(&self, req: Request) -> Result<Self::Output> {
        let timeout = extract_timeout(&req);
        let seconds = timeout.unwrap_or(30).min(600);
        match tokio::time::timeout(Duration::from_secs(seconds), self.ep.call(req)).await {
            Ok(Ok(response)) => Ok(response), // If the request completes successfully
            Ok(Err(e)) => Err(e),             // If the request returns an error
            Err(_) => {
                error!("Request timed out after {} seconds", seconds);
                Err(create_api_error_response(
                    &format!(
                        "Request timed out after {} seconds (timeout set via X-RustMailer-Timeout-Seconds header, max allowed: 600 seconds)",
                        seconds
                    ),
                    ErrorCode::RequestTimeout,
                ))
            }
        }
    }
}
