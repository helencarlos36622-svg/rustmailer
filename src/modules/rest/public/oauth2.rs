// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::{
    oauth2::{flow::OAuth2Flow, pending::OAuth2PendingEntity},
    settings::cli::SETTINGS,
};
use poem::{
    handler,
    web::{Query, Redirect},
    IntoResponse, Result,
};
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Serialize, Deserialize, Debug)]
pub struct OAuth2CallbackParams {
    state: Option<String>,
    code: Option<String>,
}
#[handler]
pub async fn oauth2_callback(
    Query(params): Query<OAuth2CallbackParams>,
) -> Result<impl IntoResponse> {
    let (state, code) = match (&params.state, &params.code) {
        (Some(state), Some(code)) => (state, code),
        (None, _) => {
            let message =
                "The state parameter is missing. Please initiate the OAuth2 process again.";
            return Ok(Redirect::temporary(format!(
                "/oauth2-result?error=missing_state&message={}",
                urlencoding::encode(message)
            ))
            .into_response());
        }
        (_, None) => {
            let message = "The authorization code is missing. Please try the OAuth2 login again.";
            return Ok(Redirect::temporary(format!(
                "/oauth2-result?error=missing_code&message={}",
                urlencoding::encode(message)
            ))
            .into_response());
        }
    };

    let pending = match OAuth2PendingEntity::get(state).await {
        Ok(Some(pending)) => pending,
        _ => {
            let message =
                "The provided state is invalid or expired. Please start the OAuth2 process again.";
            return Ok(Redirect::temporary(format!(
                "/oauth2-result?error=invalid_state&message={}",
                urlencoding::encode(message)
            ))
            .into_response());
        }
    };

    let flow = OAuth2Flow::new(pending.oauth2_id);
    if let Err(e) = flow
        .fetch_save_access_token(pending.account_id, &pending.code_verifier, code)
        .await
    {
        error!("Failed to save access token: {:#?}", e);
        let message = format!(
            "Failed to retrieve or save the access token. Error details: {:#?}",
            e
        );
        return Ok(Redirect::temporary(format!(
            "/oauth2-result?error=token_fetch_failed&message={}",
            urlencoding::encode(&message)
        ))
        .into_response());
    }

    if let Err(e) = OAuth2PendingEntity::delete(state).await {
        error!("Failed to delete pending OAuth2 entity: {}", e);
    }

    match &SETTINGS.rustmailer_oauth2_success_redirect {
        Some(url) => Ok(Redirect::temporary(url).into_response()),
        None => Ok(Redirect::temporary("/oauth2-result?success=true").into_response()),
    }
}
