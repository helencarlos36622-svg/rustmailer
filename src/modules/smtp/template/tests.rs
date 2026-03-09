// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use handlebars::Handlebars;
use serde_json::json;

#[test]
fn test1() {
    let mut handlebars = Handlebars::new();

    let template = r#"
        Users:
        {{#each users}}
        - Name: {{name}}, Email: {{email}}
        {{/each}}
    "#;
    handlebars
        .register_template_string("user_list", template)
        .unwrap();

    let data = json!({
        "users": [
            { "name": "John Doe", "email": "johndoe@example.com" },
            { "name": "Jane Smith", "email": "janesmith@example.com" }
        ]
    });

    let rendered = handlebars.render("user_list", &data).unwrap();
    println!("{}", rendered);
}
