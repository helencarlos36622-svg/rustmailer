// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::error::code::ErrorCode;
use crate::modules::error::{RustMailerError, RustMailerResult};
use crate::modules::hook::nats::{NatsConfig, NatsConnectionManager};
use crate::raise_error;
use bb8::Pool;
use std::time::Duration;

impl bb8::ManageConnection for NatsConnectionManager {
    type Connection = async_nats::jetstream::Context;

    type Error = RustMailerError;

    async fn connect(&self) -> RustMailerResult<Self::Connection> {
        self.build().await
    }
    // call this function before using the connection
    async fn is_valid(&self, conn: &mut Self::Connection) -> RustMailerResult<()> {
        conn.query_account().await.map_err(|e| {
            raise_error!(
                format!("can't query nats account info,  error: {:#?}", e),
                ErrorCode::NatsConnectionFailed
            )
        })?;
        Ok(())
    }

    fn has_broken(&self, _: &mut Self::Connection) -> bool {
        false
    }
}

pub async fn build_nats_pool(config: &NatsConfig) -> RustMailerResult<Pool<NatsConnectionManager>> {
    let manager = NatsConnectionManager::new(config);
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
