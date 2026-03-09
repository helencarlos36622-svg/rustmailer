// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::{
    grpc::service::rustmailer_grpc::{self, PagedOAuth2},
    oauth2::{
        entity::{OAuth2, OAuth2CreateRequest, OAuth2UpdateRequest},
        token::{ExternalOAuth2Request, OAuth2AccessToken},
    },
    rest::response::DataPage,
};

impl From<rustmailer_grpc::OAuth2CreateRequest> for OAuth2CreateRequest {
    fn from(value: rustmailer_grpc::OAuth2CreateRequest) -> Self {
        Self {
            description: value.description,
            client_id: value.client_id,
            client_secret: value.client_secret,
            auth_url: value.auth_url,
            token_url: value.token_url,
            redirect_uri: value.redirect_uri,
            scopes: (!value.scopes.is_empty()).then_some(value.scopes),
            extra_params: (!value.extra_params.is_empty())
                .then(|| value.extra_params.into_iter().collect()),
            enabled: value.enabled,
            use_proxy: value.use_proxy,
        }
    }
}

// 2. From RustMailerOAuth2 to OAuth2 (gRPC)
impl From<rustmailer_grpc::UpdateOAuth2Request> for OAuth2UpdateRequest {
    fn from(value: rustmailer_grpc::UpdateOAuth2Request) -> Self {
        Self {
            description: value.description,
            client_id: value.client_id,
            client_secret: value.client_secret,
            auth_url: value.auth_url,
            token_url: value.token_url,
            redirect_uri: value.redirect_uri,
            scopes: (!value.scopes.is_empty()).then_some(value.scopes),
            extra_params: (!value.extra_params.is_empty())
                .then(|| value.extra_params.into_iter().collect()),
            enabled: value.enabled,
            use_proxy: value.use_proxy,
        }
    }
}

// 3. From UpdateOAuth2Request (gRPC) to OAuth2UpdateRequestDomain
impl From<OAuth2> for rustmailer_grpc::OAuth2 {
    fn from(value: OAuth2) -> Self {
        Self {
            id: value.id,
            description: value.description,
            client_id: value.client_id,
            client_secret: value.client_secret,
            auth_url: value.auth_url,
            token_url: value.token_url,
            redirect_uri: value.redirect_uri,
            scopes: value.scopes.unwrap_or_default(),
            extra_params: value
                .extra_params
                .map(|p| p.into_iter().collect())
                .unwrap_or_default(),
            enabled: value.enabled,
            use_proxy: value.use_proxy,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<DataPage<OAuth2>> for PagedOAuth2 {
    fn from(value: DataPage<OAuth2>) -> Self {
        Self {
            current_page: value.current_page,
            page_size: value.page_size,
            total_items: value.total_items,
            items: value.items.into_iter().map(Into::into).collect(),
            total_pages: value.total_pages,
        }
    }
}

impl From<OAuth2AccessToken> for rustmailer_grpc::OAuth2AccessToken {
    fn from(value: OAuth2AccessToken) -> Self {
        Self {
            account_id: value.account_id,
            oauth2_id: value.oauth2_id,
            access_token: value.access_token,
            refresh_token: value.refresh_token,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
impl From<rustmailer_grpc::ExternalOAuth2Request> for ExternalOAuth2Request {
    fn from(value: rustmailer_grpc::ExternalOAuth2Request) -> Self {
        Self {
            oauth2_id: value.oauth2_id,
            access_token: value.access_token,
            refresh_token: value.refresh_token,
        }
    }
}
