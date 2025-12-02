use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    modules::{
        cache::{
            model::Envelope,
            vendor::gmail::{
                cache::{thread_cache_key, GMAIL_THREADS_CACHE},
                model::messages::MessageMeta,
                sync::{client::GmailClient, envelope::GmailEnvelope},
            },
        },
        error::{code::ErrorCode, RustMailerResult},
        rest::response::CursorDataPage,
    },
    raise_error,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThreadIndex {
    #[serde(rename = "historyId")]
    pub history_id: String,
    pub id: String,
    #[serde(default)]
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThreadList {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threads: Option<Vec<ThreadIndex>>,
    #[serde(rename = "nextPageToken")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
    #[serde(rename = "resultSizeEstimate")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_size_estimate: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThreadMessages {
    #[serde(rename = "historyId")]
    pub history_id: String,
    pub id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub messages: Vec<MessageMeta>,
    ///A short part of the message text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
}

pub async fn list_threads_impl(
    account_id: u64,
    use_proxy: Option<u64>,
    label_id: Option<&str>,
    page_token: Option<&str>,
    after: Option<&str>,
    max_results: u64,
) -> RustMailerResult<CursorDataPage<Envelope>> {
    let list = GmailClient::list_threads_internal(
        account_id,
        use_proxy,
        label_id,
        page_token,
        after,
        max_results,
    )
    .await?;

    let threads = match list.threads {
        Some(t) => t,
        None => {
            return Ok(CursorDataPage::new(
                None,
                Some(max_results),
                0,
                None,
                vec![],
            ))
        }
    };

    let label_map = GmailClient::for_get_label_name(account_id, use_proxy).await?;
    let mut envelopes = Vec::with_capacity(threads.len());
    for thread in threads {
        let thread_id = &thread.id;

        let cache_key = thread_cache_key(account_id, thread_id);
        let thread_messages = if let Some(cached) = GMAIL_THREADS_CACHE.get(&cache_key).await {
            (*cached).clone()
        } else {
            let fetched =
                GmailClient::get_thread_messages(account_id, use_proxy, thread_id).await?;
            GMAIL_THREADS_CACHE
                .set(cache_key.clone(), Arc::new(fetched.clone()))
                .await;
            fetched
        };

        let thread_envelopes: Vec<GmailEnvelope> = thread_messages
            .messages
            .into_iter()
            .map(GmailEnvelope::try_from)
            .collect::<Result<_, _>>()?;

        if let Some(mut envelope) = thread_envelopes
            .into_iter()
            .max_by_key(|env| env.internal_date)
        {
            envelope.account_id = account_id;
            envelopes.push(envelope.into_envelope(&label_map));
        } else {
            tracing::warn!("No valid Gmail message found in thread {}", thread_id);
        }
    }

    let total = list.result_size_estimate.ok_or_else(|| {
        raise_error!(
            "Missing 'resultSizeEstimate' in Gmail API response".into(),
            ErrorCode::InternalError
        )
    })?;

    let total_pages = (total as f64 / max_results as f64).ceil() as u64;
    Ok(CursorDataPage::new(
        list.next_page_token,
        Some(max_results),
        total,
        Some(total_pages),
        envelopes,
    ))
}

pub async fn get_thread_messages_impl(
    account_id: u64,
    use_proxy: Option<u64>,
    thread_id: &str,
) -> RustMailerResult<Vec<Envelope>> {
    let label_map = GmailClient::for_get_label_name(account_id, use_proxy).await?;

    let cache_key = thread_cache_key(account_id, thread_id);
    let thread_messages = if let Some(cached) = GMAIL_THREADS_CACHE.get(&cache_key).await {
        (*cached).clone()
    } else {
        let fetched = GmailClient::get_thread_messages(account_id, use_proxy, thread_id).await?;
        GMAIL_THREADS_CACHE
            .set(cache_key.clone(), Arc::new(fetched.clone()))
            .await;
        fetched
    };

    let thread_envelopes: Vec<GmailEnvelope> = thread_messages
        .messages
        .into_iter()
        .map(GmailEnvelope::try_from)
        .collect::<Result<_, _>>()?;

    let result = thread_envelopes
        .into_iter()
        .map(|mut e| {
            e.account_id = account_id; // Labels are ignored in remote mode; label_id has no effect.
            e.into_envelope(&label_map)
        })
        .collect();
    Ok(result)
}
