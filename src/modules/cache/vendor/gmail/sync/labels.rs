// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use itertools::Itertools;
use native_db::*;
use native_model::{native_model, Model};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use crate::{
    modules::{
        cache::imap::mailbox::MailBox,
        database::{
            async_find_impl, batch_delete_impl, batch_insert_impl, delete_impl,
            filter_by_secondary_key_impl, manager::DB_MANAGER, upsert_impl,
        },
        error::{code::ErrorCode, RustMailerResult},
    },
    raise_error, utc_now,
};

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
#[native_model(id = 7, version = 1)]
#[native_db]
pub struct GmailLabels {
    /// This `id` **must be a hash value constructed from both `account_id` and `label_id`**,
    /// ensuring global uniqueness across all accounts and labels.
    #[primary_key]
    pub id: u64,
    #[secondary_key]
    pub account_id: u64,
    pub name: String,
    pub exists: u32,
    pub unseen: u32,
    pub label_id: String,
}

impl GmailLabels {
    pub async fn upsert(label: GmailLabels) -> RustMailerResult<()> {
        upsert_impl(DB_MANAGER.envelope_db(), label).await
    }

    pub async fn batch_insert(labels: &[GmailLabels]) -> RustMailerResult<()> {
        batch_insert_impl(DB_MANAGER.envelope_db(), labels.to_vec()).await
    }

    pub async fn delete(id: u64) -> RustMailerResult<()> {
        delete_impl(DB_MANAGER.envelope_db(), move |rw| {
            rw.get()
                .primary::<GmailLabels>(id)
                .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                .ok_or_else(|| raise_error!("label missing".into(), ErrorCode::InternalError))
        })
        .await
    }

    pub async fn list_all(account_id: u64) -> RustMailerResult<Vec<GmailLabels>> {
        filter_by_secondary_key_impl(
            DB_MANAGER.envelope_db(),
            GmailLabelsKey::account_id,
            account_id,
        )
        .await
    }

    // pub async fn get_label_map(account_id: u64) -> RustMailerResult<HashMap<String, String>> {
    //     let labels = Self::list_all(account_id).await?;
    //     let map: HashMap<String, String> = labels
    //         .into_iter()
    //         .map(|label| (label.label_id, label.name))
    //         .collect();
    //     Ok(map)
    // }

    pub async fn get_by_name(account_id: u64, name: &str) -> RustMailerResult<Self> {
        let labels = Self::list_all(account_id).await?;
        labels
            .into_iter()
            .find(|label| label.name == name)
            .ok_or_else(|| {
                raise_error!(
                    format!("Label '{}' not found for account {}", name, account_id),
                    ErrorCode::MailBoxNotCached
                )
            })
    }

    pub async fn batch_delete(labels: Vec<GmailLabels>) -> RustMailerResult<()> {
        batch_delete_impl(DB_MANAGER.envelope_db(), move |rw| {
            let mut to_deleted = Vec::new();
            for label in labels {
                let retrived = rw
                    .get()
                    .primary::<GmailLabels>(label.id)
                    .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?;
                if let Some(retrived) = retrived {
                    to_deleted.push(retrived);
                }
            }
            Ok(to_deleted)
        })
        .await?;
        Ok(())
    }

    pub async fn clean(account_id: u64) -> RustMailerResult<()> {
        batch_delete_impl(DB_MANAGER.envelope_db(), move |rw| {
            let labels: Vec<GmailLabels> = rw
                .scan()
                .secondary::<GmailLabels>(GmailLabelsKey::account_id)
                .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                .start_with(account_id)
                .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                .try_collect()
                .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?;
            Ok(labels)
        })
        .await?;
        Ok(())
    }
}

impl From<GmailLabels> for MailBox {
    fn from(value: GmailLabels) -> Self {
        Self {
            id: value.id,
            account_id: value.account_id,
            name: value.name,
            delimiter: Some("/".into()),
            attributes: vec![],
            flags: vec![],
            exists: value.exists,
            unseen: Some(value.unseen),
            permanent_flags: vec![],
            uid_next: None,
            uid_validity: None,
            highest_modseq: None,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
#[native_model(id = 8, version = 1)]
#[native_db]
pub struct GmailCheckPoint {
    /// The Gmail account ID this checkpoint belongs to.
    #[primary_key]
    pub account_id: u64,

    /// The latest Gmail `historyId` for incremental synchronization.
    /// Used as `startHistoryId` in the next Gmail History API call.
    pub history_id: String,

    /// Creation timestamp in UNIX epoch milliseconds.
    /// Records when this checkpoint was initially created.
    pub created_at: i64,
}

impl GmailCheckPoint {
    pub async fn get(account_id: u64) -> RustMailerResult<GmailCheckPoint> {
        let entity = async_find_impl(DB_MANAGER.envelope_db(), account_id).await?;
        entity.ok_or_else(|| {
            raise_error!(
                format!("GmailCheckPoint not found for id={}", account_id),
                ErrorCode::ResourceNotFound
            )
        })
    }

    pub async fn find(account_id: u64) -> RustMailerResult<Option<GmailCheckPoint>> {
        async_find_impl(DB_MANAGER.envelope_db(), account_id).await
    }

    pub fn new(account_id: u64, history_id: String) -> Self {
        Self {
            account_id,
            history_id,
            created_at: utc_now!(),
        }
    }
    // Upsert is used here to overwrite the existing record
    pub async fn save(self) -> RustMailerResult<()> {
        upsert_impl(DB_MANAGER.envelope_db(), self).await
    }

    pub async fn clean(account_id: u64) -> RustMailerResult<()> {
        if Self::find(account_id).await?.is_some() {
            delete_impl(DB_MANAGER.envelope_db(), move |rw| {
                rw.get()
                    .primary::<GmailCheckPoint>(account_id)
                    .map_err(|e| raise_error!(e.to_string(), ErrorCode::InternalError))?
                    .ok_or_else(|| {
                        raise_error!(
                            "gmail history id checkpoint missing".into(),
                            ErrorCode::InternalError
                        )
                    })
            })
            .await?;
        }
        Ok(())
    }
}
