// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use rand::Rng;

use crate::utc_now;

pub fn generate_message_id() -> String {
    // Generate 16 random bytes
    let random_bytes: [u8; 16] = rand::rng().random();
    // Convert to hex
    let random_id = hex::encode(random_bytes);
    // Get current timestamp in milliseconds
    let timestamp_millis = utc_now!();
    // Format the message ID
    format!("<{}.{}@rustmailer>", timestamp_millis, random_id)
}

#[cfg(test)]
mod test {
    use crate::modules::smtp::util::generate_message_id;
    #[test]
    fn test1() {
        println!("{}", generate_message_id());
    }
}
