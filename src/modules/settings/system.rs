// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::database::manager::DB_MANAGER;
use crate::modules::database::{find_impl, upsert_impl};
use crate::modules::error::RustMailerResult;
use crate::utc_now;
use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[native_model(id = 2, version = 1)]
#[native_db]
pub struct SystemSetting {
    #[primary_key]
    pub key: String,
    pub value: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl SystemSetting {
    pub fn new(key: String, value: String) -> Self {
        Self {
            key,
            value,
            created_at: utc_now!(),
            updated_at: utc_now!(),
        }
    }
    //overwrite
    pub async fn set(self) -> RustMailerResult<()> {
        upsert_impl(DB_MANAGER.meta_db(), self).await
    }

    pub fn get(key: &str) -> RustMailerResult<Option<SystemSetting>> {
        find_impl(DB_MANAGER.meta_db(), key)
    }

    // pub async fn list() -> RustMailerResult<Vec<SystemSetting>> {
    //     list_all_impl(DB_MANAGER.metadata_db()).await
    // }

    pub fn get_existing_value(key: &str) -> RustMailerResult<Option<String>> {
        let setting = Self::get(key)?;
        Ok(setting.map(|s| s.value))
    }

    pub async fn set_value(key: &str, value: String) -> RustMailerResult<()> {
        let setting = Self::new(key.to_string(), value);
        setting.set().await
    }
}
