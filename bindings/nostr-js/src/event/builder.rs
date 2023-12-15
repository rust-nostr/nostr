// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use js_sys::Array;
use nostr::prelude::*;
use wasm_bindgen::prelude::*;

use super::{JsEvent, JsEventId, JsTags};
use crate::error::{into_err, Result};
use crate::key::{JsKeys, JsPublicKey};
use crate::types::{JsChannelId, JsContact, JsMetadata};
use crate::util;

#[wasm_bindgen(js_name = EventBuilder)]
pub struct JsEventBuilder {
    builder: EventBuilder,
}

impl Deref for JsEventBuilder {
    type Target = EventBuilder;
    fn deref(&self) -> &Self::Target {
        &self.builder
    }
}

#[wasm_bindgen(js_class = EventBuilder)]
impl JsEventBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new(kind: u64, content: String, tags: JsTags) -> Result<JsEventBuilder> {
        let tags: Vec<Vec<String>> = serde_wasm_bindgen::from_value(tags.into())?;
        let mut new_tags: Vec<Tag> = Vec::with_capacity(tags.len());
        for tag in tags.into_iter() {
            new_tags.push(Tag::try_from(tag).map_err(into_err)?);
        }

        Ok(Self {
            builder: EventBuilder::new(kind.into(), content, new_tags),
        })
    }

    #[wasm_bindgen(js_name = toEvent)]
    pub fn to_event(&self, keys: &JsKeys) -> Result<JsEvent> {
        let event = self
            .builder
            .clone()
            .to_event(keys.deref())
            .map_err(into_err)?;
        Ok(event.into())
    }

    #[wasm_bindgen(js_name = toPowEvent)]
    pub fn to_pow_event(&self, keys: &JsKeys, difficulty: u8) -> Result<JsEvent> {
        Ok(self
            .builder
            .clone()
            .to_pow_event(keys.deref(), difficulty)
            .map_err(into_err)?
            .into())
    }

    #[wasm_bindgen(js_name = setMetadata)]
    pub fn set_metadata(metadata: &JsMetadata) -> Self {
        Self {
            builder: EventBuilder::set_metadata(metadata.deref()),
        }
    }

    #[wasm_bindgen(js_name = addRecommendedRelay)]
    pub fn add_recommended_relay(url: String) -> Result<JsEventBuilder> {
        let url = Url::parse(&url).map_err(into_err)?;
        Ok(Self {
            builder: EventBuilder::add_recommended_relay(&url),
        })
    }

    #[wasm_bindgen(js_name = newTextNote)]
    pub fn new_text_note(content: String, tags: JsTags) -> Result<JsEventBuilder> {
        let tags: Vec<Vec<String>> = serde_wasm_bindgen::from_value(tags.into())?;
        let mut new_tags: Vec<Tag> = Vec::with_capacity(tags.len());
        for tag in tags.into_iter() {
            new_tags.push(Tag::try_from(tag).map_err(into_err)?);
        }

        Ok(Self {
            builder: EventBuilder::new_text_note(content, new_tags),
        })
    }

    #[wasm_bindgen(js_name = setContactList)]
    pub fn set_contact_list(list: Array) -> Result<JsEventBuilder> {
        let list = list
            .iter()
            .map(|v| Ok(util::downcast::<JsContact>(&v, "Contact")?.inner()))
            .collect::<Result<Vec<Contact>, JsError>>()?;
        Ok(Self {
            builder: EventBuilder::set_contact_list(list),
        })
    }

    #[wasm_bindgen(js_name = newEncryptedDirectMsg)]
    pub fn new_encrypted_direct_msg(
        sender_keys: &JsKeys,
        receiver_pubkey: &JsPublicKey,
        content: String,
        reply_to: Option<JsEventId>,
    ) -> Result<JsEventBuilder> {
        Ok(Self {
            builder: EventBuilder::new_encrypted_direct_msg(
                sender_keys.deref(),
                receiver_pubkey.into(),
                content,
                reply_to.map(|id| id.into()),
            )
            .map_err(into_err)?,
        })
    }

    #[wasm_bindgen]
    pub fn repost(event_id: &JsEventId, public_key: &JsPublicKey) -> Self {
        Self {
            builder: EventBuilder::repost(event_id.into(), public_key.into()),
        }
    }

    #[wasm_bindgen]
    pub fn delete(ids: Array, reason: Option<String>) -> Result<JsEventBuilder> {
        let ids = ids
            .iter()
            .map(|v| Ok(util::downcast::<JsEventId>(&v, "EventId")?.inner))
            .collect::<Result<Vec<EventId>, JsError>>()?;
        Ok(Self {
            builder: match reason {
                Some(reason) => EventBuilder::delete_with_reason(ids, reason),
                None => EventBuilder::delete(ids),
            },
        })
    }

    #[wasm_bindgen(js_name = newReaction)]
    pub fn new_reaction(event_id: &JsEventId, public_key: &JsPublicKey, content: String) -> Self {
        Self {
            builder: EventBuilder::new_reaction(event_id.into(), public_key.into(), content),
        }
    }

    #[wasm_bindgen(js_name = newChannel)]
    pub fn new_channel(metadata: &JsMetadata) -> Self {
        Self {
            builder: EventBuilder::new_channel(metadata.deref()),
        }
    }

    #[wasm_bindgen(js_name = setChannelMetadata)]
    pub fn set_channel_metadata(
        channel_id: &JsChannelId,
        relay_url: Option<String>,
        metadata: &JsMetadata,
    ) -> Result<JsEventBuilder> {
        let relay_url: Option<Url> = match relay_url {
            Some(relay_url) => Some(Url::parse(&relay_url).map_err(into_err)?),
            None => None,
        };
        Ok(Self {
            builder: EventBuilder::set_channel_metadata(
                channel_id.into(),
                relay_url,
                metadata.deref(),
            ),
        })
    }

    #[wasm_bindgen(js_name = newChannelMsg)]
    pub fn new_channel_msg(
        channel_id: &JsChannelId,
        relay_url: String,
        content: String,
    ) -> Result<JsEventBuilder> {
        let relay_url: Url = Url::parse(&relay_url).map_err(into_err)?;
        Ok(Self {
            builder: EventBuilder::new_channel_msg(channel_id.into(), relay_url, content),
        })
    }

    #[wasm_bindgen(js_name = hideChannelMsg)]
    pub fn hide_channel_msg(message_id: &JsEventId, reason: Option<String>) -> Self {
        Self {
            builder: EventBuilder::hide_channel_msg(message_id.into(), reason),
        }
    }

    #[wasm_bindgen(js_name = muteChannelUser)]
    pub fn mute_channel_user(pubkey: &JsPublicKey, reason: Option<String>) -> Self {
        Self {
            builder: EventBuilder::mute_channel_user(pubkey.into(), reason),
        }
    }

    #[wasm_bindgen]
    pub fn auth(challenge: String, relay: String) -> Result<JsEventBuilder> {
        let url = Url::parse(&relay).map_err(into_err)?;
        Ok(Self {
            builder: EventBuilder::auth(challenge, url),
        })
    }
}
