// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use std::sync::Arc;

use crate::modules::common::auth::ClientContext;
use crate::modules::error::code::ErrorCode;
use crate::modules::grpc::auth::{require_account_access, require_root};
use crate::modules::grpc::service::rustmailer_grpc::{
    DeleteAccountTemplatesRequest, DeleteTemplateRequest, EmailTemplate,
    EmailTemplateCreateRequest, Empty, GetTemplateRequest, ListAccountTemplatesRequest,
    ListTemplatesRequest, PagedEmailTemplate, TemplateSentTestRequest, TemplatesService,
    UpdateTemplateRequest,
};
use crate::modules::smtp::template::entity::EmailTemplate as RustMailerEmailTemplate;
use crate::modules::smtp::template::send::send_template_test_email;
use crate::raise_error;
use poem_grpc::{Request, Response, Status};

pub mod from;

#[derive(Default)]
pub struct RustMailerTemplatesService;

impl TemplatesService for RustMailerTemplatesService {
    async fn get_template(
        &self,
        request: Request<GetTemplateRequest>,
    ) -> Result<Response<EmailTemplate>, Status> {
        let extensions = request.extensions().clone();
        let req = request.into_inner();
        // Get ClientContext from cloned extensions
        let context = extensions.get::<Arc<ClientContext>>().ok_or_else(|| {
            raise_error!("Missing ClientContext".into(), ErrorCode::InternalError)
        })?;
        let result = RustMailerEmailTemplate::get(req.id).await?;
        if let Some(account_info) = &result.account {
            context.require_account_access(account_info.id)?;
        }
        Ok(Response::new(result.into()))
    }

    async fn remove_template(
        &self,
        request: Request<DeleteTemplateRequest>,
    ) -> Result<Response<Empty>, Status> {
        let extensions = request.extensions().clone();
        let req = request.into_inner();

        // Get ClientContext from cloned extensions
        let context = extensions.get::<Arc<ClientContext>>().ok_or_else(|| {
            raise_error!("Missing ClientContext".into(), ErrorCode::InternalError)
        })?;

        let result = RustMailerEmailTemplate::get(req.id).await?;
        if let Some(account_info) = &result.account {
            context.require_account_access(account_info.id)?;
        }

        RustMailerEmailTemplate::remove(req.id).await?;
        Ok(Response::new(Empty::default()))
    }

    async fn create_template(
        &self,
        request: Request<EmailTemplateCreateRequest>,
    ) -> Result<Response<Empty>, Status> {
        let request = request.into_inner();
        let template =
            RustMailerEmailTemplate::new(request.try_into().map_err(|e: &'static str| {
                raise_error!(e.to_string(), ErrorCode::InvalidParameter)
            })?)
            .await?;
        template.save().await?;
        Ok(Response::new(Empty::default()))
    }

    async fn update_template(
        &self,
        request: Request<UpdateTemplateRequest>,
    ) -> Result<Response<Empty>, Status> {
        let extensions = request.extensions().clone();
        let req = request.into_inner();

        // Get ClientContext from cloned extensions
        let context = extensions.get::<Arc<ClientContext>>().ok_or_else(|| {
            raise_error!("Missing ClientContext".into(), ErrorCode::InternalError)
        })?;

        let result = RustMailerEmailTemplate::get(req.id).await?;
        if let Some(account_info) = &result.account {
            context.require_account_access(account_info.id)?;
        }

        RustMailerEmailTemplate::update(
            req.id,
            req.try_into().map_err(|e: &'static str| {
                raise_error!(e.to_string(), ErrorCode::InvalidParameter)
            })?,
        )
        .await?;
        Ok(Response::new(Empty::default()))
    }

    async fn list_templates(
        &self,
        request: Request<ListTemplatesRequest>,
    ) -> Result<Response<PagedEmailTemplate>, Status> {
        let req = require_root(request)?;
        let result =
            RustMailerEmailTemplate::paginate_list(req.page, req.page_size, req.desc).await?;
        Ok(Response::new(result.into()))
    }

    async fn list_account_templates(
        &self,
        request: Request<ListAccountTemplatesRequest>,
    ) -> Result<Response<PagedEmailTemplate>, Status> {
        let req = require_account_access(request, |r| r.account_id)?;
        let result = RustMailerEmailTemplate::paginate_list_account(
            req.account_id,
            req.page,
            req.page_size,
            req.desc,
        )
        .await?;
        Ok(Response::new(result.into()))
    }

    async fn remove_account_templates(
        &self,
        request: Request<DeleteAccountTemplatesRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = require_account_access(request, |r| r.account_id)?;
        RustMailerEmailTemplate::remove_account_templates(req.account_id).await?;
        Ok(Response::new(Empty::default()))
    }

    async fn send_test_email(
        &self,
        request: Request<TemplateSentTestRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = require_account_access(request, |r| r.account_id)?;
        send_template_test_email(req.template_id, req.into()).await?;
        Ok(Response::new(Empty::default()))
    }
}
