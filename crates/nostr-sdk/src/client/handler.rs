// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use async_utility::thread;
use atomic_destructor::StealthClone;
use nostr::prelude::*;
use nostr_relay_pool::RelayPoolNotification;

use super::{Client, ClientNotification};

impl Client {
    pub(super) fn spawn_notification_handler(&self) {
        // Stealth clone the client (not increment atomic destructor counter)
        let client: Client = self.stealth_clone();

        // Spawn handler
        let _ = thread::spawn(async move {
            tracing::info!("Spawned client notification handler");

            let mut notifications = client.pool.notifications();
            while let Ok(notification) = notifications.recv().await {
                // Forward notification to client channel
                let _ = client
                    .notifications
                    .send(ClientNotification::Pool(notification.clone()));

                match notification {
                    RelayPoolNotification::Message { relay_url, message } => {
                        // Check if auto authentication (NIP42) is enabled
                        if client.opts.is_nip42_auto_authentication_enabled() {
                            if let RelayMessage::Auth { challenge } = message {
                                match client.auth(challenge, relay_url.clone()).await {
                                    Ok(_) => {
                                        tracing::info!("Authenticated to '{relay_url}' relay.");
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
                    #[cfg(feature = "nip59")]
                    RelayPoolNotification::Event { event, .. } => {
                        if client.opts.nip17_auto_decryption && event.kind == Kind::GiftWrap {
                            match client.unwrap_gift_wrap(&event).await {
                                Ok(UnwrappedGift { rumor, sender }) => {
                                    if rumor.kind == Kind::PrivateDirectMessage {
                                        if let Err(e) = client.notifications.send(
                                            ClientNotification::PrivateDirectMessage {
                                                sender,
                                                message: rumor.content,
                                                timestamp: rumor.created_at,
                                                reply_to: None,
                                            },
                                        ) {
                                            tracing::error!(
                                                "Impossible to send client notification: {e}"
                                            );
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Impossible to decrypt gift wrap: {e}")
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
