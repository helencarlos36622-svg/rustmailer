// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::account::status::AccountRunningState;
use std::sync::LazyLock;
use tokio::sync::mpsc;
use tracing::error;

pub static STATUS_DISPATCHER: LazyLock<ErrorDispatcher> = LazyLock::new(ErrorDispatcher::new);

pub struct ErrorDispatcher {
    channel: mpsc::Sender<(u64, String)>,
}

impl ErrorDispatcher {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::channel::<(u64, String)>(100);

        tokio::spawn(async move {
            while let Some((account_id, error)) = rx.recv().await {
                match AccountRunningState::append_error_message(account_id, error).await {
                    Ok(()) => {}
                    Err(error) => {
                        error!(
                            "Failed to append error for account: {}. Error: {:#?}",
                            &account_id, error
                        );
                    }
                }
            }
        });

        ErrorDispatcher { channel: tx }
    }

    pub async fn append_error(&self, account_id: u64, error: String) {
        if let Err(e) = self.channel.send((account_id, error.clone())).await {
            error!(
                "Failed to dispatch status update for account: {}, Error: {}. Channel error: {:?}",
                &account_id, error, e
            );
        }
    }
}
