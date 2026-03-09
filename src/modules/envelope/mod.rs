// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::cache::imap::sync::flow::compress_uid_list;
use crate::modules::imap::section::ImapAttachment;
use ahash::AHashSet;

pub mod detect;
pub mod extractor;

pub fn generate_uid_set(uids: Vec<u32>) -> String {
    // Insert elements into HashSet to remove duplicates
    let unique_uids: AHashSet<_> = uids.into_iter().collect();

    // Convert HashSet back to Vec, then sort
    let mut sorted_uids: Vec<u32> = unique_uids.into_iter().collect();
    sorted_uids.sort_unstable();

    compress_uid_list(sorted_uids)
}

pub struct MinimalEnvelopeMeta {
    pub size: u32,
    pub attachments: Option<Vec<ImapAttachment>>,
}
