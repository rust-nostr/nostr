// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use nostr::{RelayUrl, Url};
use uniffi::Object;

use super::{Event, EventId, Kind};
use crate::error::Result;
use crate::protocol::event::{PublicKey, Tag, Timestamp, UnsignedEvent};
use crate::protocol::key::Keys;
use crate::protocol::nips::nip01::{Coordinate, Metadata};
use crate::protocol::nips::nip09::EventDeletionRequest;
use crate::protocol::nips::nip15::{ProductData, StallData};
use crate::protocol::nips::nip34::{GitIssue, GitPatch, GitRepositoryAnnouncement};
use crate::protocol::nips::nip46::NostrConnectMessage;
use crate::protocol::nips::nip51::{
    ArticlesCuration, Bookmarks, EmojiInfo, Emojis, Interests, MuteList,
};
use crate::protocol::nips::nip53::{Image, LiveEvent};
use crate::protocol::nips::nip57::ZapRequestData;
use crate::protocol::nips::nip65::RelayMetadata;
use crate::protocol::nips::nip90::JobFeedbackData;
use crate::protocol::nips::nip94::FileMetadata;
use crate::protocol::nips::nip98::HttpData;
use crate::protocol::signer::NostrSigner;
use crate::protocol::types::{Contact, ImageDimensions};
use crate::util::parse_optional_relay_url;

#[derive(Debug, Clone, PartialEq, Eq, Object)]
#[uniffi::export(Debug, Eq)]
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

#[uniffi::export(async_runtime = "tokio")]
impl EventBuilder {
    #[uniffi::constructor]
    pub fn new(kind: &Kind, content: &str) -> Self {
        Self {
            inner: nostr::EventBuilder::new(**kind, content),
        }
    }

    /// Add tags
    ///
    /// This method extend the current tags (if any).
    pub fn tags(&self, tags: &[Arc<Tag>]) -> Self {
        let mut builder = self.clone();
        let tags = tags.iter().map(|t| t.as_ref().deref().clone());
        builder.inner = builder.inner.tags(tags);
        builder
    }

    /// Set a custom `created_at` UNIX timestamp
    pub fn custom_created_at(&self, created_at: &Timestamp) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.custom_created_at(**created_at);
        builder
    }

    /// Set POW difficulty
    ///
    /// Only values `> 0` are accepted!
    pub fn pow(&self, difficulty: u8) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.pow(difficulty);
        builder
    }

    /// Allow self-tagging
    ///
    /// When this mode is enabled, any `p` tags referencing the author’s public key will not be discarded.
    pub fn allow_self_tagging(&self) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.allow_self_tagging();
        builder
    }

    /// Deduplicate tags
    ///
    /// For more details check [`Tags::dedup`].
    pub fn dedup_tags(&self) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.dedup_tags();
        builder
    }

    /// Build, sign and return [`Event`]
    ///
    /// Check [`EventBuilder::build`] to learn more.
    pub async fn sign(&self, signer: &NostrSigner) -> Result<Event> {
        let event = self.inner.clone().sign(signer.deref()).await?;
        Ok(event.into())
    }

    /// Build, sign and return [`Event`] using [`Keys`] signer
    ///
    /// Check [`EventBuilder::build`] to learn more.
    pub fn sign_with_keys(&self, keys: &Keys) -> Result<Event> {
        let event = self.inner.clone().sign_with_keys(keys.deref())?;
        Ok(event.into())
    }

    /// Build an unsigned event
    ///
    /// By default, this method removes any `p` tags that match the author's public key.
    /// To allow self-tagging, call [`EventBuilder::allow_self_tagging`] first.
    pub fn build(&self, public_key: &PublicKey) -> UnsignedEvent {
        self.inner.clone().build(**public_key).into()
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
            let relay_url: RelayUrl = RelayUrl::parse(&url)?;
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
    pub fn text_note(content: &str) -> Self {
        Self {
            inner: nostr::EventBuilder::text_note(content),
        }
    }

    /// Text note reply
    ///
    /// This adds only the most significant tags, like:
    /// - `p` tag with the author of the `reply_to` and `root` events;
    /// - `e` tag of the `reply_to` and `root` events.
    ///
    /// Any additional necessary tag can be added with [`EventBuilder::tag`] or [`EventBuilder::tags`].
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/10.md>
    #[uniffi::constructor(default(root = None, relay_url = None))]
    pub fn text_note_reply(
        content: String,
        reply_to: &Event,
        root: Option<Arc<Event>>,
        relay_url: Option<String>,
    ) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventBuilder::text_note_reply(
                content,
                reply_to.deref(),
                root.as_ref().map(|e| e.as_ref().deref()),
                parse_optional_relay_url(relay_url)?,
            ),
        })
    }

    /// Comment
    ///
    /// This adds only the most significant tags, like:
    /// - `p` tag with the author of the `comment_to` event;
    /// - the `a`/`e` and `k` tags of the `comment_to` event;
    /// - `P` tag with the author of the `root` event;
    /// - the `A`/`E` and `K` tags of the `root` event.
    ///
    /// Any additional necessary tag can be added with [`EventBuilder::tag`] or [`EventBuilder::tags`].
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/22.md>
    #[uniffi::constructor(default(root = None, relay_url = None))]
    pub fn comment(
        content: String,
        comment_to: &Event,
        root: Option<Arc<Event>>,
        relay_url: Option<String>,
    ) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventBuilder::comment(
                content,
                comment_to.deref(),
                root.as_ref().map(|e| e.as_ref().deref()),
                parse_optional_relay_url(relay_url)?,
            ),
        })
    }

    /// Long-form text note (generally referred to as "articles" or "blog posts").
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/23.md>
    #[uniffi::constructor]
    pub fn long_form_text_note(content: &str) -> Self {
        Self {
            inner: nostr::EventBuilder::long_form_text_note(content),
        }
    }

    /// Contact/Follow list
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/02.md>
    #[uniffi::constructor]
    pub fn contact_list(contacts: Vec<Contact>) -> Result<Self> {
        let mut list = Vec::with_capacity(contacts.len());
        for contact in contacts.into_iter() {
            list.push(contact.try_into()?);
        }

        Ok(Self {
            inner: nostr::EventBuilder::contact_list(list),
        })
    }

    /// Repost
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/18.md>
    #[uniffi::constructor(default(relay_url = None))]
    pub fn repost(event: &Event, relay_url: Option<String>) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventBuilder::repost(event.deref(), parse_optional_relay_url(relay_url)?),
        })
    }

    /// Event deletion request
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/09.md>
    #[uniffi::constructor]
    pub fn delete(request: EventDeletionRequest) -> Self {
        Self {
            inner: nostr::EventBuilder::delete(request.into()),
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
        Ok(Self {
            inner: nostr::EventBuilder::channel_metadata(
                **channel_id,
                parse_optional_relay_url(relay_url)?,
                metadata.deref(),
            ),
        })
    }

    /// Channel message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    #[uniffi::constructor]
    pub fn channel_msg(channel_id: &EventId, relay_url: &str, content: &str) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventBuilder::channel_msg(
                **channel_id,
                RelayUrl::parse(relay_url)?,
                content,
            ),
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
            inner: nostr::EventBuilder::auth(challenge, RelayUrl::parse(relay_url)?),
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
    pub fn live_event(live_event: LiveEvent) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventBuilder::live_event(live_event.try_into()?),
        })
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
        Ok(Self {
            inner: nostr::EventBuilder::live_event_msg(
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
        image_dimensions: Option<ImageDimensions>,
        thumbnails: Vec<Image>,
    ) -> Result<Self> {
        let image = match image {
            Some(url) => Some(Url::parse(&url)?),
            None => None,
        };
        Ok(Self {
            inner: nostr::EventBuilder::define_badge(
                badge_id,
                name,
                description,
                image,
                image_dimensions.map(|i| i.into()),
                thumbnails
                    .into_iter()
                    // TODO: propagate error
                    .filter_map(|i: Image| {
                        Some((Url::parse(&i.url).ok()?, i.dimensions.map(|d| d.into())))
                    })
                    .collect(),
            ),
        })
    }

    /// Badge award
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/58.md>
    #[uniffi::constructor]
    pub fn award_badge(
        badge_definition: &Event,
        awarded_public_keys: &[Arc<PublicKey>],
    ) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventBuilder::award_badge(
                badge_definition.deref(),
                awarded_public_keys.iter().map(|a| ***a),
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
    pub fn job_request(kind: &Kind) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventBuilder::job_request(**kind)?,
        })
    }

    /// Data Vending Machine (DVM) - Job Result
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    #[uniffi::constructor(default(bolt11 = None))]
    pub fn job_result(
        job_request: &Event,
        payload: String,
        millisats: u64,
        bolt11: Option<String>,
    ) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventBuilder::job_result(
                job_request.deref().clone(),
                payload,
                millisats,
                bolt11,
            )?,
        })
    }

    /// Data Vending Machine (DVM) - Job Feedback
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    #[uniffi::constructor]
    pub fn job_feedback(data: &JobFeedbackData) -> Self {
        Self {
            inner: nostr::EventBuilder::job_feedback(data.deref().clone()),
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
    pub fn http_auth(data: HttpData) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventBuilder::http_auth(data.try_into()?),
        })
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

    /// Seal
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/59.md>
    #[uniffi::constructor]
    pub async fn seal(
        signer: &NostrSigner,
        receiver_public_key: &PublicKey,
        rumor: &UnsignedEvent,
    ) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventBuilder::seal(
                signer.deref(),
                receiver_public_key.deref(),
                rumor.deref().clone(),
            )
            .await?,
        })
    }

    /// Private Direct message rumor
    ///
    /// <div class="warning">
    /// This constructor compose ONLY the rumor for the private direct message!
    /// NOT USE THIS IF YOU DON'T KNOW WHAT YOU ARE DOING!
    /// </div>
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/17.md>
    #[uniffi::constructor]
    pub fn private_msg_rumor(receiver: &PublicKey, message: &str) -> Self {
        Self {
            inner: nostr::EventBuilder::private_msg_rumor(**receiver, message),
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
        // TODO: return error if invalid url
        Self {
            inner: nostr::EventBuilder::blocked_relays(
                relay.into_iter().filter_map(|u| RelayUrl::parse(&u).ok()),
            ),
        }
    }

    /// Search relays
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[uniffi::constructor]
    pub fn search_relays(relay: Vec<String>) -> Self {
        // TODO: return error if invalid url
        Self {
            inner: nostr::EventBuilder::search_relays(
                relay.into_iter().filter_map(|u| RelayUrl::parse(&u).ok()),
            ),
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
        // TODO: return error if invalid url
        Self {
            inner: nostr::EventBuilder::relay_set(
                identifier,
                relays.into_iter().filter_map(|u| RelayUrl::parse(&u).ok()),
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
        // TODO: propagate error
        Self {
            inner: nostr::EventBuilder::emoji_set(
                identifier,
                emojis.into_iter().filter_map(|e| e.try_into().ok()),
            ),
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

    /// Git Repository Announcement
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/34.md>
    #[uniffi::constructor]
    pub fn git_repository_announcement(data: GitRepositoryAnnouncement) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventBuilder::git_repository_announcement(data.into())?,
        })
    }

    /// Git Issue
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/34.md>
    #[uniffi::constructor]
    pub fn git_issue(issue: GitIssue) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventBuilder::git_issue(issue.into())?,
        })
    }

    /// Git Patch
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/34.md>
    #[uniffi::constructor]
    pub fn git_patch(patch: GitPatch) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventBuilder::git_patch(patch.try_into()?)?,
        })
    }
}
