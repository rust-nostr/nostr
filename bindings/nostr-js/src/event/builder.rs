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
use crate::nips::nip01::JsCoordinate;
use crate::nips::nip15::{JsProductData, JsStallData};
use crate::nips::nip51::{
    JsArticlesCuration, JsBookmarks, JsEmojiInfo, JsEmojis, JsInterests, JsMuteList,
};
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
    pub fn new(kind: u16, content: &str, tags: Vec<JsTag>) -> Self {
        Self {
            inner: EventBuilder::new(kind.into(), content, tags.into_iter().map(|t| t.into())),
        }
    }

    /// Add tags
    pub fn add_tags(self, tags: Vec<JsTag>) -> Self {
        self.inner
            .add_tags(tags.into_iter().map(|t| t.into()))
            .into()
    }

    /// Set a custom `created_at` UNIX timestamp
    #[wasm_bindgen(js_name = customCreatedAt)]
    pub fn custom_created_at(self, created_at: &JsTimestamp) -> Self {
        self.inner.custom_created_at(**created_at).into()
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

    /// Profile metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    pub fn metadata(metadata: &JsMetadata) -> Self {
        Self {
            inner: EventBuilder::metadata(metadata.deref()),
        }
    }

    /// Relay list metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/65.md>
    #[wasm_bindgen(js_name = relayList)]
    pub fn relay_list(relays: Vec<JsRelayListItem>) -> Result<JsEventBuilder> {
        let mut list = Vec::with_capacity(relays.len());
        for JsRelayListItem { url, metadata } in relays.into_iter() {
            let relay_url: Url = Url::parse(&url).map_err(into_err)?;
            let metadata = metadata.map(|m| m.into());
            list.push((relay_url, metadata))
        }
        Ok(Self {
            inner: EventBuilder::relay_list(list),
        })
    }

    /// Text note
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
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

    /// Long-form text note (generally referred to as "articles" or "blog posts").
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/23.md>
    #[wasm_bindgen(js_name = longFormTextNote)]
    pub fn long_form_text_note(content: &str, tags: Vec<JsTag>) -> Self {
        Self {
            inner: EventBuilder::long_form_text_note(content, tags.into_iter().map(|t| t.into())),
        }
    }

    /// Contact/Follow list
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/02.md>
    #[inline]
    #[wasm_bindgen(js_name = contactList)]
    pub fn contact_list(list: Vec<JsContact>) -> Self {
        Self {
            inner: EventBuilder::contact_list(list.into_iter().map(|c| c.into())),
        }
    }

    /// Create encrypted direct msg event
    ///
    /// <div class="warning"><strong>Unsecure!</strong> Deprecated in favor of NIP-17!</div>
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
                **receiver_pubkey,
                content,
                reply_to.map(|id| id.into()),
            )
            .map_err(into_err)?,
        })
    }

    /// Repost
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/18.md>
    pub fn repost(event: &JsEvent, relay_url: Option<String>) -> Self {
        Self {
            inner: EventBuilder::repost(event.deref(), relay_url.map(UncheckedUrl::from)),
        }
    }

    /// Event deletion
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/09.md>
    #[wasm_bindgen]
    pub fn delete(ids: Vec<JsEventId>, reason: Option<String>) -> Self {
        let ids = ids.into_iter().map(|id| *id);
        Self {
            inner: match reason {
                Some(reason) => EventBuilder::delete_with_reason(ids, reason),
                None => EventBuilder::delete(ids),
            },
        }
    }

    /// Add reaction (like/upvote, dislike/downvote or emoji) to an event
    pub fn reaction(event: &JsEvent, reaction: &str) -> Self {
        Self {
            inner: EventBuilder::reaction(event.deref(), reaction),
        }
    }

    /// Add reaction (like/upvote, dislike/downvote or emoji) to an event
    #[wasm_bindgen(js_name = reactionExtended)]
    pub fn reaction_extended(
        event_id: &JsEventId,
        public_key: &JsPublicKey,
        kind: Option<u16>,
        reaction: &str,
    ) -> Self {
        Self {
            inner: nostr::EventBuilder::reaction_extended(
                **event_id,
                **public_key,
                kind.map(|k| k.into()),
                reaction,
            ),
        }
    }

    /// Create new channel
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    pub fn channel(metadata: &JsMetadata) -> Self {
        Self {
            inner: EventBuilder::channel(metadata.deref()),
        }
    }

    /// Channel metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
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
            inner: EventBuilder::channel_metadata(**channel_id, relay_url, metadata.deref()),
        })
    }

    /// Channel message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    #[wasm_bindgen(js_name = channelMsg)]
    pub fn channel_msg(
        channel_id: &JsEventId,
        relay_url: &str,
        content: &str,
    ) -> Result<JsEventBuilder> {
        let relay_url: Url = Url::parse(relay_url).map_err(into_err)?;
        Ok(Self {
            inner: EventBuilder::channel_msg(**channel_id, relay_url, content),
        })
    }

    /// Hide message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    #[wasm_bindgen(js_name = hideChannelMsg)]
    pub fn hide_channel_msg(message_id: &JsEventId, reason: Option<String>) -> Self {
        Self {
            inner: EventBuilder::hide_channel_msg(**message_id, reason),
        }
    }

    /// Mute channel user
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    #[wasm_bindgen(js_name = muteChannelUser)]
    pub fn mute_channel_user(pubkey: &JsPublicKey, reason: Option<String>) -> Self {
        Self {
            inner: EventBuilder::mute_channel_user(**pubkey, reason),
        }
    }

    /// Authentication of clients to relays
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/42.md>
    #[wasm_bindgen]
    pub fn auth(challenge: &str, relay: &str) -> Result<JsEventBuilder> {
        let url = Url::parse(relay).map_err(into_err)?;
        Ok(Self {
            inner: EventBuilder::auth(challenge, url),
        })
    }

    // TODO: add nostr_connect method

    /// Live Event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/53.md>
    #[wasm_bindgen(js_name = liveEvent)]
    pub fn live_event(live_event: &JsLiveEvent) -> Self {
        Self {
            inner: EventBuilder::live_event(live_event.deref().clone()),
        }
    }

    /// Live Event Message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/53.md>
    #[wasm_bindgen(js_name = liveEventMsg)]
    pub fn live_event_msg(
        live_event_id: &str,
        live_event_host: &JsPublicKey,
        content: &str,
        relay_url: Option<String>,
    ) -> Result<JsEventBuilder> {
        Ok(Self {
            inner: EventBuilder::live_event_msg(
                live_event_id,
                **live_event_host,
                content,
                match relay_url {
                    Some(url) => Some(Url::from_str(&url).map_err(into_err)?),
                    None => None,
                },
            ),
        })
    }

    /// Reporting
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/56.md>
    #[wasm_bindgen]
    pub fn report(tags: Vec<JsTag>, content: &str) -> Self {
        Self {
            inner: EventBuilder::report(tags.into_iter().map(|t| t.into()), content),
        }
    }

    /// Create **public** zap request event
    ///
    /// **This event MUST NOT be broadcasted to relays**, instead must be sent to a recipient's LNURL pay callback url.
    ///
    /// To build a **private** or **anonymous** zap request use `nip57PrivateZapRequest(...)` or `nip57AnonymousZapRequest(...)` functions.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/57.md>
    #[wasm_bindgen(js_name = publicZapRequest)]
    pub fn public_zap_request(data: &JsZapRequestData) -> Self {
        Self {
            inner: EventBuilder::public_zap_request(data.deref().clone()),
        }
    }

    /// Zap Receipt
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/57.md>
    #[wasm_bindgen(js_name = zapReceipt)]
    pub fn zap_receipt(bolt11: &str, preimage: Option<String>, zap_request: &JsEvent) -> Self {
        Self {
            inner: EventBuilder::zap_receipt(bolt11, preimage, zap_request.deref()),
        }
    }

    /// Badge definition
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/58.md>
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

    /// Badge award
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/58.md>
    #[wasm_bindgen(js_name = awardBadge)]
    pub fn award_badge(
        badge_definition: &JsEvent,
        awarded_public_keys: Vec<JsTag>,
    ) -> Result<JsEventBuilder> {
        Ok(Self {
            inner: EventBuilder::award_badge(
                badge_definition.deref(),
                awarded_public_keys.into_iter().map(|t| t.into()),
            )
            .map_err(into_err)?,
        })
    }

    /// Profile badges
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/58.md>
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

    /// Data Vending Machine (DVM) - Job Request
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    #[wasm_bindgen(js_name = jobRequest)]
    pub fn job_request(kind: u16, tags: Vec<JsTag>) -> Result<JsEventBuilder> {
        Ok(Self {
            inner: EventBuilder::job_request(kind.into(), tags.into_iter().map(|t| t.into()))
                .map_err(into_err)?,
        })
    }

    /// Data Vending Machine (DVM) - Job Result
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
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

    /// Data Vending Machine (DVM) - Job Feedback
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
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

    /// File metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/94.md>
    #[wasm_bindgen(js_name = fileMetadata)]
    pub fn file_metadata(description: &str, metadata: &JsFileMetadata) -> Self {
        Self {
            inner: EventBuilder::file_metadata(description, metadata.deref().clone()),
        }
    }

    /// HTTP Auth
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/98.md>
    #[wasm_bindgen(js_name = httpAuth)]
    pub fn http_auth(data: &JsHttpData) -> Self {
        Self {
            inner: EventBuilder::http_auth(data.deref().clone()),
        }
    }

    /// Set stall data
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/15.md>
    #[wasm_bindgen(js_name = stallData)]
    pub fn stall_data(data: &JsStallData) -> Self {
        Self {
            inner: EventBuilder::stall_data(data.deref().clone()),
        }
    }

    /// Set product data
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/15.md>
    #[wasm_bindgen(js_name = productData)]
    pub fn product_data(data: &JsProductData) -> Self {
        Self {
            inner: EventBuilder::product_data(data.deref().clone()),
        }
    }

    // TODO: add seal

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

    /// Private Direct message rumor
    ///
    /// <div class="warning">
    /// This constructor compose ONLY the rumor for the private direct message!
    /// NOT USE THIS IF YOU DON'T KNOW WHAT YOU ARE DOING!
    /// </div>
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/17.md>
    #[wasm_bindgen(js_name = privateMsgRumor)]
    pub fn private_msg_rumor(
        receiver: &JsPublicKey,
        message: &str,
        reply_to: Option<JsEventId>,
    ) -> Self {
        Self {
            inner: EventBuilder::private_msg_rumor(**receiver, message, reply_to.map(|id| *id)),
        }
    }

    /// Mute list
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[wasm_bindgen(js_name = muteList)]
    pub fn mute_list(list: &JsMuteList) -> Self {
        Self {
            inner: EventBuilder::mute_list(list.clone().into()),
        }
    }

    /// Pinned notes
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[wasm_bindgen(js_name = pinnedNotes)]
    pub fn pinned_notes(ids: Vec<JsEventId>) -> Self {
        Self {
            inner: EventBuilder::pinned_notes(ids.into_iter().map(|e| e.into())),
        }
    }

    /// Bookmarks
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[wasm_bindgen(js_name = bookmarks)]
    pub fn bookmarks(list: &JsBookmarks) -> Result<JsEventBuilder> {
        Ok(Self {
            inner: EventBuilder::bookmarks(list.clone().try_into()?),
        })
    }

    /// Communities
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[wasm_bindgen(js_name = communities)]
    pub fn communities(communities: Vec<JsCoordinate>) -> Self {
        Self {
            inner: EventBuilder::communities(communities.into_iter().map(|c| c.deref().clone())),
        }
    }

    /// Public chats
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[wasm_bindgen(js_name = publicChats)]
    pub fn public_chats(chat: Vec<JsEventId>) -> Self {
        Self {
            inner: EventBuilder::public_chats(chat.into_iter().map(|e| e.into())),
        }
    }

    /// Blocked relays
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[wasm_bindgen(js_name = blockedRelays)]
    pub fn blocked_relays(relays: Vec<String>) -> Self {
        Self {
            inner: EventBuilder::blocked_relays(relays.into_iter().map(UncheckedUrl::from)),
        }
    }

    /// Search relays
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[wasm_bindgen(js_name = searchRelays)]
    pub fn search_relays(relays: Vec<String>) -> Self {
        Self {
            inner: EventBuilder::search_relays(relays.into_iter().map(UncheckedUrl::from)),
        }
    }

    /// Interests
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[wasm_bindgen]
    pub fn interests(list: &JsInterests) -> Self {
        Self {
            inner: EventBuilder::interests(list.clone().into()),
        }
    }

    /// Emojis
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[wasm_bindgen]
    pub fn emojis(list: &JsEmojis) -> Self {
        Self {
            inner: EventBuilder::emojis(list.clone().into()),
        }
    }

    /// Follow set
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[wasm_bindgen(js_name = followSet)]
    pub fn follow_set(identifier: &str, public_keys: Vec<JsPublicKey>) -> Self {
        Self {
            inner: EventBuilder::follow_set(identifier, public_keys.into_iter().map(|p| p.into())),
        }
    }

    /// Relay set
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[wasm_bindgen(js_name = relaySet)]
    pub fn relay_set(identifier: &str, relays: Vec<String>) -> Self {
        Self {
            inner: EventBuilder::relay_set(identifier, relays.into_iter().map(UncheckedUrl::from)),
        }
    }

    /// Bookmark set
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[wasm_bindgen(js_name = bookmarksSet)]
    pub fn bookmarks_set(identifier: &str, list: &JsBookmarks) -> Result<JsEventBuilder> {
        Ok(Self {
            inner: EventBuilder::bookmarks_set(identifier, list.clone().try_into()?),
        })
    }

    /// Article Curation set
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[wasm_bindgen(js_name = articlesCurationSet)]
    pub fn articles_curation_set(identifier: &str, list: &JsArticlesCuration) -> Self {
        Self {
            inner: EventBuilder::articles_curation_set(identifier, list.clone().into()),
        }
    }

    /// Videos Curation set
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[wasm_bindgen(js_name = videosCurationSet)]
    pub fn videos_curation_set(identifier: &str, video: Vec<JsCoordinate>) -> Self {
        Self {
            inner: EventBuilder::videos_curation_set(
                identifier,
                video.into_iter().map(|c| c.deref().clone()),
            ),
        }
    }

    /// Interest set
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[wasm_bindgen(js_name = interestSet)]
    pub fn interest_set(identifier: &str, hashtags: Vec<String>) -> Self {
        Self {
            inner: EventBuilder::interest_set(identifier, hashtags),
        }
    }

    /// Emoji set
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[wasm_bindgen(js_name = emojiSet)]
    pub fn emoji_set(identifier: &str, emoji: Vec<JsEmojiInfo>) -> Self {
        Self {
            inner: EventBuilder::emoji_set(identifier, emoji.into_iter().map(|e| e.into())),
        }
    }

    /// Label
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/32.md>
    #[wasm_bindgen]
    pub fn label(label_namespace: &str, labels: Vec<String>) -> Self {
        Self {
            inner: EventBuilder::label(label_namespace, labels),
        }
    }
}
