// Copyright © 2025 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use std::{collections::HashSet, sync::Arc, time::Instant};

use futures::future::join_all;
use itertools::Itertools;
use native_db::*;
use native_model::{native_model, Model};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    modules::{
        account::migration::AccountModel,
        cache::{
            imap::migration::EmailEnvelopeV3,
            model::Envelope,
            vendor::{
                gmail::sync::{client::GmailClient, envelope::GmailEnvelope},
                outlook::sync::envelope::OutlookEnvelope,
            },
        },
        database::{batch_delete_impl, manager::DB_MANAGER, paginate_secondary_scan_impl},
        error::{code::ErrorCode, RustMailerResult},
        rest::response::DataPage,
        utils::envelope_hash,
    },
    raise_error, utc_now,
};

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
#[native_model(id = 5, version = 1)]
#[native_db(primary_key(pk -> String))]
pub struct EmailThread {
    #[secondary_key(unique)]
    pub thread_id: u64,
    #[secondary_key(unique)]
    pub envelope_id: u64,
    #[secondary_key]
    pub account_id: u64,
    #[secondary_key]
    pub mailbox_id: u64,
    pub internal_date: Option<i64>,
    pub date: Option<i64>,
}
impl EmailThread {
    pub fn pk(&self) -> String {
        format!(
            "{}_{}",
            self.internal_date.unwrap_or(utc_now!()),
            self.thread_id
        )
    }

    pub fn new(
        thread_id: u64,
        envelope_id: u64,
        account_id: u64,
        mailbox_id: u64,
        internal_date: Option<i64>,
        date: Option<i64>,
    ) -> Self {
        Self {
            thread_id,
            envelope_id,
            account_id,
            mailbox_id,
            internal_date,
            date,
        }
    }

    pub async fn clean_mailbox_envelopes(account_id: u64, mailbox_id: u64) -> RustMailerResult<()> {
        const BATCH_SIZE: usize = 200;
        loop {
            let deleted = batch_delete_impl(DB_MANAGER.envelope_db(), move |rw| {
                let to_delete: Vec<EmailThread> = rw
                    .scan()
                    .secondary(EmailThreadKey::mailbox_id)
                    .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                    .start_with(mailbox_id)
                    .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                    .filter_map(Result::ok) // filter only Ok values
                    .filter(|e: &EmailThread| e.account_id == account_id)
                    .take(BATCH_SIZE)
                    .collect();
                Ok(to_delete)
            })
            .await?;
            // If this batch is empty, break the loop
            if deleted == 0 {
                break;
            }
        }
        Ok(())
    }

    pub async fn clean_account(account_id: u64) -> RustMailerResult<()> {
        const BATCH_SIZE: usize = 200;
        loop {
            let deleted = batch_delete_impl(DB_MANAGER.envelope_db(), move |rw| {
                let to_delete: Vec<EmailThread> = rw
                    .scan()
                    .secondary(EmailThreadKey::account_id)
                    .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                    .start_with(account_id)
                    .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                    .take(BATCH_SIZE)
                    .try_collect()
                    .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?;
                Ok(to_delete)
            })
            .await?;
            // If this batch is empty, break the loop
            if deleted == 0 {
                break;
            }
        }
        Ok(())
    }

    pub async fn clean_envelopes(
        account_id: u64,
        mailbox_id: u64,
        to_delete_uid: &[u32],
    ) -> RustMailerResult<()> {
        const BATCH_SIZE: usize = 200;
        let mut total_deleted = 0usize;
        let start_time = Instant::now();

        let to_delete_set: HashSet<u64> = to_delete_uid
            .iter()
            .map(|uid| envelope_hash(account_id, mailbox_id, *uid))
            .collect();

        let to_delete_set = Arc::new(to_delete_set);
        loop {
            let to_delete_set = to_delete_set.clone();
            let deleted = batch_delete_impl(DB_MANAGER.envelope_db(), move |rw| {
                let to_delete: Vec<EmailThread> = rw
                    .scan()
                    .secondary(EmailThreadKey::mailbox_id)
                    .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                    .start_with(mailbox_id)
                    .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                    .filter_map(Result::ok) // filter only Ok values
                    .filter(|e: &EmailThread| {
                        e.account_id == account_id && to_delete_set.contains(&e.envelope_id)
                    })
                    .take(BATCH_SIZE)
                    .collect();
                Ok(to_delete)
            })
            .await?;
            total_deleted += deleted;
            // If this batch is empty, break the loop
            if deleted == 0 {
                break;
            }
        }

        info!(
            "Finished deleting thread entities for mailbox_id={} account_id={} total_deleted={} in {:?}",
            mailbox_id,
            account_id,
            total_deleted,
            start_time.elapsed()
        );
        Ok(())
    }

    pub async fn list_threads_in_mailbox(
        mailbox_id: u64,
        page: u64,
        page_size: u64,
        desc: bool,
    ) -> RustMailerResult<DataPage<Envelope>> {
        let threads = paginate_secondary_scan_impl::<EmailThread>(
            DB_MANAGER.envelope_db(),
            Some(page),
            Some(page_size),
            Some(desc),
            EmailThreadKey::mailbox_id,
            mailbox_id,
        )
        .await?;

        let fetch_tasks = threads.items.into_iter().map(|thread| async move {
            EmailEnvelopeV3::get(thread.envelope_id)
                .await?
                .ok_or_else(|| {
                    raise_error!(
                        format!("Envelope not found: {}", thread.envelope_id),
                        ErrorCode::InternalError
                    )
                })
        });

        let results: RustMailerResult<Vec<EmailEnvelopeV3>> =
            join_all(fetch_tasks).await.into_iter().collect();

        let envelopes = results?;
        Ok(DataPage {
            current_page: threads.page,
            page_size: threads.page_size,
            total_items: threads.total_items,
            items: envelopes.into_iter().map(Envelope::from).collect(),
            total_pages: threads.total_pages,
        })
    }

    pub async fn list_threads_in_label(
        account: AccountModel,
        label_id: u64,
        page: u64,
        page_size: u64,
        desc: bool,
    ) -> RustMailerResult<DataPage<Envelope>> {
        let threads = paginate_secondary_scan_impl::<EmailThread>(
            DB_MANAGER.envelope_db(),
            Some(page),
            Some(page_size),
            Some(desc),
            EmailThreadKey::mailbox_id,
            label_id,
        )
        .await?;

        let fetch_tasks = threads.items.into_iter().map(|thread| async move {
            GmailEnvelope::get(thread.envelope_id)
                .await?
                .ok_or_else(|| {
                    raise_error!(
                        format!("Envelope not found: {}", thread.envelope_id),
                        ErrorCode::InternalError
                    )
                })
        });

        let results: RustMailerResult<Vec<GmailEnvelope>> =
            join_all(fetch_tasks).await.into_iter().collect();
        let map = GmailClient::for_get_label_name(account.id, account.use_proxy).await?;
        let envelopes = results?
            .into_iter()
            .map(|e| e.into_envelope(&map))
            .collect();
        Ok(DataPage {
            current_page: threads.page,
            page_size: threads.page_size,
            total_items: threads.total_items,
            items: envelopes,
            total_pages: threads.total_pages,
        })
    }

    pub async fn list_threads_in_folder(
        folder_id: u64,
        page: u64,
        page_size: u64,
        desc: bool,
    ) -> RustMailerResult<DataPage<Envelope>> {
        let threads = paginate_secondary_scan_impl::<EmailThread>(
            DB_MANAGER.envelope_db(),
            Some(page),
            Some(page_size),
            Some(desc),
            EmailThreadKey::mailbox_id,
            folder_id,
        )
        .await?;

        let fetch_tasks = threads.items.into_iter().map(|thread| async move {
            OutlookEnvelope::get(thread.envelope_id)
                .await?
                .ok_or_else(|| {
                    raise_error!(
                        format!("Envelope not found: {}", thread.envelope_id),
                        ErrorCode::InternalError
                    )
                })
        });

        let results: RustMailerResult<Vec<OutlookEnvelope>> =
            join_all(fetch_tasks).await.into_iter().collect();

        let envelopes = results?.into_iter().map(|e| e.into()).collect();
        Ok(DataPage {
            current_page: threads.page,
            page_size: threads.page_size,
            total_items: threads.total_items,
            items: envelopes,
            total_pages: threads.total_pages,
        })
    }

    pub fn need_update(&self, new_thread: &EmailThread) -> bool {
        self.internal_date
            .map_or(true, |c| new_thread.internal_date.map_or(false, |n| n > c))
    }
}
