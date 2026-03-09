// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use mail_send::smtp::message::IntoMessage;
use mail_send::{mail_builder::MessageBuilder, SmtpClientBuilder};

pub const EXT_DSN: u32 = 1 << 10;

#[tokio::test]
async fn test1() {
    rustls::crypto::CryptoProvider::install_default(rustls::crypto::ring::default_provider())
        .expect("failed to set crypto provider");
    let mut client = SmtpClientBuilder::new("smtp.zoho.com", 465)
        .implicit_tls(true)
        .credentials(("test1@zohomail.com", "xxxxxxxxxx"))
        .connect()
        .await
        .unwrap();
    let response = client.capabilities("smtp.zoho.com", false).await.unwrap();
    println!("{:#?}", response);

    //let capabilities = response.capabilities;
    let message_builder = MessageBuilder::new()
        .from(("John Doe1", "test1@zohomail.com"))
        .to(vec![("dongbin", "test1@gmail.com")])
        .subject(format!("mail send test{}", 9999))
        .html_body("<html><body><div style=\"display: none; font-size: 0; max-height: 0; overflow: hidden;\">Preview with &lt;b&gt;bold&lt;/b&gt; &amp; &quot;quotes&quot;</div><div>Main content</div></body></html>")
        ;
    let message = message_builder.into_message().unwrap();
    client.send(message).await.unwrap();
    // client.write_message(&vec).await.unwrap();
}
