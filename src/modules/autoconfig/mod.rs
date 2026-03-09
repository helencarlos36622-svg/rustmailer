// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::database::manager::DB_MANAGER;
use crate::modules::database::{async_find_impl, delete_impl, upsert_impl};
use crate::modules::error::code::ErrorCode;
use crate::raise_error;
use crate::{
    modules::autoconfig::entity::MailServerConfig, modules::error::RustMailerResult, utc_now,
};
use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

pub mod entity;
pub mod load;
#[cfg(test)]
mod tests;

const EXPIRE_TIME_MS: i64 = 30 * 24 * 60 * 60 * 1000;

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[native_model(id = 4, version = 1)]
#[native_db]
pub struct CachedMailSettings {
    #[primary_key]
    pub domain: String,
    pub config: MailServerConfig,
    pub created_at: i64,
}

impl CachedMailSettings {
    pub async fn add(domain: String, config: MailServerConfig) -> RustMailerResult<()> {
        Self {
            domain,
            config,
            created_at: utc_now!(),
        }
        .save()
        .await
    }

    async fn save(self) -> RustMailerResult<()> {
        upsert_impl(DB_MANAGER.meta_db(), self).await
    }

    pub async fn get(domain: &str) -> RustMailerResult<Option<CachedMailSettings>> {
        if let Some(found) =
            async_find_impl::<CachedMailSettings>(DB_MANAGER.meta_db(), domain.to_string()).await?
        {
            if (utc_now!() - found.created_at) > EXPIRE_TIME_MS {
                let domain = domain.to_string();
                delete_impl(DB_MANAGER.meta_db(), |rw| {
                    rw.get()
                        .primary::<CachedMailSettings>(domain)
                        .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                        .ok_or_else(|| {
                            raise_error!("auto config cache miss".into(), ErrorCode::InternalError)
                        })
                })
                .await?;
                Ok(None)
            } else {
                Ok(Some(found))
            }
        } else {
            Ok(None)
        }
    }
}
