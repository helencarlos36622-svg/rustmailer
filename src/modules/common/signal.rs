// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use std::sync::LazyLock;

use crate::modules::{
    context::Initialize, error::RustMailerResult, utils::shutdown::shutdown_signal,
};
use tokio::sync::broadcast;

pub static SIGNAL_MANAGER: LazyLock<SignalManager> = LazyLock::new(SignalManager::new);

pub struct SignalManager {
    sender: broadcast::Sender<()>,
}

impl SignalManager {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1);
        SignalManager { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.sender.subscribe()
    }
}

impl Initialize for SignalManager {
    async fn initialize() -> RustMailerResult<()> {
        tokio::spawn({
            async move {
                shutdown_signal().await;
                println!("\nSending shutdown signal...");
                let _ = SIGNAL_MANAGER.sender.send(());
            }
        });
        Ok(())
    }
}
