// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use super::RustMailerEvent;

#[test]
fn test1() {
    let examples = RustMailerEvent::generate_event_examples();
    println!("{:#?}", examples);
}
