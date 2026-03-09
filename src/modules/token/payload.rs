// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use std::collections::BTreeSet;

use crate::{
    modules::{
        account::migration::AccountModel,
        error::{code::ErrorCode, RustMailerResult},
        token::{AccessControl, AccessTokenScope},
    },
    raise_error,
};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize, Object)]
pub struct AccessTokenCreateRequest {
    /// A set of account information associated with the token.
    pub accounts: BTreeSet<u64>,
    /// An optional description of the token's purpose or usage.
    #[oai(validator(max_length = "255"))]
    pub description: Option<String>,
    /// A set of scopes defining the token's access permissions.
    pub access_scopes: BTreeSet<AccessTokenScope>,
    /// Optional access control settings
    pub acl: Option<AccessControl>,
}

impl AccessTokenCreateRequest {
    pub async fn validate(&self) -> RustMailerResult<()> {
        if let Some(acl) = &self.acl {
            acl.validate()?;
        }

        if self.accounts.is_empty() {
            return Err(raise_error!(
                "Account list cannot be empty. Please provide at least one valid account ID."
                    .into(),
                ErrorCode::InvalidParameter
            ));
        }

        let mut not_found = Vec::new();
        for account_id in &self.accounts {
            if AccountModel::find(*account_id).await?.is_none() {
                not_found.push(*account_id);
            }
        }
        if !not_found.is_empty() {
            return Err(raise_error!(
                format!("The following account IDs were not found: {}. Please provide valid account IDs.", not_found.iter().map(u64::to_string).collect::<Vec<_>>().join(", ")).into(),
                ErrorCode::InvalidParameter
            ));
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Object)]
pub struct AccessTokenUpdateRequest {
    /// A set of account information associated with the token.
    pub accounts: Option<BTreeSet<u64>>,
    /// An optional description of the token's purpose or usage.
    #[oai(validator(max_length = "255"))]
    pub description: Option<String>,
    /// A set of scopes defining the token's access permissions.
    pub access_scopes: Option<BTreeSet<AccessTokenScope>>,
    /// Optional access control settings
    pub acl: Option<AccessControl>,
}

impl AccessTokenUpdateRequest {
    pub async fn validate(&self) -> RustMailerResult<()> {
        if let Some(acl) = &self.acl {
            acl.validate()?;
        }
        if let Some(accounts) = &self.accounts {
            if accounts.is_empty() {
                return Err(raise_error!(
                    "Account list cannot be empty. Please provide at least one valid account ID."
                        .into(),
                    ErrorCode::InvalidParameter
                ));
            }

            let mut not_found = Vec::new();
            for account_id in accounts {
                if AccountModel::find(*account_id).await?.is_none() {
                    not_found.push(*account_id);
                }
            }
            if !not_found.is_empty() {
                return Err(raise_error!(
                format!("The following account IDs were not found: {}. Please provide valid account IDs.", not_found.iter().map(u64::to_string).collect::<Vec<_>>().join(", ")).into(),
                ErrorCode::InvalidParameter
            ));
            }
        }

        Ok(())
    }
}

impl AccessTokenUpdateRequest {
    pub fn should_skip_update(&self) -> bool {
        self.access_scopes.is_none()
            && self.description.is_none()
            && self.accounts.is_none()
            && self.acl.is_none()
    }
}
