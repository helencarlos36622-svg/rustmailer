// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::{
    account::{entity::MailerType, migration::AccountModel, status::AccountRunningState},
    cache::{imap::{mailbox::MailBox, manager::EnvelopeFlagsManager}, sync_type::{determine_sync_type, SyncType}},
    error::RustMailerResult,
    hook::{
        channel::{Event, EVENT_CHANNEL},
        events::{payload::AccountChange, EventPayload, EventType, RustMailerEvent},
        task::EventHookTask,
    },
};
use flow::reconcile_mailboxes;
use rebuild::{rebuild_cache, rebuild_cache_since_date, should_rebuild_cache};
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};
use sync_folders::get_sync_folders;
use tracing::{debug, info};

pub mod flow;
pub mod rebuild;
pub mod sync_folders;

static SYNC_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub async fn execute_imap_sync(account: &AccountModel) -> RustMailerResult<()> {
    assert!(
        matches!(account.mailer_type, MailerType::ImapSmtp),
        "Bug: Unexpected mailer type, expected ImapSmtp, found: {:?}",
        account.mailer_type
    );
    let start_time = Instant::now();
    let account_id = account.id;

    let sync_type = determine_sync_type(account).await?;
    if matches!(sync_type, SyncType::SkipSync) {
        return Ok(());
    }

    let remote_mailboxes = get_sync_folders(account).await?;
    let local_mailboxes = MailBox::list_all(account_id).await?;
    let mailboxes_count = local_mailboxes.len();
    let total_envelope_count = EnvelopeFlagsManager::count_account_uid_total(account_id);
    debug!(
        "Account ID: {}, fetched total count of local cached emails, elapsed time: {} seconds",
        account_id,
        start_time.elapsed().as_secs()
    );

    if should_rebuild_cache(account, mailboxes_count, total_envelope_count).await? {
        AccountRunningState::set_initial_sync_folders(
            account_id,
            remote_mailboxes.iter().map(|m| m.name.clone()).collect(),
        )
        .await?;
        match &account.date_since {
            Some(date_since) => {
                rebuild_cache_since_date(account, &remote_mailboxes, date_since).await?;
            }
            None => {
                rebuild_cache(account, &remote_mailboxes).await?;
            }
        }
        //update full sync end time here
        AccountRunningState::set_initial_sync_completed(account_id).await?;
        if EventHookTask::is_watching_account_first_sync_completed(account_id).await? {
            EVENT_CHANNEL
                .queue(Event::new(
                    account.id,
                    &account.email,
                    RustMailerEvent::new(
                        EventType::AccountFirstSyncCompleted,
                        EventPayload::AccountFirstSyncCompleted(AccountChange {
                            account_id: account_id,
                            account_email: account.email.clone(),
                        }),
                    ),
                ))
                .await;
        }

        return Ok(());
    }
    let sync_count = SYNC_COUNTER.fetch_add(1, Ordering::SeqCst);
    reconcile_mailboxes(
        account,
        &remote_mailboxes,
        &local_mailboxes,
        &sync_type,
        sync_count,
    )
    .await?;

    let elapsed_time = start_time.elapsed().as_secs();
    match sync_type {
        SyncType::FullSync => {
            info!(
                "Account{{{}}} full sync completed: {} seconds elapsed.",
                account.email, elapsed_time
            );
            AccountRunningState::set_full_sync_end(account_id).await?;
        }
        SyncType::IncrementalSync => {
            if sync_count % 10 == 0 {
                debug!(
                    "Account{{{}}} incremental sync completed: {} seconds elapsed.",
                    account.email, elapsed_time
                );
            }
            AccountRunningState::set_incremental_sync_end(account_id).await?;
        }
        SyncType::SkipSync => {
            unreachable!()
        }
    }
    Ok(())
}
