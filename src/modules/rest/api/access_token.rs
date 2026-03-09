// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::common::auth::ClientContext;
use crate::modules::rest::api::ApiTags;
use crate::modules::rest::ApiResult;
use crate::modules::token::payload::AccessTokenUpdateRequest;
use crate::modules::token::root::set_root_password;
use crate::modules::{
    token::payload::AccessTokenCreateRequest,
    token::{root::reset_root_token, AccessToken},
};
use poem_openapi::payload::PlainText;
use poem_openapi::{param::Path, payload::Json, OpenApi};

pub struct AccessTokenApi;

#[OpenApi(prefix_path = "/api/v1", tag = "ApiTags::AccessToken")]
impl AccessTokenApi {
    /// Lists all access tokens in the system.
    /// Requires root privileges.
    #[oai(
        path = "/access-token-list",
        method = "get",
        operation_id = "list_access_tokens"
    )]
    async fn list_access_tokens(
        &self,
        context: ClientContext,
    ) -> ApiResult<Json<Vec<AccessToken>>> {
        context.require_root()?;
        Ok(Json(AccessToken::list_all().await?))
    }

    /// Lists access tokens for a specific account.
    /// Requires root privileges.
    #[oai(
        path = "/access-token-list/:account_id",
        method = "get",
        operation_id = "list_account_access_tokens"
    )]
    async fn list_account_access_tokens(
        &self,
        /// The ID of the account whose tokens are to be retrieved.
        account_id: Path<u64>,
        context: ClientContext,
    ) -> ApiResult<Json<Vec<AccessToken>>> {
        context.require_root()?;
        Ok(Json(AccessToken::list_account_tokens(account_id.0).await?))
    }
    /// Deletes a specific access token.
    /// Requires root privileges.
    #[oai(
        path = "/access-token/:token",
        method = "delete",
        operation_id = "remove_access_token"
    )]
    async fn remove_access_token(
        &self,
        /// The access token to be deleted
        token: Path<String>,
        context: ClientContext,
    ) -> ApiResult<()> {
        context.require_root()?;
        Ok(AccessToken::delete(token.0.trim()).await?)
    }

    /// Creates a new access token.
    /// Requires root privileges.
    #[oai(
        path = "/access-token",
        method = "post",
        operation_id = "create_access_token"
    )]
    async fn create_access_token(
        &self,
        context: ClientContext,
        /// The request payload
        payload: Json<AccessTokenCreateRequest>,
    ) -> ApiResult<PlainText<String>> {
        context.require_root()?;
        Ok(PlainText(AccessToken::create(payload.0).await?))
    }

    /// Updates an existing access token.
    /// Requires root privileges.
    #[oai(
        path = "/access-token/:token",
        method = "post",
        operation_id = "update_access_token"
    )]
    async fn update_access_token(
        &self,
        context: ClientContext,
        /// The access token to be updated.
        token: Path<String>,
        /// The request payload.
        payload: Json<AccessTokenUpdateRequest>,
    ) -> ApiResult<()> {
        context.require_root()?;
        Ok(AccessToken::update(token.0.trim(), payload.0).await?)
    }

    /// Regenerates the root access token.
    /// Requires root privileges.
    #[oai(
        path = "/reset-root-token",
        method = "post",
        operation_id = "regenerate_root_token"
    )]
    async fn regenerate_root_token(&self, context: ClientContext) -> ApiResult<PlainText<String>> {
        context.require_root()?;
        Ok(PlainText(reset_root_token().await?))
    }

    /// Reset the Root user's password.
    /// Only callable by an already authenticated Root user.
    /// This endpoint updates the Root password to `password_str`
    /// and regenerates the `root_token`, invalidating any previous token.
    #[oai(
        path = "/reset-root-password",
        method = "post",
        operation_id = "reset_root_password"
    )]
    async fn reset_root_password(
        &self,
        password_str: PlainText<String>,
        context: ClientContext,
    ) -> ApiResult<()> {
        context.require_root()?;
        Ok(set_root_password(password_str.0.trim()).await?)
    }

    // /// Login endpoint for the Root user.
    // /// Accepts the Root password and returns the `root_token`
    // /// which should be used in subsequent requests for authentication.
    // #[oai(path = "/login", method = "post", operation_id = "login")]
    // async fn login(&self, password_str: PlainText<String>) -> ApiResult<PlainText<String>> {
    //     Ok(PlainText(check_root_password(password_str.0.trim())?))
    // }
}
