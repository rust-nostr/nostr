// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use super::{JsEvent, JsEventId, JsKind, JsTag, JsUnsignedEvent};
use crate::error::{into_err, Result};
use crate::protocol::key::{JsKeys, JsPublicKey};
use crate::protocol::nips::nip01::{JsCoordinate, JsMetadata};
use crate::protocol::nips::nip09::JsEventDeletionRequest;
use crate::protocol::nips::nip15::{JsProductData, JsStallData};
use crate::protocol::nips::nip34::{JsGitIssue, JsGitRepositoryAnnouncement};
use crate::protocol::nips::nip51::{
    JsArticlesCuration, JsBookmarks, JsEmojiInfo, JsEmojis, JsInterests, JsMuteList,
};
use crate::protocol::nips::nip53::JsLiveEvent;
use crate::protocol::nips::nip57::JsZapRequestData;
use crate::protocol::nips::nip65::JsRelayListItem;
use crate::protocol::nips::nip90::JsJobFeedbackData;
use crate::protocol::nips::nip94::JsFileMetadata;
use crate::protocol::nips::nip98::JsHttpData;
use crate::protocol::types::image::{JsImageDimensions, JsThumbnails};
use crate::protocol::types::{JsContact, JsTimestamp};
use crate::signer::JsNostrSigner;
use crate::util::parse_optional_relay_url;

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
    pub fn new(kind: &JsKind, content: &str) -> Self {
        Self {
            inner: EventBuilder::new(**kind, content),
        }
    }

    /// Add tags
    ///
    /// This method extend the current tags (if any).
    #[wasm_bindgen]
    pub fn tags(self, tags: Vec<JsTag>) -> Self {
        self.inner.tags(tags.into_iter().map(|t| t.into())).into()
    }

    /// Set a custom `created_at` UNIX timestamp
    #[wasm_bindgen(js_name = customCreatedAt)]
    pub fn custom_created_at(self, created_at: &JsTimestamp) -> Self {
        self.inner.custom_created_at(**created_at).into()
    }

    /// Set POW difficulty
    ///
    /// Only values `> 0` are accepted!
    #[wasm_bindgen]
    pub fn pow(self, difficulty: u8) -> Self {
        self.inner.pow(difficulty).into()
    }

    /// Allow self-tagging
    ///
    /// When this mode is enabled, any `p` tags referencing the author’s public key will not be discarded.
    pub fn allow_self_tagging(self) -> Self {
        self.inner.allow_self_tagging().into()
    }

    /// Deduplicate tags
    ///
    /// For more details check [`Tags::dedup`].
    pub fn dedup_tags(self) -> Self {
        self.inner.dedup_tags().into()
    }

    /// Build, sign and return event
    ///
    /// Check [`EventBuilder::build`] to learn more.
    ///
    /// **This method consumes the builder, so it will no longer be usable!**
    #[wasm_bindgen(js_name = sign)]
    pub async fn sign(self, signer: &JsNostrSigner) -> Result<JsEvent> {
        let event = self.inner.sign(signer.deref()).await.map_err(into_err)?;
        Ok(event.into())
    }

    /// Build, sign and return event using keys signer
    ///
    /// Check [`EventBuilder::build`] to learn more.
    ///
    /// **This method consumes the builder, so it will no longer be usable!**
    #[wasm_bindgen(js_name = signWithKeys)]
    pub fn sign_with_keys(self, keys: &JsKeys) -> Result<JsEvent> {
        let event = self.inner.sign_with_keys(keys.deref()).map_err(into_err)?;
        Ok(event.into())
    }

    /// Build an unsigned event
    ///
    /// By default, this method removes any `p` tags that match the author's public key.
    /// To allow self-tagging, call [`EventBuilder::allow_self_tagging`] first.
    ///
    /// **This method consumes the builder, so it will no longer be usable!**
    #[wasm_bindgen(js_name = build)]
    pub fn build(self, public_key: &JsPublicKey) -> JsUnsignedEvent {
        self.inner.build(**public_key).into()
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
            let relay_url: RelayUrl = RelayUrl::parse(&url).map_err(into_err)?;
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
    pub fn text_note(content: &str) -> Self {
        Self {
            inner: EventBuilder::text_note(content),
        }
    }

    /// Text note reply
    ///
    /// This adds only that most significant tags, like:
    /// - `p` tag with the author of the `reply_to` and `root` events;
    /// - `e` tag of the `reply_to` and `root` events.
    ///
    /// Any additional necessary tag can be added with [`EventBuilder::tag`] or [`EventBuilder::tags`].
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/10.md>
    #[wasm_bindgen(js_name = textNoteReply)]
    pub fn text_note_reply(
        content: &str,
        reply_to: &JsEvent,
        root: Option<JsEvent>,
        relay_url: Option<String>,
    ) -> Result<JsEventBuilder> {
        Ok(Self {
            inner: EventBuilder::text_note_reply(
                content,
                reply_to.deref(),
                root.as_deref(),
                parse_optional_relay_url(relay_url)?,
            ),
        })
    }

    /// Comment
    ///
    /// This adds only that most significant tags, like:
    /// - `p` tag with the author of the `comment_to` event;
    /// - the `a`/`e` and `k` tags of the `comment_to` event;
    /// - `P` tag with the author of the `root` event;
    /// - the `A`/`E` and `K` tags of the `root` event.
    ///
    /// Any additional necessary tag can be added with [`EventBuilder::tag`] or [`EventBuilder::tags`].
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/22.md>
    #[wasm_bindgen(js_name = comment)]
    pub fn comment(
        content: &str,
        comment_to: &JsEvent,
        root: Option<JsEvent>,
        relay_url: Option<String>,
    ) -> Result<JsEventBuilder> {
        Ok(Self {
            inner: EventBuilder::comment(
                content,
                comment_to.deref(),
                root.as_deref(),
                parse_optional_relay_url(relay_url)?,
            ),
        })
    }

    /// Long-form text note (generally referred to as "articles" or "blog posts").
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/23.md>
    #[wasm_bindgen(js_name = longFormTextNote)]
    pub fn long_form_text_note(content: &str) -> Self {
        Self {
            inner: EventBuilder::long_form_text_note(content),
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

    /// Repost
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/18.md>
    pub fn repost(event: &JsEvent, relay_url: Option<String>) -> Result<JsEventBuilder> {
        Ok(Self {
            inner: EventBuilder::repost(event.deref(), parse_optional_relay_url(relay_url)?),
        })
    }

    /// Event deletion
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/09.md>
    #[wasm_bindgen]
    pub fn delete(request: JsEventDeletionRequest) -> Self {
        Self {
            inner: EventBuilder::delete(request.into()),
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
        kind: Option<JsKind>,
        reaction: &str,
    ) -> Self {
        Self {
            inner: EventBuilder::reaction_extended(
                **event_id,
                **public_key,
                kind.map(|k| *k),
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
        Ok(Self {
            inner: EventBuilder::channel_metadata(
                **channel_id,
                parse_optional_relay_url(relay_url)?,
                metadata.deref(),
            ),
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
        let relay_url: RelayUrl = RelayUrl::parse(relay_url).map_err(into_err)?;
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
        let url = RelayUrl::parse(relay).map_err(into_err)?;
        Ok(Self {
            inner: EventBuilder::auth(challenge, url),
        })
    }

    // TODO: add nostr_connect method

    /// Live Event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/53.md>
    // TODO: fix this. `JsLiveEvent` can't be constructed from JS bindings
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
                parse_optional_relay_url(relay_url)?,
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
    ) -> Result<JsEventBuilder> {
        let image = match image {
            Some(url) => Some(Url::parse(&url).map_err(into_err)?),
            None => None,
        };
        Ok(Self {
            inner: EventBuilder::define_badge(
                badge_id,
                name,
                description,
                image,
                image_dimensions.map(|i| i.into()),
                // TODO: propagate error
                thumbnails
                    .into_iter()
                    .filter_map(|t| t.try_into().ok())
                    .collect(),
            ),
        })
    }

    /// Badge award
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/58.md>
    #[wasm_bindgen(js_name = awardBadge)]
    pub fn award_badge(
        badge_definition: &JsEvent,
        awarded_public_keys: Vec<JsPublicKey>,
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
    pub fn job_request(kind: &JsKind) -> Result<JsEventBuilder> {
        Ok(Self {
            inner: EventBuilder::job_request(**kind).map_err(into_err)?,
        })
    }

    /// Data Vending Machine (DVM) - Job Result
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    #[wasm_bindgen(js_name = jobResult)]
    pub fn job_result(
        job_request: &JsEvent,
        payload: String,
        millisats: f64,
        bolt11: Option<String>,
    ) -> Result<JsEventBuilder> {
        Ok(Self {
            inner: EventBuilder::job_result(
                job_request.deref().clone(),
                payload,
                millisats as u64,
                bolt11,
            )
            .map_err(into_err)?,
        })
    }

    /// Data Vending Machine (DVM) - Job Feedback
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    #[wasm_bindgen(js_name = jobFeedback)]
    pub fn job_feedback(data: &JsJobFeedbackData) -> Self {
        Self {
            inner: EventBuilder::job_feedback(data.deref().clone()),
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

    /// Seal
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/59.md>
    #[inline]
    #[wasm_bindgen]
    pub async fn seal(
        signer: &JsNostrSigner,
        receiver_public_key: &JsPublicKey,
        rumor: &JsUnsignedEvent,
    ) -> Result<JsEventBuilder> {
        Ok(Self {
            inner: EventBuilder::seal(
                signer.deref(),
                receiver_public_key.deref(),
                rumor.deref().clone(),
            )
            .await
            .map_err(into_err)?,
        })
    }

    /// Gift Wrap from seal
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/59.md>
    #[wasm_bindgen(js_name = giftWrapFromSeal)]
    pub fn gift_wrap_from_seal(
        receiver: &JsPublicKey,
        seal: &JsEvent,
        extra_tags: Option<Vec<JsTag>>,
    ) -> Result<JsEvent> {
        Ok(EventBuilder::gift_wrap_from_seal(
            receiver.deref(),
            seal.deref(),
            extra_tags.unwrap_or_default().into_iter().map(|t| t.inner),
        )
        .map_err(into_err)?
        .into())
    }

    /// Gift Wrap
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/59.md>
    #[wasm_bindgen(js_name = giftWrap)]
    pub async fn gift_wrap(
        signer: &JsNostrSigner,
        receiver: &JsPublicKey,
        rumor: &JsUnsignedEvent,
        extra_tags: Option<Vec<JsTag>>,
    ) -> Result<JsEvent> {
        Ok(EventBuilder::gift_wrap(
            signer.deref(),
            receiver.deref(),
            rumor.deref().clone(),
            extra_tags.unwrap_or_default().into_iter().map(|t| t.inner),
        )
        .await
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
    pub fn private_msg_rumor(receiver: &JsPublicKey, message: &str) -> Self {
        Self {
            inner: EventBuilder::private_msg_rumor(**receiver, message),
        }
    }

    /// Private Direct message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/17.md>
    #[wasm_bindgen(js_name = privateMsg)]
    pub async fn private_msg(
        signer: &JsNostrSigner,
        receiver: &JsPublicKey,
        message: &str,
        rumor_extra_tags: Option<Vec<JsTag>>,
    ) -> Result<JsEvent> {
        Ok(EventBuilder::private_msg(
            signer.deref(),
            **receiver,
            message,
            rumor_extra_tags
                .unwrap_or_default()
                .into_iter()
                .map(|t| t.inner),
        )
        .await
        .map_err(into_err)?
        .into())
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
        // TODO: return error if invalid url
        Self {
            inner: EventBuilder::blocked_relays(
                relays.into_iter().filter_map(|u| RelayUrl::parse(&u).ok()),
            ),
        }
    }

    /// Search relays
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[wasm_bindgen(js_name = searchRelays)]
    pub fn search_relays(relays: Vec<String>) -> Self {
        // TODO: return error if invalid url
        Self {
            inner: EventBuilder::search_relays(
                relays.into_iter().filter_map(|u| RelayUrl::parse(&u).ok()),
            ),
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
        // TODO: return error if invalid url
        Self {
            inner: EventBuilder::relay_set(
                identifier,
                relays.into_iter().filter_map(|u| RelayUrl::parse(&u).ok()),
            ),
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
        // TODO: propagate error
        Self {
            inner: EventBuilder::emoji_set(
                identifier,
                emoji.into_iter().filter_map(|e| e.try_into().ok()),
            ),
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

    /// Git Repository Announcement
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/34.md>
    #[wasm_bindgen(js_name = gitRepositoryAnnouncement)]
    pub fn git_repository_announcement(
        data: JsGitRepositoryAnnouncement,
    ) -> Result<JsEventBuilder> {
        Ok(Self {
            inner: EventBuilder::git_repository_announcement(data.into()).map_err(into_err)?,
        })
    }

    /// Git Issue
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/34.md>
    #[wasm_bindgen(js_name = gitIssue)]
    pub fn git_issue(issue: JsGitIssue) -> Result<JsEventBuilder> {
        Ok(Self {
            inner: EventBuilder::git_issue(issue.into()).map_err(into_err)?,
        })
    }
}
