// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::{
    modules::error::{code::ErrorCode, RustMailerResult},
    raise_error,
};
use async_nats::jetstream::{self};
use poem_openapi::{Enum, Object};
use regex::Regex;
use serde::{Deserialize, Serialize};

pub mod executor;
pub mod pool;

#[derive(Debug)]
pub struct NatsConnectionManager {
    config: NatsConfig,
}

impl NatsConnectionManager {
    pub fn new(config: &NatsConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub async fn build(&self) -> RustMailerResult<async_nats::jetstream::Context> {
        self.config.create_producer().await
    }
}

#[derive(Enum, Default, Hash, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum NatsAuthType {
    #[default]
    None,
    Password,
    Token,
}

#[derive(Clone, Debug, Default, Hash, Eq, PartialEq, Serialize, Deserialize, Object)]
pub struct NatsConfig {
    /// The hostname or IP address of the NATS server.
    #[oai(validator(max_length = 253, pattern = r"^[a-zA-Z0-9\-\.]+$"))]
    pub host: String,
    /// The port number on which the NATS server is listening.
    #[oai(validator(minimum(value = "1"), maximum(value = "65535")))]
    pub port: u16,
    /// The authentication type used to connect to the NATS server.
    pub auth_type: NatsAuthType,
    /// Optional token for token-based authentication with the NATS server.
    pub token: Option<String>,
    /// Optional username for user-based authentication with the NATS server.
    pub username: Option<String>,
    /// Optional password for user-based authentication with the NATS server.
    pub password: Option<String>,
    /// The name of the NATS stream to which messages are published.
    pub stream_name: String,
    /// The namespace or subject prefix used for organizing messages in the NATS server.
    pub namespace: String,
}

impl NatsConfig {
    pub fn validate(&self) -> RustMailerResult<()> {
        let pattern = r"^[a-zA-Z][a-zA-Z0-9_]*$";
        let re = Regex::new(pattern).unwrap();
        if !re.is_match(&self.namespace) {
            return Err(raise_error!("Invalid namespace: namespace can only contain letters, numbers, and underscores, and must start with a letter.".into(), ErrorCode::InvalidParameter));
        }

        match self.auth_type {
            NatsAuthType::None => {}
            NatsAuthType::Password => {
                if self.username.is_none() || self.password.is_none() {
                    return Err(raise_error!(
                        "username and Password is required when auth type is 'Password'".into(),
                        ErrorCode::InvalidParameter
                    ));
                }
            }
            NatsAuthType::Token => {
                if self.token.is_none() {
                    return Err(raise_error!(
                        "token is required when auth type is 'Token'".into(),
                        ErrorCode::InvalidParameter
                    ));
                }
            }
        }

        Ok(())
    }

    pub async fn create_producer(&self) -> RustMailerResult<async_nats::jetstream::Context> {
        let nats_url = format!("nats://{}:{}", &self.host, &self.port);

        let client = match self.auth_type {
            NatsAuthType::None => async_nats::connect(&nats_url).await.map_err(|error| {
                raise_error!(
                    format!(
                        "Failed to connect to NATS server at {} without authentication. Error: {}",
                        nats_url, error
                    ),
                    ErrorCode::InvalidParameter
                )
            })?,
            NatsAuthType::Password => {
                let username = self.username.clone().ok_or_else(|| {
                    raise_error!(
                        "Username is required for password authentication but was not provided"
                            .into(),
                        ErrorCode::InvalidParameter
                    )
                })?;
                let password = self.password.clone().ok_or_else(|| {
                    raise_error!(
                        "Password is required for password authentication but was not provided"
                            .into(),
                        ErrorCode::InvalidParameter
                    )
                })?;

                async_nats::connect_with_options(
                    &nats_url,
                    async_nats::ConnectOptions::new()
                        .user_and_password(username, password),
                )
                .await
                .map_err(|error| {
                    raise_error!(format!(
                        "Failed to connect to NATS server at {} with username/password authentication. Error: {}",
                        nats_url, error
                    ), ErrorCode::NatsConnectionFailed)
                })?
            }
            NatsAuthType::Token => {
                let token = self.token.clone().ok_or_else(|| {
                    raise_error!(
                        "Token is required for token authentication but was not provided".into(),
                        ErrorCode::InvalidParameter
                    )
                })?;

                async_nats::connect_with_options(
                    &nats_url,
                    async_nats::ConnectOptions::new().token(token),
                )
                .await
                .map_err(|error| {
                    raise_error!(format!(
                        "Failed to connect to NATS server at {} with token authentication. Error: {}",
                        nats_url, error
                    ), ErrorCode::NatsConnectionFailed)
                })?
            }
        };

        let jetstream = jetstream::new(client);

        jetstream
            .create_stream(jetstream::stream::Config {
                name: self.stream_name.to_string(),
                subjects: vec![format!("{}.>", self.namespace)],
                ..Default::default()
            })
            .await
            .map_err(|error| {
                raise_error!(
                    format!(
                        "Failed to create NATS stream '{}' in namespace '{}'. Error: {}",
                        self.stream_name, self.namespace, error
                    ),
                    ErrorCode::NatsCreateStreamFailed
                )
            })?;

        Ok(jetstream)
    }
}
