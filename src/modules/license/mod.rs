// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::account::migration::AccountModel;
use crate::modules::context::Initialize;
use crate::modules::database::manager::DB_MANAGER;
use crate::modules::database::{async_find_impl, upsert_impl};
use crate::raise_error;
use crate::{
    after_n_days_timestamp, modules::error::RustMailerResult, product_public_key, utc_now,
};
use native_db::*;
use native_model::{native_model, Model};
use poem_openapi::{Enum, Object};
use ring::signature::UnparsedPublicKey;
use serde::{Deserialize, Serialize};

use super::error::code::ErrorCode;

pub mod cache;

pub const LICENSE_KEY: &str = "rustmailer_license";

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize, Object)]
#[native_model(id = 3, version = 1)]
#[native_db]
pub struct License {
    /// Unique identifier for the license record
    #[primary_key]
    pub id: String,
    /// Original license ID from the issued license file
    pub issued_license_id: String,
    /// Licensed application name
    pub application_name: Option<String>,
    /// Customer/organization name this license was issued to
    pub customer_name: Option<String>,
    /// License activation timestamp (Unix epoch in milliseconds)
    pub created_at: i64,
    /// Type of license (Trial/Developer/Team/Enterprise)  
    pub license_type: LicenseType,
    /// License expiration timestamp (Unix epoch in milliseconds)
    pub expires_at: i64,
    /// Previous expiration timestamp for renewed licenses (milliseconds)
    pub last_expires_at: Option<i64>,
    /// Raw license file contents (root only)  
    pub license_content: Option<String>,
    /// Maximum allowed accounts  
    pub max_accounts: Option<u32>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize, Enum)]
pub enum LicenseType {
    #[default]
    Trial,
    Starter,
    Unlimited,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct LicenseContent {
    pub issued_license_id: String,
    pub application_name: Option<String>,
    pub customer_name: Option<String>,
    pub created_at: i64,
    pub license_type: LicenseType,
    pub max_accounts: Option<u32>,
    pub duration_days: u32,
    pub ignore_existing_expiry: bool,
}

impl License {
    // Default: trial license valid for 14 days
    pub async fn create_trial_license() -> RustMailerResult<()> {
        let created_at = utc_now!();
        let license = Self {
            id: LICENSE_KEY.into(),
            issued_license_id: Default::default(),
            application_name: None,
            customer_name: None,
            created_at,
            license_type: LicenseType::Trial,
            expires_at: after_n_days_timestamp!(created_at, 14),
            last_expires_at: None,
            license_content: None,
            max_accounts: None,
        };
        license.save().await
    }

    pub fn new(license_content: LicenseContent, content: String) -> Self {
        Self {
            id: LICENSE_KEY.into(),
            issued_license_id: license_content.issued_license_id,
            application_name: license_content.application_name,
            customer_name: license_content.customer_name,
            created_at: license_content.created_at,
            license_type: license_content.license_type,
            expires_at: after_n_days_timestamp!(
                license_content.created_at,
                license_content.duration_days
            ),
            last_expires_at: None,
            license_content: Some(content),
            max_accounts: license_content.max_accounts,
        }
    }

    pub fn update(
        current_license: License,
        license_content: LicenseContent,
        content: String,
    ) -> Self {
        if !license_content.ignore_existing_expiry && current_license.expires_at >= utc_now!() {
            Self {
                id: LICENSE_KEY.into(),
                issued_license_id: license_content.issued_license_id,
                application_name: license_content.application_name,
                customer_name: license_content.customer_name,
                created_at: license_content.created_at,
                license_type: license_content.license_type,
                expires_at: after_n_days_timestamp!(
                    current_license.expires_at,
                    license_content.duration_days
                ),
                last_expires_at: Some(current_license.expires_at),
                license_content: Some(content),
                max_accounts: license_content.max_accounts,
            }
        } else {
            Self {
                id: LICENSE_KEY.into(),
                issued_license_id: license_content.issued_license_id,
                application_name: license_content.application_name,
                customer_name: license_content.customer_name,
                created_at: license_content.created_at,
                license_type: license_content.license_type,
                expires_at: after_n_days_timestamp!(
                    license_content.created_at,
                    license_content.duration_days
                ),
                last_expires_at: Some(current_license.expires_at),
                license_content: Some(content),
                max_accounts: license_content.max_accounts,
            }
        }
    }

    pub async fn get_current_license() -> RustMailerResult<Option<License>> {
        async_find_impl(DB_MANAGER.meta_db(), LICENSE_KEY).await
    }

    pub async fn save(self) -> RustMailerResult<()> {
        upsert_impl(DB_MANAGER.meta_db(), self).await
    }

    pub async fn check_license(license_str: &str) -> RustMailerResult<License> {
        let license_str = license_str.replace("-----BEGIN LICENSE-----", "");
        let license_str = license_str.replace("-----END LICENSE-----", "");
        let cleaned_license = license_str.trim();
        let public_key = product_public_key!();
        let verifying_key =
            UnparsedPublicKey::new(&ring::signature::ECDSA_P256_SHA256_FIXED, public_key);
        let verifier = min_jwt::verify::ring::RsaKeyVerifier::with_rs256(&verifying_key);
        let signature_verified_jwt = min_jwt::verify(cleaned_license, &verifier)
            .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InvalidLicense))?;
        let decoded_claims = signature_verified_jwt
            .decode_claims()
            .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InvalidLicense))?;
        let license_content = serde_json::from_slice::<LicenseContent>(&decoded_claims)
            .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InvalidLicense))?;
        let current_license = License::get_current_license().await?;

        let license_str = license_str.into();
        let license = match current_license {
            Some(license) => {
                let count = AccountModel::count().await?;
                if let Some(max_accounts) = license_content.max_accounts {
                    if count > max_accounts as usize {
                        return Err(raise_error!(format!(
                                "License limit exceeded: You currently have {} accounts, but your license only allows {} accounts.\n\n\
                                 Please upgrade to a higher-tier license to manage more accounts",
                                count, max_accounts
                            ), ErrorCode::LicenseAccountLimitReached));
                    }
                }
                match license.license_type {
                    LicenseType::Trial => License::new(license_content, license_str),
                    _ => License::update(license, license_content, license_str),
                }
            }
            None => License::new(license_content, license_str),
        };
        Ok(license)
    }
}

impl Initialize for License {
    async fn initialize() -> RustMailerResult<()> {
        if License::get_current_license().await?.is_none() {
            License::create_trial_license().await?;
        }
        Ok(())
    }
}
