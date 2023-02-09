// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;

use napi::bindgen_prelude::BigInt;
use napi::Result;
use nostr::prelude::*;

use super::JsEvent;
use crate::key::JsKeys;
use crate::types::{JsContact, JsMetadata};

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

    #[napi]
    pub fn set_metadata(metadata: &JsMetadata) -> Result<Self> {
        Ok(Self {
            builder: EventBuilder::set_metadata(metadata.deref().clone()).map_err(into_err)?,
        })
    }

    #[napi]
    pub fn add_recommended_relay(url: String) -> Result<Self> {
        let url = Url::parse(&url).map_err(into_err)?;
        Ok(Self {
            builder: EventBuilder::add_recommended_relay(&url),
        })
    }

    #[napi]
    pub fn new_text_note(content: String, tags: Vec<Vec<String>>) -> Result<Self> {
        let mut new_tags: Vec<Tag> = Vec::new();
        for tag in tags.into_iter() {
            new_tags.push(Tag::try_from(tag).map_err(into_err)?);
        }

        Ok(Self {
            builder: EventBuilder::new_text_note(content, &new_tags),
        })
    }

    #[napi]
    pub fn set_contact_list(list: Vec<&JsContact>) -> Self {
        let list: Vec<Contact> = list.into_iter().map(|c| c.deref().clone()).collect();

        Self {
            builder: EventBuilder::set_contact_list(list),
        }
    }

    #[napi]
    pub fn new_encrypted_direct_msg(
        sender_keys: &JsKeys,
        receiver_pubkey: String,
        content: String,
    ) -> Result<Self> {
        Ok(Self {
            builder: EventBuilder::new_encrypted_direct_msg(
                sender_keys.deref(),
                XOnlyPublicKey::from_str(&receiver_pubkey).map_err(into_err)?,
                content,
            )
            .map_err(into_err)?,
        })
    }

    #[napi]
    pub fn repost(event_id: String, public_key: String) -> Result<Self> {
        Ok(Self {
            builder: EventBuilder::repost(
                EventId::from_hex(event_id).map_err(into_err)?,
                XOnlyPublicKey::from_str(&public_key).map_err(into_err)?,
            ),
        })
    }

    #[napi]
    pub fn delete(ids: Vec<String>, reason: Option<String>) -> Result<Self> {
        let mut new_ids: Vec<EventId> = Vec::with_capacity(ids.len());

        for id in ids.into_iter() {
            new_ids.push(EventId::from_hex(id).map_err(into_err)?);
        }

        Ok(Self {
            builder: EventBuilder::delete(new_ids, reason.as_deref()),
        })
    }

    #[napi]
    pub fn new_reaction(event_id: String, public_key: String, content: String) -> Result<Self> {
        let event_id = EventId::from_hex(event_id).map_err(into_err)?;
        let public_key = XOnlyPublicKey::from_str(&public_key).map_err(into_err)?;
        Ok(Self {
            builder: EventBuilder::new_reaction(event_id, public_key, content),
        })
    }
}
