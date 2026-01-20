use std::time::Instant;

use ahash::AHashSet;
use tracing::info;

use crate::modules::{
    account::{entity::MailerType, migration::AccountModel, status::AccountRunningState},
    cache::{
        imap::{address::AddressEntity, thread::EmailThread},
        sync_type::{determine_sync_type, SyncType},
        vendor::outlook::sync::{
            delta::{handle_delta, FolderDeltaLink},
            envelope::OutlookEnvelope,
            folders::OutlookFolder,
            rebuild::{rebuild_cache, rebuild_cache_since_date, rebuild_single_folder_cache},
            sync_folders::get_sync_folders,
        },
    },
    error::{RustMailerError, RustMailerResult},
    hook::{
        channel::{Event, EVENT_CHANNEL},
        events::{payload::AccountChange, EventPayload, EventType, RustMailerEvent},
        task::EventHookTask,
    },
    utils::mailbox_id,
};

pub mod client;
pub mod delta;
pub mod envelope;
pub mod flow;
pub mod folders;
pub mod rebuild;
pub mod sync_folders;

pub async fn execute_outlook_sync(account: &AccountModel) -> RustMailerResult<()> {
    assert!(
        matches!(account.mailer_type, MailerType::GraphApi),
        "Bug: Unexpected mailer type, expected GraphApi, found: {:?}",
        account.mailer_type
    );

    let sync_type = determine_sync_type(account).await?;
    if matches!(sync_type, SyncType::SkipSync) {
        return Ok(());
    }

    let remote_folders = get_sync_folders(account).await?;
    let remote_folders: Vec<OutlookFolder> = remote_folders
        .into_iter()
        .map(|folder| {
            let mut f: OutlookFolder = folder.try_into()?; // may fail
            f.account_id = account.id;
            f.id = mailbox_id(account.id, &f.folder_id);
            Ok::<OutlookFolder, RustMailerError>(f)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let local_folders = OutlookFolder::list_all(account.id).await?;
    let delta_links = FolderDeltaLink::get_by_account(account.id).await?;

    if should_rebuild_cache(account, &local_folders, !delta_links.is_empty()).await? {
        AccountRunningState::set_initial_sync_folders(
            account.id,
            remote_folders.iter().map(|f| f.name.clone()).collect(),
        )
        .await?;
        match &account.date_since {
            Some(date_since) => {
                rebuild_cache_since_date(account, &remote_folders, date_since).await?;
            }
            None => {
                rebuild_cache(account, &remote_folders).await?;
            }
        }
        AccountRunningState::set_initial_sync_completed(account.id).await?;
        if EventHookTask::is_watching_account_first_sync_completed(account.id).await? {
            EVENT_CHANNEL
                .queue(Event::new(
                    account.id,
                    &account.email,
                    RustMailerEvent::new(
                        EventType::AccountFirstSyncCompleted,
                        EventPayload::AccountFirstSyncCompleted(AccountChange {
                            account_id: account.id,
                            account_email: account.email.clone(),
                        }),
                    ),
                ))
                .await;
        }
        return Ok(());
    }

    handle_delta(account, &local_folders, &remote_folders).await?;

    let deleted_folders = find_deleted_folders(&local_folders, &remote_folders);
    let missing_folders = find_missing_folders(&local_folders, &remote_folders);
    if !deleted_folders.is_empty() {
        info!(
            "Account {}: Detected {} mailboxes missing from the Graph API server. \
            Now cleaning up these mailboxes and their associated metadata locally.",
            account.id,
            deleted_folders.len()
        );
        cleanup_deleted_folders(account, &deleted_folders).await?;
    }

    if !missing_folders.is_empty() {
        info!(
            count = missing_folders.len(),
            labels = ?missing_folders,
            "Inserting missing folders into database"
        );
        OutlookFolder::batch_insert(&missing_folders).await?;
        for folder in &missing_folders {
            rebuild_single_folder_cache(account, folder).await?;
        }
    }
    AccountRunningState::set_incremental_sync_end(account.id).await?;
    Ok(())
}

pub async fn should_rebuild_cache(
    account: &AccountModel,
    local_folders: &[OutlookFolder],
    has_delta: bool,
) -> RustMailerResult<bool> {
    // If both local labels and checkpoint exist, no rebuild is needed.
    if !local_folders.is_empty() && has_delta {
        return Ok(false);
    }

    info!(
        account_id = account.id,
        folder_count = local_folders.len(),
        "Rebuilding cache: cleaning local folders and checkpoints"
    );

    if !local_folders.is_empty() {
        OutlookFolder::batch_delete(local_folders.to_vec()).await?;
    }
    if has_delta {
        FolderDeltaLink::clean(account.id).await?;
    }
    if !account.minimal_sync() {
        OutlookEnvelope::clean_account(account.id).await?;
        AddressEntity::clean_account(account.id).await?;
        EmailThread::clean_account(account.id).await?;
    }
    info!(account_id = account.id, "Cache cleaning completed");
    Ok(true)
}

pub fn find_deleted_folders(
    local_folders: &[OutlookFolder],
    remote_folders: &[OutlookFolder],
) -> Vec<OutlookFolder> {
    let remote_ids: AHashSet<_> = remote_folders.iter().map(|l| &l.id).collect();

    local_folders
        .iter()
        .filter(|l| !remote_ids.contains(&l.id))
        .cloned()
        .collect()
}

pub fn find_missing_folders(
    local_folders: &[OutlookFolder],
    remote_folders: &[OutlookFolder],
) -> Vec<OutlookFolder> {
    let local_ids: AHashSet<_> = local_folders.iter().map(|l| &l.id).collect();

    remote_folders
        .iter()
        .filter(|l| !local_ids.contains(&l.id))
        .cloned()
        .collect()
}

async fn cleanup_deleted_folders(
    account: &AccountModel,
    deleted_folders: &[OutlookFolder],
) -> RustMailerResult<()> {
    let start_time = Instant::now();
    if !account.minimal_sync() {
        for folder in deleted_folders {
            OutlookEnvelope::clean_folder_envelopes(account.id, folder.id).await?;
            AddressEntity::clean_mailbox_envelopes(account.id, folder.id).await?;
            EmailThread::clean_mailbox_envelopes(account.id, folder.id).await?;
        }
    }
    OutlookFolder::batch_delete(deleted_folders.to_vec()).await?;
    let elapsed_time = start_time.elapsed().as_secs();
    info!(
        "Cleanup deleted OutlookFolders completed: {} seconds elapsed.",
        elapsed_time
    );
    Ok(())
}

// async fn cleanup_single_label(
//     account: &AccountModel,
//     folder: &OutlookFolder,
// ) -> RustMailerResult<()> {
//     let start_time = Instant::now();
//     OutlookEnvelope::clean_folder_envelopes(account.id, folder.id).await?;
//     AddressEntity::clean_mailbox_envelopes(account.id, folder.id).await?;
//     EmailThread::clean_mailbox_envelopes(account.id, folder.id).await?;
//     OutlookFolder::delete(folder.id).await?;
//     let elapsed_time = start_time.elapsed().as_secs();
//     info!(
//         "Cleanup OutlookFolders completed: {} seconds elapsed.",
//         elapsed_time
//     );
//     Ok(())
// }
