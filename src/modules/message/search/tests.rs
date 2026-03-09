// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

#[cfg(test)]
mod tests {
    use crate::modules::message::search::payload::{
        Condition, Conditions, Logic, MessageSearch, Operator,
    };
    fn cond(condition: Conditions, value: &str) -> MessageSearch {
        MessageSearch::Condition(Condition {
            condition,
            value: Some(value.to_string()),
        })
    }

    fn logic(operator: Operator, children: Vec<MessageSearch>) -> MessageSearch {
        MessageSearch::Logic(Logic { operator, children })
    }

    #[test]
    fn test_basic_conditions() {
        assert_eq!(
            cond(Conditions::From, "test@example.com")
                .to_imap_command(false)
                .unwrap(),
            "FROM \"test@example.com\""
        );

        assert_eq!(
            cond(Conditions::Subject, "hello")
                .to_imap_command(false)
                .unwrap(),
            "SUBJECT \"hello\""
        );

        assert_eq!(
            cond(Conditions::Seen, "").to_imap_command(false).unwrap(),
            "SEEN"
        );

        assert_eq!(
            cond(Conditions::Uid, "123:456")
                .to_imap_command(false)
                .unwrap(),
            "UID 123:456"
        );
    }

    #[test]
    fn test_simple_and() {
        let search = logic(
            Operator::And,
            vec![
                cond(Conditions::From, "a@example.com"),
                cond(Conditions::Subject, "urgent"),
            ],
        );

        assert_eq!(
            search.to_imap_command(true).unwrap(),
            "FROM \"a@example.com\" SUBJECT \"urgent\""
        );
    }

    #[test]
    fn test_simple_or() {
        let search = logic(
            Operator::Or,
            vec![
                cond(Conditions::From, "a@example.com"),
                cond(Conditions::From, "b@example.com"),
            ],
        );

        assert_eq!(
            search.to_imap_command(true).unwrap(),
            "OR FROM \"a@example.com\" FROM \"b@example.com\""
        );
    }

    #[test]
    fn test_simple_not() {
        let search = logic(Operator::Not, vec![cond(Conditions::Seen, "")]);

        assert_eq!(search.to_imap_command(true).unwrap(), "NOT SEEN");
    }

    #[test]
    fn test_nested_and_or() {
        let search = logic(
            Operator::And,
            vec![
                cond(Conditions::From, "boss@company.com"),
                logic(
                    Operator::Or,
                    vec![
                        cond(Conditions::Subject, "urgent"),
                        cond(Conditions::Keyword, "important"),
                    ],
                ),
            ],
        );

        assert_eq!(
            search.to_imap_command(true).unwrap(),
            "FROM \"boss@company.com\" (OR SUBJECT \"urgent\" KEYWORD \"important\")"
        );
    }

    #[test]
    fn test_deeply_nested_or() {
        let search = logic(
            Operator::Or,
            vec![
                cond(Conditions::From, "a@example.com"),
                cond(Conditions::From, "b@example.com"),
                cond(Conditions::From, "c@example.com"),
            ],
        );

        assert_eq!(
            search.to_imap_command(true).unwrap(),
            "OR FROM \"a@example.com\" (OR FROM \"b@example.com\" FROM \"c@example.com\")"
        );
    }

    #[test]
    fn test_empty_or() {
        let search = logic(Operator::Or, vec![]);
        assert!(search.to_imap_command(true).is_err());
    }

    #[test]
    fn test_single_or() {
        let search = logic(Operator::Or, vec![cond(Conditions::Seen, "")]);
        assert!(search.to_imap_command(true).is_err());
    }

    #[test]
    fn test_not_with_multiple_children() {
        let search = logic(
            Operator::Not,
            vec![cond(Conditions::Seen, ""), cond(Conditions::Flagged, "")],
        );
        assert!(search.to_imap_command(true).is_err());
    }

    #[test]
    fn test_complex_combination() {
        let search = logic(
            Operator::And,
            vec![
                cond(Conditions::From, "team@org.com"),
                logic(Operator::Not, vec![cond(Conditions::Deleted, "")]),
                logic(
                    Operator::Or,
                    vec![
                        cond(Conditions::Subject, "meeting"),
                        cond(Conditions::Body, "agenda"),
                        logic(
                            Operator::And,
                            vec![
                                cond(Conditions::Since, "2023-01-01"),
                                cond(Conditions::Before, "2023-02-01"),
                            ],
                        ),
                    ],
                ),
            ],
        );

        assert_eq!(
            search.to_imap_command(true).unwrap(),
            "FROM \"team@org.com\" (NOT DELETED) (OR SUBJECT \"meeting\" (OR BODY \"agenda\" (SINCE 01-Jan-2023 BEFORE 01-Feb-2023)))"
        );
    }

    #[test]
    fn test_date_conditions() {
        assert_eq!(
            cond(Conditions::Since, "2023-01-15")
                .to_imap_command(false)
                .unwrap(),
            "SINCE \"15-Jan-2023\""
        );

        assert_eq!(
            cond(Conditions::Larger, "1024")
                .to_imap_command(false)
                .unwrap(),
            "LARGER 1024"
        );

        assert!(cond(Conditions::Before, "invalid-date")
            .to_imap_command(false)
            .is_err());

        assert!(cond(Conditions::Smaller, "not-a-number")
            .to_imap_command(false)
            .is_err());
    }
}
