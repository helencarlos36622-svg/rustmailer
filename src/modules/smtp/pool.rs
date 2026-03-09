// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::error::RustMailerError;
use crate::modules::error::RustMailerResult;
use crate::modules::smtp::client::RustMailSmtpClient;
use crate::modules::smtp::client::Sender;
use crate::modules::smtp::manager::SmtpClientManager;
use crate::modules::smtp::manager::SmtpServerType;
use bb8::Pool;
use std::time::Duration;

impl bb8::ManageConnection for SmtpClientManager {
    type Connection = RustMailSmtpClient;
    type Error = RustMailerError;

    async fn connect(&self) -> RustMailerResult<Self::Connection> {
        self.build().await
    }

    // call this function before using the connection
    async fn is_valid(&self, conn: &mut Self::Connection) -> RustMailerResult<()> {
        conn.send_noop().await?;
        conn.reset().await
    }

    fn has_broken(&self, _: &mut Self::Connection) -> bool {
        false
    }
}

pub async fn build_smtp_pool(server: SmtpServerType) -> RustMailerResult<Pool<SmtpClientManager>> {
    let manager = SmtpClientManager::new(server);
    let pool = Pool::builder()
        .connection_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(120))
        .retry_connection(true)
        .max_size(10)
        .test_on_check_out(true)
        .build(manager)
        .await?;
    Ok(pool)
}
