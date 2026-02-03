// Copyright © 2025 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use bytes::Bytes;
use dashmap::DashMap;
use http::header::{ACCEPT, AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE};
use http::StatusCode;
use serde::Serialize;
use tracing::error;

use crate::modules::error::code::ErrorCode;
use crate::modules::hook::entity::HttpMethod;
use crate::modules::settings::proxy::Proxy;
use crate::raise_error;
use crate::{modules::error::RustMailerResult, rustmailer_version};
use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::Duration;

#[cfg(test)]
mod tests;

// This will cache clients per proxy configuration.
static HTTP_CLIENTS_CACHE: LazyLock<DashMap<u64, reqwest::Client>> = LazyLock::new(DashMap::new);

pub struct HttpClient {
    client: reqwest::Client,
}

impl HttpClient {
    pub fn create(client: reqwest::Client) -> HttpClient {
        Self { client }
    }

    fn base_builder() -> reqwest::ClientBuilder {
        reqwest::ClientBuilder::new()
            .user_agent(rustmailer_version!())
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(10))
    }

    pub async fn new(use_proxy: Option<u64>) -> RustMailerResult<HttpClient> {
        // Use proxy_id or 0 as the key for the cache
        let proxy_id = use_proxy.unwrap_or(0);
        // First, check if the HttpClient is already cached
        if let Some(client) = HTTP_CLIENTS_CACHE.get(&proxy_id) {
            return Ok(HttpClient::create(client.clone())); // Client is already cloneable, so clone the Arc here
        }
        // If not found in the cache, build a new HttpClient
        let mut builder = Self::base_builder();
        if proxy_id != 0 {
            // Only set the proxy if we have a valid proxy_id
            let proxy = Proxy::get(proxy_id).await?;
            let proxy_obj = reqwest::Proxy::all(&proxy.url).map_err(|e| {
                raise_error!(
                    format!(
                        "Failed to configure SOCKS5 proxy ({}): {:#?}. Please check",
                        &proxy.url, e
                    ),
                    ErrorCode::InternalError
                )
            })?;
            builder = builder
                .redirect(reqwest::redirect::Policy::none())
                .proxy(proxy_obj);
        }
        // Build the HttpClient
        let client = builder.build().map_err(|e| {
            raise_error!(
                format!("Failed to build HTTP client: {:#?}", e),
                ErrorCode::InternalError
            )
        })?;
        // Cache the newly created HttpClient
        HTTP_CLIENTS_CACHE.insert(proxy_id, client.clone());
        Ok(HttpClient::create(client))
    }

    pub async fn send_json_request(
        &self,
        task_info: Option<HashMap<String, String>>,
        method: HttpMethod,
        url: &str,
        payload: &serde_json::Value,
        headers: Option<HashMap<String, String>>,
    ) -> RustMailerResult<reqwest::Response> {
        let mut request_builder = match method {
            HttpMethod::Post => self.client.post(url),
            HttpMethod::Put => self.client.put(url),
        };

        if let Some(headers) = task_info {
            for (key, value) in headers {
                request_builder = request_builder.header(&key, &value);
            }
        }

        // Set headers if provided
        if let Some(headers) = headers {
            for (key, value) in headers {
                request_builder = request_builder.header(&key, &value);
            }
        }

        // Send the request with JSON payload
        let response = request_builder
            .json(payload) // Serialize the payload to JSON
            .send()
            .await
            .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?;
        Ok(response)
    }

    /// Wrapper around the Gmail API `GET` request to fetch data.
    pub async fn get(&self, url: &str, access_token: &str) -> RustMailerResult<serde_json::Value> {
        let mut attempt = 0;
        let max_attempts = 4;
        let mut delay_ms = 500;

        loop {
            attempt += 1;
            let res_result = self
                .client
                .get(url)
                .header(AUTHORIZATION, format!("Bearer {}", access_token))
                .header(CONTENT_TYPE, "application/json")
                .send()
                .await;

            match res_result {
                Ok(res) => {
                    if res.status().is_success() {
                        let json: serde_json::Value = res.json().await.map_err(|e| {
                            raise_error!(
                                format!("Failed to parse response: {:#?}", e),
                                ErrorCode::InternalError
                            )
                        })?;
                        return Ok(json);
                    } else {
                        let status = res.status();
                        let text = res.text().await.unwrap_or_default();

                        if matches!(status, StatusCode::NOT_FOUND | StatusCode::BAD_REQUEST) {
                            error!(
                                status = ?status,
                                url = %url,
                                response = %text,
                                "API client error"
                            );

                            if matches!(status, StatusCode::BAD_REQUEST) {
                                let is_failed_precondition = text.contains("failedPrecondition")
                                    || text.contains("FAILED_PRECONDITION");
                                if is_failed_precondition && attempt < max_attempts {
                                    tracing::warn!(
                                        "API call to {} returned FAILED_PRECONDITION on attempt {}. Retrying after {}ms...",
                                        url, attempt, delay_ms
                                    );
                                    tokio::time::sleep(std::time::Duration::from_millis(delay_ms))
                                        .await;
                                    delay_ms *= 2;
                                    continue;
                                }
                            }

                            return Err(raise_error!(
                                format!(
                                    "API returned client error (status {}) for {}. Response: {}",
                                    status, url, text
                                ),
                                ErrorCode::ApiCallFailed
                            ));
                        }

                        if attempt < max_attempts && status.is_server_error() {
                            tracing::warn!(
                                "API call to {} returned server error {} on attempt {}. Retrying after {}ms...",
                                url, status, attempt, delay_ms
                            );
                            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                            delay_ms *= 2;
                            continue;
                        }

                        return Err(raise_error!(
                            format!(
                                "API call to {} failed with status {}: {}",
                                url, status, text
                            ),
                            ErrorCode::ApiCallFailed
                        ));
                    }
                }
                Err(e) => {
                    if attempt < max_attempts {
                        tracing::warn!(
                            "Request to {} failed on attempt {}: {:#?}, retrying after {}ms",
                            url,
                            attempt,
                            e,
                            delay_ms
                        );
                        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                        delay_ms *= 2;
                        continue;
                    } else {
                        return Err(raise_error!(
                            format!(
                                "Request to {} failed after {} attempts: {:#?}",
                                url, attempt, e
                            ),
                            ErrorCode::ApiCallFailed
                        ));
                    }
                }
            }
        }
    }

    pub async fn get_bytes(&self, url: &str, access_token: &str) -> RustMailerResult<Bytes> {
        let mut attempt = 0;
        let max_attempts = 4;
        let mut delay_ms = 500;

        loop {
            attempt += 1;
            let res_result = self
                .client
                .get(url)
                .header(AUTHORIZATION, format!("Bearer {}", access_token))
                .header(ACCEPT, "application/octet-stream")
                .send()
                .await;

            match res_result {
                Ok(res) => {
                    if res.status().is_success() {
                        let bytes = res.bytes().await.map_err(|e| {
                            raise_error!(
                                format!("Failed to parse response: {:#?}", e),
                                ErrorCode::InternalError
                            )
                        })?;
                        return Ok(bytes);
                    } else {
                        let status = res.status();
                        let text = res.text().await.unwrap_or_default();

                        if attempt < max_attempts && status.is_server_error() {
                            tracing::warn!(
                                "API call to {} returned server error {} on attempt {}. Retrying after {}ms...",
                                url, status, attempt, delay_ms
                            );
                            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                            delay_ms *= 2;
                            continue;
                        }

                        return Err(raise_error!(
                            format!(
                                "API call to {} failed with status {}: {}",
                                url, status, text
                            ),
                            ErrorCode::ApiCallFailed
                        ));
                    }
                }
                Err(e) => {
                    if attempt < max_attempts {
                        tracing::warn!(
                            "Request to {} failed on attempt {}: {:#?}, retrying after {}ms",
                            url,
                            attempt,
                            e,
                            delay_ms
                        );
                        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                        delay_ms *= 2;
                        continue;
                    } else {
                        return Err(raise_error!(
                            format!(
                                "Request to {} failed after {} attempts: {:#?}",
                                url, attempt, e
                            ),
                            ErrorCode::ApiCallFailed
                        ));
                    }
                }
            }
        }
    }

    /// Wrapper around the Gmail API `POST` request.
    pub async fn post<T: Serialize>(
        &self,
        url: &str,
        access_token: &str,
        body: Option<&T>,
        expect_json_response: bool,
    ) -> RustMailerResult<serde_json::Value> {
        let mut builder = self
            .client
            .post(url)
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .header(CONTENT_TYPE, "application/json");
        if let Some(body) = body {
            builder = builder.json(body);
        } else {
            builder = builder.header(CONTENT_LENGTH, 0);
        }

        let res = builder.send().await.map_err(|e| {
            raise_error!(
                format!("Request failed: {:#?}", e),
                ErrorCode::InternalError
            )
        })?;

        if res.status().is_success() {
            if expect_json_response {
                let json: serde_json::Value = res.json().await.map_err(|e| {
                    raise_error!(
                        format!("Failed to parse response: {:#?}", e),
                        ErrorCode::InternalError
                    )
                })?;
                Ok(json)
            } else {
                Ok(serde_json::Value::Null)
            }
        } else {
            let status = res.status();
            let text = res.text().await.map_err(|e| {
                raise_error!(
                    format!("Failed to read error response: {:#?}", e),
                    ErrorCode::InternalError
                )
            })?;
            // Return the error with status and response text for more context
            Err(raise_error!(
                format!(
                    "API call to {} failed with status {}: {}",
                    url, status, text
                ),
                ErrorCode::ApiCallFailed
            ))
        }
    }

    pub async fn post_text<T: Serialize>(
        &self,
        url: &str,
        access_token: &str,
        body: String,
        expect_json_response: bool,
    ) -> RustMailerResult<serde_json::Value> {
        let mut builder = self
            .client
            .post(url)
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .header(CONTENT_TYPE, "text/plain");
        builder = builder.body(body);

        let res = builder.send().await.map_err(|e| {
            raise_error!(
                format!("Request failed: {:#?}", e),
                ErrorCode::InternalError
            )
        })?;

        if res.status().is_success() {
            if expect_json_response {
                let json: serde_json::Value = res.json().await.map_err(|e| {
                    raise_error!(
                        format!("Failed to parse response: {:#?}", e),
                        ErrorCode::InternalError
                    )
                })?;
                Ok(json)
            } else {
                Ok(serde_json::Value::Null)
            }
        } else {
            let status = res.status();
            let text = res.text().await.map_err(|e| {
                raise_error!(
                    format!("Failed to read error response: {:#?}", e),
                    ErrorCode::InternalError
                )
            })?;
            // Return the error with status and response text for more context
            Err(raise_error!(
                format!(
                    "API call to {} failed with status {}: {}",
                    url, status, text
                ),
                ErrorCode::ApiCallFailed
            ))
        }
    }

    /// Wrapper around the Gmail API `POST` request.
    pub async fn delete(&self, url: &str, access_token: &str) -> RustMailerResult<()> {
        let res = self
            .client
            .delete(url)
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await
            .map_err(|e| {
                raise_error!(
                    format!("Request failed: {:#?}", e),
                    ErrorCode::InternalError
                )
            })?;

        if res.status().is_success() {
            Ok(())
        } else {
            let status = res.status();
            let text = res.text().await.map_err(|e| {
                raise_error!(
                    format!("Failed to read error response: {:#?}", e),
                    ErrorCode::InternalError
                )
            })?;
            // Return the error with status and response text for more context
            Err(raise_error!(
                format!(
                    "API call to {} failed with status {}: {}",
                    url, status, text
                ),
                ErrorCode::ApiCallFailed
            ))
        }
    }

    pub async fn put<T: Serialize + ?Sized>(
        &self,
        url: &str,
        access_token: &str,
        body: &T,
    ) -> RustMailerResult<serde_json::Value> {
        let res = self
            .client
            .put(url)
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .header(CONTENT_TYPE, "application/json")
            .json(body)
            .send()
            .await
            .map_err(|e| {
                raise_error!(
                    format!("Request failed: {:#?}", e),
                    ErrorCode::InternalError
                )
            })?;

        if res.status().is_success() {
            let json: serde_json::Value = res.json().await.map_err(|e| {
                raise_error!(
                    format!("Failed to parse response: {:#?}", e),
                    ErrorCode::InternalError
                )
            })?;
            Ok(json)
        } else {
            let status = res.status();
            let text = res.text().await.map_err(|e| {
                raise_error!(
                    format!("Failed to read error response: {:#?}", e),
                    ErrorCode::InternalError
                )
            })?;
            // Return the error with status and response text for more context
            Err(raise_error!(
                format!(
                    "API call to {} failed with status {}: {}",
                    url, status, text
                ),
                ErrorCode::ApiCallFailed
            ))
        }
    }

    pub async fn patch<T: Serialize + ?Sized>(
        &self,
        url: &str,
        access_token: &str,
        body: &T,
    ) -> RustMailerResult<serde_json::Value> {
        let res = self
            .client
            .patch(url)
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .header(CONTENT_TYPE, "application/json")
            .json(body)
            .send()
            .await
            .map_err(|e| {
                raise_error!(
                    format!("Request failed: {:#?}", e),
                    ErrorCode::InternalError
                )
            })?;

        if res.status().is_success() {
            let json: serde_json::Value = res.json().await.map_err(|e| {
                raise_error!(
                    format!("Failed to parse response: {:#?}", e),
                    ErrorCode::InternalError
                )
            })?;
            Ok(json)
        } else {
            let status = res.status();
            let text = res.text().await.map_err(|e| {
                raise_error!(
                    format!("Failed to read error response: {:#?}", e),
                    ErrorCode::InternalError
                )
            })?;
            // Return the error with status and response text for more context
            Err(raise_error!(
                format!(
                    "API call to {} failed with status {}: {}",
                    url, status, text
                ),
                ErrorCode::ApiCallFailed
            ))
        }
    }
}
