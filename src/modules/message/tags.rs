// Copyright © 2025 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use std::collections::{HashMap, HashSet};

use poem_openapi::{Enum, Object};
use serde::{Deserialize, Serialize};

use crate::{
    modules::{
        account::{entity::MailerType, migration::AccountModel},
        cache::{
            imap::mailbox::{EmailFlag, EnvelopeFlag},
            vendor::{
                gmail::sync::client::GmailClient,
                outlook::sync::client::{MessageCategoryUpdate, OutlookClient},
            },
        },
        context::executors::RUST_MAIL_CONTEXT,
        envelope::generate_uid_set,
        error::{code::ErrorCode, RustMailerResult},
        mailbox::create::CreateMailboxRequest,
    },
    raise_error,
};

const MAX_MESSAGE_IDS: usize = 50;

/// Defines the type of operation to be performed on a tag/category.
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Enum)]
pub enum TagAction {
    /// Adds one or more tags to the specified messages.
    #[default]
    Add,
    /// Removes one or more tags from the specified messages.
    Remove,
    /// Sets/overwrites the entire list of tags on the specified messages.
    /// (Note: This might require fetching the existing tags first for Graph API Add/Remove logic).
    Set,
}

/// The unified request payload for batch tagging operations across different email APIs (e.g., Gmail, Graph).
/// This structure abstracts the intention of modifying tags on a batch of messages.
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
pub struct BatchTagRequest {
    /// Required: A list of unique identifiers (Message IDs) for the emails to be operated on.
    pub message_ids: Vec<String>,

    /// Required: The list of tags (which could be Label IDs for Gmail or Category Names for Graph API)
    /// to be added, removed, or set.
    pub tags: Vec<String>,

    /// Required: The action to be performed on the 'tags' list.
    pub action: TagAction,

    /// Required for IMAP operations to specify the mailbox context where the message UIDs are valid.
    /// Example: "INBOX", "Sent Items", "Project X/Subfolder"
    pub mailbox_name: Option<String>,

    /// Optional: **Used only in the Gmail API scenario.**
    /// Specifies whether a tag/Label should be automatically created if it does not exist
    /// when referenced in the request.
    /// - If set to 'None' or 'false', an error will be returned if the tag is not found.
    /// - **This field will be ignored in other MailerTypes (e.g., IMAP).**
    pub auto_create_tags: Option<bool>,
}

impl BatchTagRequest {
    pub fn validate(&self, account: &AccountModel) -> RustMailerResult<()> {
        if self.tags.is_empty() {
            return Err(raise_error!(
                "The 'tags' list cannot be empty. At least one tag must be specified.".into(),
                ErrorCode::InvalidParameter
            ));
        }

        if self.message_ids.is_empty() {
            return Err(raise_error!(
                "The 'message_ids' list must contain at least one message ID.".into(),
                ErrorCode::InvalidParameter
            ));
        }

        if self.message_ids.len() > MAX_MESSAGE_IDS {
            return Err(raise_error!(
                format!(
                    "The 'message_ids' list is too long (Max {} IDs allowed for batch operations).",
                    MAX_MESSAGE_IDS
                ),
                ErrorCode::InvalidParameter
            ));
        }

        if matches!(account.mailer_type, MailerType::ImapSmtp) {
            if self.mailbox_name.is_none() {
                return Err(raise_error!(
                    "The 'mailbox_name' field is required for IMAP/SMTP accounts to specify the folder context for message UIDs.".into(),
                    ErrorCode::InvalidParameter
                ));
            }
            for mid in &self.message_ids {
                if mid.parse::<u32>().is_err() {
                    return Err(raise_error!(
                        format!(
                            "IMAP message IDs must be valid unsigned 32-bit integers (UIDs). Found invalid ID: '{}'",
                            mid
                        ),
                        ErrorCode::InvalidParameter
                    ));
                }
            }
        }

        Ok(())
    }
}

pub async fn tag_messages_impl(account_id: u64, payload: BatchTagRequest) -> RustMailerResult<()> {
    let account = AccountModel::check_account_active(account_id, false).await?;
    let _ = &payload.validate(&account)?;

    match account.mailer_type {
        MailerType::ImapSmtp => {
            let flags: Vec<EnvelopeFlag> = payload
                .tags
                .clone()
                .into_iter()
                .map(|tag| EnvelopeFlag {
                    flag: EmailFlag::Custom,
                    custom: Some(tag),
                })
                .collect();
            let executor = RUST_MAIL_CONTEXT.imap(account_id).await?;

            let uids: Vec<u32> = payload
                .message_ids
                .iter()
                .map(|mid_str| {
                    mid_str
                        .parse::<u32>()
                        .expect("IMAP message ID failed u32 parse after validation.")
                })
                .collect();
            let uid_set = generate_uid_set(uids);

            let mut add_flags: Option<Vec<EnvelopeFlag>> = None;
            let mut remove_flags: Option<Vec<EnvelopeFlag>> = None;
            let mut overwrite_flags: Option<Vec<EnvelopeFlag>> = None;

            match payload.action {
                TagAction::Add => {
                    add_flags = Some(flags);
                }
                TagAction::Remove => {
                    remove_flags = Some(flags);
                }
                TagAction::Set => {
                    overwrite_flags = Some(flags);
                }
            }

            executor
                .uid_set_flags(
                    &uid_set,
                    &payload.mailbox_name.clone().unwrap(),
                    add_flags,
                    remove_flags,
                    overwrite_flags,
                )
                .await?;
        }
        MailerType::GmailApi => {
            let labels_map =
                GmailClient::for_lookup_label_id(account_id, account.use_proxy, true).await?;
            let tags_to_process = &payload.tags;
            let mut target_label_ids: Vec<String> = Vec::with_capacity(tags_to_process.len());
            for tag_name in tags_to_process {
                match labels_map.get(tag_name) {
                    Some(label_id) => {
                        target_label_ids.push(label_id.clone());
                    }
                    None => {
                        if let Some(true) = payload.auto_create_tags {
                            let label = GmailClient::create_label(
                                account_id,
                                account.use_proxy,
                                &CreateMailboxRequest {
                                    mailbox_name: tag_name.to_string(),
                                    parent_name: None,
                                    label_color: None,
                                },
                            )
                            .await?;
                            target_label_ids.push(label.id);
                        } else {
                            return Err(raise_error!(
                               format!(
                                    "Tag/Label name `{}` not found in Gmail labels map. \
                                    If you intend to create this label automatically, please ensure the `auto_create_tags` parameter is set to true.",
                                    tag_name
                                ),
                                ErrorCode::InvalidParameter
                            ));
                        }
                    }
                }
            }

            let mut add_ids: Vec<String> = Vec::new();
            let mut remove_ids: Vec<String> = Vec::new();
            match payload.action {
                TagAction::Add => {
                    add_ids = target_label_ids;
                }
                TagAction::Remove => {
                    remove_ids = target_label_ids;
                }
                TagAction::Set => {
                    let to_remove_labels: Vec<String> =
                        GmailClient::list_labels(account_id, account.use_proxy)
                            .await?
                            .into_iter()
                            .filter(|label| {
                                label.label_type == "user" && !tags_to_process.contains(&label.name)
                            })
                            .map(|label| label.id)
                            .collect();

                    remove_ids = to_remove_labels;
                    add_ids = target_label_ids;
                }
            }
            GmailClient::batch_modify(
                account_id,
                account.use_proxy,
                &payload.message_ids,
                add_ids,
                remove_ids,
            )
            .await?;
        }
        MailerType::GraphApi => {
            let tags_to_operate: HashSet<&String> = payload.tags.iter().collect();

            let existing_categories_map: HashMap<String, Vec<String>> = match payload.action {
                TagAction::Set => HashMap::new(),
                _ => {
                    OutlookClient::batch_get_categories(
                        account_id,
                        account.use_proxy,
                        &payload.message_ids,
                    )
                    .await?
                }
            };
            let mut update_instructions: Vec<MessageCategoryUpdate> = Vec::new();
            match payload.action {
                TagAction::Add => {
                    for mid in &payload.message_ids {
                        let current_cats = existing_categories_map
                            .get(mid)
                            .cloned()
                            .unwrap_or_default();

                        let mut new_categories_set: HashSet<String> =
                            current_cats.into_iter().collect();
                        for tag in &payload.tags {
                            new_categories_set.insert(tag.clone());
                        }

                        update_instructions.push(MessageCategoryUpdate {
                            mid: mid.clone(),
                            categories: new_categories_set.into_iter().collect(),
                        });
                    }
                }
                TagAction::Remove => {
                    for mid in &payload.message_ids {
                        let current_cats = existing_categories_map
                            .get(mid)
                            .cloned()
                            .unwrap_or_default();

                        let mut new_categories: Vec<String> = Vec::new();
                        for cat in current_cats {
                            if !tags_to_operate.contains(&cat) {
                                new_categories.push(cat);
                            }
                        }
                        update_instructions.push(MessageCategoryUpdate {
                            mid: mid.clone(),
                            categories: new_categories,
                        });
                    }
                }
                TagAction::Set => {
                    for mid in &payload.message_ids {
                        update_instructions.push(MessageCategoryUpdate {
                            mid: mid.clone(),
                            categories: payload.tags.clone(),
                        });
                    }
                }
            }
            OutlookClient::batch_modify_categories(
                account_id,
                account.use_proxy,
                &update_instructions,
            )
            .await?;
        }
    }

    Ok(())
}
