// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use anyhow::Result;
use nostr_sdk::client::Client as ClientSdk;
use nostr_sdk::relay::RelayPoolNotifications as RelayPoolNotificationsSdk;

use crate::base::event::{Contact, Event};
use crate::base::key::Keys;
use crate::base::subscription::SubscriptionFilter;
use crate::RUNTIME;

pub struct Client {
    client: ClientSdk,
}

impl Deref for Client {
    type Target = ClientSdk;
    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl Client {
    pub fn new(keys: Arc<Keys>, contacts: Option<Vec<Arc<Contact>>>) -> Self {
        let contacts = match contacts {
            Some(contacts) => {
                let mut c: Vec<nostr_sdk_base::Contact> = Vec::with_capacity(contacts.len());

                for i in contacts.iter() {
                    c.push(i.as_ref().deref().clone());
                }

                Some(c)
            }
            None => None,
        };

        Self {
            client: ClientSdk::new(keys.as_ref().deref(), contacts),
        }
    }

    pub fn generate_keys() -> Arc<Keys> {
        Arc::new(Keys::generate_from_os_random())
    }

    pub fn add_contact(&self, contact: Arc<Contact>) {
        RUNTIME.block_on(async move {
            self.client
                .add_contact(contact.as_ref().deref().clone())
                .await;
        });
    }

    pub fn remove_contact(&self, contact: Arc<Contact>) {
        RUNTIME.block_on(async move {
            self.client.remove_contact(contact.as_ref().deref()).await;
        });
    }

    pub fn add_relay(&self, url: String) -> Result<()> {
        RUNTIME.block_on(async move { self.client.add_relay(&url).await })
    }

    pub fn connect_relay(&self, url: String) {
        RUNTIME.block_on(async move {
            self.client.connect_relay(&url).await;
        });
    }

    pub fn connect_all(&self) {
        RUNTIME.block_on(async move {
            self.client.connect_all().await;
        });
    }

    pub fn subscribe(&self, filters: Vec<SubscriptionFilter>) {
        RUNTIME.block_on(async move {
            let mut new_filters: Vec<nostr_sdk_base::SubscriptionFilter> =
                Vec::with_capacity(filters.len());
            for filter in filters.into_iter() {
                new_filters.push(filter.deref().clone());
            }

            self.client.subscribe(new_filters).await;
        });
    }

    pub fn send_event(&self, event: Arc<Event>) {
        RUNTIME.block_on(async move {
            self.client.send_event(event.as_ref().deref().clone()).await;
        });
    }

    pub fn run_thread(&self) -> Result<()> {
        RUNTIME.block_on(async move {
            self.client
                .keep_alive(|notification| {
                    match notification {
                        RelayPoolNotificationsSdk::ReceivedEvent(event) => {
                            if event.kind
                                == nostr_sdk_base::Kind::Base(
                                    nostr_sdk_base::KindBase::EncryptedDirectMessage,
                                )
                            {
                                if let Ok(msg) = nostr_sdk_base::util::nip04::decrypt(
                                    &self.client.keys.secret_key()?,
                                    &event.pubkey,
                                    &event.content,
                                ) {
                                    println!("New DM: {}", msg);
                                } else {
                                    println!("Impossible to decrypt direct message");
                                }
                            } else {
                                println!("{:#?}", event);
                            }
                        }
                        RelayPoolNotificationsSdk::RelayDisconnected(url) => {
                            println!("Relay {} disconnected", url);
                        }
                    }

                    Ok(())
                })
                .await
        })
    }
}
