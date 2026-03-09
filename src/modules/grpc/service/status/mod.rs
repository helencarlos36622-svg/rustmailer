// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::grpc::service::rustmailer_grpc::{
    Empty, Notifications, ServerStatus, StatusService,
};
use crate::modules::{context::status::RustMailerStatus, version::fetch_notifications};
use poem_grpc::{Request, Response, Status};

pub mod from;

#[derive(Default)]
pub struct RustMailerStatusService;

impl StatusService for RustMailerStatusService {
    async fn get_status(&self, _request: Request<Empty>) -> Result<Response<ServerStatus>, Status> {
        Ok(Response::new(RustMailerStatus::get().into()))
    }

    async fn get_notifications(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Notifications>, Status> {
        let notifications = fetch_notifications().await?;
        Ok(Response::new(notifications.into()))
    }
}
