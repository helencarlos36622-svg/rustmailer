// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Clone)]
struct CacheEntry<T> {
    data: Arc<T>,
    created_at: Instant,
}

pub struct TimedLruCache<K, V> {
    store: Arc<RwLock<LruCache<K, CacheEntry<V>>>>,
    ttl: Duration,
}

impl<K, V> TimedLruCache<K, V>
where
    K: std::hash::Hash + Eq + Clone,
{
    /// Create a new cache with capacity and TTL.
    pub fn new(capacity: usize, ttl: Duration) -> Self {
        let store = Arc::new(RwLock::new(LruCache::new(
            NonZeroUsize::new(capacity).unwrap(),
        )));
        TimedLruCache { store, ttl }
    }

    /// Get a value from cache if not expired.
    pub async fn get(&self, key: &K) -> Option<Arc<V>> {
        let store = self.store.read().await;
        if let Some(entry) = store.peek(key) {
            if Instant::now().duration_since(entry.created_at) <= self.ttl {
                return Some(entry.data.clone());
            }
        }
        None
    }

    /// Insert a new value into the cache.
    pub async fn set(&self, key: K, data: Arc<V>) {
        let mut store = self.store.write().await;
        store.put(
            key,
            CacheEntry {
                data,
                created_at: Instant::now(),
            },
        );
    }
}
