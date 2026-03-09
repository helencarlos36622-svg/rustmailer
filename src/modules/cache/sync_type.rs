// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::{
    modules::{
        account::{entity::MailerType, migration::AccountModel, status::AccountRunningState},
        error::RustMailerResult,
    },
    utc_now,
};

/// Default interval (in minutes) for full synchronization.
pub const DEFAULT_FULL_SYNC_INTERVAL_MIN: i64 = 30;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SyncType {
    /// Full synchronization, typically used for the first sync or after major changes.
    FullSync,
    /// Incremental synchronization, typically used for updates or fetching new data since the last sync.
    IncrementalSync,

    SkipSync,
}

pub async fn determine_sync_type(account: &AccountModel) -> RustMailerResult<SyncType> {
    Ok(match AccountRunningState::get(account.id).await? {
        Some(info) => {
            let now = utc_now!();
            let incremental_sync = is_time_for_incremental_sync(
                now,
                info.last_incremental_sync_start,
                account.incremental_sync_interval_sec,
            );

            match account.mailer_type {
                MailerType::ImapSmtp => {
                    if is_time_for_full_sync(
                        now,
                        info.last_full_sync_start,
                        account
                            .full_sync_interval_min
                            .unwrap_or(DEFAULT_FULL_SYNC_INTERVAL_MIN),
                    ) {
                        AccountRunningState::set_full_sync_start(account.id).await?;
                        SyncType::FullSync
                    } else if incremental_sync {
                        AccountRunningState::set_incremental_sync_start(account.id).await?;
                        SyncType::IncrementalSync
                    } else {
                        SyncType::SkipSync
                    }
                }
                MailerType::GmailApi | MailerType::GraphApi => {
                    if incremental_sync {
                        AccountRunningState::set_incremental_sync_start(account.id).await?;
                        SyncType::IncrementalSync
                    } else {
                        SyncType::SkipSync
                    }
                }
            }
        }
        None => {
            AccountRunningState::add(account.id).await?;
            SyncType::FullSync
        }
    })
}

/// Check if it's time for a full sync based on the provided interval.
fn is_time_for_full_sync(now: i64, last_full_sync_at: i64, full_sync_interval_min: i64) -> bool {
    now - last_full_sync_at > (full_sync_interval_min * 60 * 1000)
}

/// Check if it's time for an incremental sync based on the provided interval.
fn is_time_for_incremental_sync(
    now: i64,
    last_incremental_sync_at: i64,
    incremental_sync_interval_sec: i64,
) -> bool {
    now - last_incremental_sync_at > (incremental_sync_interval_sec * 1000)
}
