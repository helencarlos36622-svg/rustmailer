// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use autoconfig::config::OAuth2Config as XOAuth2Config;
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use crate::modules::account::entity::Encryption;

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize, Object)]
pub struct ServerConfig {
    /// server hostname or IP address
    pub host: String,
    /// server port number
    pub port: u16,
    /// Connection encryption method
    pub encryption: Encryption,
}

impl ServerConfig {
    pub fn new(host: String, port: u16, encryption: Encryption) -> Self {
        Self {
            host,
            port,
            encryption,
        }
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize, Object)]
pub struct OAuth2Config {
    /// The authorization server's issuer identifier URL
    pub issuer: String,
    /// List of scopes requested by the client
    pub scope: Vec<String>,
    /// URL of the authorization server's authorization endpoint
    pub auth_url: String,
    /// URL of the authorization server's token endpoint
    pub token_url: String,
}

impl From<&XOAuth2Config> for OAuth2Config {
    fn from(value: &XOAuth2Config) -> Self {
        Self {
            issuer: value.issuer().into(),
            scope: value.scope().into_iter().map(Into::into).collect(),
            auth_url: value.auth_url().into(),
            token_url: value.token_url().into(),
        }
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize, Object)]
pub struct MailServerConfig {
    /// IMAP server configuration
    pub imap: ServerConfig,
    /// SMTP server configuration
    pub smtp: ServerConfig,
    /// OAuth 2.0 client configuration parameters
    pub oauth2: Option<OAuth2Config>,
}
