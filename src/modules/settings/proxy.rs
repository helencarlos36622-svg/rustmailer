// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use native_db::*;
use native_model::{native_model, Model};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use crate::{
    id,
    modules::{
        database::{
            async_find_impl, delete_impl, insert_impl, list_all_impl, manager::DB_MANAGER,
            update_impl,
        },
        error::{code::ErrorCode, RustMailerResult},
        utils::net::parse_proxy_addr,
    },
    raise_error, utc_now,
};

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, Object)]
#[native_model(id = 15, version = 1)]
#[native_db]
pub struct Proxy {
    /// The unique identifier for this proxy configuration.
    #[primary_key]
    pub id: u64,

    /// The proxy URL (e.g., socks5://127.0.0.1:1080) used to route network requests.
    pub url: String,

    /// The creation timestamp of this record, represented as milliseconds since the Unix epoch.
    pub created_at: i64,

    /// The last update timestamp of this record, represented as milliseconds since the Unix epoch.
    pub updated_at: i64,
}

impl Proxy {
    /// Create a new Proxy instance with the given URL and timestamps.
    pub fn new(url: String) -> Self {
        Self {
            id: id!(64),
            url,
            created_at: utc_now!(),
            updated_at: utc_now!(),
        }
    }

    pub async fn get(id: u64) -> RustMailerResult<Proxy> {
        async_find_impl(DB_MANAGER.meta_db(), id)
            .await?
            .ok_or_else(|| {
                raise_error!(
                    format!("Proxy with id={} not found", id),
                    ErrorCode::ResourceNotFound
                )
            })
    }

    pub async fn list_all() -> RustMailerResult<Vec<Proxy>> {
        list_all_impl(DB_MANAGER.meta_db()).await
    }

    pub async fn delete(id: u64) -> RustMailerResult<()> {
        delete_impl(DB_MANAGER.meta_db(), move |rw| {
            rw.get()
                .primary::<Proxy>(id)
                .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                .ok_or_else(|| raise_error!("proxy missing".into(), ErrorCode::InternalError))
        })
        .await
    }

    pub async fn update(id: u64, url: String) -> RustMailerResult<()> {
        update_impl(
            DB_MANAGER.meta_db(),
            move |rw| {
                rw.get()
                    .primary::<Proxy>(id)
                    .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                    .ok_or_else(|| {
                        raise_error!(
                            format!("Proxy with id={} not found", id),
                            ErrorCode::ResourceNotFound
                        )
                    })
            },
            move |current| {
                let mut updated = current.clone();
                updated.url = url;
                updated.updated_at = utc_now!();
                Ok(updated)
            },
        )
        .await?;
        Ok(())
    }

    pub async fn save(self) -> RustMailerResult<()> {
        self.validate()?;
        insert_impl(DB_MANAGER.meta_db(), self).await
    }

    /// Validate that the URL is a valid SOCKS5 proxy URL.
    pub fn validate(&self) -> RustMailerResult<()> {
        parse_proxy_addr(&self.url)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_proxy_urls() {
        let urls = vec![
            "socks5://127.0.0.1:1080",
            "http://127.0.0.1:8080",
        ];

        for url in urls {
            let proxy = Proxy::new(url.to_string());
            assert!(proxy.validate().is_ok(), "URL should be valid: {}", url);
        }
    }
}
