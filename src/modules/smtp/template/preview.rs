// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

pub struct EmailPreview;

impl EmailPreview {
    pub fn insert_preview_into_html(html: &str, preview: &str) -> String {
        let body_open = "<body";
        if let Some(body_start) = html.find(body_open) {
            let chars = html[body_start + body_open.len()..].chars();
            let mut tag_end = body_start + body_open.len();
            for c in chars {
                tag_end += c.len_utf8();
                if c == '>' {
                    break;
                }
            }
            let preview_div = format!(
                "<div style=\"display: none; font-size: 0; max-height: 0; overflow: hidden;\">{}</div>",
                Self::escape_html(preview)
            );
            return format!("{}{}{}", &html[..tag_end], preview_div, &html[tag_end..]);
        }
        format!(
            "<div style=\"display: none; font-size: 0; max-height: 0; overflow: hidden;\">{}</div>{}",
            Self::escape_html(preview),
            html
        )
    }

    fn escape_html(text: &str) -> String {
        text.chars()
            .map(|c| match c {
                '<' => "&lt;".to_string(),
                '>' => "&gt;".to_string(),
                '&' => "&amp;".to_string(),
                '"' => "&quot;".to_string(),
                '\'' => "&#x27;".to_string(),
                _ => c.to_string(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::EmailPreview;

    #[test]
    fn test_insert_preview_with_body() {
        let html = "<html><body><p>Hello World</p></body></html>";
        let preview = "This is a preview!";
        let result = EmailPreview::insert_preview_into_html(html, preview);

        let expected = "<html><body><div style=\"display: none; font-size: 0; max-height: 0; overflow: hidden;\">This is a preview!</div><p>Hello World</p></body></html>";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_insert_preview_with_body_attributes() {
        let html = "<html><body class=\"dark\" style=\"color: blue;\"><p>Content</p></body></html>";
        let preview = "Preview text here";
        let result = EmailPreview::insert_preview_into_html(html, preview);

        let expected = "<html><body class=\"dark\" style=\"color: blue;\"><div style=\"display: none; font-size: 0; max-height: 0; overflow: hidden;\">Preview text here</div><p>Content</p></body></html>";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_insert_preview_no_body() {
        let html = "<p>Simple content</p>";
        let preview = "No body preview";
        let result = EmailPreview::insert_preview_into_html(html, preview);

        let expected = "<div style=\"display: none; font-size: 0; max-height: 0; overflow: hidden;\">No body preview</div><p>Simple content</p>";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_insert_preview_with_html_entities() {
        let html = "<html><body><div>Main content</div></body></html>";
        let preview = "Preview with <b>bold</b> & \"quotes\"";
        let result = EmailPreview::insert_preview_into_html(html, preview);

        let expected = "<html><body><div style=\"display: none; font-size: 0; max-height: 0; overflow: hidden;\">Preview with &lt;b&gt;bold&lt;/b&gt; &amp; &quot;quotes&quot;</div><div>Main content</div></body></html>";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_empty_html() {
        let html = "";
        let preview = "Empty preview";
        let result = EmailPreview::insert_preview_into_html(html, preview);

        let expected = "<div style=\"display: none; font-size: 0; max-height: 0; overflow: hidden;\">Empty preview</div>";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_preview_with_newlines() {
        let html = "<html><body><p>Text</p></body></html>";
        let preview = "Line1\nLine2";
        let result = EmailPreview::insert_preview_into_html(html, preview);

        let expected = "<html><body><div style=\"display: none; font-size: 0; max-height: 0; overflow: hidden;\">Line1\nLine2</div><p>Text</p></body></html>";
        assert_eq!(result, expected);
    }
}
