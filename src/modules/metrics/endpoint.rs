// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use poem::{
    http::{Method, StatusCode},
    Endpoint, Request, Response, Result,
};
use prometheus::{default_registry, Encoder, TextEncoder};

use crate::modules::{common::auth::authorize_access, token::AccessTokenScope};

pub struct PrometheusEndpoint;

impl Endpoint for PrometheusEndpoint {
    type Output = Response;

    async fn call(&self, req: Request) -> Result<Self::Output> {
        if req.method() != Method::GET {
            return Ok(StatusCode::METHOD_NOT_ALLOWED.into());
        }
        authorize_access(&req, Some(AccessTokenScope::Metrics)).await?;
        let encoder = TextEncoder::new();
        let metric_families = default_registry().gather();
        let mut result = Vec::new();
        match encoder.encode(&metric_families, &mut result) {
            Ok(()) => Ok(Response::builder()
                .content_type(encoder.format_type())
                .body(result)),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR.into()),
        }
    }
}
