// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use std::{io::Cursor, sync::LazyLock};

use image::{ExtendedColorType, ImageBuffer, ImageEncoder, Rgba};
use poem::{
    handler,
    web::{headers::UserAgent, Path, RealIp, Redirect, TypedHeader},
    IntoResponse, Response,
};

use tracing::{error, warn};

use crate::modules::{
    hook::{
        channel::{Event, EVENT_CHANNEL},
        events::{
            payload::{EmailLinkClicked, EmailOpened},
            EventPayload, EventType, RustMailerEvent,
        },
        task::EventHookTask,
    },
    metrics::{RUSTMAILER_EMAIL_CLICKS_TOTAL, RUSTMAILER_EMAIL_OPENS_TOTAL},
    smtp::track::{EmailTracker, TrackType},
};

// Static 1x1 transparent PNG
static TRANSPARENT_PIXEL: LazyLock<Vec<u8>> = LazyLock::new(|| {
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_pixel(1, 1, Rgba([0, 0, 0, 0]));
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    let encoder = image::codecs::png::PngEncoder::new(&mut cursor);
    encoder
        .write_image(img.as_raw(), 1, 1, ExtendedColorType::Rgba8)
        .expect("Failed to encode PNG");
    buffer
});

#[handler]
pub async fn get_tracking_code(
    Path(id): Path<String>,
    RealIp(ip): RealIp,
    user_agent: TypedHeader<UserAgent>,
) -> Response {
    match EmailTracker::decrypt_payload(&id) {
        Ok(payload) => {
            match payload.track_type {
                TrackType::Click => {
                    RUSTMAILER_EMAIL_CLICKS_TOTAL.inc();
                    let url = payload.url.clone().unwrap_or_default();
                    if url.is_empty() {
                        warn!(
                            account_id = %payload.account_id,
                            message_id = %payload.message_id,
                            "Click track without URL"
                        );
                        // Return empty 200 response instead of redirect
                        return Response::builder()
                            .status(http::StatusCode::OK)
                            .content_type("text/plain")
                            .body("")
                            .into_response();
                    }

                    match EventHookTask::is_watching_email_link_clicked(payload.account_id).await {
                        Ok(watched) => {
                            if watched {
                                EVENT_CHANNEL
                                    .queue(Event::new(
                                        payload.account_id,
                                        &payload.account_email,
                                        RustMailerEvent::new(
                                            EventType::EmailLinkClicked,
                                            EventPayload::EmailLinkClicked(EmailLinkClicked {
                                                campaign_id: payload.campaign_id,
                                                recipient: payload.recipient,
                                                message_id: payload.message_id.clone(),
                                                url,
                                                remote_ip: ip.map(|i| i.to_string()),
                                                user_agent: user_agent.0.to_string(),
                                            }),
                                        ),
                                    ))
                                    .await;
                            }
                        }
                        Err(e) => {
                            error!(
                                account_id = %payload.account_id,
                                message_id = %payload.message_id,
                                error = %e,
                                "Failed to check event_watched for EmailLinkClicked"
                            );
                        }
                    }

                    // Redirect to the target URL
                    Redirect::temporary(&payload.url.unwrap_or_default()).into_response()
                }
                TrackType::Open => {
                    RUSTMAILER_EMAIL_OPENS_TOTAL.inc();
                    match EventHookTask::is_watching_email_opened(payload.account_id).await {
                        Ok(watched) => {
                            if watched {
                                EVENT_CHANNEL
                                    .queue(Event::new(
                                        payload.account_id,
                                        &payload.account_email,
                                        RustMailerEvent::new(
                                            EventType::EmailOpened,
                                            EventPayload::EmailOpened(EmailOpened {
                                                campaign_id: payload.campaign_id,
                                                recipient: payload.recipient,
                                                message_id: payload.message_id.clone(),
                                                remote_ip: ip.map(|i| i.to_string()),
                                                user_agent: user_agent.0.to_string(),
                                            }),
                                        ),
                                    ))
                                    .await;
                            }
                        }
                        Err(e) => {
                            error!(
                                account_id = %payload.account_id,
                                message_id = %payload.message_id,
                                error = %e,
                                "Failed to check event_watched for EmailOpened"
                            );
                        }
                    };

                    // Return cached transparent PNG
                    Response::builder()
                        .content_type("image/png")
                        .header("Pragma", "no-cache")
                        .header("Cache-Control", "no-cache, no-store, must-revalidate")
                        .header("Expires", "0")
                        .body(TRANSPARENT_PIXEL.clone())
                }
            }
        }
        Err(e) => {
            warn!(tracking_id = %id, error = %e, "Invalid tracking payload");
            Response::builder()
                .status(http::StatusCode::OK)
                .content_type("text/plain")
                .body("Invalid tracking payload")
                .into_response()
        }
    }
}
