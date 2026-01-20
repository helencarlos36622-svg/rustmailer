// Copyright © 2025 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use http::header::{ACCEPT, CONTENT_TYPE};
use poem_grpc::{ClientConfig, CompressionEncoding};
use reqwest::{header::AUTHORIZATION, Client};
use serde_json::json;
use std::{fs::File, future::Future, io::Write, pin::Pin, time::Duration};

use crate::{
    modules::{
        cache::vendor::outlook::model::{
            CategoriesListResponse, MailFolder, MailFoldersResponse, MessageListResponse,
        },
        common::rustls::RustMailerTls,
        context::Initialize,
        error::{code::ErrorCode, RustMailerResult},
        grpc::service::rustmailer_grpc::{GetOAuth2TokensRequest, OAuth2ServiceClient},
    },
    raise_error, rustmailer_version,
};

async fn access_token() -> String {
    RustMailerTls::initialize().await.unwrap();

    let cfg = ClientConfig::builder()
        .uri("http://localhost:16630")
        .build()
        .unwrap();
    let mut grpc_client = OAuth2ServiceClient::new(cfg);
    grpc_client.set_accept_compressed([CompressionEncoding::GZIP]);
    grpc_client.set_send_compressed(CompressionEncoding::GZIP);

    let request = GetOAuth2TokensRequest {
        account_id: 3815676991897752,
    };

    let mut request = poem_grpc::Request::new(request);
    request.metadata_mut().insert(
        AUTHORIZATION,
        format!("Bearer {}", "2mY4irNCahQXeSarHYje1P1W"),
    );
    let result = grpc_client.get_o_auth2_tokens(request).await.unwrap();
    result.access_token.clone().unwrap()
}

#[tokio::test]
async fn test1() {
    let access_token = access_token().await;
    let url = "https://graph.microsoft.com/v1.0/me/mailFolders";
    let mut builder = reqwest::ClientBuilder::new()
        .user_agent(rustmailer_version!())
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(10));

    let proxy_obj = reqwest::Proxy::all("http://127.0.0.1:22307").unwrap();
    builder = builder
        .redirect(reqwest::redirect::Policy::none())
        .proxy(proxy_obj);
    let client = builder.build().unwrap();

    let res = client
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await
        .unwrap();

    if res.status().is_success() {
        let body: serde_json::Value = res.json().await.unwrap();
        let json = serde_json::to_string_pretty(&body).unwrap();
        println!("Response = {}", json);
    } else {
        eprintln!("Error: {} - {:?}", res.status(), res.text().await.unwrap());
    }
}

#[tokio::test]
async fn test2() {
    let access_token = access_token().await;
    let url = "https://graph.microsoft.com/v1.0/me/mailFolders/AQMkADAwATMwMAItNzE0OC1jZTEzLTAwAi0wMAoALgAAA_KUk7xWPSBEntPHShr61lgBAOo9V4GwHndCjf0x1uoIcwUAAAIBCgAAAA==/childFolders";
    let mut builder = reqwest::ClientBuilder::new()
        .user_agent(rustmailer_version!())
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(10));

    let proxy_obj = reqwest::Proxy::all("http://127.0.0.1:22307").unwrap();
    builder = builder
        .redirect(reqwest::redirect::Policy::none())
        .proxy(proxy_obj);
    let client = builder.build().unwrap();

    let res = client
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await
        .unwrap();

    if res.status().is_success() {
        let body: serde_json::Value = res.json().await.unwrap();
        let json = serde_json::to_string_pretty(&body).unwrap();
        println!("Response = {}", json);
    } else {
        eprintln!("Error: {} - {:?}", res.status(), res.text().await.unwrap());
    }
}

fn fetch_recursive<'a>(
    client: &'a Client,
    folder_id: Option<&'a str>,
    prefix: &'a str,
    output: &'a mut Vec<MailFolder>,
    access_token: &'a str,
) -> Pin<Box<dyn Future<Output = RustMailerResult<()>> + 'a>> {
    Box::pin(async move {
        let folders_response: MailFoldersResponse = match folder_id {
            Some(id) => {
                let url =
                    format!("https://graph.microsoft.com/v1.0/me/mailFolders/{id}/childFolders");
                let res = client
                    .get(url)
                    .header(AUTHORIZATION, format!("Bearer {}", access_token))
                    .header(CONTENT_TYPE, "application/json")
                    .send()
                    .await
                    .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?;

                if res.status().is_success() {
                    let body: MailFoldersResponse = res
                        .json()
                        .await
                        .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?;
                    body
                } else {
                    return Err(raise_error!("".into(), ErrorCode::InternalError));
                }
            }
            None => {
                let url = "https://graph.microsoft.com/v1.0/me/mailFolders";
                let res = client
                    .get(url)
                    .header(AUTHORIZATION, format!("Bearer {}", access_token))
                    .header(CONTENT_TYPE, "application/json")
                    .send()
                    .await
                    .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?;
                if res.status().is_success() {
                    let body: MailFoldersResponse = res
                        .json()
                        .await
                        .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?;
                    body
                } else {
                    return Err(raise_error!("".into(), ErrorCode::InternalError));
                }
            }
        };

        for mut folder in folders_response.value {
            let full_name = if prefix.is_empty() {
                folder.display_name.clone()
            } else {
                format!("{}/{}", prefix, folder.display_name)
            };
            folder.display_name = full_name.clone();
            output.push(folder.clone());

            if folder.child_folder_count.unwrap_or(0) > 0 {
                fetch_recursive(client, Some(&folder.id), &full_name, output, access_token).await?;
            }
        }
        Ok(())
    })
}

pub async fn fetch_flattened_mailfolders(
    client: &Client,
    access_token: &str,
) -> RustMailerResult<Vec<MailFolder>> {
    let mut result = Vec::new();
    fetch_recursive(client, None, "", &mut result, access_token).await?;
    Ok(result)
}

#[tokio::test]
async fn test3() {
    let access_token = access_token().await;
    let mut builder = reqwest::ClientBuilder::new()
        .user_agent(rustmailer_version!())
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(10));

    let proxy_obj = reqwest::Proxy::all("http://127.0.0.1:22307").unwrap();
    builder = builder
        .redirect(reqwest::redirect::Policy::none())
        .proxy(proxy_obj);
    let client = builder.build().unwrap();

    let result = fetch_flattened_mailfolders(&client, &access_token)
        .await
        .unwrap();
    println!("{:#?}", result);
}

#[tokio::test]
async fn test9() {
    let access_token = access_token().await;
    let url = "https://graph.microsoft.com/v1.0/me/mailFolders/inbox";
    let mut builder = reqwest::ClientBuilder::new()
        .user_agent(rustmailer_version!())
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(10));

    let proxy_obj = reqwest::Proxy::all("http://127.0.0.1:22307").unwrap();
    builder = builder
        .redirect(reqwest::redirect::Policy::none())
        .proxy(proxy_obj);
    let client = builder.build().unwrap();

    let res = client
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await
        .unwrap();

    if res.status().is_success() {
        let body: serde_json::Value = res.json().await.unwrap();
        let json = serde_json::to_string_pretty(&body).unwrap();
        println!("Response = {}", json);
    } else {
        eprintln!("Error: {} - {:?}", res.status(), res.text().await.unwrap());
    }
}

#[tokio::test]
async fn fetch_delta() {
    let access_token = access_token().await;
    let mut url = "https://graph.microsoft.com/v1.0/me/mailFolders/inbox/messages/delta?$select=id&$orderBy=receivedDateTime desc".to_string();
    let client = reqwest::Client::builder()
        .user_agent(rustmailer_version!())
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(10))
        .proxy(reqwest::Proxy::all("http://127.0.0.1:22307").unwrap())
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let mut nextlink_count = 0;

    loop {
        let res = client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await
            .unwrap();

        if res.status().is_success() {
            let body: serde_json::Value = res.json().await.unwrap();
            if let Some(value) = body.get("value") {
                println!(
                    "This page contains {} messages",
                    value.as_array().unwrap().len()
                );
            }

            if let Some(next_link) = body.get("@odata.nextLink") {
                url = next_link.as_str().unwrap().to_string();
                nextlink_count += 1;
                println!("Following @odata.nextLink: {}", url);
            } else if let Some(delta_link) = body.get("@odata.deltaLink") {
                println!("Reached @odata.deltaLink: {}", delta_link.as_str().unwrap());
                println!("@odata.nextLink was called {} times", nextlink_count);
                break;
            } else {
                println!("No nextLink or deltaLink, ending loop.");
                break;
            }
        } else {
            eprintln!("Error: {} - {:?}", res.status(), res.text().await.unwrap());
            break;
        }
    }
}

#[tokio::test]
async fn fetch_delta2() {
    let access_token = access_token().await;
    let mut url =
        "https://graph.microsoft.com/v1.0/me/mailFolders('inbox')/messages/delta?$deltatoken=gGpE9bWe1qFWcyG3HvbWkkunspLvA4xxmWqkeJWDye811LoJPSEs2ZjUPG0pKudlwejI_rsTDgNJAu_zA2ex-FgAPO2Di5TtbAbBZHRD2W4wUQqFja-VKSwMWTD2WTQ40_86JhGVgDhA_GpxYC-BBtGflJttseEOr6eCZyMVGlI.i7MKxGEvGLJfbZxhsubwycVsTVIyTbv4AIJH78wjsog"
            .to_string();
    let client = reqwest::Client::builder()
        .user_agent(rustmailer_version!())
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(10))
        .proxy(reqwest::Proxy::all("http://127.0.0.1:22307").unwrap())
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let mut nextlink_count = 0;

    loop {
        let res = client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await
            .unwrap();

        if res.status().is_success() {
            let body: serde_json::Value = res.json().await.unwrap();

            println!("{:#?}", &body);
            if let Some(value) = body.get("value") {
                println!(
                    "This page contains {} messages",
                    value.as_array().unwrap().len()
                );
            }

            if let Some(next_link) = body.get("@odata.nextLink") {
                url = next_link.as_str().unwrap().to_string();
                nextlink_count += 1;
                println!("Following @odata.nextLink: {}", url);
            } else if let Some(delta_link) = body.get("@odata.deltaLink") {
                println!("Reached @odata.deltaLink: {}", delta_link.as_str().unwrap());
                println!("@odata.nextLink was called {} times", nextlink_count);
                break;
            } else {
                println!("No nextLink or deltaLink, ending loop.");
                break;
            }
        } else {
            eprintln!("Error: {} - {:?}", res.status(), res.text().await.unwrap());
            break;
        }
    }
}

#[tokio::test]
async fn list_messages() {
    let access_token = access_token().await;
    let mut url =
        "https://graph.microsoft.com/v1.0/me/mailFolders/inbox/messages?$top=3&\
               $orderBy=receivedDateTime desc&\
               $filter=receivedDateTime ge 2025-10-01T00:00:00Z&\
               $select=id,isRead,conversationId,internetMessageId,from,body,toRecipients,ccRecipients,\
               bccRecipients,replyTo,sender,subject,receivedDateTime,sentDateTime,isRead,bodyPreview,categories&\
               $expand=attachments($select=id,name,contentType,size,isInline,microsoft.graph.fileAttachment/contentId)"
            .to_string();
    let client = reqwest::Client::builder()
        .user_agent(rustmailer_version!())
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(10))
        .proxy(reqwest::Proxy::all("http://127.0.0.1:22307").unwrap())
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let mut nextlink_count = 0;

    let res = client
        .get(&url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await
        .unwrap();

    if res.status().is_success() {
        let body: MessageListResponse = res.json().await.unwrap();
        println!("{:#?}", body);
    } else {
        eprintln!("Error: {} - {:?}", res.status(), res.text().await.unwrap());
    }
}

#[tokio::test]
async fn get_message() {
    let access_token = access_token().await;
    let mut url =
        "https://graph.microsoft.com/v1.0/me/messages/AQMkADAwATMwMAItNzE0OC1jZTEzLTAwAi0wMAoARgAAA_KUk7xWPSBEntPHShr61lgHAOo9V4GwHndCjf0x1uoIcwUAAAIBDAAAAOo9V4GwHndCjf0x1uoIcwUAAVpj5Y4AAAA=?\
               $select=id,isRead,conversationId,internetMessageId,from,body,toRecipients,ccRecipients,\
               bccRecipients,replyTo,sender,subject,receivedDateTime,sentDateTime,isRead,bodyPreview,categories&\
               $expand=attachments($select=id,name,contentType,size,isInline,microsoft.graph.fileAttachment/contentId)"
            .to_string();
    let client = reqwest::Client::builder()
        .user_agent(rustmailer_version!())
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(10))
        .proxy(reqwest::Proxy::all("http://127.0.0.1:22307").unwrap())
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let mut nextlink_count = 0;

    let res = client
        .get(&url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await
        .unwrap();

    if res.status().is_success() {
        let body: serde_json::Value = res.json().await.unwrap();
        println!("{:#?}", body);
    } else {
        eprintln!("Error: {} - {:?}", res.status(), res.text().await.unwrap());
    }
}

#[tokio::test]
async fn get_attachment() {
    let access_token = access_token().await;
    let attachment_id =  "AQMkADAwATMwMAItNzE0OC1jZTEzLTAwAi0wMAoARgAAA_KUk7xWPSBEntPHShr61lgHAOo9V4GwHndCjf0x1uoIcwUAAAIBDAAAAOo9V4GwHndCjf0x1uoIcwUAAYiJVIAAAAABEgAQAFv-yJzy5mVEmtL1_F7Z6DM=";
    let url = format!(
        "https://graph.microsoft.com/v1.0/me/messages/{}/attachments/{}",
        "AQMkADAwATMwMAItNzE0OC1jZTEzLTAwAi0wMAoARgAAA_KUk7xWPSBEntPHShr61lgHAOo9V4GwHndCjf0x1uoIcwUAAAIBDAAAAOo9V4GwHndCjf0x1uoIcwUAAYiJVIAAAAA=", attachment_id
    );
    let client = reqwest::Client::builder()
        .user_agent(rustmailer_version!())
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(10))
        .proxy(reqwest::Proxy::all("http://127.0.0.1:22307").unwrap())
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let mut nextlink_count = 0;

    let res = client
        .get(&url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await
        .unwrap();

    if res.status().is_success() {
        let body: serde_json::Value = res.json().await.unwrap();
        println!("{:#?}", body);
    } else {
        eprintln!("Error: {} - {:?}", res.status(), res.text().await.unwrap());
    }
}

#[tokio::test]
async fn create_reply_draft() {
    let access_token = access_token().await;
    let url = format!(
        "https://graph.microsoft.com/v1.0/me/messages/{}/createReply",
        "AQMkADAwATMwMAItNzE0OC1jZTEzLTAwAi0wMAoARgAAA_KUk7xWPSBEntPHShr61lgHAOo9V4GwHndCjf0x1uoIcwUAAAIBDAAAAOo9V4GwHndCjf0x1uoIcwUAAauT-moAAAA="
    );
    let client = reqwest::Client::builder()
        .user_agent(rustmailer_version!())
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(10))
        .proxy(reqwest::Proxy::all("http://127.0.0.1:22307").unwrap())
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let res = client
        .post(&url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await
        .unwrap();

    if res.status().is_success() {
        let body: serde_json::Value = res.json().await.unwrap();
        println!("{:#?}", body);
    } else {
        eprintln!("Error: {} - {:?}", res.status(), res.text().await.unwrap());
    }
}

#[tokio::test]
async fn patch_reply_draft() {
    let access_token = access_token().await;
    let url = format!(
        "https://graph.microsoft.com/v1.0/me/messages/{}",
        "AQMkADAwATMwMAItNzE0OC1jZTEzLTAwAi0wMAoARgAAA_KUk7xWPSBEntPHShr61lgHAOo9V4GwHndCjf0x1uoIcwUAAAIBDwAAAOo9V4GwHndCjf0x1uoIcwUAAauT6uAAAAA="
    );
    let client = reqwest::Client::builder()
        .user_agent(rustmailer_version!())
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(10))
        .proxy(reqwest::Proxy::all("http://127.0.0.1:22307").unwrap())
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let data = json!({
        "body": {
            "contentType": "text",
            "content": "hello world",
        },
        "isRead": false
    });

    let res = client
        .patch(&url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .json(&data)
        .send()
        .await
        .unwrap();

    if res.status().is_success() {
        let body: serde_json::Value = res.json().await.unwrap();
        println!("{:#?}", body);
    } else {
        eprintln!("Error: {} - {:?}", res.status(), res.text().await.unwrap());
    }
}

#[tokio::test]
async fn get_raw_message() {
    let access_token = access_token().await;
    let url = format!(
        "https://graph.microsoft.com/v1.0/me/messages/{}/$value",
        "AQMkADAwATMwMAItNzE0OC1jZTEzLTAwAi0wMAoARgAAA_KUk7xWPSBEntPHShr61lgHAOo9V4GwHndCjf0x1uoIcwUAAAIBDAAAAOo9V4GwHndCjf0x1uoIcwUAAYiJVIEAAAA="
    );
    let client = reqwest::Client::builder()
        .user_agent(rustmailer_version!())
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(10))
        .proxy(reqwest::Proxy::all("http://127.0.0.1:22307").unwrap())
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let res = client
        .get(&url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(ACCEPT, "application/octet-stream")
        .send()
        .await
        .unwrap();

    if res.status().is_success() {
        let bytes = res.bytes().await.unwrap();
        let mut file = File::create("e:\\message.eml").unwrap();
        file.write_all(&bytes).unwrap();
    } else {
        eprintln!("Error: {} - {:?}", res.status(), res.text().await.unwrap());
    }
}

#[tokio::test]
async fn copy_message() {
    let access_token = access_token().await;
    let url = format!(
        "https://graph.microsoft.com/v1.0/me/messages/{}/copy",
        "AQMkADAwATMwMAItNzE0OC1jZTEzLTAwAi0wMAoARgAAA_KUk7xWPSBEntPHShr61lgHAOo9V4GwHndCjf0x1uoIcwUAAAIBDAAAAOo9V4GwHndCjf0x1uoIcwUAAYiJVIEAAAA="
    );
    let client = reqwest::Client::builder()
        .user_agent(rustmailer_version!())
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(10))
        .proxy(reqwest::Proxy::all("http://127.0.0.1:22307").unwrap())
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let data = json!({
      "destinationId": "AQMkADAwATMwMAItNzE0OC1jZTEzLTAwAi0wMAoALgAAA_KUk7xWPSBEntPHShr61lgBAOo9V4GwHndCjf0x1uoIcwUAAYiJQtoAAAA="
    });

    let res = client
        .post(&url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .json(&data)
        .send()
        .await
        .unwrap();

    if res.status().is_success() {
        let body: serde_json::Value = res.json().await.unwrap();
        println!("{:#?}", body);
    } else {
        eprintln!("Error: {} - {:?}", res.status(), res.text().await.unwrap());
    }
}

#[tokio::test]
async fn move_message() {
    let access_token = access_token().await;
    let url = format!(
        "https://graph.microsoft.com/v1.0/me/messages/{}/move",
        "AQMkADAwATMwMAItNzE0OC1jZTEzLTAwAi0wMAoARgAAA_KUk7xWPSBEntPHShr61lgHAOo9V4GwHndCjf0x1uoIcwUAAAIBDAAAAOo9V4GwHndCjf0x1uoIcwUAAYiJVIEAAAA="
    );
    let client = reqwest::Client::builder()
        .user_agent(rustmailer_version!())
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(10))
        .proxy(reqwest::Proxy::all("http://127.0.0.1:22307").unwrap())
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let data = json!({
      "destinationId": "AQMkADAwATMwMAItNzE0OC1jZTEzLTAwAi0wMAoALgAAA_KUk7xWPSBEntPHShr61lgBAOo9V4GwHndCjf0x1uoIcwUAAYiJQtoAAAA="
    });

    let res = client
        .post(&url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .json(&data)
        .send()
        .await
        .unwrap();

    if res.status().is_success() {
        let body: serde_json::Value = res.json().await.unwrap();
        println!("{:#?}", body);
    } else {
        eprintln!("Error: {} - {:?}", res.status(), res.text().await.unwrap());
    }
}

#[tokio::test]
async fn get_thread_messages() {
    let access_token = access_token().await;
    let mut url =
        "https://graph.microsoft.com/v1.0/me/mailFolders/inbox/messages?$top=3&\
               $orderBy=receivedDateTime desc&\
               $filter=receivedDateTime ge 2025-10-01T00:00:00Z&\
               $select=id,isRead,conversationId,internetMessageId,from,body,toRecipients,ccRecipients,\
               bccRecipients,replyTo,sender,subject,receivedDateTime,sentDateTime,isRead,bodyPreview,categories&\
               $expand=attachments($select=id,name,contentType,size,isInline,microsoft.graph.fileAttachment/contentId)"
            .to_string();
    let client = reqwest::Client::builder()
        .user_agent(rustmailer_version!())
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(10))
        .proxy(reqwest::Proxy::all("http://127.0.0.1:22307").unwrap())
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let mut nextlink_count = 0;

    let res = client
        .get(&url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await
        .unwrap();

    if res.status().is_success() {
        let body: MessageListResponse = res.json().await.unwrap();
        println!("{:#?}", body);
    } else {
        eprintln!("Error: {} - {:?}", res.status(), res.text().await.unwrap());
    }
}

#[tokio::test]
async fn list_categories() {
    let access_token = access_token().await;
    let url = "https://graph.microsoft.com/v1.0/me/outlook/masterCategories".to_string();
    let client = reqwest::Client::builder()
        .user_agent(rustmailer_version!())
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(10))
        .proxy(reqwest::Proxy::all("http://127.0.0.1:22307").unwrap())
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let res = client
        .get(&url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await
        .unwrap();

    if res.status().is_success() {
        let body: CategoriesListResponse = res.json().await.unwrap();
        println!("{:#?}", body);
    } else {
        eprintln!("Error: {} - {:?}", res.status(), res.text().await.unwrap());
    }
}

#[tokio::test]
async fn create_categories() {
    let access_token = access_token().await;
    let url = "https://graph.microsoft.com/v1.0/me/outlook/masterCategories".to_string();
    let client = reqwest::Client::builder()
        .user_agent(rustmailer_version!())
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(10))
        .proxy(reqwest::Proxy::all("http://127.0.0.1:22307").unwrap())
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let data = json!({
      "color": "Preset19",
      "displayName": "hangzhou"
    });

    let res = client
        .post(&url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .json(&data)
        .send()
        .await
        .unwrap();

    if res.status().is_success() {
        let body: serde_json::Value = res.json().await.unwrap();
        println!("{:#?}", body);
    } else {
        eprintln!("Error: {} - {:?}", res.status(), res.text().await.unwrap());
    }
}
