// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::{
    account::{migration::AccountModel, since::DateSince},
    cache::vendor::outlook::sync::{
        client::OutlookClient,
        delta::FolderDeltaLink,
        flow::{fetch_and_save_full_folder, fetch_and_save_since_date},
        folders::OutlookFolder,
    },
    error::RustMailerResult,
};
use std::time::Instant;
use tracing::{error, info, warn};

pub async fn rebuild_cache(
    account: &AccountModel,
    remote_folders: &[OutlookFolder],
) -> RustMailerResult<()> {
    let start_time = Instant::now();
    let mut total_inserted = 0;

    let account_id = account.id;
    let use_proxy = account.use_proxy;
    OutlookFolder::batch_insert(remote_folders).await?;
    for folder in remote_folders {
        if !account.minimal_sync() {
            if folder.exists > 0 {
                match fetch_and_save_full_folder(account, folder, folder.exists, true).await {
                    Ok(inserted) => {
                        total_inserted += inserted;
                    }
                    Err(e) => {
                        warn!(
                    "Account {}: Failed to sync label '{}'. Error: {:#?}. Removing label entry.",
                    account.id, &folder.name, e
                );
                        if let Err(del_err) = OutlookFolder::delete(folder.id).await {
                            error!(
                                "Account {}: Failed to delete label '{}' after sync error: {}",
                                account.id, &folder.name, del_err
                            );
                        }
                    }
                }
            } else {
                warn!(
                "Account {}: folder '{}' on the remote server has no emails. Skipping fetch for this folder.",
                account.id, &folder.name
            );
            }
        }

        let delta_link =
            OutlookClient::get_delta_link(account_id, use_proxy, &folder.folder_id).await?;
        FolderDeltaLink::upsert(account_id, &folder.folder_id, &delta_link).await?;
    }

    if !account.minimal_sync() {
        let elapsed_time = start_time.elapsed().as_secs();
        info!(
            "Rebuild account cache completed: {} envelopes inserted. {} secs elapsed. \
            This is a full data fetch as there was no local cache data available.",
            total_inserted, elapsed_time
        );
    }
    Ok(())
}

pub async fn rebuild_cache_since_date(
    account: &AccountModel,
    remote_folders: &[OutlookFolder],
    date_since: &DateSince,
) -> RustMailerResult<()> {
    let start_time = Instant::now();
    let mut total_inserted = 0;
    let date = date_since.since_outlook_date()?;

    let account_id = account.id;
    let use_proxy = account.use_proxy;

    OutlookFolder::batch_insert(remote_folders).await?;
    for folder in remote_folders {
        if !account.minimal_sync() {
            if folder.exists > 0 {
                match fetch_and_save_since_date(account, date.as_str(), folder, true).await {
                    Ok(inserted) => {
                        total_inserted += inserted;
                    }
                    Err(e) => {
                        warn!(
                    "Account {}: Failed to sync mailbox '{}'. Error: {}. Removing mailbox entry.",
                    account.id, &folder.name, e
                );
                        if let Err(del_err) = OutlookFolder::delete(folder.id).await {
                            error!(
                                "Account {}: Failed to delete mailbox '{}' after sync error: {}",
                                account.id, &folder.name, del_err
                            );
                        }
                    }
                }
            } else {
                warn!(
                "Account {}: Mailbox '{}' on the remote server has no emails. Skipping fetch for this mailbox.",
                account.id, &folder.name
            );
            }
        }

        let delta_link =
            OutlookClient::get_delta_link(account_id, use_proxy, &folder.folder_id).await?;
        FolderDeltaLink::upsert(account_id, &folder.folder_id, &delta_link).await?;
    }
    if !account.minimal_sync() {
        let elapsed_time = start_time.elapsed().as_secs();
        info!(
            "Rebuild account cache completed: {} envelopes inserted. {} secs elapsed. \
        Data fetched from server starting from the specified date: {}.",
            total_inserted, elapsed_time, date
        );
    }
    Ok(())
}

pub async fn rebuild_single_folder_cache(
    account: &AccountModel,
    folder: &OutlookFolder,
) -> RustMailerResult<()> {
    if folder.exists > 0 {
        if !account.minimal_sync() {
            match &account.date_since {
                Some(date_since) => {
                    let date = date_since.since_outlook_date()?;
                    match fetch_and_save_since_date(account, date.as_str(), folder, true).await {
                        Ok(inserted) => {
                            info!(
                            "Account {}: folder '{}' synced successfully. {} messages inserted.",
                            account.id, folder.name, inserted
                        );
                            return Ok(());
                        }
                        Err(e) => {
                            warn!(
                                    "Account {}: Failed to sync mailbox '{}'. Error: {}. Removing folder entry.",
                                    account.id, &folder.name, e
                                );
                            if let Err(del_err) = OutlookFolder::delete(folder.id).await {
                                error!(
                                "Account {}: Failed to delete mailbox '{}' after sync error: {}",
                                account.id, &folder.name, del_err
                            );
                            }
                        }
                    }
                }
                None => {
                    match fetch_and_save_full_folder(account, folder, folder.exists, true).await {
                        Ok(inserted) => {
                            info!(
                        "Account {}: folder '{}' synced successfully. {} messages inserted.",
                        account.id, folder.name, inserted
                    );
                            return Ok(());
                        }
                        Err(e) => {
                            warn!(
                        "Account {}: Failed to sync folder '{}'. Error: {:#?}. Removing folder entry.",
                        account.id, &folder.name, e
                    );
                            if let Err(del_err) = OutlookFolder::delete(folder.id).await {
                                error!(
                                    "Account {}: Failed to delete folder '{}' after sync error: {}",
                                    account.id, &folder.name, del_err
                                );
                            }
                        }
                    }
                }
            }
        }

        let delta_link =
            OutlookClient::get_delta_link(account.id, account.use_proxy, &folder.folder_id).await?;
        FolderDeltaLink::upsert(account.id, &folder.folder_id, &delta_link).await?;
    }
    Ok(())
}
