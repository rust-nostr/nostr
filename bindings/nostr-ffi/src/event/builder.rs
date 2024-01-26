// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use nostr::{Contact as ContactSdk, UncheckedUrl, Url};
use uniffi::Object;

use super::{Event, EventId};
use crate::error::Result;
use crate::helper::unwrap_or_clone_arc;
use crate::key::Keys;
use crate::nips::nip15::{ProductData, StallData};
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
    pub fn new(kind: u64, content: String, tags: Vec<Arc<Tag>>) -> Result<Arc<Self>> {
        let tags = tags.into_iter().map(|t| t.as_ref().deref().clone());
        Ok(Arc::new(Self {
            inner: nostr::EventBuilder::new(kind.into(), content, tags),
        }))
    }

    /// Set a custom `created_at` UNIX timestamp
    pub fn custom_created_at(self: Arc<Self>, created_at: Arc<Timestamp>) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.custom_created_at(**created_at);
        builder
    }

    pub fn to_event(&self, keys: Arc<Keys>) -> Result<Arc<Event>> {
        let event = self.inner.clone().to_event(keys.deref())?;
        Ok(Arc::new(event.into()))
    }

    pub fn to_pow_event(&self, keys: Arc<Keys>, difficulty: u8) -> Result<Arc<Event>> {
        Ok(Arc::new(
            self.inner
                .clone()
                .to_pow_event(keys.deref(), difficulty)?
                .into(),
        ))
    }

    pub fn to_unsigned_event(&self, public_key: Arc<PublicKey>) -> Arc<UnsignedEvent> {
        Arc::new(
            self.inner
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
            self.inner
                .clone()
                .to_unsigned_pow_event(*public_key.as_ref().deref(), difficulty)
                .into(),
        )
    }

    #[uniffi::constructor]
    pub fn metadata(metadata: Arc<Metadata>) -> Arc<Self> {
        Arc::new(Self {
            inner: nostr::EventBuilder::metadata(metadata.as_ref().deref()),
        })
    }

    #[uniffi::constructor]
    pub fn relay_list(list: HashMap<String, Option<RelayMetadata>>) -> Arc<Self> {
        let iter = list
            .into_iter()
            .map(|(url, r)| (UncheckedUrl::from(url), r.map(|r| r.into())));
        Arc::new(Self {
            inner: nostr::EventBuilder::relay_list(iter),
        })
    }

    #[uniffi::constructor]
    pub fn text_note(content: String, tags: Vec<Arc<Tag>>) -> Result<Arc<Self>> {
        let tags = tags.into_iter().map(|t| t.as_ref().deref().clone());
        Ok(Arc::new(Self {
            inner: nostr::EventBuilder::text_note(content, tags),
        }))
    }

    #[uniffi::constructor]
    pub fn long_form_text_note(content: String, tags: Vec<Arc<Tag>>) -> Result<Arc<Self>> {
        let tags = tags.into_iter().map(|t| t.as_ref().deref().clone());
        Ok(Arc::new(Self {
            inner: nostr::EventBuilder::long_form_text_note(content, tags),
        }))
    }

    #[uniffi::constructor]
    pub fn contact_list(list: Vec<Arc<Contact>>) -> Arc<Self> {
        let list: Vec<ContactSdk> = list
            .into_iter()
            .map(|c| c.as_ref().deref().clone())
            .collect();

        Arc::new(Self {
            inner: nostr::EventBuilder::contact_list(list),
        })
    }

    /// Create encrypted direct msg event
    #[uniffi::constructor]
    pub fn encrypted_direct_msg(
        sender_keys: Arc<Keys>,
        receiver_pubkey: Arc<PublicKey>,
        content: String,
        reply_to: Option<Arc<EventId>>,
    ) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            inner: nostr::EventBuilder::encrypted_direct_msg(
                sender_keys.deref(),
                *receiver_pubkey.as_ref().deref(),
                content,
                reply_to.map(|id| id.as_ref().into()),
            )?,
        }))
    }

    #[uniffi::constructor]
    pub fn repost(event_id: Arc<EventId>, public_key: Arc<PublicKey>) -> Arc<Self> {
        Arc::new(Self {
            inner: nostr::EventBuilder::repost(
                event_id.as_ref().into(),
                *public_key.as_ref().deref(),
            ),
        })
    }

    /// Create delete event
    #[uniffi::constructor]
    pub fn delete(ids: Vec<Arc<EventId>>, reason: Option<String>) -> Arc<Self> {
        let ids: Vec<nostr::EventId> = ids.into_iter().map(|e| e.as_ref().into()).collect();
        Arc::new(Self {
            inner: match reason {
                Some(reason) => nostr::EventBuilder::delete_with_reason(ids, reason),
                None => nostr::EventBuilder::delete(ids),
            },
        })
    }

    #[uniffi::constructor]
    pub fn reaction(
        event_id: Arc<EventId>,
        public_key: Arc<PublicKey>,
        content: String,
    ) -> Arc<Self> {
        Arc::new(Self {
            inner: nostr::EventBuilder::reaction(
                event_id.as_ref().into(),
                *public_key.as_ref().deref(),
                content,
            ),
        })
    }

    #[uniffi::constructor]
    pub fn channel(metadata: Arc<Metadata>) -> Arc<Self> {
        Arc::new(Self {
            inner: nostr::EventBuilder::channel(metadata.as_ref().deref()),
        })
    }

    #[uniffi::constructor]
    pub fn channel_metadata(
        channel_id: Arc<EventId>,
        relay_url: Option<String>,
        metadata: Arc<Metadata>,
    ) -> Result<Arc<Self>> {
        let relay_url = match relay_url {
            Some(url) => Some(Url::parse(&url)?),
            None => None,
        };
        Ok(Arc::new(Self {
            inner: nostr::EventBuilder::channel_metadata(
                **channel_id,
                relay_url,
                metadata.as_ref().deref(),
            ),
        }))
    }

    #[uniffi::constructor]
    pub fn channel_msg(
        channel_id: Arc<EventId>,
        relay_url: String,
        content: String,
    ) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            inner: nostr::EventBuilder::channel_msg(**channel_id, Url::parse(&relay_url)?, content),
        }))
    }

    #[uniffi::constructor]
    pub fn hide_channel_msg(message_id: Arc<EventId>, reason: Option<String>) -> Arc<Self> {
        Arc::new(Self {
            inner: nostr::EventBuilder::hide_channel_msg(message_id.as_ref().into(), reason),
        })
    }

    #[uniffi::constructor]
    pub fn mute_channel_user(public_key: Arc<PublicKey>, reason: Option<String>) -> Arc<Self> {
        Arc::new(Self {
            inner: nostr::EventBuilder::mute_channel_user(*public_key.as_ref().deref(), reason),
        })
    }

    #[uniffi::constructor]
    pub fn auth(challenge: String, relay_url: String) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            inner: nostr::EventBuilder::auth(challenge, Url::parse(&relay_url)?),
        }))
    }

    #[uniffi::constructor]
    pub fn nostr_connect(
        sender_keys: Arc<Keys>,
        receiver_pubkey: Arc<PublicKey>,
        msg: NostrConnectMessage,
    ) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            inner: nostr::EventBuilder::nostr_connect(
                sender_keys.as_ref().deref(),
                **receiver_pubkey,
                msg.try_into()?,
            )?,
        }))
    }

    #[uniffi::constructor]
    pub fn live_event(live_event: LiveEvent) -> Arc<Self> {
        Arc::new(Self {
            inner: nostr::EventBuilder::live_event(live_event.into()),
        })
    }

    #[uniffi::constructor]
    pub fn live_event_msg(
        live_event_id: String,
        live_event_host: Arc<PublicKey>,
        content: String,
        relay_url: Option<String>,
        tags: Vec<Arc<Tag>>,
    ) -> Result<Arc<Self>> {
        let relay_url = match relay_url {
            Some(url) => Some(Url::parse(&url)?),
            None => None,
        };
        let tags = tags
            .into_iter()
            .map(|t| t.as_ref().deref().clone())
            .collect();
        Ok(Arc::new(Self {
            inner: nostr::EventBuilder::live_event_msg(
                live_event_id,
                **live_event_host,
                content,
                relay_url,
                tags,
            ),
        }))
    }

    #[uniffi::constructor]
    pub fn report(tags: Vec<Arc<Tag>>, content: String) -> Arc<Self> {
        let tags = tags.into_iter().map(|t| t.as_ref().deref().clone());
        Arc::new(Self {
            inner: nostr::EventBuilder::report(tags, content),
        })
    }

    /// Create **public** zap request event
    ///
    /// **This event MUST NOT be broadcasted to relays**, instead must be sent to a recipient's LNURL pay callback url.
    ///
    /// To build a **private** or **anonymous** zap request use `nip57_private_zap_request(...)` or `nip57_anonymous_zap_request(...)` functions.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/57.md>
    #[uniffi::constructor]
    pub fn public_zap_request(data: Arc<ZapRequestData>) -> Arc<Self> {
        Arc::new(Self {
            inner: nostr::EventBuilder::public_zap_request(data.as_ref().deref().clone()),
        })
    }

    #[uniffi::constructor]
    pub fn zap_receipt(
        bolt11: String,
        preimage: Option<String>,
        zap_request: Arc<Event>,
    ) -> Arc<Self> {
        Arc::new(Self {
            inner: nostr::EventBuilder::zap_receipt(
                bolt11,
                preimage,
                zap_request.as_ref().deref().clone(),
            ),
        })
    }

    #[uniffi::constructor]
    pub fn define_badge(
        badge_id: String,
        name: Option<String>,
        description: Option<String>,
        image: Option<String>,
        image_dimensions: Option<Arc<ImageDimensions>>,
        thumbnails: Vec<Image>,
    ) -> Arc<Self> {
        Arc::new(Self {
            inner: nostr::EventBuilder::define_badge(
                badge_id,
                name,
                description,
                image.map(UncheckedUrl::from),
                image_dimensions.map(|i| i.as_ref().into()),
                thumbnails
                    .into_iter()
                    .map(|i: Image| {
                        (
                            UncheckedUrl::from(i.url),
                            i.dimensions.map(|d| d.as_ref().into()),
                        )
                    })
                    .collect(),
            ),
        })
    }

    #[uniffi::constructor]
    pub fn award_badge(
        badge_definition: Arc<Event>,
        awarded_pubkeys: Vec<Arc<Tag>>,
    ) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            inner: nostr::EventBuilder::award_badge(
                badge_definition.as_ref().deref(),
                awarded_pubkeys
                    .into_iter()
                    .map(|a| a.as_ref().deref().clone()),
            )?,
        }))
    }

    #[uniffi::constructor]
    pub fn profile_badges(
        badge_definitions: Vec<Arc<Event>>,
        badge_awards: Vec<Arc<Event>>,
        pubkey_awarded: &PublicKey,
    ) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            inner: nostr::EventBuilder::profile_badges(
                badge_definitions
                    .into_iter()
                    .map(|b| b.as_ref().deref().clone())
                    .collect(),
                badge_awards
                    .into_iter()
                    .map(|b| b.as_ref().deref().clone())
                    .collect(),
                pubkey_awarded.deref(),
            )?,
        }))
    }

    /// Data Vending Machine - Job Request
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    #[uniffi::constructor]
    pub fn job_request(kind: u64, tags: Vec<Arc<Tag>>) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            inner: nostr::EventBuilder::job_request(
                kind.into(),
                tags.into_iter().map(|t| t.as_ref().deref().clone()),
            )?,
        }))
    }

    #[uniffi::constructor]
    pub fn job_result(
        job_request: Arc<Event>,
        amount_millisats: u64,
        bolt11: Option<String>,
    ) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            inner: nostr::EventBuilder::job_result(
                job_request.as_ref().deref().clone(),
                amount_millisats,
                bolt11,
            )?,
        }))
    }

    #[uniffi::constructor]
    pub fn job_feedback(
        job_request: Arc<Event>,
        status: DataVendingMachineStatus,
        extra_info: Option<String>,
        amount_millisats: u64,
        bolt11: Option<String>,
        payload: Option<String>,
    ) -> Arc<Self> {
        Arc::new(Self {
            inner: nostr::EventBuilder::job_feedback(
                job_request.as_ref().deref(),
                status.into(),
                extra_info,
                amount_millisats,
                bolt11,
                payload,
            ),
        })
    }

    #[uniffi::constructor]
    pub fn file_metadata(description: String, metadata: Arc<FileMetadata>) -> Arc<Self> {
        Arc::new(Self {
            inner: nostr::EventBuilder::file_metadata(
                description,
                metadata.as_ref().deref().clone(),
            ),
        })
    }

    #[uniffi::constructor]
    pub fn http_auth(data: HttpData) -> Self {
        Self {
            inner: nostr::EventBuilder::http_auth(data.into()),
        }
    }

    #[uniffi::constructor]
    pub fn stall_data(data: StallData) -> Self {
        Self {
            inner: nostr::EventBuilder::stall_data(data.into()),
        }
    }

    #[uniffi::constructor]
    pub fn product_data(data: ProductData) -> Self {
        Self {
            inner: nostr::EventBuilder::product_data(data.into()),
        }
    }
}
