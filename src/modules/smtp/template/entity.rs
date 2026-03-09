// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::account::migration::AccountModel;
use crate::modules::database::manager::DB_MANAGER;
use crate::modules::database::{
    batch_delete_impl, delete_impl, paginate_query_primary_scan_all_impl,
    paginate_secondary_scan_impl, secondary_find_impl, update_impl,
};

use crate::modules::error::code::ErrorCode;
use crate::modules::rest::response::DataPage;
use crate::modules::smtp::template::payload::{TemplateCreateRequest, TemplateUpdateRequest};
use crate::modules::token::AccountInfo;
use crate::{id, raise_error};
use crate::{modules::database::insert_impl, modules::error::RustMailerResult, utc_now};
use handlebars::Handlebars;
use itertools::Itertools;
use native_db::*;
use native_model::{native_model, Model};
use poem_openapi::{Enum, Object};
use serde::{Deserialize, Serialize};

const NOT_ASSIGNED: u64 = 0;

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
#[native_model(id = 6, version = 1)]
#[native_db(primary_key(pk -> String), secondary_key(account_id_key -> u64))]
pub struct EmailTemplate {
    /// Unique identifier for the template, used as a secondary key.
    #[secondary_key(unique)]
    pub id: u64,
    /// Optional description of the template for additional context.
    pub description: Option<String>,
    /// Associated account information, if any. `None` indicates the template is public.
    pub account: Option<AccountInfo>,
    /// Subject line of the email template.
    pub subject: String,
    /// Optional preview text for the email, used in email clients.
    pub preview: Option<String>,
    /// Format of the HTML email content, either Markdown or HTML. Defaults to HTML if not specified.
    pub format: Option<MessageFormat>,
    /// Plain text content of the email, if provided.
    pub text: Option<String>,
    /// HTML content of the email, if provided.
    pub html: Option<String>,
    /// Timestamp of when the template was created (in Unix epoch milliseconds).
    pub created_at: i64,
    /// Timestamp of when the template was last updated (in Unix epoch milliseconds).
    pub updated_at: i64,
    /// Timestamp of when the template was last accessed (in Unix epoch milliseconds).
    pub last_access_at: i64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Enum)]
pub enum MessageFormat {
    /// Content is formatted in Markdown.
    Markdown,
    /// Content is formatted in HTML (default).
    #[default]
    Html,
}

impl EmailTemplate {
    fn pk(&self) -> String {
        format!("{}_{}", self.created_at, self.id)
    }

    fn account_id_key(&self) -> u64 {
        self.account
            .clone()
            .map(|info| info.id)
            .unwrap_or(NOT_ASSIGNED)
    }

    pub async fn new(value: TemplateCreateRequest) -> RustMailerResult<Self> {
        let account_info = if let Some(account_id) = value.account_id {
            AccountModel::get(account_id).await.map(|account| {
                Some(AccountInfo {
                    id: account_id,
                    email: account.email,
                })
            })?
        } else {
            None
        };

        Ok(Self {
            id: id!(96),
            description: value.description,
            account: account_info,
            subject: value.subject,
            preview: value.preview,
            html: value.html,
            text: value.text,
            format: value.format,
            created_at: utc_now!(),
            updated_at: utc_now!(),
            last_access_at: Default::default(),
        })
    }

    pub async fn paginate_list_account(
        account_id: u64,
        page: Option<u64>,
        page_size: Option<u64>,
        desc: Option<bool>,
    ) -> RustMailerResult<DataPage<EmailTemplate>> {
        paginate_secondary_scan_impl(
            DB_MANAGER.meta_db(),
            page,
            page_size,
            desc,
            EmailTemplateKey::account_id_key,
            account_id,
        )
        .await
        .map(DataPage::from)
    }

    pub async fn save(self) -> RustMailerResult<()> {
        self.validate_templates()?;
        if let Some(account) = &self.account {
            Self::check_account_id(account.id).await?;
        }
        //check name
        if Self::find(self.id).await?.is_some() {
            return Err(raise_error!(
                format!("template with id '{}' already exists", self.id).into(),
                ErrorCode::AlreadyExists
            ));
        }
        insert_impl(DB_MANAGER.meta_db(), self).await
    }

    pub async fn remove_account_templates(account_id: u64) -> RustMailerResult<()> {
        batch_delete_impl(DB_MANAGER.meta_db(), move |rw| {
            let templates: Vec<EmailTemplate> = rw
                .scan()
                .secondary::<EmailTemplate>(EmailTemplateKey::account_id_key)
                .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                .start_with(account_id)
                .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                .try_collect()
                .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?;
            Ok(templates)
        })
        .await?;
        Ok(())
    }

    pub async fn find(id: u64) -> RustMailerResult<Option<EmailTemplate>> {
        secondary_find_impl(DB_MANAGER.meta_db(), EmailTemplateKey::id, id).await
    }

    pub async fn get(id: u64) -> RustMailerResult<EmailTemplate> {
        Self::find(id).await?.ok_or_else(|| {
            raise_error!(
                format!("Template id='{id}' not found."),
                ErrorCode::ResourceNotFound
            )
        })
    }

    pub async fn remove(id: u64) -> RustMailerResult<()> {
        delete_impl(DB_MANAGER.meta_db(), move |rw| {
            rw.get().secondary::<EmailTemplate>(EmailTemplateKey::id, id)
            .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
            .ok_or_else(||raise_error!(format!("The email template with id={id} that you want to delete was not found."), ErrorCode::ResourceNotFound))
        }).await
    }

    pub async fn paginate_list(
        page: Option<u64>,
        page_size: Option<u64>,
        desc: Option<bool>,
    ) -> RustMailerResult<DataPage<EmailTemplate>> {
        paginate_query_primary_scan_all_impl(DB_MANAGER.meta_db(), page, page_size, desc)
            .await
            .map(DataPage::from)
    }

    pub async fn update(id: u64, request: TemplateUpdateRequest) -> RustMailerResult<()> {
        if let Some(text) = &request.text {
            Self::validate_template("text", text)?;
        }

        if let Some(html) = &request.html {
            Self::validate_template("html", html)?;
        }

        if let Some(subject) = &request.subject {
            Self::validate_template("subject", subject)?;
        }

        if let Some(preview) = &request.preview {
            Self::validate_template("preview", preview)?;
        }

        update_impl(
            DB_MANAGER.meta_db(),
            move |rw| {
                rw.get()
                    .secondary::<EmailTemplate>(EmailTemplateKey::id, id)
                    .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                    .ok_or_else(|| {
                        raise_error!(format!("The email template with id={} that you want to modify was not found.", id), ErrorCode::ResourceNotFound)
                    })
            },
            |current| Ok(Self::apply_update(current, request)),
        )
        .await?;
        Ok(())
    }

    async fn check_account_id(account_id: u64) -> RustMailerResult<()> {
        let _ = AccountModel::get(account_id).await?;
        Ok(())
    }

    fn validate_templates(&self) -> RustMailerResult<()> {
        if let Some(text) = &self.text {
            Self::validate_template("text", text)?;
        }

        if let Some(html) = &self.html {
            Self::validate_template("html", html)?;
            if self.format.is_none() {
                return Err(raise_error!(
                    "Content format must be specified when 'html' is set. Expected 'Markdown' or 'Html'."
                        .into(), ErrorCode::InvalidParameter
                ));
            }
        }

        Self::validate_template("subject", &self.subject)?;

        if let Some(preview) = &self.preview {
            Self::validate_template("preview", preview)?;
        }
        Ok(())
    }

    fn validate_template(field: &str, content: &str) -> RustMailerResult<()> {
        if content.is_empty() {
            return Err(raise_error!(
                format!("field: {} is empty.", field),
                ErrorCode::InvalidParameter
            ));
        }

        let mut handlebars = Handlebars::new();
        match handlebars.register_template_string("test", content) {
            Ok(_) => Ok(()),
            Err(e) => Err(raise_error!(
                format!("field: {} , template error: {:#?}", field, e),
                ErrorCode::InvalidParameter
            )),
        }
    }

    // This function updates the properties of an EmailTemplate with new values from an EmailTemplateRequest.
    fn apply_update(old: &EmailTemplate, request: TemplateUpdateRequest) -> EmailTemplate {
        let mut new = old.clone();

        if request.description.is_some() {
            new.description = request.description;
        }

        if let Some(subject) = request.subject {
            new.subject = subject;
        }

        if let Some(preview) = request.preview {
            new.preview = Some(preview);
        }

        if let Some(text) = request.text {
            new.text = Some(text);
        }

        if let Some(html) = request.html {
            new.html = Some(html);
        }

        if let Some(format) = request.format {
            new.format = Some(format);
        }
        new.updated_at = utc_now!();
        new
    }
}
