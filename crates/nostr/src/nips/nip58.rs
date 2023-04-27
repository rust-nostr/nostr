//! NIP58
//!
//! <https://github.com/nostr-protocol/nips/blob/master/58.md>

use crate::{event::builder::Error as BuilderError, Event, EventBuilder, Keys, Kind, Tag};

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
