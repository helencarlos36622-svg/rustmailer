// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use mail_send::mail_builder::headers::raw::Raw as XRaw;
use mail_send::mail_builder::headers::text::Text as XText;
use mail_send::mail_builder::headers::url::URL as XURL;
use mail_send::mail_builder::headers::HeaderType;
use poem_openapi::{Object, Union};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

// Define the HeaderType enum, representing different possible header value types.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, Union)]
#[oai(discriminator_name = "type")]
pub enum HeaderValue {
    Raw(Raw),
    Text(Text),
    Url(Url),
}

impl HeaderValue {
    pub fn to_header_type(&self) -> HeaderType<'static> {
        match self {
            HeaderValue::Raw(raw) => HeaderType::Raw(XRaw::new(Cow::Owned(raw.raw.to_string()))),
            HeaderValue::Text(text) => {
                HeaderType::Text(XText::new(Cow::Owned(text.text.to_string())))
            }
            HeaderValue::Url(url) => HeaderType::URL(XURL::new_list(
                url.url
                    .iter()
                    .map(|s| Cow::Owned(s.to_string()))
                    .collect::<Vec<_>>()
                    .into_iter(),
            )),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
pub struct Raw {
    pub raw: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
pub struct Text {
    pub text: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
pub struct Url {
    pub url: Vec<String>,
}
