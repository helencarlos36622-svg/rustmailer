// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::error::code::ErrorCode;
use crate::modules::license::{License, LicenseType};
use crate::{modules::error::RustMailerResult, raise_error, utc_now};
use dashmap::DashMap;
use std::sync::LazyLock;

const CACHE_TTL: i64 = 60 * 60 * 1000;

static LICENSE_CACHE: LazyLock<DashMap<&'static str, CachedLicense>> =
    LazyLock::new(|| DashMap::new());

#[derive(Clone)]
pub struct CachedLicense {
    pub license: License,
    pub updated_at: i64,
}

impl CachedLicense {
    pub fn new(license: License) -> Self {
        Self {
            license,
            updated_at: utc_now!() - 2 * CACHE_TTL,
        }
    }

    #[inline]
    fn is_invalid(&self, now: i64) -> bool {
        match self.license.license_type {
            LicenseType::Trial => self.license.expires_at < now,
            _ => self.license.expires_at + 30 * 24 * 60 * 60 * 1000 < now,
        }
    }

    #[inline]
    fn is_stale(&self, now: i64) -> bool {
        now - self.updated_at >= CACHE_TTL
    }

    pub async fn check_license_validity() -> RustMailerResult<()> {
        let now = utc_now!();

        // Check if the license exists in cache
        if let Some(cached) = LICENSE_CACHE.get("global") {
            // If the cached license is already invalid, return immediately
            if cached.is_invalid(now) {
                return Err(raise_error!(
                    "License has expired".into(),
                    ErrorCode::LicenseExpired
                ));
            }

            // If the cache is still fresh and license is valid, return success
            if !cached.is_stale(now) {
                return Ok(());
            }

            // Otherwise, fall through to refresh the license
        }

        // Either no cached license or it's stale, fetch from source
        let license = License::get_current_license().await?.ok_or_else(|| {
            raise_error!(
                "No valid license found".into(),
                ErrorCode::MissingConfiguration
            )
        })?;

        // Update the cache with the latest license
        let cache_entry = CachedLicense::new(license.clone());
        LICENSE_CACHE.insert("global", cache_entry.clone());

        // Re-check validity of the refreshed license
        if cache_entry.is_invalid(now) {
            Err(raise_error!(
                "License has expired".into(),
                ErrorCode::LicenseExpired
            ))
        } else {
            Ok(())
        }
    }
}
