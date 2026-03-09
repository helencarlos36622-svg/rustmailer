// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::error::code::ErrorCode;
use crate::modules::error::RustMailerResult;
use crate::raise_error;
// use mail_send::mail_auth;
use mail_send::smtp::message::IntoMessage;
use mail_send::SmtpClient;
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;

pub enum RustMailSmtpClient {
    Plain(SmtpClient<TcpStream>),
    Tls(SmtpClient<TlsStream<TcpStream>>),
}

pub(crate) trait Sender {
    async fn send_noop(&mut self) -> RustMailerResult<()>;
    async fn reset(&mut self) -> RustMailerResult<()>;
    async fn send_email<'x>(&mut self, message: impl IntoMessage<'x>) -> RustMailerResult<()>;
    async fn capabilities(&mut self, host: &str) -> RustMailerResult<u32>;
    // async fn send_signed_email<'x, V: mail_auth::common::crypto::SigningKey>(
    //     &mut self,
    //     message: impl IntoMessage<'x>,
    //     signer: &mail_auth::dkim::DkimSigner<V, mail_auth::dkim::Done>,
    // ) -> RustMailerResult<()>;
}

impl Sender for RustMailSmtpClient {
    async fn send_noop(&mut self) -> RustMailerResult<()> {
        match self {
            RustMailSmtpClient::Plain(smtp_client) => smtp_client
                .noop()
                .await
                .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::SmtpCommandFailed)),
            RustMailSmtpClient::Tls(smtp_client) => smtp_client
                .noop()
                .await
                .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::SmtpCommandFailed)),
        }
    }

    async fn reset(&mut self) -> RustMailerResult<()> {
        match self {
            RustMailSmtpClient::Plain(smtp_client) => smtp_client
                .rset()
                .await
                .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::SmtpCommandFailed)),
            RustMailSmtpClient::Tls(smtp_client) => smtp_client
                .rset()
                .await
                .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::SmtpCommandFailed)),
        }
    }

    async fn send_email<'x>(&mut self, message: impl IntoMessage<'x>) -> RustMailerResult<()> {
        match self {
            RustMailSmtpClient::Plain(smtp_client) => smtp_client
                .send(message)
                .await
                .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::SmtpCommandFailed)),
            RustMailSmtpClient::Tls(smtp_client) => smtp_client
                .send(message)
                .await
                .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::SmtpCommandFailed)),
        }
    }

    async fn capabilities(&mut self, host: &str) -> RustMailerResult<u32> {
        let response =
            match self {
                RustMailSmtpClient::Plain(smtp_client) => smtp_client
                    .capabilities(host, false)
                    .await
                    .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::SmtpCommandFailed))?,
                RustMailSmtpClient::Tls(smtp_client) => smtp_client
                    .capabilities(host, false)
                    .await
                    .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::SmtpCommandFailed))?,
            };
        Ok(response.capabilities)
    }

    // async fn send_signed_email<'x, V: mail_auth::common::crypto::SigningKey>(
    //     &mut self,
    //     message: impl IntoMessage<'x>,
    //     signer: &mail_auth::dkim::DkimSigner<V, mail_auth::dkim::Done>,
    // ) -> RustMailerResult<()> {
    //     match self {
    //         RustMailSmtpClient::Plain(smtp_client) => smtp_client
    //             .send_signed(message, signer)
    //             .await
    //             .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::SmtpCommandFailed)),
    //         RustMailSmtpClient::Tls(smtp_client) => smtp_client
    //             .send_signed(message, signer)
    //             .await
    //             .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::SmtpCommandFailed)),
    //     }
    // }
}
