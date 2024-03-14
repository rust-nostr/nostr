// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use nostr::{Contact as ContactSdk, UncheckedUrl, Url};
use uniffi::Object;

use super::{Event, EventId, Kind};
use crate::error::Result;
use crate::helper::unwrap_or_clone_arc;
use crate::key::Keys;
use crate::nips::nip01::Coordinate;
use crate::nips::nip15::{ProductData, StallData};
use crate::nips::nip51::{ArticlesCuration, Bookmarks, Emojis, Interests, MuteList};
use crate::nips::nip53::LiveEvent;
use crate::nips::nip57::ZapRequestData;
use crate::nips::nip90::DataVendingMachineStatus;
use crate::nips::nip98::HttpData;
use crate::types::{Contact, Metadata};
use crate::{
    FileMetadata, Image, ImageDimensions, NostrConnectMessage, PublicKey, RelayMetadata, Tag,
    Timestamp, UnsignedEvent,
};

#[derive(Clone, Object)]
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

#[uniffi::export]
impl EventBuilder {
    #[uniffi::constructor]
    pub fn new(kind: &Kind, content: &str, tags: &[Arc<Tag>]) -> Result<Self> {
        let tags = tags.iter().map(|t| t.as_ref().deref().clone());
        Ok(Self {
            inner: nostr::EventBuilder::new(**kind, content, tags),
        })
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

    #[uniffi::constructor]
    pub fn metadata(metadata: &Metadata) -> Self {
        Self {
            inner: nostr::EventBuilder::metadata(metadata.deref()),
        }
    }

    #[uniffi::constructor]
    pub fn relay_list(list: HashMap<String, Option<RelayMetadata>>) -> Self {
        let iter = list
            .into_iter()
            .map(|(url, r)| (UncheckedUrl::from(url), r.map(|r| r.into())));
        Self {
            inner: nostr::EventBuilder::relay_list(iter),
        }
    }

    #[uniffi::constructor]
    pub fn text_note(content: &str, tags: &[Arc<Tag>]) -> Result<Self> {
        let tags = tags.iter().map(|t| t.as_ref().deref().clone());
        Ok(Self {
            inner: nostr::EventBuilder::text_note(content, tags),
        })
    }

    /// Text note reply
    ///
    /// If no `root` is passed, the `rely_to` will be used for root `e` tag.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/10.md>
    #[uniffi::constructor]
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

    #[uniffi::constructor]
    pub fn long_form_text_note(content: &str, tags: &[Arc<Tag>]) -> Result<Self> {
        let tags = tags.iter().map(|t| t.as_ref().deref().clone());
        Ok(Self {
            inner: nostr::EventBuilder::long_form_text_note(content, tags),
        })
    }

    #[uniffi::constructor]
    pub fn contact_list(list: &[Arc<Contact>]) -> Self {
        let list: Vec<ContactSdk> = list.iter().map(|c| c.as_ref().deref().clone()).collect();

        Self {
            inner: nostr::EventBuilder::contact_list(list),
        }
    }

    /// Create encrypted direct msg event
    #[uniffi::constructor]
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

    #[uniffi::constructor]
    pub fn repost(event: &Event, relay_url: Option<String>) -> Self {
        Self {
            inner: nostr::EventBuilder::repost(event.deref(), relay_url.map(UncheckedUrl::from)),
        }
    }

    /// Create delete event
    #[uniffi::constructor]
    pub fn delete(ids: &[Arc<EventId>], reason: Option<String>) -> Self {
        let ids = ids.iter().map(|e| ***e);
        Self {
            inner: match reason {
                Some(reason) => nostr::EventBuilder::delete_with_reason(ids, reason),
                None => nostr::EventBuilder::delete(ids),
            },
        }
    }

    #[uniffi::constructor]
    pub fn reaction(event: &Event, reaction: &str) -> Self {
        Self {
            inner: nostr::EventBuilder::reaction(event.deref(), reaction),
        }
    }

    #[uniffi::constructor]
    pub fn channel(metadata: &Metadata) -> Self {
        Self {
            inner: nostr::EventBuilder::channel(metadata.deref()),
        }
    }

    #[uniffi::constructor]
    pub fn channel_metadata(
        channel_id: &EventId,
        relay_url: Option<String>,
        metadata: &Metadata,
    ) -> Result<Self> {
        let relay_url = match relay_url {
            Some(url) => Some(Url::parse(&url)?),
            None => None,
        };
        Ok(Self {
            inner: nostr::EventBuilder::channel_metadata(**channel_id, relay_url, metadata.deref()),
        })
    }

    #[uniffi::constructor]
    pub fn channel_msg(channel_id: &EventId, relay_url: &str, content: &str) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventBuilder::channel_msg(**channel_id, Url::parse(relay_url)?, content),
        })
    }

    #[uniffi::constructor]
    pub fn hide_channel_msg(message_id: &EventId, reason: Option<String>) -> Self {
        Self {
            inner: nostr::EventBuilder::hide_channel_msg(**message_id, reason),
        }
    }

    #[uniffi::constructor]
    pub fn mute_channel_user(public_key: &PublicKey, reason: Option<String>) -> Self {
        Self {
            inner: nostr::EventBuilder::mute_channel_user(**public_key, reason),
        }
    }

    #[uniffi::constructor]
    pub fn auth(challenge: &str, relay_url: &str) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventBuilder::auth(challenge, Url::parse(relay_url)?),
        })
    }

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

    #[uniffi::constructor]
    pub fn live_event(live_event: LiveEvent) -> Self {
        Self {
            inner: nostr::EventBuilder::live_event(live_event.into()),
        }
    }

    #[uniffi::constructor]
    pub fn live_event_msg(
        live_event_id: &str,
        live_event_host: &PublicKey,
        content: &str,
        relay_url: Option<String>,
        tags: &[Arc<Tag>],
    ) -> Result<Self> {
        let relay_url = match relay_url {
            Some(url) => Some(Url::parse(&url)?),
            None => None,
        };
        let tags = tags.iter().map(|t| t.as_ref().deref().clone()).collect();
        Ok(Self {
            inner: nostr::EventBuilder::live_event_msg(
                live_event_id,
                **live_event_host,
                content,
                relay_url,
                tags,
            ),
        })
    }

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

    #[uniffi::constructor]
    pub fn zap_receipt(bolt11: String, preimage: Option<String>, zap_request: &Event) -> Self {
        Self {
            inner: nostr::EventBuilder::zap_receipt(bolt11, preimage, zap_request.deref().clone()),
        }
    }

    #[uniffi::constructor]
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

    #[uniffi::constructor]
    pub fn award_badge(badge_definition: &Event, awarded_pubkeys: &[Arc<Tag>]) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventBuilder::award_badge(
                badge_definition.deref(),
                awarded_pubkeys.iter().map(|a| a.as_ref().deref().clone()),
            )?,
        })
    }

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

    /// Data Vending Machine - Job Request
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

    #[uniffi::constructor]
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

    #[uniffi::constructor]
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

    #[uniffi::constructor]
    pub fn file_metadata(description: &str, metadata: &FileMetadata) -> Self {
        Self {
            inner: nostr::EventBuilder::file_metadata(description, metadata.deref().clone()),
        }
    }

    #[uniffi::constructor]
    pub fn http_auth(data: HttpData) -> Self {
        Self {
            inner: nostr::EventBuilder::http_auth(data.into()),
        }
    }

    #[uniffi::constructor]
    pub fn stall_data(data: &StallData) -> Self {
        Self {
            inner: nostr::EventBuilder::stall_data(data.deref().clone()),
        }
    }

    #[uniffi::constructor]
    pub fn product_data(data: ProductData) -> Self {
        Self {
            inner: nostr::EventBuilder::product_data(data.into()),
        }
    }

    /// GiftWrapped Sealed Direct message
    #[uniffi::constructor]
    pub fn sealed_direct(receiver: &PublicKey, message: &str) -> Self {
        Self {
            inner: nostr::EventBuilder::sealed_direct(**receiver, message),
        }
    }

    #[uniffi::constructor]
    pub fn mute_list(list: MuteList) -> Self {
        Self {
            inner: nostr::EventBuilder::mute_list(list.into()),
        }
    }

    #[uniffi::constructor]
    pub fn pinned_notes(ids: Vec<Arc<EventId>>) -> Self {
        Self {
            inner: nostr::EventBuilder::pinned_notes(ids.into_iter().map(|e| **e)),
        }
    }

    #[uniffi::constructor]
    pub fn bookmarks(list: Bookmarks) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventBuilder::bookmarks(list.try_into()?),
        })
    }

    #[uniffi::constructor]
    pub fn communities(communities: Vec<Arc<Coordinate>>) -> Self {
        Self {
            inner: nostr::EventBuilder::communities(
                communities.into_iter().map(|c| c.as_ref().into()),
            ),
        }
    }

    #[uniffi::constructor]
    pub fn public_chats(chat: Vec<Arc<EventId>>) -> Self {
        Self {
            inner: nostr::EventBuilder::public_chats(chat.into_iter().map(|e| **e)),
        }
    }

    #[uniffi::constructor]
    pub fn blocked_relays(relay: Vec<String>) -> Self {
        Self {
            inner: nostr::EventBuilder::blocked_relays(relay.into_iter().map(UncheckedUrl::from)),
        }
    }

    #[uniffi::constructor]
    pub fn search_relays(relay: Vec<String>) -> Self {
        Self {
            inner: nostr::EventBuilder::search_relays(relay.into_iter().map(UncheckedUrl::from)),
        }
    }

    #[uniffi::constructor]
    pub fn interests(list: Interests) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventBuilder::interests(list.into()),
        })
    }

    #[uniffi::constructor]
    pub fn emojis(list: Emojis) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventBuilder::emojis(list.into()),
        })
    }

    #[uniffi::constructor]
    pub fn follow_sets(publick_key: Vec<Arc<PublicKey>>) -> Self {
        Self {
            inner: nostr::EventBuilder::follow_sets(publick_key.into_iter().map(|p| **p)),
        }
    }

    #[uniffi::constructor]
    pub fn relay_sets(relay: Vec<String>) -> Self {
        Self {
            inner: nostr::EventBuilder::relay_sets(relay.into_iter().map(UncheckedUrl::from)),
        }
    }

    #[uniffi::constructor]
    pub fn bookmarks_sets(list: Bookmarks) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventBuilder::bookmarks_sets(list.try_into()?),
        })
    }

    #[uniffi::constructor]
    pub fn articles_curation_sets(list: ArticlesCuration) -> Self {
        Self {
            inner: nostr::EventBuilder::articles_curation_sets(list.into()),
        }
    }
}
