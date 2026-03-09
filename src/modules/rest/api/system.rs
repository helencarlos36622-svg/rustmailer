// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::cache::disk::DISK_CACHE;
use crate::modules::common::auth::ClientContext;
use crate::modules::error::code::ErrorCode;
use crate::modules::overview::Overview;
use crate::modules::rest::api::ApiTags;
use crate::modules::rest::ApiResult;
use crate::modules::settings::proxy::Proxy;
use crate::modules::version::{fetch_notifications, Notifications};
use crate::raise_error;
use poem_openapi::param::Path;
use poem_openapi::payload::{Json, PlainText};
use poem_openapi::OpenApi;

pub struct SystemApi;

#[OpenApi(prefix_path = "/api/v1", tag = "ApiTags::System")]
impl SystemApi {
    /// Retrieves important system notifications for the RustMail service.
    /// This endpoint returns a consolidated view of all critical system notifications including:
    /// - Available version updates
    /// - License expiration warnings
    #[oai(
        method = "get",
        path = "/notifications",
        operation_id = "get_notifications"
    )]
    async fn get_notifications(&self) -> ApiResult<Json<Notifications>> {
        let notification = fetch_notifications()
            .await
            .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?;
        Ok(Json(notification))
    }

    /// Retrieves an overview of RustMail service metrics.
    /// This endpoint returns a consolidated view of all key metrics including:
    /// - IMAP traffic (sent and received)
    /// - Email sent counts (success and failure)
    /// - Email sent bytes
    /// - New email arrivals
    /// - Mail flag changes
    /// - Email opens
    /// - Email clicks
    /// - Event dispatch counts (success and failure for HTTP and NATS)
    #[oai(method = "get", path = "/overview", operation_id = "get_overview")]
    async fn get_overview(&self) -> ApiResult<Json<Overview>> {
        let metrics = Overview::get()
            .await
            .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?;
        Ok(Json(metrics))
    }

    /// Get the full list of SOCKS5 proxy configurations.
    #[oai(method = "get", path = "/list-proxy", operation_id = "list_proxy")]
    async fn list_proxy(&self) -> ApiResult<Json<Vec<Proxy>>> {
        let proxies = Proxy::list_all()
            .await
            .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?;
        Ok(Json(proxies))
    }

    /// Delete a specific proxy configuration by ID. Requires root permission.
    #[oai(path = "/proxy/:id", method = "delete", operation_id = "remove_proxy")]
    async fn remove_proxy(
        &self,
        /// The name of the OAuth2 configuration to retrieve
        id: Path<u64>,
        context: ClientContext,
    ) -> ApiResult<()> {
        context.require_root()?;
        Ok(Proxy::delete(id.0).await?)
    }

    /// Retrieve a specific proxy configuration by ID
    #[oai(path = "/proxy/:id", method = "get", operation_id = "get_proxy")]
    async fn get_proxy(
        &self,
        /// The name of the OAuth2 configuration to retrieve
        id: Path<u64>,
        context: ClientContext,
    ) -> ApiResult<Json<Proxy>> {
        context.require_root()?;
        Ok(Json(Proxy::get(id.0).await?))
    }

    /// Create a new proxy configuration. Requires root permission.
    #[oai(path = "/proxy", method = "post", operation_id = "create_proxy")]
    async fn create_proxy(&self, url: PlainText<String>, context: ClientContext) -> ApiResult<()> {
        context.require_root()?;
        let entity = Proxy::new(url.0);
        Ok(entity.save().await?)
    }

    /// Update the URL of a specific proxy by ID. Requires root permission.
    #[oai(path = "/proxy/:id", method = "post", operation_id = "update_proxy")]
    async fn update_proxy(
        &self,
        id: Path<u64>,
        url: PlainText<String>,
        context: ClientContext,
    ) -> ApiResult<()> {
        context.require_root()?;
        Ok(Proxy::update(id.0, url.0).await?)
    }

    /// Delete all entries in the disk cache. Requires root permission.
    /// The disk cache stores temporary files such as email bodies, attachments,
    /// and outgoing emails waiting to be sent. This operation will clear all
    /// cached data, freeing disk space but removing all temporary content.
    #[oai(
        path = "/disk-cache",
        method = "delete",
        operation_id = "clear_disk_cache"
    )]
    async fn clear_disk_cache(&self, context: ClientContext) -> ApiResult<()> {
        context.require_root()?;
        Ok(DISK_CACHE.clear().await?)
    }
}
