// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::{
    autoconfig::entity::{MailServerConfig, OAuth2Config, ServerConfig},
    grpc::service::rustmailer_grpc,
};

impl From<ServerConfig> for rustmailer_grpc::ServerConfig {
    fn from(value: ServerConfig) -> Self {
        Self {
            host: value.host,
            port: value.port as u32,
            encryption: value.encryption.into(),
        }
    }
}

impl From<OAuth2Config> for rustmailer_grpc::OAuth2Config {
    fn from(value: OAuth2Config) -> Self {
        Self {
            issuer: value.issuer,
            scope: value.scope,
            auth_url: value.auth_url,
            token_url: value.token_url,
        }
    }
}

impl From<MailServerConfig> for rustmailer_grpc::MailServerConfig {
    fn from(value: MailServerConfig) -> Self {
        Self {
            imap: Some(value.imap.into()),
            smtp: Some(value.smtp.into()),
            oauth2: value.oauth2.map(Into::into),
        }
    }
}
