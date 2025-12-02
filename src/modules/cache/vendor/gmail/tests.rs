// Copyright © 2025 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use ahash::AHashMap;
use mail_send::{
    mail_builder::{headers::address::Address, MessageBuilder},
    smtp::message::IntoMessage,
};
use poem_grpc::{ClientConfig, CompressionEncoding};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};
use std::{borrow::Cow, time::Duration};

use crate::{
    base64_encode,
    modules::{
        cache::{
            imap::migration::EmailEnvelopeV3,
            vendor::gmail::{
                model::{
                    history::HistoryList,
                    messages::{FullMessage, MessageList, MessageMeta, PartBody},
                    thread::{ThreadList, ThreadMessages},
                },
                sync::envelope::GmailEnvelope,
            },
        },
        common::{rustls::RustMailerTls, Addr},
        context::Initialize,
        grpc::service::rustmailer_grpc::{GetOAuth2TokensRequest, OAuth2ServiceClient},
    },
    rustmailer_version,
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
        account_id: 4727355769996270,
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
    let url = "https://gmail.googleapis.com/gmail/v1/users/me/labels/Label_6886728075529239043";
    let url =
        "https://gmail.googleapis.com/gmail/v1/users/me/messages/198e590baf688394?format=full";
    let url =
        "https://gmail.googleapis.com/gmail/v1/users/me/messages?labelIds=INBOX&maxResults=10";
    let url = "https://gmail.googleapis.com/gmail/v1/users/me/labels/Label_6886728075529239043";
    let url = "https://gmail.googleapis.com/gmail/v1/users/me/labels";
    let url = "https://gmail.googleapis.com/gmail/v1/users/me/messages?labelIds=SENT&maxResults=20";

    let url = "https://gmail.googleapis.com/gmail/v1/users/me/labels";
    let url =
        "https://gmail.googleapis.com/gmail/v1/users/me/messages/19961e858d35c4e6?format=full";
    let mut builder = reqwest::ClientBuilder::new()
        .user_agent(rustmailer_version!())
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(10));

    let proxy_obj = reqwest::Proxy::all("socks5://127.0.0.1:22308").unwrap();
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
        let body: Value = res.json().await.unwrap();
        let pretty = serde_json::to_string_pretty(&body).unwrap();
        println!("Response = {}", pretty);
    } else {
        eprintln!("Error: {} - {:?}", res.status(), res.text().await.unwrap());
    }
}

#[tokio::test]
async fn test11() {
    let access_token = access_token().await;
    let url = "https://gmail.googleapis.com/gmail/v1/users/me/labels/Label_6886728075529239043";
    let url =
        "https://gmail.googleapis.com/gmail/v1/users/me/messages/198e590baf688394?format=full";
    let url =
        "https://gmail.googleapis.com/gmail/v1/users/me/messages?labelIds=INBOX&maxResults=10";
    let url = "https://gmail.googleapis.com/gmail/v1/users/me/labels/Label_6886728075529239043";
    let url = "https://gmail.googleapis.com/gmail/v1/users/me/labels";
    let url = "https://gmail.googleapis.com/gmail/v1/users/me/messages?labelIds=SENT&maxResults=20";
    let url =
        "https://gmail.googleapis.com/gmail/v1/users/me/messages/1980d79e5bb7c585/attachments/ANGjdJ-eQccvPmRcUJhAEUrokYpWYnvrhpW1pq0nB75AsZ-WUXO_Yvxp8Y8WaYCW1WF0JwFYT3hFxnx2g2H1CAlAeUKa5BIJqjA-DVnT-kIACeVvq5gWnLrZJ6ux9_DW7OeApbTaLG1iVR0KU-rK986vfRJKK4g1JTqXDfQUU4D23B6qciJTkLHFTMcJ-HaO1FQ6Gj3LB-InOYz2oeSeuqJbEjTJetmIXsoPKy2b5Fzg1Eu3H1PwjqFC1WbLFwM8iAPLccrPe6SxqgpI1suPRkhRiqxvr5PyIohyFATp150Mnt7ec4TBInksFpihpTxK0cMfdti4gYv3eoGe2eDtgu5dcRBcSHJicWKN0VXLcCiqLgUo4mm4m6dy6-eU3v1BybESPZWuGMLEBjQWt4T6";
    let mut builder = reqwest::ClientBuilder::new()
        .user_agent(rustmailer_version!())
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(10));

    let proxy_obj = reqwest::Proxy::all("socks5://127.0.0.1:22308").unwrap();
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
        let body: Value = res.json().await.unwrap();
        let pretty = serde_json::to_string_pretty(&body).unwrap();
        let body: PartBody = serde_json::from_value(body).unwrap();
        println!("Response = {:#?}", body);
    } else {
        eprintln!("Error: {} - {:?}", res.status(), res.text().await.unwrap());
    }
}

#[tokio::test]
async fn test2() {
    let access_token = access_token().await;
    let url =
        "https://gmail.googleapis.com/gmail/v1/users/me/messages?labelIds=INBOX&q=before:2025/08/28&maxResults=20";
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
        let body: Value = res.json().await.unwrap();
        let detail: MessageList = serde_json::from_value(body).unwrap();
        println!("Response = {:#?}", detail);
    } else {
        eprintln!("Error: {} - {:?}", res.status(), res.text().await.unwrap());
    }
}

#[tokio::test]
async fn test3() {
    let access_token = access_token().await;
    let url =
        "https://gmail.googleapis.com/gmail/v1/users/me/messages/19abb5fe57ca4856?format=metadata&metadataHeaders=Message-Id&metadataHeaders=From&metadataHeaders=To&metadataHeaders=Cc&metadataHeaders=Bcc&metadataHeaders=Subject&metadataHeaders=Date&metadataHeaders=Mime-Version&metadataHeaders=Reply-To&metadataHeaders=In-Reply-To&metadataHeaders=References";
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
        let body: Value = res.json().await.unwrap();
        let detail: MessageMeta = serde_json::from_value(body).unwrap();
        let envelope: GmailEnvelope = detail.try_into().unwrap();
        println!("Response = {:#?}", envelope);
        let envelope: EmailEnvelopeV3 = envelope.into_v3(&AHashMap::new());
        println!("Response = {:#?}", envelope);
    } else {
        eprintln!("Error: {} - {:?}", res.status(), res.text().await.unwrap());
    }
}

#[tokio::test]
async fn test4() {
    let access_token = access_token().await;
    let url =
        "https://gmail.googleapis.com/gmail/v1/users/me/history?startHistoryId=45284&labelId=INBOX&maxResults=2";
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
        let body: Value = res.json().await.unwrap();
        let json = serde_json::to_string_pretty(&body).unwrap();
        println!("Response = {}", json);
        let list: HistoryList = serde_json::from_value(body).unwrap();
        println!("Response = {:#?}", list);
    } else {
        //eprintln!("Error: {} - {:?}", res.status(), res.text().await.unwrap());
        let text = res.text().await.unwrap();
        if text.contains("Requested entity was not found") {
            println!("history id is invalid");
        }
    }
}

#[tokio::test]
async fn test5() {
    let examples = [
        "Quinn Eckart <jira@lifebuoy.atlassian.net>",
        "justemail@example.com",
        "<only@example.com>",
    ];

    for s in examples {
        let addr = Addr::parse(s);
        println!("{:?}", addr);
    }
}

#[tokio::test]
async fn test6() {
    let access_token = access_token().await;
    let url = "https://gmail.googleapis.com/gmail/v1/users/me/drafts";
    let mut builder = reqwest::ClientBuilder::new()
        .user_agent(rustmailer_version!())
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(10));

    let proxy_obj = reqwest::Proxy::all("socks5://127.0.0.1:22308").unwrap();
    builder = builder
        .redirect(reqwest::redirect::Policy::none())
        .proxy(proxy_obj);
    let client = builder.build().unwrap();

    let from = Address::new_address(
        Some(Cow::Owned("rustmailer".to_string())),
        Cow::Owned("rustmailer.git@gmail.com".to_string()),
    );
    let to = Address::new_address(
        Some(Cow::Owned("noreply".to_string())),
        Cow::Owned("noreply@medium.com".to_string()),
    );
    let subject = "Re: 👋 Welcome to Medium".to_string();
    let mut builder = MessageBuilder::new()
        .from(from)
        .to(Address::from(to.clone()))
        .subject(subject.clone());

    builder = builder.in_reply_to("5JgiBu1_TSC_RIro8-xLWg@geopod-ismtpd-4".to_string());
    let references = vec!["5JgiBu1_TSC_RIro8-xLWg@geopod-ismtpd-4".to_string()];
    builder = builder.references(references);
    builder = builder.text_body("wowwowwowwowwowwowwowwowwowwowwowwow");

    let message = builder.into_message().unwrap();

    let raw_encoded = base64_encode!(&message.body);

    let body = json!({
        "message": {
            "threadId": "19720f0b9bd3822c",
            "raw": raw_encoded
        }
    });

    let res = client
        .post(url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .json(&body)
        .send()
        .await
        .unwrap();

    if res.status().is_success() {
        let body: Value = res.json().await.unwrap();
        let json = serde_json::to_string_pretty(&body).unwrap();
        println!("Response = {}", json);
    } else {
        eprintln!("Error: {} - {:?}", res.status(), res.text().await.unwrap());
    }
}

#[tokio::test]
async fn test7() {
    let access_token = access_token().await;
    let url = "https://gmail.googleapis.com/gmail/v1/users/me/labels";
    let mut builder = reqwest::ClientBuilder::new()
        .user_agent(rustmailer_version!())
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(10));

    let proxy_obj = reqwest::Proxy::all("socks5://127.0.0.1:22308").unwrap();
    builder = builder
        .redirect(reqwest::redirect::Policy::none())
        .proxy(proxy_obj);
    let client = builder.build().unwrap();

    let body = json!({
          "name": "test_label_name1",
          "messageListVisibility": "show",
          "labelListVisibility": "labelShow",
          "type": "user",
          "color": {
            "textColor": "#e66550",
            "backgroundColor": "#fcdee8"
          }
    });

    let res = client
        .post(url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .json(&body)
        .send()
        .await
        .unwrap();

    if res.status().is_success() {
        let body: Value = res.json().await.unwrap();
        let json = serde_json::to_string_pretty(&body).unwrap();
        println!("Response = {}", json);
    } else {
        eprintln!("Error: {} - {:?}", res.status(), res.text().await.unwrap());
    }
}

#[tokio::test]
async fn test8() {
    let access_token = access_token().await;
    let url = "https://gmail.googleapis.com/gmail/v1/users/me/labels/Label_4";
    let mut builder = reqwest::ClientBuilder::new()
        .user_agent(rustmailer_version!())
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(10));

    let proxy_obj = reqwest::Proxy::all("socks5://127.0.0.1:22308").unwrap();
    builder = builder
        .redirect(reqwest::redirect::Policy::none())
        .proxy(proxy_obj);
    let client = builder.build().unwrap();

    let res = client
        .delete(url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await
        .unwrap();

    if res.status().is_success() {
        // let body: Value = res.json().await.unwrap();
        // let json = serde_json::to_string_pretty(&body).unwrap();
        // println!("Response = {}", json);
    } else {
        eprintln!("Error: {} - {:?}", res.status(), res.text().await.unwrap());
    }
}

#[tokio::test]
async fn test9() {
    let access_token = access_token().await;
    let url = format!(
        "https://gmail.googleapis.com/gmail/v1/users/me/messages/19961e858d35c4e6?format=raw"
    );
    let mut builder = reqwest::ClientBuilder::new()
        .user_agent(rustmailer_version!())
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(10));

    let proxy_obj = reqwest::Proxy::all("socks5://127.0.0.1:22308").unwrap();
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
        let body: Value = res.json().await.unwrap();
        let json = serde_json::to_string_pretty(&body).unwrap();
        println!("Response = {}", json);

        let full_message: FullMessage = serde_json::from_value(body).unwrap();
        println!("Response = {:#?}", full_message);
    } else {
        eprintln!("Error: {} - {:?}", res.status(), res.text().await.unwrap());
    }
}

#[tokio::test]
async fn list_threads() {
    let access_token = access_token().await;
    let url =
        "https://gmail.googleapis.com/gmail/v1/users/me/threads?labelIds=INBOX&q=after:2025/07/28&maxResults=20";
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
        let body: Value = res.json().await.unwrap();
        //println!("Response = {:#?}", body);
        let list: ThreadList = serde_json::from_value(body).unwrap();
        println!("Response = {:#?}", list);
    } else {
        eprintln!("Error: {} - {:?}", res.status(), res.text().await.unwrap());
    }
}

#[tokio::test]
async fn test13() {
    let access_token = access_token().await;
    let url =
        "https://gmail.googleapis.com/gmail/v1/users/me/threads/19ad6ed3d0afba92?format=metadata&metadataHeaders=Message-Id&metadataHeaders=From&metadataHeaders=To&metadataHeaders=Cc&metadataHeaders=Bcc&metadataHeaders=Subject&metadataHeaders=Date&metadataHeaders=Mime-Version&metadataHeaders=Reply-To&metadataHeaders=In-Reply-To&metadataHeaders=References";
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
        let body: Value = res.json().await.unwrap();
        println!("Response = {:#?}", body);
        let messages: ThreadMessages = serde_json::from_value(body).unwrap();
        println!("Response = {:#?}", messages);

        // let envelope: GmailEnvelope = detail.try_into().unwrap();
        // println!("Response = {:#?}", envelope);
    } else {
        eprintln!("Error: {} - {:?}", res.status(), res.text().await.unwrap());
    }
}
