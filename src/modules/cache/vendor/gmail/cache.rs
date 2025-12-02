// Copyright © 2025 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use std::sync::LazyLock;
use std::time::Duration;

use ahash::AHashMap;

use crate::modules::cache::vendor::gmail::model::thread::ThreadMessages;
use crate::modules::common::lru::TimedLruCache;

pub static GMAIL_LABELS_CACHE: LazyLock<TimedLruCache<u64, AHashMap<String, String>>> =
    LazyLock::new(|| TimedLruCache::new(100, Duration::from_secs(60)));

// Gmail thread message cache.
// Key format: "{account_id}:{thread_id}"
// Stores the full message list of a thread to avoid repeated Gmail API calls.
pub static GMAIL_THREADS_CACHE: LazyLock<TimedLruCache<String, ThreadMessages>> =
    LazyLock::new(|| TimedLruCache::new(2000, Duration::from_secs(180)));

pub fn thread_cache_key(account_id: u64, thread_id: &str) -> String {
    format!("{}:{}", account_id, thread_id)
}
