// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::autoconfig::entity::MailServerConfig;
use crate::modules::autoconfig::load::resolve_autoconfig;
use crate::modules::error::code::ErrorCode;
use crate::modules::rest::api::ApiTags;
use crate::modules::rest::ApiResult;
use crate::raise_error;
use poem::web::Path;
use poem_openapi::payload::Json;
use poem_openapi::OpenApi;

pub struct AutoConfigApi;

#[OpenApi(prefix_path = "/api/v1", tag = "ApiTags::AutoConfig")]
impl AutoConfigApi {
    /// Retrieve mail server configuration for a given email address
    /// 
    #[oai(
        path = "/autoconfig/:email_address",
        method = "get",
        operation_id = "autoconfig"
    )]
    async fn autoconfig(
        &self,
        /// The email address to lookup configuration for
        email_address: Path<String>
    ) -> ApiResult<Json<MailServerConfig>> {
        let result = resolve_autoconfig(email_address.0.trim())
            .await?
            .ok_or_else(|| {
                raise_error!(
                    "Unable to find account configuration information in the backend.".into(),
                    ErrorCode::ResourceNotFound
                )
            })?;
        Ok(Json(result))
    }
}
