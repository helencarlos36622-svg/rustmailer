// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use std::collections::BTreeSet;

use crate::modules::account::payload::{
    filter_accessible_accounts, AccountCreateRequest, AccountUpdateRequest, MinimalAccount,
};
use crate::modules::account::status::AccountRunningState;
use crate::modules::account::migration::AccountModel;
use crate::modules::common::auth::ClientContext;
use crate::modules::common::paginated::paginate_vec;
use crate::modules::error::code::ErrorCode;
use crate::modules::rest::api::ApiTags;
use crate::modules::rest::response::DataPage;
use crate::modules::rest::ApiResult;
use crate::modules::token::{AccessToken, AccountInfo};
use crate::raise_error;
use poem::web::Path;
use poem_openapi::param::Query;
use poem_openapi::payload::Json;
use poem_openapi::OpenApi;

pub struct AccountApi;

#[OpenApi(prefix_path = "/api/v1", tag = "ApiTags::Account")]
impl AccountApi {
    /// Get account details by account ID
    #[oai(
        path = "/account/:account_id",
        method = "get",
        operation_id = "get_account"
    )]
    async fn get_account(
        &self,
        /// The account ID to retrieve
        account_id: Path<u64>,
        context: ClientContext,
    ) -> ApiResult<Json<AccountModel>> {
        let account_id = account_id.0;
        context.require_account_access(account_id)?;
        Ok(Json(AccountModel::get(account_id).await?))
    }

    /// Delete an account by ID - WARNING: This permanently removes the account and all associated resources
    #[oai(
        path = "/account/:account_id",
        method = "delete",
        operation_id = "remove_account"
    )]
    async fn remove_account(
        &self,
        /// The account ID to delete
        account_id: Path<u64>,
        context: ClientContext,
    ) -> ApiResult<()> {
        let account_id = account_id.0;
        context.require_account_access(account_id)?;
        Ok(AccountModel::delete(account_id).await?)
    }

    /// Create a new account
    #[oai(path = "/account", method = "post", operation_id = "create_account")]
    async fn create_account(
        &self,
        /// Account creation request payload
        payload: Json<AccountCreateRequest>,
        context: ClientContext,
    ) -> ApiResult<Json<AccountModel>> {
        let account = AccountModel::create_account(payload.0).await?;
        if let Some(access_token) = &context.access_token {
            let account_info = AccountInfo {
                id: account.id,
                email: account.email.clone(),
            };
            AccessToken::grant_account_access(&access_token.token, account_info).await?;
        }
        Ok(Json(account))
    }

    /// Update an existing account
    #[oai(
        path = "/account/:account_id",
        method = "post",
        operation_id = "update_account"
    )]
    async fn update_account(
        &self,
        /// The account ID to update
        account_id: Path<u64>,
        /// Account update request payload
        payload: Json<AccountUpdateRequest>,
        context: ClientContext,
    ) -> ApiResult<()> {
        let account_id = account_id.0;
        context.require_account_access(account_id)?;
        Ok(AccountModel::update(account_id, payload.0, true).await?)
    }

    /// List accounts with optional pagination parameters
    #[oai(
        path = "/list-accounts",
        method = "get",
        operation_id = "list_accounts"
    )]
    async fn list_accounts(
        &self,
        /// Optional. The page number to retrieve (starting from 1).
        page: Query<Option<u64>>,
        /// Optional. The number of items per page.
        page_size: Query<Option<u64>>,
        /// Optional. Whether to sort the list in descending order.
        desc: Query<Option<bool>>,
        context: ClientContext,
    ) -> ApiResult<Json<DataPage<AccountModel>>> {
        let accessible_accounts = context.accessible_accounts()?;

        if accessible_accounts.is_none() {
            return Ok(Json(
                AccountModel::paginate_list(page.0, page_size.0, desc.0).await?,
            ));
        }

        let all_accounts = AccountModel::list_all().await?;
        let allowed_ids: BTreeSet<u64> =
            accessible_accounts.unwrap().iter().map(|a| a.id).collect();

        let mut filtered_accounts: Vec<AccountModel> = all_accounts
            .into_iter()
            .filter(|acct| allowed_ids.contains(&acct.id))
            .collect();

        let sort_desc = desc.0.unwrap_or(true);
        filtered_accounts.sort_by(|a, b| {
            if sort_desc {
                b.created_at.cmp(&a.created_at)
            } else {
                a.created_at.cmp(&b.created_at)
            }
        });
        let page_data =
            paginate_vec(&filtered_accounts, page.0, page_size.0).map(DataPage::from)?;
        Ok(Json(page_data))
    }

    /// Get the running state of an account
    #[oai(
        path = "/account-state/:account_id",
        method = "get",
        operation_id = "account_state"
    )]
    async fn account_state(
        &self,
        /// The account ID to check state for
        account_id: Path<u64>,
        context: ClientContext,
    ) -> ApiResult<Json<AccountRunningState>> {
        let account_id = account_id.0;
        context.require_account_access(account_id)?;
        let state = AccountRunningState::get(account_id).await?.ok_or_else(|| {
            raise_error!(
                "account running state is not found".into(),
                ErrorCode::ResourceNotFound
            )
        })?;
        Ok(Json(state))
    }

    /// Get a minimal list of active accounts for use in selectors when creating account-related resources
    /// This endpoint provides a lightweight list of accounts containing only essential information (id and name).
    /// It's primarily designed for UI selectors/dropdowns when creating or associating resources with accounts.
    #[oai(
        path = "/minimal-account-list",
        method = "get",
        operation_id = "minimal_accounts_list"
    )]
    async fn minimal_accounts_list(
        &self,
        context: ClientContext,
    ) -> ApiResult<Json<Vec<MinimalAccount>>> {
        let accessible_accounts = context.accessible_accounts()?;

        let minimal_list = AccountModel::minimal_list().await?;
        let result = match accessible_accounts {
            Some(set) => filter_accessible_accounts(&minimal_list, set),
            None => minimal_list,
        };
        Ok(Json(result))
    }
}
