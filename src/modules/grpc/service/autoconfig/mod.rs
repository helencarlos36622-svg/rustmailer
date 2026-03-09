// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use std::str::FromStr;

use crate::modules::error::code::ErrorCode;
use crate::modules::grpc::service::rustmailer_grpc::{AutoconfigRequest, MailServerConfig};
use crate::modules::{
    autoconfig::load::resolve_autoconfig, grpc::service::rustmailer_grpc::AutoConfigService,
};
use crate::raise_error;
use email_address::EmailAddress;
use poem_grpc::{Request, Response, Status};

pub mod from;

#[derive(Default)]
pub struct RustMailerAutoConfigService;

impl AutoConfigService for RustMailerAutoConfigService {
    async fn get_autoconfig(
        &self,
        request: Request<AutoconfigRequest>,
    ) -> Result<Response<MailServerConfig>, Status> {
        let email = request.into_inner().email_address;

        let _ = EmailAddress::from_str(&email).map_err(|_| {
            raise_error!(
                "Invalid email address format".into(),
                ErrorCode::InvalidParameter
            )
        })?;

        let result = resolve_autoconfig(&email)
            .await?
            .map(|c| c.into())
            .ok_or_else(|| {
                raise_error!(
                    "We couldn't find automatic configuration settings for your email provider."
                        .into(),
                    ErrorCode::ResourceNotFound
                )
            })?;
        Ok(Response::new(result))
    }
}
