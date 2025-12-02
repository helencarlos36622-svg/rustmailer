// Copyright © 2025 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::{
    calculate_hash, id,
    modules::{
        cache::{
            imap::{
                address::AddressEntity,
                migration::EmailEnvelopeV3,
                thread::{EmailThread, EmailThreadKey},
            },
            model::Envelope,
        },
        common::Addr,
        database::{
            batch_delete_impl, delete_impl, filter_by_secondary_key_impl, manager::DB_MANAGER,
            paginate_secondary_scan_impl, secondary_find_impl, upsert_impl, with_transaction,
        },
        error::{code::ErrorCode, RustMailerResult},
        rest::response::DataPage,
        utils::envelope_hash_from_id,
    },
    raise_error,
};
use ahash::AHashMap;
use itertools::Itertools;
use native_db::*;
use native_model::{native_model, Model};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::info;

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
#[native_model(id = 6, version = 1)]
#[native_db(primary_key(pk -> String), secondary_key(create_envelope_id -> u64, unique))]
pub struct GmailEnvelope {
    /// The ID of the account owning this email within RustMailer.
    /// This corresponds to the local RustMailer account ID, not the Gmail account itself.
    #[secondary_key]
    pub account_id: u64,
    /// The unique internal identifier of the label within RustMailer’s local cache
    /// where the email is associated.
    ///
    /// This is **not** the Gmail label ID; it only references the label stored locally.
    #[secondary_key]
    pub label_id: u64,
    /// This is the human-readable label used in Gmail to categorize emails.
    pub label_name: String,
    /// The Gmail message ID as returned by the `messages.list` or `messages.get` API.
    /// This ID uniquely identifies the email within the account and mailbox.
    pub id: String,
    /// The date and time when Gmail received the email, as a Unix timestamp in milliseconds.
    /// Corresponds to the API field `internalDate`. May be `None` if unavailable.
    pub internal_date: i64,
    /// The size of the email in bytes. Corresponds to the `sizeEstimate` from the API.
    pub size: u32,
    /// Blind carbon copy (BCC) recipient(s), if any. Each `Addr` contains name and email.
    pub bcc: Option<Vec<Addr>>,
    /// Carbon copy (CC) recipient(s), if any. Each `Addr` contains name and email.
    pub cc: Option<Vec<Addr>>,
    /// The date the email was sent, as a Unix timestamp in milliseconds.
    /// Extracted from the `Date` header if present. May be `None` if the header is missing or unparseable.
    pub date: Option<i64>,
    /// The sender's address, as specified in the `From` header.
    pub from: Option<Addr>,
    /// The message ID of the email to which this email is a reply, if applicable.
    /// Corresponds to the `In-Reply-To` header.
    pub in_reply_to: Option<String>,
    /// The actual sender's address, if different from the `From` field.
    /// Extracted from the `Sender` header, if present.
    pub sender: Option<Addr>,
    /// The globally unique message ID of the email.
    /// Corresponds to the `Message-ID` header. Useful for threading and deduplication.
    pub message_id: Option<String>,
    /// The subject of the email, if present.
    pub subject: Option<String>,
    /// The identifier of the thread this email belongs to.
    /// Derived from `in_reply_to`, `references`, or `message_id`.
    #[secondary_key]
    pub thread_id: u64,
    /// The MIME version of the email (e.g., "1.0"), if specified.
    /// Corresponds to the `Mime-Version` header.
    pub mime_version: Option<String>,
    /// List of message IDs referenced by this email, used for threading.
    /// Corresponds to the `References` header.
    pub references: Option<Vec<String>>,
    /// The address(es) to which replies should be sent, if specified.
    /// Corresponds to the `Reply-To` header.
    pub reply_to: Option<Vec<Addr>>,
    /// Primary recipient(s) of the email, corresponding to the `To` header.
    pub to: Option<Vec<Addr>>,
    /// A short snippet (preview) of the email body.
    /// Corresponds to the API `snippet` field. Typically the first few hundred characters of the message body.
    pub snippet: Option<String>,
    /// The Gmail history ID associated with this email.
    /// Useful for incremental synchronization via `history.list`.
    pub history_id: String,
    /// The Gmail API thread ID associated with this email.
    pub gmail_thread_id: String,
    /// A list of labels applied to the message.
    ///
    /// Each element is a string representing a Gmail label ID (e.g., "INBOX", "UNREAD").
    /// This field reflects the current labels associated with the email.
    pub label_ids: Vec<String>,
}

impl GmailEnvelope {
    pub fn pk(&self) -> String {
        format!(
            "{}_{}",
            self.internal_date,
            envelope_hash_from_id(self.account_id, self.label_id, &self.id)
        )
    }

    pub fn create_envelope_id(&self) -> u64 {
        envelope_hash_from_id(self.account_id, self.label_id, &self.id)
    }

    pub fn compute_thread_id(&self) -> u64 {
        if self.in_reply_to.is_some() && self.references.as_ref().map_or(false, |r| !r.is_empty()) {
            return calculate_hash!(&self.references.as_ref().unwrap()[0]);
        }
        if let Some(message_id) = self.message_id.as_ref() {
            return calculate_hash!(message_id);
        }
        id!(128)
    }

    pub async fn delete(account_id: u64, label_id: u64, mid: &str) -> RustMailerResult<()> {
        let mid = mid.to_string();
        delete_impl(DB_MANAGER.envelope_db(), move |rw| {
            rw.get()
                .secondary::<GmailEnvelope>(
                    GmailEnvelopeKey::create_envelope_id,
                    envelope_hash_from_id(account_id, label_id, &mid),
                )
                .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                .ok_or_else(|| {
                    raise_error!("gmail envelope missing".into(), ErrorCode::InternalError)
                })
        })
        .await
    }

    pub async fn find(
        account_id: u64,
        label_id: u64,
        mid: &str,
    ) -> RustMailerResult<Option<GmailEnvelope>> {
        secondary_find_impl(
            DB_MANAGER.envelope_db(),
            GmailEnvelopeKey::create_envelope_id,
            envelope_hash_from_id(account_id, label_id, mid),
        )
        .await
    }

    pub async fn get(envelope_id: u64) -> RustMailerResult<Option<GmailEnvelope>> {
        secondary_find_impl(
            DB_MANAGER.envelope_db(),
            GmailEnvelopeKey::create_envelope_id,
            envelope_id,
        )
        .await
    }

    pub async fn get_thread(
        account_id: u64,
        thread_id: u64,
    ) -> RustMailerResult<Vec<GmailEnvelope>> {
        let envelopes = filter_by_secondary_key_impl::<GmailEnvelope>(
            DB_MANAGER.envelope_db(),
            GmailEnvelopeKey::thread_id,
            thread_id,
        )
        .await?;

        let mut result = Vec::with_capacity(envelopes.len());
        for e in envelopes {
            if e.account_id == account_id {
                result.push(e);
            }
        }
        // Sort by internal_date in descending order
        result.sort_by(|a, b| b.internal_date.cmp(&a.internal_date));

        Ok(result)
    }

    pub async fn list_messages_in_label(
        label_id: u64,
        page: u64,
        page_size: u64,
        desc: bool,
    ) -> RustMailerResult<DataPage<GmailEnvelope>> {
        paginate_secondary_scan_impl(
            DB_MANAGER.envelope_db(),
            Some(page),
            Some(page_size),
            Some(desc),
            GmailEnvelopeKey::label_id,
            label_id,
        )
        .await
        .map(DataPage::from)
    }

    pub async fn upsert(envelope: GmailEnvelope) -> RustMailerResult<()> {
        upsert_impl(DB_MANAGER.envelope_db(), envelope).await
    }

    pub async fn save_envelopes(envelopes: Vec<GmailEnvelope>) -> RustMailerResult<()> {
        with_transaction(DB_MANAGER.envelope_db(), move |rw| {
            for mut e in envelopes {
                e.thread_id = e.compute_thread_id();
                let envelope_id = e.create_envelope_id();
                // Idempotent write
                if rw
                    .get()
                    .secondary::<GmailEnvelope>(GmailEnvelopeKey::create_envelope_id, envelope_id)
                    .map_err(|err| raise_error!(format!("{:#?}", err), ErrorCode::InternalError))?
                    .is_none()
                {
                    rw.insert::<GmailEnvelope>(e.clone()).map_err(|err| {
                        raise_error!(format!("{:#?}", err), ErrorCode::InternalError)
                    })?;

                    let address_entities = AddressEntity::extract2(&e);
                    let thread = EmailThread::new(
                        e.thread_id,
                        envelope_id,
                        e.account_id,
                        e.label_id,
                        Some(e.internal_date),
                        e.date,
                    );
                    // --- Thread upsert ---
                    match rw
                        .get()
                        .secondary::<EmailThread>(EmailThreadKey::thread_id, thread.thread_id)
                        .map_err(|err| {
                            raise_error!(format!("{:#?}", err), ErrorCode::InternalError)
                        })? {
                        Some(current) => {
                            // Only replace if current.internal_date is older than new internal_date
                            if current.need_update(&thread) {
                                rw.remove(current).map_err(|err| {
                                    raise_error!(format!("{:#?}", err), ErrorCode::InternalError)
                                })?;
                                rw.insert::<EmailThread>(thread).map_err(|err| {
                                    raise_error!(format!("{:#?}", err), ErrorCode::InternalError)
                                })?;
                            }
                        }
                        None => {
                            rw.insert::<EmailThread>(thread).map_err(|err| {
                                raise_error!(format!("{:#?}", err), ErrorCode::InternalError)
                            })?;
                        }
                    }

                    // --- Store address entities ---
                    for addr in address_entities {
                        rw.insert::<AddressEntity>(addr).map_err(|err| {
                            raise_error!(format!("{:#?}", err), ErrorCode::InternalError)
                        })?;
                    }
                }
            }
            Ok(())
        })
        .await
    }

    pub fn clean_angle_brackets(s: &str) -> &str {
        s.trim().trim_matches(|c| c == '<' || c == '>')
    }

    pub fn parse_addr_list(s: &str) -> Vec<Addr> {
        s.split(',')
            .map(|part| part.trim())
            .filter(|part| !part.is_empty())
            .map(Addr::parse)
            .collect()
    }

    pub async fn clean_label_envelopes(account_id: u64, label_id: u64) -> RustMailerResult<()> {
        const BATCH_SIZE: usize = 200;
        let mut total_deleted = 0usize;
        let start_time = Instant::now();
        loop {
            let deleted = batch_delete_impl(DB_MANAGER.envelope_db(), move |rw| {
                let to_delete: Vec<GmailEnvelope> = rw
                    .scan()
                    .secondary(GmailEnvelopeKey::label_id)
                    .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                    .start_with(label_id)
                    .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                    .filter_map(Result::ok) // filter only Ok values
                    .filter(|e: &GmailEnvelope| e.account_id == account_id)
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
            "Finished deleting gmail envelopes for label_id={} account_id={} total_deleted={} in {:?}",
            label_id,
            account_id,
            total_deleted,
            start_time.elapsed()
        );
        Ok(())
    }

    pub async fn clean_account(account_id: u64) -> RustMailerResult<()> {
        const BATCH_SIZE: usize = 200;
        let mut total_deleted = 0usize;
        let start_time = Instant::now();
        loop {
            let deleted = batch_delete_impl(DB_MANAGER.envelope_db(), move |rw| {
                let to_delete: Vec<GmailEnvelope> = rw
                    .scan()
                    .secondary(GmailEnvelopeKey::account_id)
                    .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                    .start_with(account_id)
                    .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                    .take(BATCH_SIZE)
                    .try_collect()
                    .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?;
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
            "Finished deleting gmail envelopes for account_id={} total_deleted={} in {:?}",
            account_id,
            total_deleted,
            start_time.elapsed()
        );
        Ok(())
    }

    pub fn into_v3(self, label_map: &AHashMap<String, String>) -> EmailEnvelopeV3 {
        let labels: Vec<String> = self
            .label_ids
            .into_iter()
            .filter_map(|id| label_map.get(&id).cloned())
            .collect();

        EmailEnvelopeV3 {
            account_id: self.account_id,
            mailbox_id: self.label_id,
            mailbox_name: self.label_name,
            uid: 0,
            internal_date: Some(self.internal_date),
            size: self.size,
            flags: vec![],
            flags_hash: 0,
            bcc: self.bcc,
            cc: self.cc,
            date: self.date,
            from: self.from,
            in_reply_to: self.in_reply_to,
            sender: self.sender,
            return_address: None,
            message_id: self.message_id,
            subject: self.subject,
            thread_name: None,
            thread_id: self.thread_id,
            mime_version: self.mime_version,
            references: self.references,
            reply_to: self.reply_to,
            to: self.to,
            attachments: None,
            body_meta: None,
            received: None,
            mid: Some(self.id),
            labels,
        }
    }

    pub fn into_envelope(self, label_map: &AHashMap<String, String>) -> Envelope {
        let is_read = self.label_ids.iter().any(|f| f == "UNREAD");

        let labels: Vec<String> = self
            .label_ids
            .into_iter()
            .filter_map(|id| label_map.get(&id).cloned())
            .collect();

        Envelope {
            id: self.id,
            account_id: self.account_id,
            mailbox_id: self.label_id,
            mailbox_name: self.label_name,
            internal_date: Some(self.internal_date),
            size: self.size,
            flags: None,
            flags_hash: None,
            bcc: self.bcc,
            cc: self.cc,
            date: self.date,
            from: self.from,
            in_reply_to: self.in_reply_to,
            sender: self.sender,
            return_address: None,
            message_id: self.message_id,
            subject: self.subject,
            thread_name: None,
            thread_id: self.gmail_thread_id,
            mime_version: self.mime_version,
            references: self.references,
            reply_to: self.reply_to,
            to: self.to,
            attachments: None,
            body_meta: None,
            received: None,
            is_read,
            labels,
        }
    }
}
