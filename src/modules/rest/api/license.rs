// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::common::auth::ClientContext;
use crate::modules::error::code::ErrorCode;
use crate::modules::license::License;
use crate::modules::rest::api::ApiTags;
use crate::modules::rest::ApiResult;
use crate::raise_error;
use poem_openapi::payload::Json;
use poem_openapi::{payload::PlainText, OpenApi};
pub struct LicenseApi;

#[OpenApi(prefix_path = "/api/v1", tag = "ApiTags::License")]
impl LicenseApi {
    /// Retrieve current license information
    /// Requires root privileges.
    #[oai(path = "/license", method = "get", operation_id = "get_license")]
    async fn get_license(&self, context: ClientContext) -> ApiResult<Json<License>> {
        context.require_root()?;
        let result = License::get_current_license().await?.ok_or_else(|| {
            raise_error!(
                "License not found. This should not happen.".into(),
                ErrorCode::ResourceNotFound
            )
        })?;
        Ok(Json(result))
    }

    /// Upload and activate a new license
    /// Requires root privileges.
    #[oai(path = "/license", method = "post", operation_id = "set_license")]
    async fn set_license(
        &self,
        context: ClientContext,
        ///Raw license key string (text/plain content-type)
        license_str: PlainText<String>,
    ) -> ApiResult<Json<License>> {
        context.require_root()?;
        let license = License::check_license(&license_str).await?;
        license.clone().save().await?;
        Ok(Json(license))
    }
}
