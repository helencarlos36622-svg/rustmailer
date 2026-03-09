// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::account::entity::{AuthType, Encryption};
use crate::modules::account::migration::AccountModel;
use crate::modules::error::code::ErrorCode;
use crate::modules::error::RustMailerResult;
use crate::modules::oauth2::token::OAuth2AccessToken;
use crate::modules::settings::proxy::Proxy;
use crate::modules::smtp::client::RustMailSmtpClient;
use crate::modules::smtp::mta::entity::Mta;
use crate::modules::utils::net::parse_proxy_addr;
use crate::{decrypt, raise_error};
use mail_send::smtp::tls::build_tls_connector;
use mail_send::smtp::AssertReply;
use mail_send::{Credentials, SmtpClient, SmtpClientBuilder};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_socks::tcp::Socks5Stream;

pub const EXT_START_TLS: u32 = 1 << 24;

pub struct SmtpClientManager {
    server: SmtpServerType,
}

pub enum SmtpServerType {
    Mta(u64),
    Account(u64),
}

impl SmtpClientManager {
    pub fn new(server: SmtpServerType) -> Self {
        Self { server }
    }

    async fn build_mta(mta_id: u64) -> RustMailerResult<RustMailSmtpClient> {
        let mta = Mta::get(mta_id).await?.ok_or_else(|| {
            raise_error!(
                format!("MTA '{}' not found", mta_id),
                ErrorCode::ResourceNotFound
            )
        })?;

        let encrypted_password = mta.credentials.password.ok_or_else(|| {
            raise_error!(
                "mta password missing".into(),
                ErrorCode::MissingConfiguration
            )
        })?;

        let credentials =
            Credentials::new(mta.credentials.username, decrypt!(&encrypted_password)?);

        let timeout = Duration::from_secs(30);
        if let Some(proxy_id) = &mta.use_proxy {
            let proxy = Proxy::get(*proxy_id).await?;
            let proxy = parse_proxy_addr(&proxy.url)?;

            let socks_stream =
                Socks5Stream::connect(proxy, format!("{}:{}", &mta.server.host, mta.server.port))
                    .await
                    .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?;

            let tcp_stream = socks_stream.into_inner();
            return Self::connect(
                mta.server.encryption,
                &mta.server.host,
                timeout,
                tcp_stream,
                credentials,
            )
            .await
            .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::SmtpConnectionFailed));
        }

        let builder = SmtpClientBuilder::new(mta.server.host, mta.server.port)
            .credentials(credentials)
            .timeout(timeout);

        let client = match mta.server.encryption {
            Encryption::Ssl => {
                let client = builder.implicit_tls(true).connect().await.map_err(|e| {
                    raise_error!(format!("{:#?}", e), ErrorCode::SmtpConnectionFailed)
                })?;
                RustMailSmtpClient::Tls(client)
            }
            Encryption::StartTls => {
                let client = builder.implicit_tls(false).connect().await.map_err(|e| {
                    raise_error!(format!("{:#?}", e), ErrorCode::SmtpConnectionFailed)
                })?;
                RustMailSmtpClient::Tls(client)
            }
            Encryption::None => {
                let client = builder.connect_plain().await.map_err(|e| {
                    raise_error!(format!("{:#?}", e), ErrorCode::SmtpConnectionFailed)
                })?;
                RustMailSmtpClient::Plain(client)
            }
        };

        Ok(client)
    }

    async fn build_client(account_id: u64) -> RustMailerResult<RustMailSmtpClient> {
        let account = AccountModel::get(account_id).await?;

        let smtp = account
            .smtp
            .as_ref()
            .expect("BUG: account.smtp is None, but it should always be present here");

        let credentials = match smtp.auth.auth_type {
            AuthType::Password => {
                let password = smtp.auth.password.as_ref().ok_or_else(|| {
                    raise_error!(
                        "smtp auth type is Password, but password not set".into(),
                        ErrorCode::MissingConfiguration
                    )
                })?;
                Credentials::new(account.email, decrypt!(&password)?)
            }
            AuthType::OAuth2 => {
                let record = OAuth2AccessToken::get(account_id).await?;
                let access_token = record.and_then(|r| r.access_token).ok_or_else(|| {
                    raise_error!(
                        "SMTP auth type is OAuth2, but OAuth2 authorization is not yet complete."
                            .into(),
                        ErrorCode::MissingConfiguration
                    )
                })?;

                Credentials::new_xoauth2(account.email, access_token)
            }
        };

        let timeout = Duration::from_secs(30);
        if let Some(proxy_id) = smtp.use_proxy {
            let proxy = Proxy::get(proxy_id).await?;
            let proxy = parse_proxy_addr(&proxy.url)?;
            let socks_stream = Socks5Stream::connect(proxy, format!("{}:{}", smtp.host, smtp.port))
                .await
                .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?;

            let tcp_stream = socks_stream.into_inner();
            return Self::connect(
                smtp.encryption.clone(),
                &smtp.host,
                timeout,
                tcp_stream,
                credentials,
            )
            .await
            .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::SmtpConnectionFailed));
        }

        let builder = SmtpClientBuilder::new(smtp.host.clone(), smtp.port)
            .credentials(credentials)
            .timeout(timeout);

        let client = match smtp.encryption {
            Encryption::Ssl => {
                let client = builder.implicit_tls(true).connect().await.map_err(|e| {
                    raise_error!(format!("{:#?}", e), ErrorCode::SmtpConnectionFailed)
                })?;
                RustMailSmtpClient::Tls(client)
            }
            Encryption::StartTls => {
                let client = builder.implicit_tls(false).connect().await.map_err(|e| {
                    raise_error!(format!("{:#?}", e), ErrorCode::SmtpConnectionFailed)
                })?;
                RustMailSmtpClient::Tls(client)
            }
            Encryption::None => {
                let client = builder.connect_plain().await.map_err(|e| {
                    raise_error!(format!("{:#?}", e), ErrorCode::SmtpConnectionFailed)
                })?;
                RustMailSmtpClient::Plain(client)
            }
        };

        Ok(client)
    }
    pub async fn build(&self) -> RustMailerResult<RustMailSmtpClient> {
        match self.server {
            SmtpServerType::Mta(mta_id) => Self::build_mta(mta_id).await,
            SmtpServerType::Account(account_id) => Self::build_client(account_id).await,
        }
    }

    async fn connect(
        encryption: Encryption,
        host: &str,
        timeout: Duration,
        tcp_stream: TcpStream,
        credentials: Credentials<String>,
    ) -> Result<RustMailSmtpClient, mail_send::Error> {
        tokio::time::timeout(timeout, async {
            let mut client = SmtpClient {
                stream: tcp_stream,
                timeout: timeout,
            };

            let local_host = gethostname::gethostname()
                .to_str()
                .unwrap_or("[127.0.0.1]")
                .to_string();
            let tls_connector = build_tls_connector(false);
            match encryption {
                Encryption::Ssl => {
                    let mut client = client.into_tls(&tls_connector, host).await?;
                    // Read greeting
                    client.read().await?.assert_positive_completion()?;
                    let capabilities = client.capabilities(&local_host, false).await?;
                    // Authenticate
                    client.authenticate(&credentials, &capabilities).await?;
                    Ok(RustMailSmtpClient::Tls(client))
                }
                Encryption::StartTls => {
                    // Read greeting
                    client.read().await?.assert_positive_completion()?;
                    // Send EHLO
                    let response = client.ehlo(&local_host).await?;
                    if response.has_capability(EXT_START_TLS) {
                        let mut client = client.start_tls(&tls_connector, host).await?;
                        let capabilities = client.capabilities(&local_host, false).await?;
                        // Authenticate
                        client.authenticate(&credentials, &capabilities).await?;
                        Ok(RustMailSmtpClient::Tls(client))
                    } else {
                        return Err(mail_send::Error::MissingStartTls);
                    }
                }
                Encryption::None => {
                    // Read greeting
                    client.read().await?.assert_positive_completion()?;
                    let capabilities = client.capabilities(&local_host, false).await?;
                    // Authenticate
                    client.authenticate(&credentials, &capabilities).await?;
                    Ok(RustMailSmtpClient::Plain(client))
                }
            }
        })
        .await
        .map_err(|_| mail_send::Error::Timeout)?
    }
}
