// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::{
    account::{migration::AccountModel, status::AccountRunningState},
    cache::vendor::outlook::sync::{
        client::OutlookClient, envelope::OutlookEnvelope, folders::OutlookFolder,
    },
    error::RustMailerResult,
};

const ENVELOPE_BATCH_SIZE: u32 = 20;

pub async fn fetch_and_save_since_date(
    account: &AccountModel,
    date: &str,
    folder: &OutlookFolder,
    initial: bool,
) -> RustMailerResult<usize> {
    let mut inserted_count = 0;
    let account_id = account.id;
    let use_proxy = account.use_proxy;
    let folder_limit = account.folder_limit.unwrap_or(u32::max_value());
    let mut page = 1u32;
    let page_size = ENVELOPE_BATCH_SIZE;

    loop {
        if inserted_count >= folder_limit as usize {
            break;
        }

        let resp = OutlookClient::list_messages(
            account_id,
            use_proxy,
            &folder.folder_id,
            page as u64,
            page_size as u64,
            Some(date),
        )
        .await?;
        if page == 1 && initial {
            AccountRunningState::set_initial_current_syncing_folder(
                account_id,
                folder.name.clone(),
                None,
            )
            .await?;
        }

        if initial {
            AccountRunningState::set_current_sync_batch_number(account_id, page).await?;
        }

        if !resp.value.is_empty() {
            let envelopes: Vec<OutlookEnvelope> = resp
                .value
                .into_iter()
                .map(|m| {
                    let mut envelope: OutlookEnvelope = m.try_into()?;
                    envelope.account_id = account_id;
                    envelope.folder_id = folder.id;
                    envelope.folder_name = folder.name.clone();
                    Ok(envelope)
                })
                .collect::<RustMailerResult<Vec<OutlookEnvelope>>>()?;
            inserted_count += envelopes.len();
            OutlookEnvelope::save_envelopes(envelopes).await?;
        }
        if resp.next_link.is_none() {
            break;
        }
        page += 1;
    }
    Ok(inserted_count)
}

pub async fn fetch_and_save_full_folder(
    account: &AccountModel,
    folder: &OutlookFolder,
    total: u32,
    initial: bool,
) -> RustMailerResult<usize> {
    let folder_limit = account.folder_limit;
    let total_to_fetch = match folder_limit {
        Some(limit) if limit < total => total.min(limit.max(100)),
        _ => total,
    };
    let page_size = ENVELOPE_BATCH_SIZE;
    let total_batches = total_to_fetch.div_ceil(page_size);
    let mut inserted_count = 0;
    let account_id = account.id;
    let use_proxy = account.use_proxy;
    // If this is the first synchronization, set the initial state for the account
    if initial {
        AccountRunningState::set_initial_current_syncing_folder(
            account_id,
            folder.name.clone(),
            Some(total_batches),
        )
        .await?;
    }
    let mut page = 1u32;
    loop {
        // Stop if we have already fetched enough messages
        if inserted_count as u32 >= total_to_fetch {
            break;
        }
        let resp = OutlookClient::list_messages(
            account_id,
            use_proxy,
            &folder.folder_id,
            page as u64,
            page_size as u64,
            None,
        )
        .await?;
        if initial {
            AccountRunningState::set_current_sync_batch_number(account_id, page).await?;
        }
        if !resp.value.is_empty() {
            let envelopes: Vec<OutlookEnvelope> = resp
                .value
                .into_iter()
                .map(|m| {
                    let mut envelope: OutlookEnvelope = m.try_into()?;
                    envelope.account_id = account_id;
                    envelope.folder_id = folder.id;
                    envelope.folder_name = folder.name.clone();
                    Ok(envelope)
                })
                .collect::<RustMailerResult<Vec<OutlookEnvelope>>>()?;
            inserted_count += envelopes.len();
            OutlookEnvelope::save_envelopes(envelopes).await?;
        }

        if resp.next_link.is_none() {
            break;
        }
        page += 1;
    }
    Ok(inserted_count)
}
