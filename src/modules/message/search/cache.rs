// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use std::sync::LazyLock;
use std::time::Duration;

use crate::modules::common::lru::TimedLruCache;

pub static IMAP_SEARCH_CACHE: LazyLock<TimedLruCache<String, (Vec<String>, u64)>> =
    LazyLock::new(|| TimedLruCache::new(100, Duration::from_secs(120)));
