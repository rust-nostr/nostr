// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use async_utility::thread;
use atomic_destructor::StealthClone;
use nostr_relay_pool::prelude::*;

use super::Client;

impl Client {
    pub(super) fn spawn_notification_handler(&self) {
        // Stealth clone the client (not increment atomic destructor counter)
        let client: Client = self.stealth_clone();

        // Spawn handler
        let _ = thread::spawn(async move {
            tracing::info!("Spawned client notification handler");

            let mut notifications = client.pool.notifications();
            while let Ok(notification) = notifications.recv().await {
                match notification {
                    RelayPoolNotification::Message { relay_url, message } => {
                        // Check if auto authentication (NIP42) is enabled
                        if client.opts.is_nip42_auto_authentication_enabled() {
                            if let RelayMessage::Auth { challenge } = message {
                                match client.auth(challenge, relay_url.clone()).await {
                                    Ok(..) => {
                                        tracing::info!("Authenticated to '{relay_url}' relay.");

                                        if let Ok(relay) = client.relay(relay_url).await {
                                            let opts = RelaySendOptions::new()
                                                .skip_send_confirmation(true);
                                            if let Err(e) = relay.resubscribe(opts).await {
                                                tracing::error!(
                                                    "Impossible to resubscribe to '{}': {e}",
                                                    relay.url()
                                                );
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!(
                                            "Can't authenticate to '{relay_url}' relay: {e}"
                                        );
                                    }
                                }
                            }
                        }
                    }
                    RelayPoolNotification::Shutdown => break,
                    _ => (),
                }
            }

            tracing::info!("Client notification handler terminated.");
        });
    }
}
