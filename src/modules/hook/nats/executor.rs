// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::error::code::ErrorCode;
use crate::modules::hook::events::EventType;
use crate::modules::hook::nats::NatsConfig;
use crate::{
    modules::{error::RustMailerResult, hook::nats::NatsConnectionManager},
    raise_error,
};
#[cfg(test)]
use async_nats::jetstream::stream::Info;
use bb8::Pool;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use tracing::{debug, error};

use super::pool::build_nats_pool;

pub static NATS_EXECUTORS: LazyLock<NatsContextExecutors> =
    LazyLock::new(NatsContextExecutors::new);

pub struct NatsContextExecutors {
    nats: DashMap<NatsConfig, Arc<NatsExecutor>>,
}

impl NatsContextExecutors {
    pub fn new() -> Self {
        Self {
            nats: DashMap::new(),
        }
    }

    pub async fn get(&self, config: &NatsConfig) -> RustMailerResult<Arc<NatsExecutor>> {
        if let Some(executor) = self.nats.get(config) {
            return Ok(executor.value().clone());
        }

        let pool = build_nats_pool(config).await?;
        let executor = Arc::new(NatsExecutor::new(config.clone(), pool));

        match self.nats.try_entry(config.clone()) {
            Some(dashmap::mapref::entry::Entry::Occupied(entry)) => Ok(entry.get().clone()),
            Some(dashmap::mapref::entry::Entry::Vacant(entry)) => {
                entry.insert(executor.clone());
                Ok(executor)
            }
            None => Err(raise_error!(
                "DashMap locked".into(),
                ErrorCode::InternalError
            )),
        }
    }
}

pub struct NatsExecutor {
    config: NatsConfig,
    pool: Pool<NatsConnectionManager>,
}

impl NatsExecutor {
    pub fn new(config: NatsConfig, pool: Pool<NatsConnectionManager>) -> Self {
        Self { config, pool }
    }

    #[cfg(test)]
    pub async fn stream_info(&self) -> RustMailerResult<Info> {
        use crate::modules::error::code::ErrorCode;

        let context = self.pool.get().await?;
        let mut stream = context
            .get_stream(&self.config.stream_name)
            .await
            .map_err(|e| {
                raise_error!(
                    format!("Failed to get stream. error: {:#?}", e),
                    ErrorCode::NatsRequestFailed
                )
            })?;
        stream.info().await.map(|info| info.clone()).map_err(|e| {
            raise_error!(
                format!("failed to get stream info, {:#?}", e),
                ErrorCode::NatsRequestFailed
            )
        })
    }

    pub async fn publish(
        &self,
        task_info: Option<HashMap<String, String>>,
        event_type: EventType,
        payload: serde_json::Value,
    ) -> RustMailerResult<()> {
        let topic = format!("{}.{}", self.config.namespace, event_type);

        let mut headers = async_nats::HeaderMap::new();
        if let Some(task_info) = task_info {
            for (key, value) in task_info {
                headers.append(key, value);
            }
        }
        self.pool
            .get()
            .await?
            .publish_with_headers(topic, headers, payload.to_string().into())
            .await
            .map_err(|e| {
                error!("Failed to publish event to NATS: {:?}", e);
                raise_error!(
                    format!("{:#?}", e),
                    crate::modules::error::code::ErrorCode::NatsRequestFailed
                )
            })?;
        debug!("Successfully published event: {}", event_type);
        Ok(())
    }
}
