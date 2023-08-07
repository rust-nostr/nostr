// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::url::Url;
use nostr::{ChannelId, Contact as ContactSdk, EventBuilder as EventBuilderSdk};
use uniffi::Object;

use super::{Event, EventId};
use crate::error::Result;
use crate::key::Keys;
use crate::types::{Contact, Metadata};
use crate::{FileMetadata, PublicKey, Tag, UnsignedEvent};

#[derive(Debug, Object)]
pub struct EventBuilder {
    builder: EventBuilderSdk,
}

impl From<EventBuilderSdk> for EventBuilder {
    fn from(builder: EventBuilderSdk) -> Self {
        Self { builder }
    }
}

impl Deref for EventBuilder {
    type Target = EventBuilderSdk;
    fn deref(&self) -> &Self::Target {
        &self.builder
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
            builder: EventBuilderSdk::new(kind.into(), content, &tags),
        }))
    }

    pub fn to_event(&self, keys: Arc<Keys>) -> Result<Arc<Event>> {
        let event = self.builder.clone().to_event(keys.deref())?;
        Ok(Arc::new(event.into()))
    }

    pub fn to_pow_event(&self, keys: Arc<Keys>, difficulty: u8) -> Result<Arc<Event>> {
        Ok(Arc::new(
            self.builder
                .clone()
                .to_pow_event(keys.deref(), difficulty)?
                .into(),
        ))
    }

    pub fn to_unsigned_event(&self, public_key: Arc<PublicKey>) -> Arc<UnsignedEvent> {
        Arc::new(
            self.builder
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
            self.builder
                .clone()
                .to_unsigned_pow_event(*public_key.as_ref().deref(), difficulty)
                .into(),
        )
    }
}

#[uniffi::export]
impl EventBuilder {
    #[uniffi::constructor]
    pub fn set_metadata(metadata: Arc<Metadata>) -> Arc<Self> {
        Arc::new(Self {
            builder: EventBuilderSdk::set_metadata(metadata.as_ref().deref().clone()),
        })
    }

    #[uniffi::constructor]
    pub fn add_recommended_relay(url: String) -> Result<Arc<Self>> {
        let url = Url::parse(&url)?;

        Ok(Arc::new(Self {
            builder: EventBuilderSdk::add_recommended_relay(&url),
        }))
    }

    #[uniffi::constructor]
    pub fn new_text_note(content: String, tags: Vec<Arc<Tag>>) -> Result<Arc<Self>> {
        let tags = tags
            .into_iter()
            .map(|t| t.as_ref().deref().clone())
            .collect::<Vec<_>>();
        Ok(Arc::new(Self {
            builder: EventBuilderSdk::new_text_note(content, &tags),
        }))
    }

    #[uniffi::constructor]
    pub fn long_form_text_note(content: String, tags: Vec<Arc<Tag>>) -> Result<Arc<Self>> {
        let tags = tags
            .into_iter()
            .map(|t| t.as_ref().deref().clone())
            .collect::<Vec<_>>();
        Ok(Arc::new(Self {
            builder: EventBuilderSdk::long_form_text_note(content, &tags),
        }))
    }

    #[uniffi::constructor]
    pub fn set_contact_list(list: Vec<Arc<Contact>>) -> Arc<Self> {
        let list: Vec<ContactSdk> = list
            .into_iter()
            .map(|c| c.as_ref().deref().clone())
            .collect();

        Arc::new(Self {
            builder: EventBuilderSdk::set_contact_list(list),
        })
    }

    #[uniffi::constructor]
    pub fn new_encrypted_direct_msg(
        sender_keys: Arc<Keys>,
        receiver_pubkey: Arc<PublicKey>,
        content: String,
        reply: Option<Arc<EventId>>,
    ) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            builder: EventBuilderSdk::new_encrypted_direct_msg(
                sender_keys.deref(),
                *receiver_pubkey.as_ref().deref(),
                content,
                reply.map(|id| id.as_ref().into()),
            )?,
        }))
    }

    #[uniffi::constructor]
    pub fn repost(event_id: Arc<EventId>, public_key: Arc<PublicKey>) -> Arc<Self> {
        Arc::new(Self {
            builder: EventBuilderSdk::repost(
                event_id.as_ref().into(),
                *public_key.as_ref().deref(),
            ),
        })
    }

    #[uniffi::constructor]
    pub fn delete(ids: Vec<Arc<EventId>>, reason: Option<String>) -> Arc<Self> {
        let ids: Vec<nostr::EventId> = ids.into_iter().map(|e| e.as_ref().into()).collect();
        Arc::new(Self {
            builder: EventBuilderSdk::delete(ids, reason.as_deref()),
        })
    }

    #[uniffi::constructor]
    pub fn new_reaction(
        event_id: Arc<EventId>,
        public_key: Arc<PublicKey>,
        content: String,
    ) -> Arc<Self> {
        Arc::new(Self {
            builder: EventBuilderSdk::new_reaction(
                event_id.as_ref().into(),
                *public_key.as_ref().deref(),
                content,
            ),
        })
    }

    #[uniffi::constructor]
    pub fn new_channel(metadata: Arc<Metadata>) -> Arc<Self> {
        Arc::new(Self {
            builder: EventBuilderSdk::new_channel(metadata.as_ref().deref().clone()),
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
            builder: EventBuilderSdk::set_channel_metadata(
                ChannelId::from_hex(channel_id)?,
                relay_url,
                metadata.as_ref().deref().clone(),
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
            builder: EventBuilderSdk::new_channel_msg(
                ChannelId::from_hex(channel_id)?,
                Url::parse(&relay_url)?,
                content,
            ),
        }))
    }

    #[uniffi::constructor]
    pub fn hide_channel_msg(message_id: Arc<EventId>, reason: Option<String>) -> Arc<Self> {
        Arc::new(Self {
            builder: EventBuilderSdk::hide_channel_msg(message_id.as_ref().into(), reason),
        })
    }

    #[uniffi::constructor]
    pub fn mute_channel_user(public_key: Arc<PublicKey>, reason: Option<String>) -> Arc<Self> {
        Arc::new(Self {
            builder: EventBuilderSdk::mute_channel_user(*public_key.as_ref().deref(), reason),
        })
    }

    #[uniffi::constructor]
    pub fn auth(challenge: String, relay_url: String) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            builder: EventBuilderSdk::auth(challenge, Url::parse(&relay_url)?),
        }))
    }

    // TODO: add nostr_connect method

    #[uniffi::constructor]
    pub fn report(tags: Vec<Arc<Tag>>, content: String) -> Arc<Self> {
        let tags = tags
            .into_iter()
            .map(|t| t.as_ref().deref().clone())
            .collect::<Vec<_>>();
        Arc::new(Self {
            builder: EventBuilderSdk::report(&tags, content),
        })
    }

    #[uniffi::constructor]
    pub fn new_zap_request(
        pubkey: Arc<PublicKey>,
        event_id: Option<Arc<EventId>>,
        amount: Option<u64>,
        lnurl: Option<String>,
    ) -> Arc<Self> {
        Arc::new(Self {
            builder: EventBuilderSdk::new_zap_request(
                *pubkey.as_ref().deref(),
                event_id.map(|id| id.as_ref().into()),
                amount,
                lnurl,
            ),
        })
    }

    #[uniffi::constructor]
    pub fn new_zap(bolt11: String, preimage: Option<String>, zap_request: Arc<Event>) -> Arc<Self> {
        Arc::new(Self {
            builder: EventBuilderSdk::new_zap(
                bolt11,
                preimage,
                zap_request.as_ref().deref().clone(),
            ),
        })
    }

    #[uniffi::constructor]
    pub fn file_metadata(description: String, metadata: Arc<FileMetadata>) -> Arc<Self> {
        Arc::new(Self {
            builder: EventBuilderSdk::file_metadata(description, metadata.as_ref().deref().clone()),
        })
    }
}
