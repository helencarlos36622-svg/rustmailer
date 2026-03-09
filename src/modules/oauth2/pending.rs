// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::{
    modules::{
        database::{
            batch_delete_impl, delete_impl, async_find_impl, insert_impl, manager::DB_MANAGER,
        },
        error::{code::ErrorCode, RustMailerResult},
    },
    raise_error, utc_now,
};
use itertools::Itertools;
use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

const EXPIRATION_DURATION_MS: i64 = 24 * 60 * 60 * 1000;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[native_model(id = 9, version = 1)]
#[native_db]
pub struct OAuth2PendingEntity {
    /// Unique identifier for the OAuth2 request record
    pub oauth2_id: u64,

    pub account_id: u64,
    /// CSRF protection state parameter used to verify the integrity of the authorization request
    #[primary_key]
    pub state: String,

    /// PKCE code verifier used in the authorization code exchange process to ensure security
    pub code_verifier: String,

    /// Timestamp when the OAuth2 request was created, used to determine request expiration
    pub created_at: i64,
}

impl OAuth2PendingEntity {
    pub fn new(
        oauth2_id: u64,
        account_id: u64,
        state: String,
        code_verifier: String,
    ) -> Self {
        Self {
            oauth2_id,
            account_id,
            state,
            code_verifier,
            created_at: utc_now!(),
        }
    }

    pub async fn save(self) -> RustMailerResult<()> {
        insert_impl(DB_MANAGER.meta_db(), self).await
    }

    pub async fn delete(state: &str) -> RustMailerResult<()> {
        let state = state.to_string();
        delete_impl(DB_MANAGER.meta_db(), move |rw| {
            rw.get().primary::<OAuth2PendingEntity>(state.clone())
            .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
            .ok_or_else(|| raise_error!(format!(
                "The oauth2 pending entity with state={state} that you want to delete was not found."
            ), ErrorCode::ResourceNotFound))
        }).await
    }

    pub async fn clean() -> RustMailerResult<()> {
        batch_delete_impl(DB_MANAGER.meta_db(), |rw| {
            let all: Vec<OAuth2PendingEntity> = rw
                .scan()
                .primary()
                .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                .all()
                .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                .try_collect()
                .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?;

            let now = utc_now!();
            let to_delete: Vec<OAuth2PendingEntity> = all
                .into_iter()
                .filter(|e| now - e.created_at > EXPIRATION_DURATION_MS)
                .collect();
            Ok(to_delete)
        })
        .await?;
        Ok(())
    }

    pub async fn get(state: &str) -> RustMailerResult<Option<OAuth2PendingEntity>> {
        let entity =
            async_find_impl::<OAuth2PendingEntity>(DB_MANAGER.meta_db(), state.to_string())
                .await?;

        match entity {
            Some(entity) => {
                let state = state.to_string();
                if utc_now!() - entity.created_at > EXPIRATION_DURATION_MS {
                    delete_impl(DB_MANAGER.meta_db(), move |rw| {
                        rw.get()
                            .primary::<OAuth2PendingEntity>(state)
                            .map_err(|e| {
                                raise_error!(format!("{:#?}", e), ErrorCode::InternalError)
                            })?
                            .ok_or_else(|| {
                                raise_error!(
                                    "OAuth2 pending entity not found".into(),
                                    ErrorCode::ResourceNotFound
                                )
                            })
                    })
                    .await?;
                    return Ok(None);
                }
                Ok(Some(entity))
            }
            None => Ok(None),
        }
    }
}
