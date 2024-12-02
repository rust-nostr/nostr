// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use async_utility::task;
use atomic_destructor::StealthClone;
use nostr_relay_pool::prelude::*;

use super::Client;

impl Client {
    pub(super) fn spawn_notification_handler(&self) {
        // Stealth clone the client (not increment atomic destructor counter)
        let client: Client = self.stealth_clone();

        // Spawn handler
        task::spawn(async move {
            tracing::info!("Spawned client notification handler");

            let mut notifications = client.pool.notifications();
            while let Ok(notification) = notifications.recv().await {
                match notification {
                    // Check if relay message is AUTH
                    RelayPoolNotification::Message {
                        relay_url,
                        message: RelayMessage::Auth { challenge },
                    } => {
                        // Check if auto authentication (NIP42) is enabled
                        if client.opts.is_nip42_auto_authentication_enabled() {
                            // Auth
                            match client.auth(challenge, relay_url.clone()).await {
                                Ok(..) => {
                                    tracing::info!("Authenticated to '{relay_url}' relay.");

                                    if let Ok(relay) = client.relay(relay_url).await {
                                        if let Err(e) = relay.resubscribe().await {
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
                    RelayPoolNotification::Shutdown => break,
                    _ => (),
                }
            }

            tracing::info!("Client notification handler terminated.");
        });
    }
}
