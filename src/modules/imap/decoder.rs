// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use mail_parser::parsers::MessageStream;

pub fn try_decode_string(encoded: &str) -> String {
    // Check the string format
    if encoded.starts_with("=?") && encoded.ends_with("?=") {
        // Remove the first equals sign and try to decode
        let modified_encoded = &encoded[1..]; // Only take the slice
        if let Some(result) = MessageStream::new(modified_encoded.as_bytes()).decode_rfc2047() {
            return result; // Return the result if decoding is successful
        }
    }
    // If the format does not match or decoding fails, return the original string
    encoded.to_string()
}
