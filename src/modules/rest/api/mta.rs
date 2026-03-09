// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::common::auth::ClientContext;
use crate::modules::error::code::ErrorCode;
use crate::modules::rest::api::ApiTags;
use crate::modules::rest::response::DataPage;
use crate::modules::rest::ApiResult;
use crate::modules::smtp::mta::entity::Mta;
use crate::modules::smtp::mta::payload::{
    MTACreateRequest, MTAUpdateRequest, SendTestEmailRequest,
};
use crate::modules::smtp::mta::send::send_test_email;
use crate::raise_error;
use poem::web::Path;
use poem_openapi::param::Query;
use poem_openapi::payload::Json;
use poem_openapi::OpenApi;
pub struct MTAApi;

#[OpenApi(prefix_path = "/api/v1", tag = "ApiTags::Mta")]
impl MTAApi {
    /// Retrieves the MTA (Mail Transfer Agent) configuration by its unique name.
    #[oai(path = "/mta/:id", method = "get", operation_id = "get_mta")]
    async fn get_mta(
        &self,
        /// The unique name identifier of the MTA.
        id: Path<u64>,
    ) -> ApiResult<Json<Mta>> {
        let id = id.0;
        let mta = Mta::get(id).await?.ok_or_else(|| {
            raise_error!(
                format!("MTA with id {id} not found"),
                ErrorCode::ResourceNotFound
            )
        })?;
        Ok(Json(mta))
    }

    /// Deletes an existing MTA configuration identified by its name.
    /// Requires root privileges.
    #[oai(path = "/mta/:id", method = "delete", operation_id = "remove_mta")]
    async fn remove_mta(
        &self,
        /// The unique name identifier of the MTA to delete.
        id: Path<u64>,
        context: ClientContext,
    ) -> ApiResult<()> {
        context.require_root()?;
        Ok(Mta::delete(id.0).await?)
    }

    /// Creates a new MTA configuration.
    /// Requires root privileges.
    #[oai(path = "/mta", method = "post", operation_id = "create_mta")]
    async fn create_mta(
        &self,
        /// The MTA creation request payload.
        request: Json<MTACreateRequest>,
        context: ClientContext,
    ) -> ApiResult<()> {
        context.require_root()?;
        let entity = Mta::new(request.0)?;
        Ok(entity.save().await?)
    }

    /// Updates an existing MTA configuration by its name.
    /// Requires root privileges.
    #[oai(path = "/mta/:id", method = "post", operation_id = "update_mta")]
    async fn update_mta(
        &self,
        /// The unique name identifier of the MTA to update.
        id: Path<u64>,
        /// The MTA update request payload.
        request: Json<MTAUpdateRequest>,
        context: ClientContext,
    ) -> ApiResult<()> {
        context.require_root()?;
        Ok(Mta::update(id.0, request.0).await?)
    }

    /// Retrieves a list of all MTA
    #[oai(path = "/list-mta", method = "get", operation_id = "list_mta")]
    async fn list_mta(
        &self,
        /// Optional. The page number to retrieve (starting from 1).
        page: Query<Option<u64>>,
        /// Optional. The number of items per page.
        page_size: Query<Option<u64>>,
        /// Optional. Whether to sort the list in descending order.
        desc: Query<Option<bool>>,
    ) -> ApiResult<Json<DataPage<Mta>>> {
        Ok(Json(Mta::paginate_list(page.0, page_size.0, desc.0).await?))
    }

    /// Sends a test email using the specified Mail Transfer Agent (MTA).
    #[oai(
        path = "/mta-send-test/:id",
        method = "post",
        operation_id = "send_test_email"
    )]
    async fn send_test_email(
        &self,
        /// The unique name identifier of the MTA to test.
        id: Path<u64>,
        /// request payload.
        request: Json<SendTestEmailRequest>,
    ) -> ApiResult<()> {
        send_test_email(id.0, request.0).await?;
        Ok(())
    }
}
