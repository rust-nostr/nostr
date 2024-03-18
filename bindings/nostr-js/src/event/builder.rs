// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;
use std::str::FromStr;

use nostr::prelude::*;
use wasm_bindgen::prelude::*;

use super::tag::{JsImageDimensions, JsThumbnails};
use super::{JsEvent, JsEventId, JsTag, JsUnsignedEvent};
use crate::error::{into_err, Result};
use crate::key::{JsKeys, JsPublicKey};
use crate::nips::nip15::{JsProductData, JsStallData};
use crate::nips::nip53::JsLiveEvent;
use crate::nips::nip57::JsZapRequestData;
use crate::nips::nip65::JsRelayListItem;
use crate::nips::nip90::JsDataVendingMachineStatus;
use crate::nips::nip94::JsFileMetadata;
use crate::nips::nip98::JsHttpData;
use crate::types::{JsContact, JsMetadata, JsTimestamp};

#[wasm_bindgen(js_name = EventBuilder)]
pub struct JsEventBuilder {
    inner: EventBuilder,
}

impl Deref for JsEventBuilder {
    type Target = EventBuilder;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<EventBuilder> for JsEventBuilder {
    fn from(inner: EventBuilder) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = EventBuilder)]
impl JsEventBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new(kind: f64, content: &str, tags: Vec<JsTag>) -> Self {
        Self {
            inner: EventBuilder::new(kind.into(), content, tags.into_iter().map(|t| t.into())),
        }
    }

    /// Set a custom `created_at` UNIX timestamp
    #[wasm_bindgen(js_name = customCreatedAt)]
    pub fn custom_created_at(self, created_at: JsTimestamp) -> Self {
        self.inner.custom_created_at(*created_at).into()
    }

    /// Build `Event`
    ///
    /// **This method consume the builder, so it will no longer be usable!**
    #[wasm_bindgen(js_name = toEvent)]
    pub fn to_event(self, keys: &JsKeys) -> Result<JsEvent> {
        let event = self.inner.to_event(keys.deref()).map_err(into_err)?;
        Ok(event.into())
    }

    /// Build `UnsignedEvent`
    ///
    /// **This method consume the builder, so it will no longer be usable!**
    #[wasm_bindgen(js_name = toUnsignedEvent)]
    pub fn to_unsigned_event(self, public_key: &JsPublicKey) -> JsUnsignedEvent {
        self.inner.to_unsigned_event(**public_key).into()
    }

    /// Build POW `Event`
    ///
    /// **This method consume the builder, so it will no longer be usable!**
    #[wasm_bindgen(js_name = toPowEvent)]
    pub fn to_pow_event(self, keys: &JsKeys, difficulty: u8) -> Result<JsEvent> {
        Ok(self
            .inner
            .to_pow_event(keys.deref(), difficulty)
            .map_err(into_err)?
            .into())
    }

    /// Build Unisgned POW Event
    ///
    /// **This method consume the builder, so it will no longer be usable!**
    #[wasm_bindgen(js_name = toUnsignedPowEvent)]
    pub fn to_unsigned_pow_event(
        self,
        public_key: &JsPublicKey,
        difficulty: u8,
    ) -> JsUnsignedEvent {
        self.inner
            .to_unsigned_pow_event(**public_key, difficulty)
            .into()
    }

    pub fn metadata(metadata: &JsMetadata) -> Self {
        Self {
            inner: EventBuilder::metadata(metadata.deref()),
        }
    }

    #[wasm_bindgen(js_name = relayList)]
    pub fn relay_list(relays: Vec<JsRelayListItem>) -> Self {
        Self {
            inner: EventBuilder::relay_list(relays.into_iter().map(|r| r.into())),
        }
    }

    #[wasm_bindgen(js_name = textNote)]
    pub fn text_note(content: &str, tags: Vec<JsTag>) -> Self {
        Self {
            inner: EventBuilder::text_note(content, tags.into_iter().map(|t| t.into())),
        }
    }

    /// Text note reply
    ///
    /// If no `root` is passed, the `rely_to` will be used for root `e` tag.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/10.md>
    #[wasm_bindgen(js_name = textNoteReply)]
    pub fn text_note_reply(
        content: &str,
        reply_to: &JsEvent,
        root: Option<JsEvent>,
        relay_url: Option<String>,
    ) -> Self {
        Self {
            inner: EventBuilder::text_note_reply(
                content,
                reply_to.deref(),
                root.as_deref(),
                relay_url.map(UncheckedUrl::from),
            ),
        }
    }

    #[wasm_bindgen(js_name = longFormTextNote)]
    pub fn long_form_text_note(content: &str, tags: Vec<JsTag>) -> Self {
        Self {
            inner: EventBuilder::long_form_text_note(content, tags.into_iter().map(|t| t.into())),
        }
    }

    #[wasm_bindgen(js_name = contactList)]
    pub fn contact_list(list: Vec<JsContact>) -> Self {
        let list = list.into_iter().map(|c| c.inner());
        Self {
            inner: EventBuilder::contact_list(list),
        }
    }

    #[wasm_bindgen(js_name = encryptedDirectMsg)]
    pub fn encrypted_direct_msg(
        sender_keys: &JsKeys,
        receiver_pubkey: &JsPublicKey,
        content: &str,
        reply_to: Option<JsEventId>,
    ) -> Result<JsEventBuilder> {
        Ok(Self {
            inner: EventBuilder::encrypted_direct_msg(
                sender_keys.deref(),
                receiver_pubkey.into(),
                content,
                reply_to.map(|id| id.into()),
            )
            .map_err(into_err)?,
        })
    }

    /// Repost
    pub fn repost(event: &JsEvent, relay_url: Option<String>) -> Self {
        Self {
            inner: EventBuilder::repost(event.deref(), relay_url.map(UncheckedUrl::from)),
        }
    }

    #[wasm_bindgen]
    pub fn delete(ids: Vec<JsEventId>, reason: Option<String>) -> Self {
        let ids = ids.into_iter().map(|id| id.inner);
        Self {
            inner: match reason {
                Some(reason) => EventBuilder::delete_with_reason(ids, reason),
                None => EventBuilder::delete(ids),
            },
        }
    }

    pub fn reaction(event: &JsEvent, reaction: &str) -> Self {
        Self {
            inner: EventBuilder::reaction(event.deref(), reaction),
        }
    }

    pub fn channel(metadata: &JsMetadata) -> Self {
        Self {
            inner: EventBuilder::channel(metadata.deref()),
        }
    }

    #[wasm_bindgen(js_name = channelMetadata)]
    pub fn channel_metadata(
        channel_id: &JsEventId,
        relay_url: Option<String>,
        metadata: &JsMetadata,
    ) -> Result<JsEventBuilder> {
        let relay_url: Option<Url> = match relay_url {
            Some(relay_url) => Some(Url::parse(&relay_url).map_err(into_err)?),
            None => None,
        };
        Ok(Self {
            inner: EventBuilder::channel_metadata(channel_id.into(), relay_url, metadata.deref()),
        })
    }

    #[wasm_bindgen(js_name = channelMsg)]
    pub fn channel_msg(
        channel_id: &JsEventId,
        relay_url: &str,
        content: &str,
    ) -> Result<JsEventBuilder> {
        let relay_url: Url = Url::parse(relay_url).map_err(into_err)?;
        Ok(Self {
            inner: EventBuilder::channel_msg(channel_id.into(), relay_url, content),
        })
    }

    #[wasm_bindgen(js_name = hideChannelMsg)]
    pub fn hide_channel_msg(message_id: &JsEventId, reason: Option<String>) -> Self {
        Self {
            inner: EventBuilder::hide_channel_msg(message_id.into(), reason),
        }
    }

    #[wasm_bindgen(js_name = muteChannelUser)]
    pub fn mute_channel_user(pubkey: &JsPublicKey, reason: Option<String>) -> Self {
        Self {
            inner: EventBuilder::mute_channel_user(pubkey.into(), reason),
        }
    }

    #[wasm_bindgen]
    pub fn auth(challenge: &str, relay: &str) -> Result<JsEventBuilder> {
        let url = Url::parse(relay).map_err(into_err)?;
        Ok(Self {
            inner: EventBuilder::auth(challenge, url),
        })
    }

    #[wasm_bindgen(js_name = liveEvent)]
    pub fn live_event(live_event: JsLiveEvent) -> Self {
        Self {
            inner: EventBuilder::live_event(live_event.into()),
        }
    }

    #[wasm_bindgen(js_name = liveEventMsg)]
    pub fn live_event_msg(
        live_event_id: String,
        live_event_host: JsPublicKey,
        content: String,
        relay_url: Option<String>,
        tags: Vec<JsTag>,
    ) -> Result<JsEventBuilder> {
        Ok(Self {
            inner: EventBuilder::live_event_msg(
                live_event_id,
                live_event_host.deref().to_owned(),
                content,
                match relay_url {
                    Some(url) => Some(Url::from_str(&url).map_err(into_err)?),
                    None => None,
                },
                tags.into_iter().map(|t| t.into()).collect(),
            ),
        })
    }

    #[wasm_bindgen]
    pub fn report(tags: Vec<JsTag>, content: String) -> Self {
        Self {
            inner: EventBuilder::report(tags.into_iter().map(|t| t.into()), content),
        }
    }

    #[wasm_bindgen(js_name = publicZapRequest)]
    pub fn public_zap_request(data: JsZapRequestData) -> Self {
        Self {
            inner: EventBuilder::public_zap_request(data.deref().clone()),
        }
    }

    #[wasm_bindgen(js_name = zapReceipt)]
    pub fn zap_receipt(bolt11: String, preimage: Option<String>, zap_request: JsEvent) -> Self {
        Self {
            inner: EventBuilder::zap_receipt(bolt11, preimage, zap_request.deref().to_owned()),
        }
    }

    #[wasm_bindgen(js_name = defineBadge)]
    pub fn define_badge(
        badge_id: String,
        name: Option<String>,
        description: Option<String>,
        image: Option<String>,
        image_dimensions: Option<JsImageDimensions>,
        thumbnails: Vec<JsThumbnails>,
    ) -> Self {
        Self {
            inner: EventBuilder::define_badge(
                badge_id,
                name,
                description,
                image.map(UncheckedUrl::from),
                image_dimensions.map(|i| i.into()),
                thumbnails.into_iter().map(|t| t.into()).collect(),
            ),
        }
    }

    #[wasm_bindgen(js_name = awardBadge)]
    pub fn award_badge(
        badge_definition: &JsEvent,
        awarded_pubkeys: Vec<JsTag>,
    ) -> Result<JsEventBuilder> {
        Ok(Self {
            inner: EventBuilder::award_badge(
                badge_definition.deref(),
                awarded_pubkeys.into_iter().map(|t| t.into()),
            )
            .map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = profileBadges)]
    pub fn profile_badges(
        badge_definitions: Vec<JsEvent>,
        badge_awards: Vec<JsEvent>,
        pubkey_awarded: &JsPublicKey,
    ) -> Result<JsEventBuilder> {
        Ok(Self {
            inner: EventBuilder::profile_badges(
                badge_definitions.into_iter().map(|e| e.into()).collect(),
                badge_awards.into_iter().map(|e| e.into()).collect(),
                pubkey_awarded.deref(),
            )
            .map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = jobRequest)]
    pub fn job_request(kind: f64, tags: Vec<JsTag>) -> Result<JsEventBuilder> {
        Ok(Self {
            inner: EventBuilder::job_request(kind.into(), tags.into_iter().map(|t| t.into()))
                .map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = jobResult)]
    pub fn job_result(
        job_request: &JsEvent,
        amount_millisats: f64,
        bolt11: Option<String>,
    ) -> Result<JsEventBuilder> {
        Ok(Self {
            inner: EventBuilder::job_result(
                job_request.deref().clone(),
                amount_millisats as u64,
                bolt11,
            )
            .map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = jobFeedback)]
    pub fn job_feedback(
        job_request: &JsEvent,
        status: JsDataVendingMachineStatus,
        extra_info: Option<String>,
        amount_millisats: u64,
        bolt11: Option<String>,
        payload: Option<String>,
    ) -> Self {
        Self {
            inner: EventBuilder::job_feedback(
                job_request.deref(),
                status.into(),
                extra_info,
                amount_millisats,
                bolt11,
                payload,
            ),
        }
    }

    #[wasm_bindgen(js_name = fileMetadata)]
    pub fn file_metadata(description: String, metadata: JsFileMetadata) -> Self {
        Self {
            inner: EventBuilder::file_metadata(description, metadata.deref().clone()),
        }
    }

    #[wasm_bindgen(js_name = httpAuth)]
    pub fn http_auth(data: JsHttpData) -> Self {
        Self {
            inner: EventBuilder::http_auth(data.into()),
        }
    }

    #[wasm_bindgen(js_name = stallData)]
    pub fn stall_data(data: &JsStallData) -> Self {
        Self {
            inner: EventBuilder::stall_data(data.deref().clone()),
        }
    }

    #[wasm_bindgen(js_name = productData)]
    pub fn product_data(data: JsProductData) -> Self {
        Self {
            inner: EventBuilder::product_data(data.into()),
        }
    }

    /// Gift Wrap from seal
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/59.md>
    #[wasm_bindgen(js_name = giftWrapFromSeal)]
    pub fn gift_wrap_from_seal(
        receiver: &JsPublicKey,
        seal: &JsEvent,
        expiration: Option<JsTimestamp>,
    ) -> Result<JsEvent> {
        Ok(EventBuilder::gift_wrap_from_seal(
            receiver.deref(),
            seal.deref(),
            expiration.map(|t| *t),
        )
        .map_err(into_err)?
        .into())
    }

    /// Gift Wrap
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/59.md>
    #[wasm_bindgen(js_name = giftWrap)]
    pub fn gift_wrap(
        sender_keys: &JsKeys,
        receiver: &JsPublicKey,
        rumor: &JsUnsignedEvent,
        expiration: Option<JsTimestamp>,
    ) -> Result<JsEvent> {
        Ok(EventBuilder::gift_wrap(
            sender_keys.deref(),
            receiver.deref(),
            rumor.deref().clone(),
            expiration.map(|t| *t),
        )
        .map_err(into_err)?
        .into())
    }

    /// GiftWrapped Sealed Direct message
    #[wasm_bindgen(js_name = sealedDirect)]
    pub fn sealed_direct(receiver: &JsPublicKey, message: &str) -> Self {
        Self {
            inner: EventBuilder::sealed_direct(**receiver, message),
        }
    }
}
