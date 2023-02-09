// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;

use napi::bindgen_prelude::BigInt;
use napi::Result;
use nostr::prelude::*;

use super::{JsEvent, JsEventId};
use crate::key::{JsKeys, JsPublicKey};
use crate::types::{JsChannelId, JsContact, JsMetadata};

use crate::error::into_err;

#[napi(js_name = "EventBuilder")]
pub struct JsEventBuilder {
    builder: EventBuilder,
}

impl Deref for JsEventBuilder {
    type Target = EventBuilder;
    fn deref(&self) -> &Self::Target {
        &self.builder
    }
}

#[napi]
impl JsEventBuilder {
    #[napi(constructor)]
    pub fn new(kind: BigInt, content: String, tags: Vec<Vec<String>>) -> Result<Self> {
        let kind: u64 = kind.get_u64().1;
        let mut new_tags: Vec<Tag> = Vec::with_capacity(tags.len());
        for tag in tags.into_iter() {
            new_tags.push(Tag::try_from(tag).map_err(into_err)?);
        }

        Ok(Self {
            builder: EventBuilder::new(kind.into(), content, &new_tags),
        })
    }

    #[napi]
    pub fn to_event(&self, keys: &JsKeys) -> Result<JsEvent> {
        let event = self
            .builder
            .clone()
            .to_event(keys.deref())
            .map_err(into_err)?;
        Ok(event.into())
    }

    #[napi]
    pub fn to_pow_event(&self, keys: &JsKeys, difficulty: u8) -> Result<JsEvent> {
        Ok(self
            .builder
            .clone()
            .to_pow_event(keys.deref(), difficulty)
            .map_err(into_err)?
            .into())
    }

    #[napi(factory)]
    pub fn set_metadata(metadata: &JsMetadata) -> Result<Self> {
        Ok(Self {
            builder: EventBuilder::set_metadata(metadata.deref().clone()).map_err(into_err)?,
        })
    }

    #[napi(factory)]
    pub fn add_recommended_relay(url: String) -> Result<Self> {
        let url = Url::parse(&url).map_err(into_err)?;
        Ok(Self {
            builder: EventBuilder::add_recommended_relay(&url),
        })
    }

    #[napi(factory)]
    pub fn new_text_note(content: String, tags: Vec<Vec<String>>) -> Result<Self> {
        let mut new_tags: Vec<Tag> = Vec::new();
        for tag in tags.into_iter() {
            new_tags.push(Tag::try_from(tag).map_err(into_err)?);
        }

        Ok(Self {
            builder: EventBuilder::new_text_note(content, &new_tags),
        })
    }

    #[napi(factory)]
    pub fn set_contact_list(list: Vec<&JsContact>) -> Self {
        let list: Vec<Contact> = list.into_iter().map(|c| c.deref().clone()).collect();
        Self {
            builder: EventBuilder::set_contact_list(list),
        }
    }

    #[napi(factory)]
    pub fn new_encrypted_direct_msg(
        sender_keys: &JsKeys,
        receiver_pubkey: &JsPublicKey,
        content: String,
    ) -> Result<Self> {
        Ok(Self {
            builder: EventBuilder::new_encrypted_direct_msg(
                sender_keys.deref(),
                receiver_pubkey.into(),
                content,
            )
            .map_err(into_err)?,
        })
    }

    #[napi(factory)]
    pub fn repost(event_id: &JsEventId, public_key: &JsPublicKey) -> Self {
        Self {
            builder: EventBuilder::repost(event_id.into(), public_key.into()),
        }
    }

    #[napi(factory)]
    pub fn delete(ids: Vec<&JsEventId>, reason: Option<String>) -> Self {
        let ids: Vec<EventId> = ids.into_iter().map(|id| id.into()).collect();
        Self {
            builder: EventBuilder::delete(ids, reason.as_deref()),
        }
    }

    #[napi(factory)]
    pub fn new_reaction(event_id: &JsEventId, public_key: &JsPublicKey, content: String) -> Self {
        Self {
            builder: EventBuilder::new_reaction(event_id.into(), public_key.into(), content),
        }
    }

    #[napi(factory)]
    pub fn new_channel(metadata: &JsMetadata) -> Result<Self> {
        Ok(Self {
            builder: EventBuilder::new_channel(metadata.deref().clone()).map_err(into_err)?,
        })
    }

    #[napi(factory)]
    pub fn set_channel_metadata(
        channel_id: &JsChannelId,
        relay_url: Option<String>,
        metadata: &JsMetadata,
    ) -> Result<Self> {
        let relay_url: Option<Url> = match relay_url {
            Some(relay_url) => Some(Url::parse(&relay_url).map_err(into_err)?),
            None => None,
        };
        Ok(Self {
            builder: EventBuilder::set_channel_metadata(
                channel_id.into(),
                relay_url,
                metadata.deref().clone(),
            )
            .map_err(into_err)?,
        })
    }

    #[napi(factory)]
    pub fn new_channel_msg(
        channel_id: &JsChannelId,
        relay_url: Option<String>,
        content: String,
    ) -> Result<Self> {
        let relay_url: Option<Url> = match relay_url {
            Some(relay_url) => Some(Url::parse(&relay_url).map_err(into_err)?),
            None => None,
        };
        Ok(Self {
            builder: EventBuilder::new_channel_msg(channel_id.into(), relay_url, content),
        })
    }

    #[napi(factory)]
    pub fn hide_channel_msg(message_id: &JsEventId, reason: Option<String>) -> Self {
        Self {
            builder: EventBuilder::hide_channel_msg(message_id.into(), reason),
        }
    }

    #[napi(factory)]
    pub fn mute_channel_user(pubkey: &JsPublicKey, reason: Option<String>) -> Self {
        Self {
            builder: EventBuilder::mute_channel_user(pubkey.into(), reason),
        }
    }

    #[napi(factory)]
    pub fn auth(challenge: String, relay: String) -> Result<Self> {
        let url = Url::parse(&relay).map_err(into_err)?;
        Ok(Self {
            builder: EventBuilder::auth(challenge, url),
        })
    }
}
