// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use std::time::Duration;

use poem::listener::{Listener, TcpListener};
use poem::middleware::CatchPanic;
use poem::{EndpointExt, Server};

use crate::modules::common::auth::ApiGuard;
use crate::modules::common::log::Tracing;
use crate::modules::common::timeout::Timeout;
use crate::modules::common::tls::rustls_config;
use crate::modules::error::code::ErrorCode;
use crate::modules::grpc::service::hook::RustMailerEventHooksService;
use crate::modules::grpc::service::rustmailer_grpc::EventHooksServiceServer;
use crate::modules::settings::cli::CompressionAlgorithm;
use crate::modules::{
    error::RustMailerResult,
    grpc::service::{
        account::RustMailerAccountService,
        autoconfig::RustMailerAutoConfigService,
        mailbox::RustMailerMailboxService,
        message::RustMailerMessageService,
        mta::RustMailerMtaService,
        oauth2::RustMailerOAuth2Service,
        rustmailer_grpc::{
            AccountServiceServer, AutoConfigServiceServer, MailboxServiceServer,
            MessageServiceServer, MtaServiceServer, OAuth2ServiceServer, SendMailServiceServer,
            StatusServiceServer, TemplatesServiceServer, FILE_DESCRIPTOR_SET,
        },
        send::RustMailerSendMailService,
        status::RustMailerStatusService,
        template::RustMailerTemplatesService,
    },
    settings::cli::SETTINGS,
    utils::shutdown::shutdown_signal,
};
use crate::raise_error;
use poem_grpc::{CompressionEncoding, Reflection, RouteGrpc};

macro_rules! add_service {
    ($route:expr, $service:ty, $impl:expr) => {
        match SETTINGS.rustmailer_grpc_compression {
            CompressionAlgorithm::None => $route.add_service(<$service>::new($impl)),
            CompressionAlgorithm::Gzip => $route.add_service(
                <$service>::new($impl)
                    .send_compressed(CompressionEncoding::GZIP)
                    .accept_compressed([CompressionEncoding::GZIP]),
            ),
            CompressionAlgorithm::Brotli => $route.add_service(
                <$service>::new($impl)
                    .send_compressed(CompressionEncoding::BROTLI)
                    .accept_compressed([CompressionEncoding::BROTLI]),
            ),
            CompressionAlgorithm::Zstd => $route.add_service(
                <$service>::new($impl)
                    .send_compressed(CompressionEncoding::ZSTD)
                    .accept_compressed([CompressionEncoding::ZSTD]),
            ),
            CompressionAlgorithm::Deflate => $route.add_service(
                <$service>::new($impl)
                    .send_compressed(CompressionEncoding::DEFLATE)
                    .accept_compressed([CompressionEncoding::DEFLATE]),
            ),
        }
    };
}

pub async fn start_grpc_server() -> RustMailerResult<()> {
    let mut route = RouteGrpc::new().add_service(
        Reflection::new()
            .add_file_descriptor_set(FILE_DESCRIPTOR_SET)
            .build(),
    );
    route = add_service!(
        route,
        AccountServiceServer<RustMailerAccountService>,
        RustMailerAccountService
    );
    route = add_service!(
        route,
        EventHooksServiceServer<RustMailerEventHooksService>,
        RustMailerEventHooksService
    );
    route = add_service!(
        route,
        AutoConfigServiceServer<RustMailerAutoConfigService>,
        RustMailerAutoConfigService
    );
    route = add_service!(
        route,
        MailboxServiceServer<RustMailerMailboxService>,
        RustMailerMailboxService
    );
    route = add_service!(
        route,
        MessageServiceServer<RustMailerMessageService>,
        RustMailerMessageService
    );
    route = add_service!(
        route,
        MtaServiceServer<RustMailerMtaService>,
        RustMailerMtaService
    );
    route = add_service!(
        route,
        OAuth2ServiceServer<RustMailerOAuth2Service>,
        RustMailerOAuth2Service
    );
    route = add_service!(
        route,
        TemplatesServiceServer<RustMailerTemplatesService>,
        RustMailerTemplatesService
    );
    route = add_service!(
        route,
        StatusServiceServer<RustMailerStatusService>,
        RustMailerStatusService
    );
    route = add_service!(
        route,
        SendMailServiceServer<RustMailerSendMailService>,
        RustMailerSendMailService
    );
    let route = route
        .with(ApiGuard)
        .with(Timeout)
        .with(Tracing)
        .with(CatchPanic::new());

    let listener = TcpListener::bind((
        SETTINGS
            .rustmailer_bind_ip
            .clone()
            .unwrap_or("0.0.0.0".into()),
        SETTINGS.rustmailer_grpc_port as u16,
    ));

    let listener = if SETTINGS.rustmailer_enable_grpc_https {
        listener.rustls(rustls_config()?).boxed()
    } else {
        listener.boxed()
    };

    let server = Server::new(listener)
        .name("RustMailer Grpc Service")
        .idle_timeout(Duration::from_secs(60))
        .run_with_graceful_shutdown(route, shutdown_signal(), Some(Duration::from_secs(5)));

    println!(
        "RustMailer Grpc Service is now running on port {}.",
        SETTINGS.rustmailer_grpc_port
    );
    server
        .await
        .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))
}
