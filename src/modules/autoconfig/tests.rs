// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

#[tokio::test]
async fn test() {
    let config = autoconfig::from_addr("test@163.com").await.unwrap();
    println!("{:#?}", config);
}
