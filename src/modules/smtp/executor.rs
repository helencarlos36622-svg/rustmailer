// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::smtp::client::Sender;
use crate::modules::{error::RustMailerResult, smtp::manager::SmtpClientManager};
use bb8::Pool;
use mail_send::smtp::message::IntoMessage;

pub struct SmtpExecutor {
    pool: Pool<SmtpClientManager>,
}

impl SmtpExecutor {
    pub fn new(pool: Pool<SmtpClientManager>) -> Self {
        Self { pool }
    }

    pub async fn send_email<'x>(&self, message: impl IntoMessage<'x>) -> RustMailerResult<()> {
        let mut client = self.pool.get().await?;
        client.send_email(message).await
    }

    pub async fn capabilities(&self, host: &str) -> RustMailerResult<u32> {
        let mut client = self.pool.get().await?;
        client.capabilities(host).await
    }

    // pub async fn send_signed_email<'x, V: mail_auth::common::crypto::SigningKey>(
    //     &mut self,
    //     message: impl IntoMessage<'x>,
    //     signer: &mail_auth::dkim::DkimSigner<V, mail_auth::dkim::Done>,
    // ) -> RustMailerResult<()> {
    //     let mut client = self.pool.get().await?;
    //     client.send_signed_email(message, signer).await
    // }
}
