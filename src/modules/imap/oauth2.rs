// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

#[derive(Debug)]
pub struct OAuth2 {
    user: String,
    access_token: String,
}

impl OAuth2 {
    pub fn new(user: String, access_token: String) -> Self {
        Self { user, access_token }
    }
}

impl async_imap::Authenticator for OAuth2 {
    type Response = String;

    fn process(&mut self, _data: &[u8]) -> Self::Response {
        format!(
            "user={}\x01auth=Bearer {}\x01\x01",
            self.user, self.access_token
        )
    }
}
