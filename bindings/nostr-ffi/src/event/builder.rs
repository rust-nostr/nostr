// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use nostr::util::EventIdOrCoordinate;
use nostr::{Contact as ContactSdk, UncheckedUrl, Url};
use uniffi::Object;

use super::{Event, EventId, Kind};
use crate::error::Result;
use crate::helper::unwrap_or_clone_arc;
use crate::key::Keys;
use crate::nips::nip01::Coordinate;
use crate::nips::nip15::{ProductData, StallData};
use crate::nips::nip51::{ArticlesCuration, Bookmarks, EmojiInfo, Emojis, Interests, MuteList};
use crate::nips::nip53::LiveEvent;
use crate::nips::nip57::ZapRequestData;
use crate::nips::nip90::DataVendingMachineStatus;
use crate::nips::nip98::HttpData;
use crate::types::{Contact, Metadata};
use crate::{
    FileMetadata, Image, ImageDimensions, NostrConnectMessage, PublicKey, RelayMetadata, Tag,
    Timestamp, UnsignedEvent,
};

#[derive(Debug, Clone, PartialEq, Eq, Object, o2o::o2o)]
#[from_owned(nostr::EventBuilder| return Self { inner: @ })]
#[uniffi::export(Debug, Eq)]
pub struct EventBuilder {
    inner: nostr::EventBuilder,
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
    pub fn new(kind: &Kind, content: &str, tags: &[Arc<Tag>]) -> Self {
        let tags = tags.iter().map(|t| t.as_ref().deref().clone());
        Self {
            inner: nostr::EventBuilder::new(**kind, content, tags),
        }
    }

    /// Add tags
    pub fn add_tags(self: Arc<Self>, tags: &[Arc<Tag>]) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        let tags = tags.iter().map(|t| t.as_ref().deref().clone());
        builder.inner = builder.inner.add_tags(tags);
        builder
    }

    /// Set a custom `created_at` UNIX timestamp
    pub fn custom_created_at(self: Arc<Self>, created_at: &Timestamp) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.custom_created_at(**created_at);
        builder
    }

    pub fn to_event(&self, keys: &Keys) -> Result<Event> {
        let event = self.inner.clone().to_event(keys.deref())?;
        Ok(event.into())
    }

    pub fn to_pow_event(&self, keys: &Keys, difficulty: u8) -> Result<Event> {
        Ok(self
            .inner
            .clone()
            .to_pow_event(keys.deref(), difficulty)?
            .into())
    }

    pub fn to_unsigned_event(&self, public_key: &PublicKey) -> UnsignedEvent {
        self.inner.clone().to_unsigned_event(**public_key).into()
    }

    pub fn to_unsigned_pow_event(&self, public_key: &PublicKey, difficulty: u8) -> UnsignedEvent {
        self.inner
            .clone()
            .to_unsigned_pow_event(**public_key, difficulty)
            .into()
    }

    /// Profile metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[uniffi::constructor]
    pub fn metadata(metadata: &Metadata) -> Self {
        Self {
            inner: nostr::EventBuilder::metadata(metadata.deref()),
        }
    }

    /// Relay list metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/65.md>
    #[uniffi::constructor]
    pub fn relay_list(map: HashMap<String, Option<RelayMetadata>>) -> Result<Self> {
        let mut list = Vec::with_capacity(map.len());
        for (url, metadata) in map.into_iter() {
            let relay_url: Url = Url::parse(&url)?;
            let metadata = metadata.map(|m| m.into());
            list.push((relay_url, metadata))
        }
        Ok(Self {
            inner: nostr::EventBuilder::relay_list(list),
        })
    }

    /// Text note
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[uniffi::constructor]
    pub fn text_note(content: &str, tags: &[Arc<Tag>]) -> Self {
        let tags = tags.iter().map(|t| t.as_ref().deref().clone());
        Self {
            inner: nostr::EventBuilder::text_note(content, tags),
        }
    }

    /// Text note reply
    ///
    /// If no `root` is passed, the `rely_to` will be used for root `e` tag.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/10.md>
    #[uniffi::constructor(default(root = None, relay_url = None))]
    pub fn text_note_reply(
        content: String,
        reply_to: &Event,
        root: Option<Arc<Event>>,
        relay_url: Option<String>,
    ) -> Self {
        Self {
            inner: nostr::EventBuilder::text_note_reply(
                content,
                reply_to.deref(),
                root.as_ref().map(|e| e.as_ref().deref()),
                relay_url.map(UncheckedUrl::from),
            ),
        }
    }

    /// Long-form text note (generally referred to as "articles" or "blog posts").
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/23.md>
    #[uniffi::constructor]
    pub fn long_form_text_note(content: &str, tags: &[Arc<Tag>]) -> Self {
        let tags = tags.iter().map(|t| t.as_ref().deref().clone());
        Self {
            inner: nostr::EventBuilder::long_form_text_note(content, tags),
        }
    }

    /// Contact/Follow list
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/02.md>
    #[uniffi::constructor]
    pub fn contact_list(list: &[Arc<Contact>]) -> Self {
        let list: Vec<ContactSdk> = list.iter().map(|c| c.as_ref().deref().clone()).collect();

        Self {
            inner: nostr::EventBuilder::contact_list(list),
        }
    }

    /// Create encrypted direct msg event
    ///
    /// <div class="warning"><strong>Unsecure!</strong> Deprecated in favor of NIP-17!</div>
    #[uniffi::constructor(default(reply_to = None))]
    pub fn encrypted_direct_msg(
        sender_keys: &Keys,
        receiver_pubkey: &PublicKey,
        content: &str,
        reply_to: Option<Arc<EventId>>,
    ) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventBuilder::encrypted_direct_msg(
                sender_keys.deref(),
                **receiver_pubkey,
                content,
                reply_to.map(|id| **id),
            )?,
        })
    }

    /// Repost
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/18.md>
    #[uniffi::constructor(default(relay_url = None))]
    pub fn repost(event: &Event, relay_url: Option<String>) -> Self {
        Self {
            inner: nostr::EventBuilder::repost(event.deref(), relay_url.map(UncheckedUrl::from)),
        }
    }

    /// Event deletion
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/09.md>
    #[uniffi::constructor(default(ids = [], coordinates = [], reason = None))]
    pub fn delete(
        ids: &[Arc<EventId>],
        coordinates: &[Arc<Coordinate>],
        reason: Option<String>,
    ) -> Self {
        let coordinates = coordinates
            .iter()
            .map(|c| c.as_ref().deref().clone())
            .map(EventIdOrCoordinate::from);
        let ids = ids
            .iter()
            .map(|e| ***e)
            .map(EventIdOrCoordinate::from)
            .chain(coordinates);
        Self {
            inner: match reason {
                Some(reason) => nostr::EventBuilder::delete_with_reason(ids, reason),
                None => nostr::EventBuilder::delete(ids),
            },
        }
    }

    /// Add reaction (like/upvote, dislike/downvote or emoji) to an event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/25.md>
    #[uniffi::constructor]
    pub fn reaction(event: &Event, reaction: &str) -> Self {
        Self {
            inner: nostr::EventBuilder::reaction(event.deref(), reaction),
        }
    }

    /// Add reaction (like/upvote, dislike/downvote or emoji) to an event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/25.md>
    #[uniffi::constructor(default(kind = None))]
    pub fn reaction_extended(
        event_id: &EventId,
        public_key: &PublicKey,
        reaction: &str,
        kind: Option<Arc<Kind>>,
    ) -> Self {
        Self {
            inner: nostr::EventBuilder::reaction_extended(
                **event_id,
                **public_key,
                kind.map(|k| **k),
                reaction,
            ),
        }
    }

    /// Create new channel
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    #[uniffi::constructor]
    pub fn channel(metadata: &Metadata) -> Self {
        Self {
            inner: nostr::EventBuilder::channel(metadata.deref()),
        }
    }

    /// Channel metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    #[uniffi::constructor(default(relay_url = None))]
    pub fn channel_metadata(
        channel_id: &EventId,
        metadata: &Metadata,
        relay_url: Option<String>,
    ) -> Result<Self> {
        let relay_url = match relay_url {
            Some(url) => Some(Url::parse(&url)?),
            None => None,
        };
        Ok(Self {
            inner: nostr::EventBuilder::channel_metadata(**channel_id, relay_url, metadata.deref()),
        })
    }

    /// Channel message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    #[uniffi::constructor]
    pub fn channel_msg(channel_id: &EventId, relay_url: &str, content: &str) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventBuilder::channel_msg(**channel_id, Url::parse(relay_url)?, content),
        })
    }

    /// Hide message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    #[uniffi::constructor(default(reason = None))]
    pub fn hide_channel_msg(message_id: &EventId, reason: Option<String>) -> Self {
        Self {
            inner: nostr::EventBuilder::hide_channel_msg(**message_id, reason),
        }
    }

    /// Mute channel user
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    #[uniffi::constructor(default(reason = None))]
    pub fn mute_channel_user(public_key: &PublicKey, reason: Option<String>) -> Self {
        Self {
            inner: nostr::EventBuilder::mute_channel_user(**public_key, reason),
        }
    }

    /// Authentication of clients to relays
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/42.md>
    #[uniffi::constructor]
    pub fn auth(challenge: &str, relay_url: &str) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventBuilder::auth(challenge, Url::parse(relay_url)?),
        })
    }

    /// Nostr Connect / Nostr Remote Signing
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/46.md>
    #[uniffi::constructor]
    pub fn nostr_connect(
        sender_keys: &Keys,
        receiver_pubkey: &PublicKey,
        msg: NostrConnectMessage,
    ) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventBuilder::nostr_connect(
                sender_keys.deref(),
                **receiver_pubkey,
                msg.try_into()?,
            )?,
        })
    }

    /// Live Event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/53.md>
    #[uniffi::constructor]
    pub fn live_event(live_event: LiveEvent) -> Self {
        Self {
            inner: nostr::EventBuilder::live_event(live_event.into()),
        }
    }

    /// Live Event Message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/53.md>
    #[uniffi::constructor(default(relay_url = None))]
    pub fn live_event_msg(
        live_event_id: &str,
        live_event_host: &PublicKey,
        content: &str,
        relay_url: Option<String>,
    ) -> Result<Self> {
        let relay_url = match relay_url {
            Some(url) => Some(Url::parse(&url)?),
            None => None,
        };
        Ok(Self {
            inner: nostr::EventBuilder::live_event_msg(
                live_event_id,
                **live_event_host,
                content,
                relay_url,
            ),
        })
    }

    /// Reporting
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/56.md>
    #[uniffi::constructor]
    pub fn report(tags: &[Arc<Tag>], content: &str) -> Self {
        let tags = tags.iter().map(|t| t.as_ref().deref().clone());
        Self {
            inner: nostr::EventBuilder::report(tags, content),
        }
    }

    /// Create **public** zap request event
    ///
    /// **This event MUST NOT be broadcasted to relays**, instead must be sent to a recipient's LNURL pay callback url.
    ///
    /// To build a **private** or **anonymous** zap request use `nip57_private_zap_request(...)` or `nip57_anonymous_zap_request(...)` functions.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/57.md>
    #[uniffi::constructor]
    pub fn public_zap_request(data: &ZapRequestData) -> Self {
        Self {
            inner: nostr::EventBuilder::public_zap_request(data.deref().clone()),
        }
    }

    /// Zap Receipt
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/57.md>
    #[uniffi::constructor]
    pub fn zap_receipt(bolt11: &str, preimage: Option<String>, zap_request: &Event) -> Self {
        Self {
            inner: nostr::EventBuilder::zap_receipt(bolt11, preimage, zap_request.deref()),
        }
    }

    /// Badge definition
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/58.md>
    #[uniffi::constructor(default(name = None, description = None, image = None, image_dimensions = None, thumbnails = []))]
    pub fn define_badge(
        badge_id: String,
        name: Option<String>,
        description: Option<String>,
        image: Option<String>,
        image_dimensions: Option<Arc<ImageDimensions>>,
        thumbnails: Vec<Image>,
    ) -> Self {
        Self {
            inner: nostr::EventBuilder::define_badge(
                badge_id,
                name,
                description,
                image.map(UncheckedUrl::from),
                image_dimensions.map(|i| **i),
                thumbnails
                    .into_iter()
                    .map(|i: Image| (UncheckedUrl::from(i.url), i.dimensions.map(|d| **d)))
                    .collect(),
            ),
        }
    }

    /// Badge award
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/58.md>
    #[uniffi::constructor]
    pub fn award_badge(badge_definition: &Event, awarded_pubkeys: &[Arc<Tag>]) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventBuilder::award_badge(
                badge_definition.deref(),
                awarded_pubkeys.iter().map(|a| a.as_ref().deref().clone()),
            )?,
        })
    }

    /// Profile badges
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/58.md>
    #[uniffi::constructor]
    pub fn profile_badges(
        badge_definitions: &[Arc<Event>],
        badge_awards: &[Arc<Event>],
        pubkey_awarded: &PublicKey,
    ) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventBuilder::profile_badges(
                badge_definitions
                    .iter()
                    .map(|b| b.as_ref().deref().clone())
                    .collect(),
                badge_awards
                    .iter()
                    .map(|b| b.as_ref().deref().clone())
                    .collect(),
                pubkey_awarded.deref(),
            )?,
        })
    }

    /// Data Vending Machine (DVM) - Job Request
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    #[uniffi::constructor]
    pub fn job_request(kind: &Kind, tags: &[Arc<Tag>]) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventBuilder::job_request(
                **kind,
                tags.iter().map(|t| t.as_ref().deref().clone()),
            )?,
        })
    }

    /// Data Vending Machine (DVM) - Job Result
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    #[uniffi::constructor(default(bolt11 = None))]
    pub fn job_result(
        job_request: &Event,
        amount_millisats: u64,
        bolt11: Option<String>,
    ) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventBuilder::job_result(
                job_request.deref().clone(),
                amount_millisats,
                bolt11,
            )?,
        })
    }

    /// Data Vending Machine (DVM) - Job Feedback
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    #[uniffi::constructor(default(bolt11 = None, payload = None))]
    pub fn job_feedback(
        job_request: &Event,
        status: DataVendingMachineStatus,
        extra_info: Option<String>,
        amount_millisats: u64,
        bolt11: Option<String>,
        payload: Option<String>,
    ) -> Self {
        Self {
            inner: nostr::EventBuilder::job_feedback(
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
    #[uniffi::constructor]
    pub fn file_metadata(description: &str, metadata: &FileMetadata) -> Self {
        Self {
            inner: nostr::EventBuilder::file_metadata(description, metadata.deref().clone()),
        }
    }

    /// HTTP Auth
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/98.md>
    #[uniffi::constructor]
    pub fn http_auth(data: HttpData) -> Self {
        Self {
            inner: nostr::EventBuilder::http_auth(data.into()),
        }
    }

    /// Set stall data
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/15.md>
    #[uniffi::constructor]
    pub fn stall_data(data: &StallData) -> Self {
        Self {
            inner: nostr::EventBuilder::stall_data(data.deref().clone()),
        }
    }

    /// Set product data
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/15.md>
    #[uniffi::constructor]
    pub fn product_data(data: ProductData) -> Self {
        Self {
            inner: nostr::EventBuilder::product_data(data.into()),
        }
    }

    // TODO: add seal

    /// Private Direct message rumor
    ///
    /// <div class="warning">
    /// This constructor compose ONLY the rumor for the private direct message!
    /// NOT USE THIS IF YOU DON'T KNOW WHAT YOU ARE DOING!
    /// </div>
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/17.md>
    #[uniffi::constructor(default(reply_to = None))]
    pub fn private_msg_rumor(
        receiver: &PublicKey,
        message: &str,
        reply_to: Option<Arc<EventId>>,
    ) -> Self {
        Self {
            inner: nostr::EventBuilder::private_msg_rumor(
                **receiver,
                message,
                reply_to.map(|id| **id),
            ),
        }
    }

    /// Mute list
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[uniffi::constructor]
    pub fn mute_list(list: MuteList) -> Self {
        Self {
            inner: nostr::EventBuilder::mute_list(list.into()),
        }
    }

    /// Pinned notes
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[uniffi::constructor]
    pub fn pinned_notes(ids: Vec<Arc<EventId>>) -> Self {
        Self {
            inner: nostr::EventBuilder::pinned_notes(ids.into_iter().map(|e| **e)),
        }
    }

    /// Bookmarks
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[uniffi::constructor]
    pub fn bookmarks(list: Bookmarks) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventBuilder::bookmarks(list.try_into()?),
        })
    }

    /// Communities
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[uniffi::constructor]
    pub fn communities(communities: Vec<Arc<Coordinate>>) -> Self {
        Self {
            inner: nostr::EventBuilder::communities(
                communities.into_iter().map(|c| c.as_ref().deref().clone()),
            ),
        }
    }

    /// Public chats
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[uniffi::constructor]
    pub fn public_chats(chat: Vec<Arc<EventId>>) -> Self {
        Self {
            inner: nostr::EventBuilder::public_chats(chat.into_iter().map(|e| **e)),
        }
    }

    /// Blocked relays
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[uniffi::constructor]
    pub fn blocked_relays(relay: Vec<String>) -> Self {
        Self {
            inner: nostr::EventBuilder::blocked_relays(relay.into_iter().map(UncheckedUrl::from)),
        }
    }

    /// Search relays
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[uniffi::constructor]
    pub fn search_relays(relay: Vec<String>) -> Self {
        Self {
            inner: nostr::EventBuilder::search_relays(relay.into_iter().map(UncheckedUrl::from)),
        }
    }

    /// Interests
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[uniffi::constructor]
    pub fn interests(list: Interests) -> Self {
        Self {
            inner: nostr::EventBuilder::interests(list.into()),
        }
    }

    /// Emojis
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[uniffi::constructor]
    pub fn emojis(list: Emojis) -> Self {
        Self {
            inner: nostr::EventBuilder::emojis(list.into()),
        }
    }

    /// Follow set
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[uniffi::constructor]
    pub fn follow_set(identifier: &str, public_keys: Vec<Arc<PublicKey>>) -> Self {
        Self {
            inner: nostr::EventBuilder::follow_set(
                identifier,
                public_keys.into_iter().map(|p| **p),
            ),
        }
    }

    /// Relay set
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[uniffi::constructor]
    pub fn relay_set(identifier: &str, relays: Vec<String>) -> Self {
        Self {
            inner: nostr::EventBuilder::relay_set(
                identifier,
                relays.into_iter().map(UncheckedUrl::from),
            ),
        }
    }

    /// Bookmark set
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[uniffi::constructor]
    pub fn bookmarks_set(identifier: &str, list: Bookmarks) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventBuilder::bookmarks_set(identifier, list.try_into()?),
        })
    }

    /// Article Curation set
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[uniffi::constructor]
    pub fn articles_curation_set(identifier: &str, list: ArticlesCuration) -> Self {
        Self {
            inner: nostr::EventBuilder::articles_curation_set(identifier, list.into()),
        }
    }

    /// Videos Curation set
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[uniffi::constructor]
    pub fn videos_curation_set(identifier: &str, video: Vec<Arc<Coordinate>>) -> Self {
        Self {
            inner: nostr::EventBuilder::videos_curation_set(
                identifier,
                video.into_iter().map(|c| c.as_ref().deref().clone()),
            ),
        }
    }

    /// Interest set
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[uniffi::constructor]
    pub fn interest_set(identifier: &str, hashtags: Vec<String>) -> Self {
        Self {
            inner: nostr::EventBuilder::interest_set(identifier, hashtags),
        }
    }

    /// Emoji set
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[uniffi::constructor]
    pub fn emoji_set(identifier: &str, emojis: Vec<EmojiInfo>) -> Self {
        Self {
            inner: nostr::EventBuilder::emoji_set(identifier, emojis.into_iter().map(|e| e.into())),
        }
    }

    /// Label
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/32.md>
    #[uniffi::constructor]
    pub fn label(label_namespace: String, labels: Vec<String>) -> Self {
        Self {
            inner: nostr::EventBuilder::label(label_namespace, labels),
        }
    }
}
