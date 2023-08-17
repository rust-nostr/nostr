// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::url::Url;
use nostr::{ChannelId, Contact as ContactSdk, EventBuilder as EventBuilderSdk};

use super::{Event, EventId};
use crate::error::Result;
use crate::key::Keys;
use crate::nips::nip57::ZapRequestData;
use crate::types::{Contact, Metadata};
use crate::{FileMetadata, Kind, PublicKey, Tag, UnsignedEvent};

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

impl EventBuilder {
    pub fn new(kind: Kind, content: String, tags: Vec<Arc<Tag>>) -> Result<Self> {
        let tags = tags
            .into_iter()
            .map(|t| t.as_ref().deref().clone())
            .collect::<Vec<_>>();
        Ok(Self {
            builder: EventBuilderSdk::new(kind.into(), content, &tags),
        })
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

impl EventBuilder {
    pub fn set_metadata(metadata: Arc<Metadata>) -> Self {
        Self {
            builder: EventBuilderSdk::set_metadata(metadata.as_ref().deref().clone()),
        }
    }

    pub fn add_recommended_relay(url: String) -> Result<Self> {
        let url = Url::parse(&url)?;

        Ok(Self {
            builder: EventBuilderSdk::add_recommended_relay(&url),
        })
    }

    pub fn new_text_note(content: String, tags: Vec<Arc<Tag>>) -> Result<Self> {
        let tags = tags
            .into_iter()
            .map(|t| t.as_ref().deref().clone())
            .collect::<Vec<_>>();
        Ok(Self {
            builder: EventBuilderSdk::new_text_note(content, &tags),
        })
    }

    pub fn long_form_text_note(content: String, tags: Vec<Arc<Tag>>) -> Result<Self> {
        let tags = tags
            .into_iter()
            .map(|t| t.as_ref().deref().clone())
            .collect::<Vec<_>>();
        Ok(Self {
            builder: EventBuilderSdk::long_form_text_note(content, &tags),
        })
    }

    pub fn set_contact_list(list: Vec<Arc<Contact>>) -> Self {
        let list: Vec<ContactSdk> = list
            .into_iter()
            .map(|c| c.as_ref().deref().clone())
            .collect();

        Self {
            builder: EventBuilderSdk::set_contact_list(list),
        }
    }

    /// Create encrypted direct msg event
    pub fn new_encrypted_direct_msg(
        sender_keys: Arc<Keys>,
        receiver_pubkey: Arc<PublicKey>,
        content: String,
        reply: Option<Arc<EventId>>,
    ) -> Result<Self> {
        Ok(Self {
            builder: EventBuilderSdk::new_encrypted_direct_msg(
                sender_keys.deref(),
                *receiver_pubkey.as_ref().deref(),
                content,
                reply.map(|id| id.as_ref().into()),
            )?,
        })
    }

    pub fn repost(event_id: Arc<EventId>, public_key: Arc<PublicKey>) -> Self {
        Self {
            builder: EventBuilderSdk::repost(
                event_id.as_ref().into(),
                *public_key.as_ref().deref(),
            ),
        }
    }

    /// Create delete event
    pub fn delete(ids: Vec<Arc<EventId>>, reason: Option<String>) -> Self {
        let ids: Vec<nostr::EventId> = ids.into_iter().map(|e| e.as_ref().into()).collect();
        Self {
            builder: EventBuilderSdk::delete(ids, reason.as_deref()),
        }
    }

    pub fn new_reaction(
        event_id: Arc<EventId>,
        public_key: Arc<PublicKey>,
        content: String,
    ) -> Self {
        Self {
            builder: EventBuilderSdk::new_reaction(
                event_id.as_ref().into(),
                *public_key.as_ref().deref(),
                content,
            ),
        }
    }

    pub fn new_channel(metadata: Arc<Metadata>) -> Self {
        Self {
            builder: EventBuilderSdk::new_channel(metadata.as_ref().deref().clone()),
        }
    }

    pub fn set_channel_metadata(
        channel_id: String,
        relay_url: Option<String>,
        metadata: Arc<Metadata>,
    ) -> Result<Self> {
        let relay_url = match relay_url {
            Some(url) => Some(Url::parse(&url)?),
            None => None,
        };
        Ok(Self {
            builder: EventBuilderSdk::set_channel_metadata(
                ChannelId::from_hex(channel_id)?,
                relay_url,
                metadata.as_ref().deref().clone(),
            ),
        })
    }

    pub fn new_channel_msg(channel_id: String, relay_url: String, content: String) -> Result<Self> {
        Ok(Self {
            builder: EventBuilderSdk::new_channel_msg(
                ChannelId::from_hex(channel_id)?,
                Url::parse(&relay_url)?,
                content,
            ),
        })
    }

    pub fn hide_channel_msg(message_id: Arc<EventId>, reason: Option<String>) -> Self {
        Self {
            builder: EventBuilderSdk::hide_channel_msg(message_id.as_ref().into(), reason),
        }
    }

    pub fn mute_channel_user(public_key: Arc<PublicKey>, reason: Option<String>) -> Self {
        Self {
            builder: EventBuilderSdk::mute_channel_user(*public_key.as_ref().deref(), reason),
        }
    }

    pub fn auth(challenge: String, relay_url: String) -> Result<Self> {
        Ok(Self {
            builder: EventBuilderSdk::auth(challenge, Url::parse(&relay_url)?),
        })
    }

    // TODO: add nostr_connect method

    pub fn report(tags: Vec<Arc<Tag>>, content: String) -> Self {
        let tags = tags
            .into_iter()
            .map(|t| t.as_ref().deref().clone())
            .collect::<Vec<_>>();
        Self {
            builder: EventBuilderSdk::report(&tags, content),
        }
    }

    pub fn new_zap_request(data: Arc<ZapRequestData>) -> Self {
        Self {
            builder: EventBuilderSdk::new_zap_request(data.as_ref().deref().clone()),
        }
    }

    pub fn new_zap_receipt(
        bolt11: String,
        preimage: Option<String>,
        zap_request: Arc<Event>,
    ) -> Self {
        Self {
            builder: EventBuilderSdk::new_zap_receipt(
                bolt11,
                preimage,
                zap_request.as_ref().deref().clone(),
            ),
        }
    }

    pub fn file_metadata(description: String, metadata: Arc<FileMetadata>) -> Self {
        Self {
            builder: EventBuilderSdk::file_metadata(description, metadata.as_ref().deref().clone()),
        }
    }
}
