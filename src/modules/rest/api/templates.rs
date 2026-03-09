// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::common::auth::ClientContext;
use crate::modules::rest::api::ApiTags;
use crate::modules::rest::response::DataPage;
use crate::modules::rest::ApiResult;
use crate::modules::smtp::template::entity::EmailTemplate;
use crate::modules::smtp::template::payload::{
    TemplateCreateRequest, TemplateSentTestRequest, TemplateUpdateRequest,
};
use crate::modules::smtp::template::send::send_template_test_email;
use poem::web::Path;
use poem_openapi::param::Query;
use poem_openapi::payload::Json;
use poem_openapi::OpenApi;
pub struct TempaltesApi;

#[OpenApi(prefix_path = "/api/v1", tag = "ApiTags::Template")]
impl TempaltesApi {
    /// Retrieves an email template by its name.
    /// Returns the template if found, or a `ResourceNotFound` error if no template matches the provided name.
    #[oai(path = "/template/:id", method = "get", operation_id = "get_template")]
    async fn get_template(
        &self,
        ///The name of the email template to retrieve
        id: Path<u64>,
        context: ClientContext,
    ) -> ApiResult<Json<EmailTemplate>> {
        let template = EmailTemplate::get(id.0).await?;
        if let Some(account_info) = &template.account {
            context.require_account_access(account_info.id)?;
        }
        Ok(Json(template))
    }

    /// Deletes an email template by its name.
    /// Removes the specified template if it exists and the client has access. Returns a `ResourceNotFound` error if the template is not found.
    #[oai(
        path = "/template/:id",
        method = "delete",
        operation_id = "remove_template"
    )]
    async fn remove_template(
        &self,
        ///The name of the email template to retrieve
        id: Path<u64>,
        context: ClientContext,
    ) -> ApiResult<()> {
        let template = EmailTemplate::get(id.0).await?;
        if let Some(account_info) = &template.account {
            context.require_account_access(account_info.id)?;
        }

        Ok(EmailTemplate::remove(id.0).await?)
    }

    /// Creates a new email template.
    /// Saves a new email template based on the provided request data.
    #[oai(path = "/template", method = "post", operation_id = "create_template")]
    async fn create_template(
        &self,
        ///JSON payload containing the data needed to create a new email template
        request: Json<TemplateCreateRequest>,
    ) -> ApiResult<()> {
        let entity = EmailTemplate::new(request.0).await?;
        Ok(entity.save().await?)
    }

    /// Updates an existing email template by its name.
    /// Modifies the specified template with the provided update data if it exists and the client has access. Returns a `ResourceNotFound` error if the template is not found.
    #[oai(
        path = "/template/:id",
        method = "post",
        operation_id = "update_template"
    )]
    async fn update_template(
        &self,
        ///The name of the email template to update
        id: Path<u64>,
        ///JSON payload containing the updated template data.
        payload: Json<TemplateUpdateRequest>,
        context: ClientContext,
    ) -> ApiResult<()> {
        let template = EmailTemplate::get(id.0).await?;
        if let Some(account_info) = &template.account {
            context.require_account_access(account_info.id)?;
        }
        Ok(EmailTemplate::update(id.0, payload.0).await?)
    }

    /// Lists all email templates with pagination.
    /// Retrieves a paginated list of all email templates.
    /// Requires root privileges.
    #[oai(
        path = "/list-template",
        method = "get",
        operation_id = "list_templates"
    )]
    async fn list_templates(
        &self,
        /// Optional. The page number to retrieve (starting from 1).
        page: Query<Option<u64>>,
        /// Optional. The number of items per page.
        page_size: Query<Option<u64>>,
        /// Optional. Whether to sort the list in descending order.
        desc: Query<Option<bool>>,
        context: ClientContext,
    ) -> ApiResult<Json<DataPage<EmailTemplate>>> {
        context.require_root()?;
        Ok(Json(
            EmailTemplate::paginate_list(page.0, page_size.0, desc.0).await?,
        ))
    }

    /// Lists email templates associated with a specific account.
    /// Retrieves a paginated list of templates for the specified account ID. Requires access to the specified account.
    #[oai(
        path = "/account-templates/:account_id",
        method = "get",
        operation_id = "list_account_templates"
    )]
    async fn list_account_templates(
        &self,
        ///The ID of the account whose templates are to be listed
        account_id: Path<u64>,
        /// Optional. The page number to retrieve (starting from 1).
        page: Query<Option<u64>>,
        /// Optional. The number of items per page.
        page_size: Query<Option<u64>>,
        /// Optional. Whether to sort the list in descending order.
        desc: Query<Option<bool>>,
        context: ClientContext,
    ) -> ApiResult<Json<DataPage<EmailTemplate>>> {
        let account_id = account_id.0;
        context.require_account_access(account_id)?;
        Ok(Json(
            EmailTemplate::paginate_list_account(account_id, page.0, page_size.0, desc.0).await?,
        ))
    }

    /// Deletes all email templates associated with a specific account.
    /// Removes all templates linked to the specified account ID. Requires access to the specified account.
    #[oai(
        path = "/account-templates/:account_id",
        method = "delete",
        operation_id = "remove_account_templates"
    )]
    async fn remove_account_templates(
        &self,
        ///The ID of the account whose templates are to be deleted
        account_id: Path<u64>,
        context: ClientContext,
    ) -> ApiResult<()> {
        let account_id = account_id.0;
        context.require_account_access(account_id)?;
        Ok(EmailTemplate::remove_account_templates(account_id).await?)
    }

    /// Send a test email using a specific template
    /// This endpoint allows sending a test email to verify template rendering and delivery.
    #[oai(
        path = "/template-send-test/:id",
        method = "post",
        operation_id = "template_send_test_email"
    )]
    async fn template_send_test_email(
        &self,
        /// The unique name identifier of the MTA to test.
        id: Path<u64>,
        /// request payload.
        request: Json<TemplateSentTestRequest>,
        context: ClientContext,
    ) -> ApiResult<()> {
        context.require_account_access(request.0.account_id)?;
        send_template_test_email(id.0, request.0).await?;
        Ok(())
    }
}
