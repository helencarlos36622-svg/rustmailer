// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use tracing::{debug, warn};

use crate::{
    modules::{
        account::migration::AccountModel,
        cache::{
            imap::sync::sync_folders::detect_mailbox_changes,
            vendor::outlook::{model::MailFolder, sync::client::OutlookClient},
        },
        error::{code::ErrorCode, RustMailerResult},
    },
    raise_error,
};

pub async fn get_sync_folders(account: &AccountModel) -> RustMailerResult<Vec<MailFolder>> {
    let all_mail_folders = OutlookClient::list_mailfolders(account.id, account.use_proxy).await?;
    debug!(
        "Account {}: Retrieved {} visible folders from Graph API: {:?}",
        account.id,
        all_mail_folders.len(),
        all_mail_folders
            .iter()
            .map(|l| &l.display_name)
            .collect::<Vec<_>>()
    );
    if all_mail_folders.is_empty() {
        warn!(
            "Account {}: No mailfolders returned from Graph API.",
            account.id
        );
        return Err(
            raise_error!(
                format!(
                    "No mailfolders returned from Graph API for account {}. This is unexpected and may indicate an issue with the Graph API or data.",
                    account.id
                ),
                ErrorCode::InternalError
            )
        );
    }

    detect_mailbox_changes(
        account,
        all_mail_folders
            .iter()
            .map(|f| f.display_name.clone())
            .collect(),
    )
    .await?;

    let subscribed = &account.sync_folders;
    debug!(
        "Account {}: Current subscribed sync folders: {:?}",
        account.id, subscribed
    );
    // Filter folders according to the subscription list; matched_folders will not include any folders outside of it.
    let mut matched_folders: Vec<MailFolder> = if !subscribed.is_empty() {
        all_mail_folders
            .clone()
            .into_iter()
            .filter(|f| subscribed.contains(&f.id))
            .collect()
    } else {
        Vec::new()
    };
    debug!(
        "Account {}: Matched folders after subscription filter: {:?}",
        account.id,
        matched_folders
            .iter()
            .map(|l| &l.display_name)
            .collect::<Vec<_>>()
    );
    // If there are no subscriptions, default to the two special folders: inbox and sentitems
    if matched_folders.is_empty() {
        matched_folders = all_mail_folders
            .into_iter()
            .filter(|folder| folder.display_name == "INBOX" || folder.display_name == "SENTITEMS")
            .collect();

        debug!(
            "Account {}: Matched folders after default inbox/sentitems filter: {:?}",
            account.id,
            matched_folders
                .iter()
                .map(|f| &f.display_name)
                .collect::<Vec<_>>()
        );
        if !matched_folders.is_empty() {
            let sync_folders: Vec<String> = matched_folders.iter().map(|n| n.id.clone()).collect();
            AccountModel::update_sync_folders(account.id, sync_folders).await?;
        } else {
            warn!("Account {}: No mailfolder found from Graph API. This is unexpected — Graph API should at least provide inbox/sentitems.", account.id);
            return Err(
                raise_error!(
                    format!("No mailfolder found for account {} via Graph API. This is unexpected — Graph API should at least provide inbox/sentitems.", &account.id), 
                    ErrorCode::InternalError
                )
            );
        }
    }
    Ok(matched_folders)
}
