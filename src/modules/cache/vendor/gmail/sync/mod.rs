// Copyright © 2025 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

pub mod client;
pub mod envelope;
pub mod flow;
pub mod history;
pub mod labels;
pub mod rebuild;
pub mod sync_labels;

use std::time::Instant;

use ahash::AHashSet;
use tracing::info;

use crate::modules::{
    account::{entity::MailerType, migration::AccountModel, status::AccountRunningState},
    cache::{
        imap::{address::AddressEntity, thread::EmailThread},
        sync_type::{determine_sync_type, SyncType},
        vendor::gmail::sync::{
            client::GmailClient,
            envelope::GmailEnvelope,
            history::handle_history,
            labels::{GmailCheckPoint, GmailLabels},
            rebuild::{rebuild_cache, rebuild_cache_since_date, rebuild_single_label_cache},
            sync_labels::get_sync_labels,
        },
    },
    error::RustMailerResult,
    hook::{
        channel::{Event, EVENT_CHANNEL},
        events::{payload::AccountChange, EventPayload, EventType, RustMailerEvent},
        task::EventHookTask,
    },
    utils::mailbox_id,
};

pub async fn execute_gmail_sync(account: &AccountModel) -> RustMailerResult<()> {
    assert!(
        matches!(account.mailer_type, MailerType::GmailApi),
        "Bug: Unexpected mailer type, expected GmailApi, found: {:?}",
        account.mailer_type
    );

    let sync_type = determine_sync_type(account).await?;
    if matches!(sync_type, SyncType::SkipSync) {
        return Ok(());
    }

    let remote_labels = get_sync_labels(account).await?;
    let remote_labels: Vec<GmailLabels> = remote_labels
        .into_iter()
        .map(|label| {
            let mut label: GmailLabels = label.into();
            label.account_id = account.id;
            label.id = mailbox_id(account.id, &label.label_id);
            label
        })
        .collect();

    let local_labels = GmailLabels::list_all(account.id).await?;

    let checkpoint = GmailCheckPoint::find(account.id).await?;
    if should_rebuild_cache(account, &local_labels, checkpoint).await? {
        AccountRunningState::set_initial_sync_folders(
            account.id,
            remote_labels
                .iter()
                .map(|label| label.name.clone())
                .collect(),
        )
        .await?;
        match &account.date_since {
            Some(date_since) => {
                rebuild_cache_since_date(account, &remote_labels, date_since).await?;
            }
            None => {
                rebuild_cache(account, &remote_labels).await?;
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

    handle_history(account, &local_labels, &remote_labels).await?;

    let deleted_labels = find_deleted_labels(&local_labels, &remote_labels);
    let missing_labels = find_missing_labels(&local_labels, &remote_labels);

    if !deleted_labels.is_empty() {
        info!(
            "Account {}: Detected {} mailboxes missing from the Gmail server (not found in the Gmail API response). \
            Now cleaning up these mailboxes and their associated metadata locally.",
            account.id, deleted_labels.len()
        );
        cleanup_deleted_labels(account, &deleted_labels).await?;
    }

    if !missing_labels.is_empty() {
        info!(
            count = missing_labels.len(),
            labels = ?missing_labels,
            "Inserting missing Gmail labels into database"
        );
        GmailLabels::batch_insert(&missing_labels).await?;
        if !account.minimal_sync() {
            for label in &missing_labels {
                //During incremental synchronization, if any labels are found missing or not fully synchronized, the checkpoint does not need to be updated.
                rebuild_single_label_cache(account, label).await?;
            }
        }
    }
    AccountRunningState::set_incremental_sync_end(account.id).await?;
    Ok(())
}

pub async fn should_rebuild_cache(
    account: &AccountModel,
    local_labels: &[GmailLabels],
    checkpoint: Option<GmailCheckPoint>,
) -> RustMailerResult<bool> {
    // If both local labels and checkpoint exist, no rebuild is needed.
    if !local_labels.is_empty() && checkpoint.is_some() {
        return Ok(false);
    }

    info!(
        account_id = account.id,
        label_count = local_labels.len(),
        "Rebuilding cache: cleaning local labels and checkpoints"
    );

    if !local_labels.is_empty() {
        GmailLabels::batch_delete(local_labels.to_vec()).await?;
    }
    if checkpoint.is_some() {
        GmailCheckPoint::clean(account.id).await?;
    }

    if !account.minimal_sync() {
        GmailEnvelope::clean_account(account.id).await?;
        AddressEntity::clean_account(account.id).await?;
        EmailThread::clean_account(account.id).await?;
    }

    info!(account_id = account.id, "Cache cleaning completed");

    Ok(true)
}

pub fn find_deleted_labels(
    local_labels: &[GmailLabels],
    remote_labels: &[GmailLabels],
) -> Vec<GmailLabels> {
    let remote_ids: AHashSet<_> = remote_labels.iter().map(|l| &l.id).collect();

    local_labels
        .iter()
        .filter(|l| !remote_ids.contains(&l.id))
        .cloned()
        .collect()
}

pub fn find_missing_labels(
    local_labels: &[GmailLabels],
    remote_labels: &[GmailLabels],
) -> Vec<GmailLabels> {
    let local_ids: AHashSet<_> = local_labels.iter().map(|l| &l.id).collect();

    remote_labels
        .iter()
        .filter(|l| !local_ids.contains(&l.id))
        .cloned()
        .collect()
}

async fn cleanup_deleted_labels(
    account: &AccountModel,
    deleted_labels: &[GmailLabels],
) -> RustMailerResult<()> {
    let start_time = Instant::now();

    if !account.minimal_sync() {
        for label in deleted_labels {
            GmailEnvelope::clean_label_envelopes(account.id, label.id).await?;
            AddressEntity::clean_mailbox_envelopes(account.id, label.id).await?;
            EmailThread::clean_mailbox_envelopes(account.id, label.id).await?;
        }
    }
    GmailLabels::batch_delete(deleted_labels.to_vec()).await?;
    let elapsed_time = start_time.elapsed().as_secs();
    info!(
        "Cleanup deleted GmailLabels completed: {} seconds elapsed.",
        elapsed_time
    );
    Ok(())
}

async fn cleanup_single_label(account: &AccountModel, label: &GmailLabels) -> RustMailerResult<()> {
    let start_time = Instant::now();
    if !account.minimal_sync() {
        GmailEnvelope::clean_label_envelopes(account.id, label.id).await?;
        AddressEntity::clean_mailbox_envelopes(account.id, label.id).await?;
        EmailThread::clean_mailbox_envelopes(account.id, label.id).await?;
    }
    GmailLabels::delete(label.id).await?;
    let elapsed_time = start_time.elapsed().as_secs();
    info!(
        "Cleanup GmailLabels completed: {} seconds elapsed.",
        elapsed_time
    );
    Ok(())
}

async fn save_latest_history_id(account: &AccountModel) -> RustMailerResult<()> {
    // Fetch the most recent message
    let result =
        GmailClient::list_messages(account.id, account.use_proxy, None, None, None, 1).await?;
    let first_message = match result.messages.as_ref().and_then(|m| m.first()) {
        Some(msg) => msg,
        None => return Ok(()),
    };

    // Retrieve the message details to get its history_id
    let message =
        GmailClient::get_message(account.id, account.use_proxy, &first_message.id).await?;
    let start_history_id = &message.history_id;

    // Fetch the latest history starting from the message's history_id
    let history_list = GmailClient::list_history(
        account.id,
        account.use_proxy,
        None,
        start_history_id,
        None,
        1,
    )
    .await?;

    // Save the latest history_id to ensure future incremental sync starts from the correct point
    GmailCheckPoint::new(account.id, history_list.history_id)
        .save()
        .await
}
