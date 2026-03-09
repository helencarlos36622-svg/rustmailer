// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.


// These bounce subject prefixes are inspired by the go-sisimai project:
// https://github.com/sisimai/go-sisimai
// They are used to identify common bounce or feedback message subjects.
pub const BOUNCE_PREFIXES: [&str; 32] = [
    "abuse-report",
    "auto",
    "auto-reply",
    "automatic-reply",
    "aws-notification",
    "complaint-about",
    "delivery-failure",
    "delivery-notification",
    "delivery-status",
    "dmarc-ietf-dmarc",
    "email-feedback",
    "failed-delivery",
    "failure-delivery",
    "failure-notice",
    "loop-alert",
    "mail-could",
    "mail-delivery",
    "mail-failure",
    "mail-system",
    "message-delivery",
    "message-frozen",
    "non-recapitabile",
    "non-remis",
    "notice",
    "postmaster-notify",
    "returned-mail",
    "there-was",
    "undeliverable",
    "undeliverable-mail",
    "undeliverable-message",
    "undelivered-mail",
    "warning",
];

pub fn analyze_subject_for_bounce(subject: Option<String>) -> bool {
    let subject = match subject {
        Some(s) if !s.is_empty() => s,
        _ => return false,
    };

    // Normalize subject line in one pass
    let subject = subject.trim().to_lowercase();

    // Remove forwarding prefixes more efficiently
    let subject = subject
        .strip_prefix("fwd:")
        .or_else(|| subject.strip_prefix("fw:"))
        .unwrap_or(&subject)
        .trim();

    // Clean up special characters only if needed
    let subject = if subject.contains(&['[', ']', '_'][..]) {
        subject.replace(&['[', ']', '_'][..], " ")
    } else {
        subject.to_string()
    };

    // Normalize whitespace more efficiently
    let subject = subject.split_whitespace().collect::<Vec<_>>().join(" ");

    // Extract title parts with less allocation
    let mut words = subject.splitn(3, ' ');
    let first_word = words.next().unwrap_or("");

    let title = if let Some(colon_pos) = first_word.find(':') {
        &first_word[..colon_pos]
    } else {
        let part2 = words.next().unwrap_or("");
        if part2.is_empty() {
            first_word
        } else {
            &format!("{}-{}", first_word, part2)
        }
    };

    // Final title cleanup without allocation if possible
    let title = title.trim_matches(|c| matches!(c, ':' | ',' | '*' | '"'));

    // Check against known prefixes
    BOUNCE_PREFIXES.contains(&title)
}
