// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use std::sync::LazyLock;

use crate::{
    calculate_hash,
    modules::{
        cache::{
            imap::{
                address::AddressEntity,
                envelope::EmailEnvelope,
                migration::{EmailEnvelopeV2, EmailEnvelopeV3},
                minimal::MinimalEnvelope,
                thread::EmailThread,
            },
            vendor::{
                gmail::sync::{
                    envelope::GmailEnvelope,
                    labels::{GmailCheckPoint, GmailLabels},
                },
                outlook::sync::{
                    delta::FolderDeltaLink, envelope::OutlookEnvelope, folders::OutlookFolder,
                },
            },
        },
        database::ModelsAdapter,
    },
};
use ahash::{AHashMap, AHashSet};
use mailbox::{EmailFlag, EnvelopeFlag, MailBox};
use native_db::Models;
pub mod address;
pub mod envelope;
pub mod mailbox;
pub mod manager;
pub mod migration;
pub mod minimal;
pub mod sync;
pub mod task;
#[cfg(test)]
mod tests;
pub mod thread;

pub static ENVELOPE_MODELS: LazyLock<Models> = LazyLock::new(|| {
    let mut adapter = ModelsAdapter::new();
    adapter.register_model::<EmailEnvelope>();
    adapter.register_model::<EmailEnvelopeV2>();
    adapter.register_model::<EmailEnvelopeV3>();
    adapter.register_model::<MailBox>();
    adapter.register_model::<MinimalEnvelope>();
    adapter.register_model::<AddressEntity>();
    adapter.register_model::<EmailThread>();
    adapter.register_model::<GmailEnvelope>();
    adapter.register_model::<GmailLabels>();
    adapter.register_model::<GmailCheckPoint>();
    adapter.register_model::<OutlookFolder>();
    adapter.register_model::<FolderDeltaLink>();
    adapter.register_model::<OutlookEnvelope>();
    adapter.models
});

// Recent flag is removed
#[inline]
pub fn flags_to_hash(flags: &[EnvelopeFlag]) -> u64 {
    assert!(
        !flags.iter().any(|f| matches!(f.flag, EmailFlag::Recent)),
        "Flags must not contain `Recent` type."
    );

    if flags.is_empty() {
        return 0;
    }

    let mut sorted_flags: Vec<String> = flags.iter().map(|f| f.to_string()).collect();
    sorted_flags.sort();
    let flags_str: String = sorted_flags.join(",");
    calculate_hash!(&flags_str)
}

pub fn find_flag_updates(
    uid_flags_hash: &AHashMap<u32, u64>,
    remote_uid_flags: Vec<(u32, (u64, Vec<EnvelopeFlag>))>,
) -> Vec<(u32, Vec<EnvelopeFlag>)> {
    let mut to_update_flags = Vec::new();
    for (uid, (new_hash, flags)) in remote_uid_flags {
        if let Some(&existing_hash) = uid_flags_hash.get(&uid) {
            // If the hash is different, mark for flag update
            if existing_hash != new_hash {
                to_update_flags.push((uid, flags));
            }
        }
    }
    to_update_flags
}

type UidFlagsHash = AHashMap<u32, u64>;
type RemoteUidFlags = Vec<(u32, (u64, Vec<EnvelopeFlag>))>;
type FlagUpdates = Vec<(u32, Vec<EnvelopeFlag>)>;
type NewUids = Vec<(u32, u64)>;
type UidSet = AHashSet<u32>;
type DiffResult = (FlagUpdates, NewUids, UidSet);

pub fn diff(uid_flags_hash: &UidFlagsHash, remote_uid_flags: RemoteUidFlags) -> DiffResult {
    let mut to_update_flags = Vec::new();
    let mut to_add = Vec::new();
    let mut all_uids = AHashSet::new();

    for (uid, (new_hash, flags)) in remote_uid_flags {
        all_uids.insert(uid); // Collect all UIDs from the current batch

        // Check if the UID exists locally
        if let Some(&existing_hash) = uid_flags_hash.get(&uid) {
            // If the hash is different, mark for flag update
            if existing_hash != new_hash {
                to_update_flags.push((uid, flags));
            }
        } else {
            // If UID is missing locally, mark for addition
            to_add.push((uid, new_hash));
        }
    }

    (to_update_flags, to_add, all_uids)
}

pub fn find_missing_remote_uids(
    uid_flags_hash: &AHashMap<u32, u64>,
    all_uids: &AHashSet<u32>,
) -> Vec<u32> {
    uid_flags_hash
        .keys()
        .filter(|uid| !all_uids.contains(uid))
        .cloned()
        .collect()
}

pub fn find_deleted_mailboxes(
    local_mailboxes: &[MailBox],
    server_mailboxes: &[MailBox],
) -> Vec<MailBox> {
    let server_names: AHashSet<_> = server_mailboxes.iter().map(|m| &m.name).collect();
    local_mailboxes
        .iter()
        .filter(|m| !server_names.contains(&m.name))
        .cloned()
        .collect()
}

pub fn find_missing_mailboxes(
    local_mailboxes: &[MailBox],
    server_mailboxes: &[MailBox],
) -> Vec<MailBox> {
    let local_names: AHashSet<_> = local_mailboxes.iter().map(|m| &m.name).collect();
    server_mailboxes
        .iter()
        .filter(|m| !local_names.contains(&m.name))
        .cloned()
        .collect()
}

pub fn find_intersecting_mailboxes(
    local_mailboxes: &[MailBox],
    remote_mailboxes: &[MailBox],
) -> Vec<(MailBox, MailBox)> {
    let local_map: AHashMap<_, _> = local_mailboxes
        .iter()
        .map(|m| (m.name.clone(), m.clone()))
        .collect();
    remote_mailboxes
        .iter()
        .filter_map(|m| {
            local_map
                .get(&m.name)
                .map(|local_mailbox| (local_mailbox.clone(), m.clone()))
        })
        .collect()
}
