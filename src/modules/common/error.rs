// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use poem::{Endpoint, IntoResponse, Middleware, Request, Response, Result};

use crate::modules::error::handler::error_handler;

pub struct ErrorCapture;

pub struct ErrorCaptureEndpoint<E> {
    ep: E,
}

impl<E: Endpoint> Middleware<E> for ErrorCapture {
    type Output = ErrorCaptureEndpoint<E>;

    fn transform(&self, ep: E) -> Self::Output {
        ErrorCaptureEndpoint { ep }
    }
}

impl<E: Endpoint> Endpoint for ErrorCaptureEndpoint<E> {
    type Output = Response;

    async fn call(&self, req: Request) -> Result<Self::Output> {
        match self.ep.call(req).await {
            Ok(response) => Ok(response.into_response()),
            Err(error) => Ok(error_handler(error).await.into_response()),
        }
    }
}
