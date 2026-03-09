// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::error::code::ErrorCode;
use crate::modules::smtp::template::entity::{EmailTemplate, MessageFormat};
use crate::modules::smtp::template::preview::EmailPreview;
use crate::{modules::error::RustMailerResult, raise_error};
use handlebars::Handlebars;
use pulldown_cmark::{html, Parser};
use serde_json::Value;
pub struct Templates;

impl Templates {
    pub fn render(
        template: &EmailTemplate,
        data: &Option<Value>,
    ) -> RustMailerResult<(String, Option<String>, Option<String>)> {
        match data {
            None => Ok((
                template.subject.clone(),
                template.text.clone(),
                template.html.clone(),
            )),
            Some(data) => {
                let mut handlebars = Handlebars::new();

                let register_template = |hb: &mut Handlebars, name: &str, content: &str| {
                    hb.register_template_string(name, content).map_err(|e| {
                        raise_error!(
                            format!("Handlebars register '{name}' error: {e}"),
                            ErrorCode::InternalError
                        )
                    })
                };

                register_template(&mut handlebars, "subject", &template.subject)?;
                if let Some(text) = &template.text {
                    register_template(&mut handlebars, "text", text)?;
                }
                if let Some(html) = &template.html {
                    register_template(&mut handlebars, "html", html)?;
                }
                if let Some(preview) = &template.preview {
                    register_template(&mut handlebars, "preview", preview)?;
                }

                let render_template = |hb: &Handlebars, name: &str| {
                    hb.render(name, data).map_err(|e| {
                        raise_error!(
                            format!("Handlebars '{name}' render error: {e}"),
                            ErrorCode::InternalError
                        )
                    })
                };

                let subject = render_template(&handlebars, "subject")?;
                let text = template
                    .text
                    .as_ref()
                    .map(|_| render_template(&handlebars, "text"))
                    .transpose()?;
                let mut html = template
                    .html
                    .as_ref()
                    .map(|_| render_template(&handlebars, "html"))
                    .transpose()?;

                if let Some(format) = &template.format {
                    if let Some(html_content) = &mut html {
                        if matches!(format, MessageFormat::Markdown) {
                            let mut html_output = String::new();
                            html::push_html(&mut html_output, Parser::new(html_content));
                            *html_content = html_output;
                        }

                        if template.preview.is_some() {
                            let preview_content = render_template(&handlebars, "preview")?;
                            *html_content = EmailPreview::insert_preview_into_html(
                                html_content,
                                &preview_content,
                            );
                        }
                    }
                }

                Ok((subject, text, html))
            }
        }
    }
}
