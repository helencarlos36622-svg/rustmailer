// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use std::time::Duration;

use crate::{modules::{common::http::HttpClient, error::code::ErrorCode}, raise_error, rustmailer_version};

#[tokio::test]
async fn test_connect_timeout() {
    use crate::modules::hook::entity::HttpMethod;
    use serde_json::json;

    let client = HttpClient::new(None).await.unwrap();

    let url = "http://10.255.255.1:81/";
    let payload = json!({ "test": "timeout" });

    let result = client
        .send_json_request(None, HttpMethod::Post, url, &payload, None)
        .await;

    match result {
        Err(e) => {
            let err_str = e.to_string();
            println!("Caught error: {}", err_str);
        }
        Ok(_) => panic!("Expected timeout, but got successful response"),
    }
}

#[tokio::test]
async fn test_send_to_debug_any_json() {
    use crate::modules::hook::entity::HttpMethod;
    use serde_json::json;

    let client = HttpClient::new(None).await.unwrap();

    let url = "http://127.0.0.1:15630/api/v1/debug-any-json";
    let payload = json!({
        "message": "Hello, debug!",
        "timestamp": 1688888888,
        "nested": {
            "level": "info",
            "enabled": true
        }
    });

    let result = client
        .send_json_request(None, HttpMethod::Post, url, &payload, None)
        .await;

    match result {
        Ok(response) => {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            println!("Response Status: {}", status);
            println!("Response Body: {}", text);
            assert!(status.is_success(), "Expected success response");
        }
        Err(e) => panic!("Request failed: {}", e),
    }
}

#[tokio::test]
async fn test_connect_use_proxy() {
    use crate::modules::hook::entity::HttpMethod;
    use serde_json::json;

    let mut builder = reqwest::ClientBuilder::new()
        .user_agent(rustmailer_version!())
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(10));

    let proxy_obj = reqwest::Proxy::all("http://127.0.0.1:22307")
        .map_err(|e| {
            raise_error!(
                format!(
                    "Failed to configure SOCKS5 proxy ({}): {:#?}. Please check",
                    "socks5://127.0.0.1:22308", e
                ),
                ErrorCode::InternalError
            )
        })
        .unwrap();

    builder = builder
        .redirect(reqwest::redirect::Policy::none())
        .proxy(proxy_obj);

    let client = builder
        .build()
        .map_err(|e| {
            raise_error!(
                format!("Failed to build HTTP client: {:#?}", e),
                ErrorCode::InternalError
            )
        })
        .unwrap();
    let client = HttpClient::create(client);
    let url = "https://discord.com/api/webhooks/1397150752484622416/9yb6QJSJkszn-uiDge3No3ri9B2-shKMKOT1ruijnPbVtd_k9HAuqspn8C2cOXIqu4l5";
    let payload = json!({
        "avatar_url": "https://github.com/rustmailer.png",
        "content": "hello world",
        "embeds": [
            {
                "color": 16711680,
                "description": "you've got a new email in user@example.com",
                "fields": [
                    {
                        "name": "Sender",
                        "value": "sender@example.com"
                    },
                    {
                        "name": "subject",
                        "value": "Meeting Notes"
                    },
                    {
                        "name": "mailbox",
                        "value": "INBOX"
                    },
                    {
                        "name": "date",
                        "value": "2025-07-22 09:31:32"
                    }
                ],
                "footer": {
                    "icon_url": "https://github.com/rustmailer.png",
                    "text": "RustMailer New Email Notification"
                },
                "timestamp": "2025-07-22T12:15:57.148Z",
                "title": "New Email Notification"
            },
            {
                "description": "Type: application/pdf",
                "title": "Attachment: notes.pdf"
            }
        ],
        "username": "RustMailer"
    });

    let result = client
        .send_json_request(None, HttpMethod::Post, url, &payload, None)
        .await;

    match result {
        Err(e) => {
            let err_str = e.to_string();
            println!("Caught error: {}", err_str);
        }
        Ok(_) => println!("send ok"),
    }
}
