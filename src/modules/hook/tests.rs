// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use std::collections::BTreeMap;
use vrl::{
    compiler::{compile, state::RuntimeState, Context, TargetValue, TimeZone},
    value,
    value::{Secrets, Value},
};

use crate::{
    id,
    modules::{
        common::Addr,
        hook::{
            events::{payload::MailboxDeletion, EventPayload, EventType, RustMailerEvent},
            nats::{executor::NATS_EXECUTORS, NatsAuthType, NatsConfig},
        },
    },
    utc_now,
};

// Helper function to run a VRL script and return the result as a Value
fn run_vrl_script(input: Value, script: &str) -> Result<Value, String> {
    let fns = vrl::stdlib::all();
    let result = compile(script, &fns).map_err(|e| format!("Compile error: {:#?}", e))?;

    let mut target = TargetValue {
        value: input,
        metadata: Value::Object(BTreeMap::new()),
        secrets: Secrets::default(),
    };
    let mut state = RuntimeState::default();
    let timezone = TimeZone::default();
    let mut ctx = Context::new(&mut target, &mut state, &timezone);

    result
        .program
        .resolve(&mut ctx)
        .map_err(|e| format!("Runtime error: {}", e))
}

// Helper function to create mock email input
fn mock_email(from: &str, subject: &str, body: &str) -> Value {
    value!({
        "tenant_id": "tenant_a",
        "email_id": "123",
        "from": from,
        "subject": subject,
        "body": body
    })
}
#[test]
fn test_classify_order_email() {
    // Scenario: Classify emails with "order" in subject as "order"
    let input = mock_email(
        "seller@alibaba.com",
        "Order Confirmation",
        "Order No: ABC123\nAmount: 1000",
    );
    let script = r#"
        if contains(string!(.subject), "Order") {
            .category = "order"
        } else {
            .category = "other"
        }
        .category
    "#;

    let result = run_vrl_script(input, script).unwrap();
    assert_eq!(result, value!("order"));
}

#[test]
fn test_classify_non_order_email() {
    // Scenario: Classify non-order emails as "other"
    let input = mock_email(
        "promo@alibaba.com",
        "Promotion",
        "Welcome to our 11.11 sale!",
    );
    let script = r#"
        if contains(string!(.subject), "Order") {
            .category = "order"
        } else {
            .category = "other"
        }
        .category
    "#;

    let result = run_vrl_script(input, script).unwrap();
    assert_eq!(result, value!("other"));
}

#[test]
fn test_extract_order_id() {
    // Scenario: Extract order ID from email body
    let input = mock_email(
        "seller@alibaba.com",
        "Order Confirmation",
        "Order No: ABC123\nAmount: 1000",
    );
    let script = r#"
        if contains(string!(.body), "Order No") {
            .category = "order"
            . |= parse_regex!(.body, r'Order No: (?P<order_id>[0-9a-fA-F]+)')
        } else {
            .category = "other"
        }
        .order_id
    "#;

    let result = run_vrl_script(input, script).unwrap();
    assert_eq!(result, value!("ABC123"));
}

#[test]
fn test_extract_multiple_fields() {
    // Scenario: Extract order ID and amount, return as object
    let input = mock_email(
        "seller@alibaba.com",
        "Order Confirmation",
        "Order No: XYZ789\nAmount: 2000",
    );
    let script = r#"
        if contains(string!(.body), "Order No") {
            .category = "order"
            . |= parse_regex!(.body, r'Order No: (?P<order_id>[0-9a-zA-Z]+)')
            . |= parse_regex!(.body, r'Amount: (?P<amount>[0-9a-zA-Z]+)')
        } else {
            .category = "other"
        }
        { "order_id": .order_id, "amount": .amount }
    "#;

    let result = run_vrl_script(input, script).unwrap();
    assert_eq!(result, value!({ "order_id": "XYZ789", "amount": "2000" }));
}

#[test]
fn test_filter_spam_email() {
    // Scenario: Drop spam emails from specific domains
    let input = mock_email(
        "noreply@spam.com",
        "Promotion Offer",
        "Click now to get your coupon!",
    );
    let script = r#"
        if contains(string!(.from), "spam.com") {
            .category = null
        } else {
            .category = "inbox"
        }
        .category
    "#;

    let result = run_vrl_script(input, script).unwrap();
    assert_eq!(result, value!(null));
}

#[test]
fn test_handle_missing_regex_match() {
    // Scenario: Handle emails without order ID
    let input = mock_email(
        "support@alibaba.com",
        "Customer Support Reply",
        "Thank you for your feedback!",
    );
    let script = r#"
        .category = "other"
        .order_id = null
        if match(string!(.body), r'Order No: [0-9a-zA-Z]+') {
            . |= parse_regex!(.body, r'Order No: (?P<order_id>[0-9a-zA-Z]+)')
        }
        .order_id
    "#;

    let result = run_vrl_script(input, script).unwrap();
    assert_eq!(result, value!(null));
}

#[tokio::test]
async fn test_create_jetstream_producer_and_send_message() {
    let config = NatsConfig {
        host: "127.0.0.1".to_string(),
        port: 4222,
        auth_type: NatsAuthType::None,
        token: None,
        username: None,
        password: None,
        stream_name: "test_stream".to_string(),
        namespace: "test.ns".to_string(),
    };

    let nats = NATS_EXECUTORS.get(&config).await.unwrap();

    let addr = |email: &str| Addr {
        name: Some("Test User".to_string()),
        address: Some(email.to_string()),
    };

    let payload = MailboxDeletion {
        account_id: id!(64),
        account_email: "test@example.com".into(),
        mailbox_names: vec!["test_mailbox".into()],
    };

    let event = RustMailerEvent {
        event_id: id!(96),
        event_type: EventType::MailboxDeletion,
        instance_url: "http://localhost:15630".into(),
        timestamp: utc_now!(),
        payload: EventPayload::MailboxDeletion(payload),
    };

    nats.publish(
        None,
        EventType::MailboxDeletion,
        event.to_json_value().unwrap(),
    )
    .await
    .expect("Failed to publish message");

    let info = nats.stream_info().await.expect("Failed to get stream");
    println!("Current message count in stream: {}", info.state.messages);
}
