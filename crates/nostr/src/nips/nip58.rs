//! NIP58
//!
//! <https://github.com/nostr-protocol/nips/blob/master/58.md>

use secp256k1::XOnlyPublicKey;

use crate::event::builder::Error as BuilderError;
use crate::{Event, EventBuilder, Keys, Kind, Tag, UncheckedUrl};

#[derive(Debug, thiserror::Error)]
/// [`BadgeAward`] error
pub enum Error {
    /// Invalid kind
    #[error("invalid kind")]
    InvalidKind,
    /// Identifier tag not found
    #[error("identifier tag not found")]
    IdentifierTagNotFound,
    /// Event builder Error
    #[error(transparent)]
    Event(#[from] crate::event::builder::Error),
}

/// Simple struct to hold `width` x `height.
pub struct ImageDimensions(u64, u64);

/// [`BadgeDefinition`] event builder
pub struct BadgeDefinitionBuilder {
    badge_id: String,
    name: Option<String>,
    image: Option<String>,
    image_dimensions: Option<ImageDimensions>,
    description: Option<String>,
    thumbs: Option<Vec<(String, Option<ImageDimensions>)>>,
}

impl BadgeDefinitionBuilder {
    /// New [`BadgeDefinitionBuilder`]
    pub fn new(badge_id: String) -> Self {
        Self {
            badge_id,
            name: None,
            image: None,
            image_dimensions: None,
            description: None,
            thumbs: None,
        }
    }

    /// Set name
    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Set image
    pub fn image(mut self, image: String) -> Self {
        self.image = Some(image);
        self
    }

    /// Set `[ImageDimensions]`
    pub fn image_dimensions(mut self, image_dimensions: ImageDimensions) -> Self {
        self.image_dimensions = Some(image_dimensions);
        self
    }

    /// Set description
    pub fn description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set thumbnails with their optional `[ImageDimensions]`
    pub fn thumbs(mut self, thumbs: Vec<(String, Option<ImageDimensions>)>) -> Self {
        self.thumbs = Some(thumbs);
        self
    }

    /// Build [`Event`]
    pub fn build(self, keys: &Keys) -> Result<BadgeDefinition, BuilderError> {
        let mut tags: Vec<Tag> = Vec::new();
        let badge_id = Tag::Identifier(self.badge_id);
        tags.push(badge_id);

        if let Some(name) = self.name {
            let name_tag = Tag::Name(name);
            tags.push(name_tag);
        };

        if let Some(image) = self.image {
            let image_tag = if let Some(width_height) = self.image_dimensions {
                let ImageDimensions(width, height) = width_height;
                Tag::Image(image, Some((width, height)))
            } else {
                Tag::Image(image, None)
            };
            tags.push(image_tag);
        }

        if let Some(description) = self.description {
            let description_tag = Tag::Description(description);
            tags.push(description_tag);
        }
        if let Some(thumbs) = self.thumbs {
            for thumb in thumbs {
                let thumb_url = thumb.0;
                let thumb_tag = if let Some(width_height) = thumb.1 {
                    let ImageDimensions(width, height) = width_height;
                    Tag::Thumb(thumb_url, Some((width, height)))
                } else {
                    Tag::Thumb(thumb_url, None)
                };
                tags.push(thumb_tag);
            }
        }

        let event_builder = EventBuilder::new(Kind::BadgeDefinition, String::new(), &tags);
        let event = event_builder.to_event(keys)?;
        Ok(BadgeDefinition(event))
    }
}

/// Badge definition event as specified in NIP-58
pub struct BadgeDefinition(Event);

/// Badge award event as specified in NIP-58
pub struct BadgeAward(Event);

impl BadgeAward {
    ///
    pub fn new(
        badge_definition: &Event,
        awarded_pub_keys: Vec<Tag>,
        keys: &Keys,
    ) -> Result<BadgeAward, Error> {
        let badge_id = match badge_definition.kind {
            Kind::BadgeDefinition => badge_definition.tags.iter().find_map(|t| match t {
                Tag::Identifier(id) => Some(id),
                _ => None,
            }),
            _ => return Err(Error::InvalidKind),
        }
        .ok_or(Error::IdentifierTagNotFound)?;

        let awarded_pub_keys: Vec<Tag> = awarded_pub_keys
            .into_iter()
            .filter(|e| matches!(e, Tag::PubKey(..)))
            .collect();

        if awarded_pub_keys.is_empty() {
            return Err(Error::InvalidKind);
        }

        let a_tag = Tag::A {
            kind: Kind::BadgeDefinition,
            public_key: keys.public_key(),
            identifier: badge_id.to_owned(),
            relay_url: None,
        };
        let mut tags = vec![a_tag];
        tags.extend(awarded_pub_keys);

        let event_builder = EventBuilder::new(Kind::BadgeAward, String::new(), &tags);
        let event = event_builder.to_event(keys)?;

        Ok(BadgeAward(event))
    }
}

///  Profile Badges event as specified in NIP-58
pub struct ProfileBadgesEvent(Event);

/// [`ProfileBadgesEvent`] errors
#[derive(Debug, thiserror::Error)]
pub enum ProfileBadgesEventError {
    /// Invalid length
    #[error("invalid length")]
    InvalidLength,
    /// Invalid kind
    #[error("invalid kind")]
    InvalidKind,
    /// Mismatched badge definition or award
    #[error("mismatched badge definition/award")]
    MismatchedBadgeDefinitionOrAward,
    /// Badge awards lack the awarded public key
    #[error("badge award events lack the awarded public key")]
    BadgeAwardsLackAwardedPublicKey,
    /// Badge awards lack the awarded public key
    #[error("badge award event lacks `a` tag")]
    BadgeAwardMissingATag,
    /// Badge Definition Event error
    #[error(transparent)]
    BadgeDefinitionError(#[from] Error),
    /// Event builder Error
    #[error(transparent)]
    EventBuilder(#[from] crate::event::builder::Error),
}

impl ProfileBadgesEvent {
    /// Helper function to filter events for a specific [`Kind`]
    pub(crate) fn filter_for_kind(events: Vec<Event>, kind_needed: &Kind) -> Vec<Event> {
        events
            .into_iter()
            .filter(|e| e.kind == *kind_needed)
            .collect()
    }

    fn extract_identifier(tags: Vec<Tag>) -> Option<Tag> {
        tags.iter()
            .find(|tag| matches!(tag, Tag::Identifier(_)))
            .cloned()
    }

    fn extract_awarded_public_key(
        tags: &[Tag],
        awarded_public_key: &XOnlyPublicKey,
    ) -> Option<(XOnlyPublicKey, Option<UncheckedUrl>)> {
        tags.iter().find_map(|t| match t {
            Tag::PubKey(pub_key, unchecked_url) if pub_key == awarded_public_key => {
                Some((*pub_key, unchecked_url.clone()))
            }
            _ => None,
        })
    }

    /// Create a new [`ProfileBadgesEvent`] from badge definition and awards events
    /// [`badge_definitions`] and [`badge_awards`] must be ordered, so on the same position they refer to the same badge
    pub fn new(
        badge_definitions: Vec<Event>,
        badge_awards: Vec<Event>,
        pubkey_awarded: &XOnlyPublicKey,
        keys: &Keys,
    ) -> Result<ProfileBadgesEvent, ProfileBadgesEventError> {
        if badge_definitions.len() != badge_awards.len() {
            return Err(ProfileBadgesEventError::InvalidLength);
        }

        let mut badge_awards = ProfileBadgesEvent::filter_for_kind(badge_awards, &Kind::BadgeAward);
        if badge_awards.is_empty() {
            return Err(ProfileBadgesEventError::InvalidKind);
        }

        for award in &badge_awards {
            if !award.tags.iter().any(|t| match t {
                Tag::PubKey(pub_key, _) => pub_key == pubkey_awarded,
                _ => false,
            }) {
                return Err(ProfileBadgesEventError::BadgeAwardsLackAwardedPublicKey);
            }
        }

        let mut badge_definitions =
            ProfileBadgesEvent::filter_for_kind(badge_definitions, &Kind::BadgeDefinition);
        if badge_definitions.is_empty() {
            return Err(ProfileBadgesEventError::InvalidKind);
        }

        // Add identifier `d` tag
        let id_tag = Tag::Identifier("profile_badges".to_owned());
        let mut tags: Vec<Tag> = vec![id_tag];

        let badge_definitions_identifiers = badge_definitions
            .iter_mut()
            .map(|event| {
                let tags = core::mem::take(&mut event.tags);
                let id = Self::extract_identifier(tags).ok_or(
                    ProfileBadgesEventError::BadgeDefinitionError(Error::IdentifierTagNotFound),
                )?;

                Ok((event.clone(), id))
            })
            .collect::<Result<Vec<(Event, Tag)>, ProfileBadgesEventError>>();
        let badge_definitions_identifiers = badge_definitions_identifiers.map_err(|_| {
            ProfileBadgesEventError::BadgeDefinitionError(Error::IdentifierTagNotFound)
        })?;

        let badge_awards_identifiers = badge_awards
            .iter_mut()
            .map(|event| {
                let tags = core::mem::take(&mut event.tags);
                let (_, relay_url) = Self::extract_awarded_public_key(&tags, pubkey_awarded)
                    .ok_or(ProfileBadgesEventError::BadgeAwardsLackAwardedPublicKey)?;
                let (id, a_tag) = tags
                    .iter()
                    .find_map(|t| match t {
                        Tag::A { identifier, .. } => Some((identifier.clone(), t.clone())),
                        _ => None,
                    })
                    .ok_or(ProfileBadgesEventError::BadgeAwardMissingATag)?;
                Ok((event.clone(), id, a_tag, relay_url))
            })
            .collect::<Result<Vec<(Event, String, Tag, Option<UncheckedUrl>)>, ProfileBadgesEventError>>();
        let badge_awards_identifiers = badge_awards_identifiers?;

        // This collection has been filtered for the needed tags
        let users_badges: Vec<(_, _)> =
            core::iter::zip(badge_definitions_identifiers, badge_awards_identifiers).collect();

        for (badge_definition, badge_award) in users_badges {
            match (&badge_definition, &badge_award) {
                ((_, Tag::Identifier(identifier)), (_, badge_id, ..)) if badge_id != identifier => {
                    return Err(ProfileBadgesEventError::MismatchedBadgeDefinitionOrAward);
                }
                (
                    (_, Tag::Identifier(identifier)),
                    (badge_award_event, badge_id, a_tag, relay_url),
                ) if badge_id == identifier => {
                    let badge_definition_event_tag = a_tag.clone().to_owned();
                    let badge_award_event_tag =
                        Tag::Event(badge_award_event.id, relay_url.clone(), None);
                    tags.extend_from_slice(&[badge_definition_event_tag, badge_award_event_tag]);
                }
                _ => {}
            }
        }

        // Badge definitions and awards have been validated

        let event_builder = EventBuilder::new(Kind::ProfileBadges, String::new(), &tags);
        let event = event_builder.to_event(keys)?;

        Ok(ProfileBadgesEvent(event))
    }
}
