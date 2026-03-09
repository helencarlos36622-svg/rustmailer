// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use dashmap::DashMap;
use governor::{
    clock::{QuantaClock, QuantaInstant},
    middleware::NoOpMiddleware,
    state::{InMemoryState, NotKeyed},
    NotUntil, Quota, RateLimiter,
};
use std::{
    num::NonZero,
    sync::{Arc, LazyLock},
    time::Duration,
};

use crate::modules::token::RateLimit;

pub static RATE_LIMITER_MANAGER: LazyLock<TokenRateLimiter> = LazyLock::new(TokenRateLimiter::new);

pub struct TokenRateLimiter {
    limiters: Arc<
        DashMap<
            String,
            (
                Arc<RateLimiter<NotKeyed, InMemoryState, QuantaClock, NoOpMiddleware>>,
                RateLimit,
            ),
        >,
    >,
}

impl TokenRateLimiter {
    pub fn new() -> Self {
        TokenRateLimiter {
            limiters: Arc::new(DashMap::new()),
        }
    }

    pub async fn check(
        &self,
        token: &str,
        limit: RateLimit,
    ) -> Result<(), NotUntil<QuantaInstant>> {
        let limiter = self.get_or_update_limiter(token, limit).await;
        limiter.check()
    }

    async fn get_or_update_limiter(
        &self,
        token: &str,
        limit: RateLimit,
    ) -> Arc<RateLimiter<NotKeyed, InMemoryState, QuantaClock, NoOpMiddleware>> {
        self.limiters
            .entry(token.to_string())
            .and_modify(|(existing_limiter, current_limit)| {
                if current_limit.interval != limit.interval || current_limit.quota != limit.quota {
                    let quota = Quota::with_period(Duration::from_secs(limit.interval))
                        .unwrap()
                        .allow_burst(NonZero::new(limit.quota).unwrap());
                    *existing_limiter = Arc::new(RateLimiter::direct_with_clock(
                        quota,
                        QuantaClock::default(),
                    ));
                    *current_limit = RateLimit {
                        interval: limit.interval,
                        quota: limit.quota,
                    };
                }
            })
            .or_insert({
                let quota = Quota::with_period(Duration::from_secs(limit.interval))
                    .unwrap()
                    .allow_burst(NonZero::new(limit.quota).unwrap());
                (
                    Arc::new(RateLimiter::direct_with_clock(
                        quota,
                        QuantaClock::default(),
                    )),
                    limit,
                )
            })
            .value()
            .0
            .clone()
    }
}