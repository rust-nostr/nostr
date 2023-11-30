// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use nostr::url::Url;
use nostr::{ChannelId, Contact as ContactSdk, UncheckedUrl};
use uniffi::Object;

use super::{Event, EventId};
use crate::error::Result;
use crate::key::Keys;
use crate::nips::nip57::ZapRequestData;
use crate::types::{Contact, Metadata};
use crate::{FileMetadata, NostrConnectMessage, PublicKey, RelayMetadata, Tag, UnsignedEvent};

#[derive(Object)]
pub struct EventBuilder {
    inner: nostr::EventBuilder,
}

impl From<nostr::EventBuilder> for EventBuilder {
    fn from(inner: nostr::EventBuilder) -> Self {
        Self { inner }
    }
}

impl Deref for EventBuilder {
    type Target = nostr::EventBuilder;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[uniffi::export]
impl EventBuilder {
    #[uniffi::constructor]
    pub fn new(kind: u64, content: String, tags: Vec<Arc<Tag>>) -> Result<Arc<Self>> {
        let tags = tags
            .into_iter()
            .map(|t| t.as_ref().deref().clone())
            .collect::<Vec<_>>();
        Ok(Arc::new(Self {
            inner: nostr::EventBuilder::new(kind.into(), content, &tags),
        }))
    }

    pub fn to_event(&self, keys: Arc<Keys>) -> Result<Arc<Event>> {
        let event = self.inner.clone().to_event(keys.deref())?;
        Ok(Arc::new(event.into()))
    }

    pub fn to_pow_event(&self, keys: Arc<Keys>, difficulty: u8) -> Result<Arc<Event>> {
        Ok(Arc::new(
            self.inner
                .clone()
                .to_pow_event(keys.deref(), difficulty)?
                .into(),
        ))
    }

    pub fn to_unsigned_event(&self, public_key: Arc<PublicKey>) -> Arc<UnsignedEvent> {
        Arc::new(
            self.inner
                .clone()
                .to_unsigned_event(*public_key.as_ref().deref())
                .into(),
        )
    }

    pub fn to_unsigned_pow_event(
        &self,
        public_key: Arc<PublicKey>,
        difficulty: u8,
    ) -> Arc<UnsignedEvent> {
        Arc::new(
            self.inner
                .clone()
                .to_unsigned_pow_event(*public_key.as_ref().deref(), difficulty)
                .into(),
        )
    }

    #[uniffi::constructor]
    pub fn set_metadata(metadata: Arc<Metadata>) -> Arc<Self> {
        Arc::new(Self {
            inner: nostr::EventBuilder::set_metadata(metadata.as_ref().deref()),
        })
    }

    #[uniffi::constructor]
    pub fn add_recommended_relay(url: String) -> Result<Arc<Self>> {
        let url = Url::parse(&url)?;
        Ok(Arc::new(Self {
            inner: nostr::EventBuilder::add_recommended_relay(&url),
        }))
    }

    #[uniffi::constructor]
    pub fn relay_list(list: HashMap<String, Option<RelayMetadata>>) -> Arc<Self> {
        let iter = list
            .into_iter()
            .map(|(url, r)| (UncheckedUrl::from(url), r.map(|r| r.into())));
        Arc::new(Self {
            inner: nostr::EventBuilder::relay_list(iter),
        })
    }

    #[uniffi::constructor]
    pub fn new_text_note(content: String, tags: Vec<Arc<Tag>>) -> Result<Arc<Self>> {
        let tags = tags
            .into_iter()
            .map(|t| t.as_ref().deref().clone())
            .collect::<Vec<_>>();
        Ok(Arc::new(Self {
            inner: nostr::EventBuilder::new_text_note(content, &tags),
        }))
    }

    #[uniffi::constructor]
    pub fn long_form_text_note(content: String, tags: Vec<Arc<Tag>>) -> Result<Arc<Self>> {
        let tags = tags
            .into_iter()
            .map(|t| t.as_ref().deref().clone())
            .collect::<Vec<_>>();
        Ok(Arc::new(Self {
            inner: nostr::EventBuilder::long_form_text_note(content, &tags),
        }))
    }

    #[uniffi::constructor]
    pub fn set_contact_list(list: Vec<Arc<Contact>>) -> Arc<Self> {
        let list: Vec<ContactSdk> = list
            .into_iter()
            .map(|c| c.as_ref().deref().clone())
            .collect();

        Arc::new(Self {
            inner: nostr::EventBuilder::set_contact_list(list),
        })
    }

    /// Create encrypted direct msg event
    #[uniffi::constructor]
    pub fn new_encrypted_direct_msg(
        sender_keys: Arc<Keys>,
        receiver_pubkey: Arc<PublicKey>,
        content: String,
        reply_to: Option<Arc<EventId>>,
    ) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            inner: nostr::EventBuilder::new_encrypted_direct_msg(
                sender_keys.deref(),
                *receiver_pubkey.as_ref().deref(),
                content,
                reply_to.map(|id| id.as_ref().into()),
            )?,
        }))
    }

    #[uniffi::constructor]
    pub fn repost(event_id: Arc<EventId>, public_key: Arc<PublicKey>) -> Arc<Self> {
        Arc::new(Self {
            inner: nostr::EventBuilder::repost(
                event_id.as_ref().into(),
                *public_key.as_ref().deref(),
            ),
        })
    }

    /// Create delete event
    #[uniffi::constructor]
    pub fn delete(ids: Vec<Arc<EventId>>, reason: Option<String>) -> Arc<Self> {
        let ids: Vec<nostr::EventId> = ids.into_iter().map(|e| e.as_ref().into()).collect();
        Arc::new(Self {
            inner: match reason {
                Some(reason) => nostr::EventBuilder::delete_with_reason(ids, reason),
                None => nostr::EventBuilder::delete(ids),
            },
        })
    }

    #[uniffi::constructor]
    pub fn new_reaction(
        event_id: Arc<EventId>,
        public_key: Arc<PublicKey>,
        content: String,
    ) -> Arc<Self> {
        Arc::new(Self {
            inner: nostr::EventBuilder::new_reaction(
                event_id.as_ref().into(),
                *public_key.as_ref().deref(),
                content,
            ),
        })
    }

    #[uniffi::constructor]
    pub fn new_channel(metadata: Arc<Metadata>) -> Arc<Self> {
        Arc::new(Self {
            inner: nostr::EventBuilder::new_channel(metadata.as_ref().deref()),
        })
    }

    #[uniffi::constructor]
    pub fn set_channel_metadata(
        channel_id: String,
        relay_url: Option<String>,
        metadata: Arc<Metadata>,
    ) -> Result<Arc<Self>> {
        let relay_url = match relay_url {
            Some(url) => Some(Url::parse(&url)?),
            None => None,
        };
        Ok(Arc::new(Self {
            inner: nostr::EventBuilder::set_channel_metadata(
                ChannelId::from_hex(channel_id)?,
                relay_url,
                metadata.as_ref().deref(),
            ),
        }))
    }

    #[uniffi::constructor]
    pub fn new_channel_msg(
        channel_id: String,
        relay_url: String,
        content: String,
    ) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            inner: nostr::EventBuilder::new_channel_msg(
                ChannelId::from_hex(channel_id)?,
                Url::parse(&relay_url)?,
                content,
            ),
        }))
    }

    #[uniffi::constructor]
    pub fn hide_channel_msg(message_id: Arc<EventId>, reason: Option<String>) -> Arc<Self> {
        Arc::new(Self {
            inner: nostr::EventBuilder::hide_channel_msg(message_id.as_ref().into(), reason),
        })
    }

    #[uniffi::constructor]
    pub fn mute_channel_user(public_key: Arc<PublicKey>, reason: Option<String>) -> Arc<Self> {
        Arc::new(Self {
            inner: nostr::EventBuilder::mute_channel_user(*public_key.as_ref().deref(), reason),
        })
    }

    #[uniffi::constructor]
    pub fn auth(challenge: String, relay_url: String) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            inner: nostr::EventBuilder::auth(challenge, Url::parse(&relay_url)?),
        }))
    }

    #[uniffi::constructor]
    pub fn nostr_connect(
        sender_keys: Arc<Keys>,
        receiver_pubkey: Arc<PublicKey>,
        msg: NostrConnectMessage,
    ) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            inner: nostr::EventBuilder::nostr_connect(
                sender_keys.as_ref().deref(),
                **receiver_pubkey,
                msg.try_into()?,
            )?,
        }))
    }

    #[uniffi::constructor]
    pub fn report(tags: Vec<Arc<Tag>>, content: String) -> Arc<Self> {
        let tags = tags
            .into_iter()
            .map(|t| t.as_ref().deref().clone())
            .collect::<Vec<_>>();
        Arc::new(Self {
            inner: nostr::EventBuilder::report(&tags, content),
        })
    }

    #[uniffi::constructor]
    pub fn new_zap_request(data: Arc<ZapRequestData>) -> Arc<Self> {
        Arc::new(Self {
            inner: nostr::EventBuilder::new_zap_request(data.as_ref().deref().clone()),
        })
    }

    #[uniffi::constructor]
    pub fn new_zap_receipt(
        bolt11: String,
        preimage: Option<String>,
        zap_request: Arc<Event>,
    ) -> Arc<Self> {
        Arc::new(Self {
            inner: nostr::EventBuilder::new_zap_receipt(
                bolt11,
                preimage,
                zap_request.as_ref().deref().clone(),
            ),
        })
    }

    #[uniffi::constructor]
    pub fn file_metadata(description: String, metadata: Arc<FileMetadata>) -> Arc<Self> {
        Arc::new(Self {
            inner: nostr::EventBuilder::file_metadata(
                description,
                metadata.as_ref().deref().clone(),
            ),
        })
    }
}
