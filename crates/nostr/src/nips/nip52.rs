// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP-52: Calendar Events
//!
//! <https://github.com/nostr-protocol/nips/blob/master/52.md>

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;

use crate::nips::nip01::Coordinate;
use crate::types::{RelayUrl, Url};
use crate::{Alphabet, EventId, ImageDimensions, PublicKey, SingleLetterTag, Tag, TagKind, TagStandard, Timestamp};

/// Check if a string matches YYYY-MM-DD format
fn is_valid_date_format(s: &str) -> bool {
    let bytes = s.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[..4].iter().all(|b| b.is_ascii_digit())
        && bytes[5..7].iter().all(|b| b.is_ascii_digit())
        && bytes[8..10].iter().all(|b| b.is_ascii_digit())
}

const START_TZID_STR: &str = "start_tzid";
const END_TZID_STR: &str = "end_tzid";
const FB_STR: &str = "fb";

/// NIP-52 Error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Identifier (`d` tag) missing
    IdentifierMissing,
    /// Title missing
    TitleMissing,
    /// Start missing
    StartMissing,
    /// Status missing
    StatusMissing,
    /// Coordinate (`a` tag) missing
    CoordinateMissing,
    /// Unknown RSVP status
    UnknownRsvpStatus(String),
    /// Unknown free/busy value
    UnknownFreeBusy(String),
    /// Invalid date format (expected YYYY-MM-DD)
    InvalidDateFormat(String),
    /// Invalid timestamp
    InvalidTimestamp(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IdentifierMissing => f.write_str("Missing identifier (d tag)"),
            Self::TitleMissing => f.write_str("Missing title"),
            Self::StartMissing => f.write_str("Missing start"),
            Self::StatusMissing => f.write_str("Missing status"),
            Self::CoordinateMissing => f.write_str("Missing coordinate (a tag)"),
            Self::UnknownRsvpStatus(s) => write!(f, "Unknown RSVP status: {s}"),
            Self::UnknownFreeBusy(s) => write!(f, "Unknown free/busy value: {s}"),
            Self::InvalidDateFormat(s) => write!(f, "Invalid date format (expected YYYY-MM-DD): {s}"),
            Self::InvalidTimestamp(s) => write!(f, "Invalid timestamp: {s}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

/// Calendar Event RSVP Status
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CalendarEventRsvpStatus {
    /// Accepted
    Accepted,
    /// Declined
    Declined,
    /// Tentative
    Tentative,
}

impl fmt::Display for CalendarEventRsvpStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl CalendarEventRsvpStatus {
    /// Get as `&str`
    pub fn as_str(&self) -> &str {
        match self {
            Self::Accepted => "accepted",
            Self::Declined => "declined",
            Self::Tentative => "tentative",
        }
    }
}

impl FromStr for CalendarEventRsvpStatus {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "accepted" => Ok(Self::Accepted),
            "declined" => Ok(Self::Declined),
            "tentative" => Ok(Self::Tentative),
            s => Err(Error::UnknownRsvpStatus(s.to_string())),
        }
    }
}

/// Free/Busy indicator
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FreeBusy {
    /// Free
    Free,
    /// Busy
    Busy,
}

impl fmt::Display for FreeBusy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FreeBusy {
    /// Get as `&str`
    pub fn as_str(&self) -> &str {
        match self {
            Self::Free => "free",
            Self::Busy => "busy",
        }
    }
}

impl FromStr for FreeBusy {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "free" => Ok(Self::Free),
            "busy" => Ok(Self::Busy),
            s => Err(Error::UnknownFreeBusy(s.to_string())),
        }
    }
}

/// Date-Based Calendar Event (kind 31922)
///
/// The event description is stored in the Nostr event's `content` field
/// and is not modeled here (handled at the `Event` level).
///
/// <https://github.com/nostr-protocol/nips/blob/master/52.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DateBasedCalendarEvent {
    /// Identifier (`d` tag)
    pub id: String,
    /// Title
    pub title: String,
    /// Start date (YYYY-MM-DD)
    pub start: String,
    /// End date (YYYY-MM-DD, exclusive)
    pub end: Option<String>,
    /// Summary
    pub summary: Option<String>,
    /// Image
    pub image: Option<(Url, Option<ImageDimensions>)>,
    /// Locations (repeatable)
    pub locations: Vec<String>,
    /// Geohash
    pub geohash: Option<String>,
    /// Participants (pubkey, optional relay URL, optional role)
    pub participants: Vec<(PublicKey, Option<RelayUrl>, Option<String>)>,
    /// Hashtags
    pub hashtags: Vec<String>,
    /// References
    pub references: Vec<String>,
    /// Coordinates (`a` tags, optional) — references to kind:31924 calendars requesting inclusion
    pub coordinates: Vec<(Coordinate, Option<RelayUrl>)>,
}

impl DateBasedCalendarEvent {
    /// Create a new date-based calendar event
    pub fn new<S1, S2, S3>(id: S1, title: S2, start: S3) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<String>,
    {
        Self {
            id: id.into(),
            title: title.into(),
            start: start.into(),
            end: None,
            summary: None,
            image: None,
            locations: Vec::new(),
            geohash: None,
            participants: Vec::new(),
            hashtags: Vec::new(),
            references: Vec::new(),
            coordinates: Vec::new(),
        }
    }
}

impl From<DateBasedCalendarEvent> for Vec<Tag> {
    fn from(event: DateBasedCalendarEvent) -> Self {
        let DateBasedCalendarEvent {
            id,
            title,
            start,
            end,
            summary,
            image,
            locations,
            geohash,
            participants,
            hashtags,
            references,
            coordinates,
        } = event;

        let mut tags = Vec::new();

        tags.push(Tag::identifier(id));
        tags.push(Tag::from_standardized_without_cell(TagStandard::Title(
            title,
        )));
        tags.push(Tag::custom(TagKind::Start, [&start]));

        if let Some(end) = end {
            tags.push(Tag::custom(TagKind::End, [&end]));
        }

        if let Some(summary) = summary {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Summary(
                summary,
            )));
        }

        if let Some((image, dim)) = image {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Image(
                image, dim,
            )));
        }

        for location in locations {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Location(
                location,
            )));
        }

        if let Some(geohash) = geohash {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Geohash(
                geohash,
            )));
        }

        for (pubkey, relay_url, role) in participants {
            tags.push(Tag::from_standardized_without_cell(TagStandard::PublicKey {
                public_key: pubkey,
                relay_url,
                alias: role,
                uppercase: false,
            }));
        }

        for hashtag in hashtags {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Hashtag(
                hashtag,
            )));
        }

        for reference in references {
            tags.push(Tag::from_standardized_without_cell(
                TagStandard::Reference(reference),
            ));
        }

        for (coordinate, relay_url) in coordinates {
            tags.push(Tag::from_standardized_without_cell(
                TagStandard::Coordinate {
                    coordinate,
                    relay_url,
                    uppercase: false,
                },
            ));
        }

        tags
    }
}

impl TryFrom<Vec<Tag>> for DateBasedCalendarEvent {
    type Error = Error;

    fn try_from(tags: Vec<Tag>) -> Result<Self, Self::Error> {
        let id: &str = tags
            .iter()
            .find(|t| t.kind() == TagKind::d())
            .and_then(|t| t.content())
            .ok_or(Error::IdentifierMissing)?;

        let mut event = DateBasedCalendarEvent {
            id: id.to_string(),
            title: String::new(),
            start: String::new(),
            end: None,
            summary: None,
            image: None,
            locations: Vec::new(),
            geohash: None,
            participants: Vec::new(),
            hashtags: Vec::new(),
            references: Vec::new(),
            coordinates: Vec::new(),
        };

        let mut has_title = false;
        let mut has_start = false;

        for tag in tags.into_iter() {
            match tag.kind() {
                TagKind::Start => {
                    if let Some(content) = tag.content() {
                        if !is_valid_date_format(content) {
                            return Err(Error::InvalidDateFormat(content.to_string()));
                        }
                        event.start = content.to_string();
                        has_start = true;
                    }
                }
                TagKind::End => {
                    if let Some(content) = tag.content() {
                        if !is_valid_date_format(content) {
                            return Err(Error::InvalidDateFormat(content.to_string()));
                        }
                        event.end = Some(content.to_string());
                    }
                }
                _ => {
                    if let Some(std_tag) = tag.to_standardized() {
                        match std_tag {
                            TagStandard::Title(title) => {
                                event.title = title;
                                has_title = true;
                            }
                            TagStandard::Summary(summary) => event.summary = Some(summary),
                            TagStandard::Image(url, dim) => event.image = Some((url, dim)),
                            TagStandard::Location(loc) => event.locations.push(loc),
                            TagStandard::Geohash(g) => event.geohash = Some(g),
                            TagStandard::PublicKey {
                                public_key,
                                relay_url,
                                alias,
                                uppercase: false,
                            } => {
                                event.participants.push((public_key, relay_url, alias));
                            }
                            TagStandard::Hashtag(h) => event.hashtags.push(h),
                            TagStandard::Reference(r) => event.references.push(r),
                            TagStandard::Coordinate {
                                coordinate,
                                relay_url,
                                uppercase: false,
                            } => {
                                event.coordinates.push((coordinate, relay_url));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        if !has_title {
            return Err(Error::TitleMissing);
        }
        if !has_start {
            return Err(Error::StartMissing);
        }

        Ok(event)
    }
}

/// Time-Based Calendar Event (kind 31923)
///
/// The event description is stored in the Nostr event's `content` field
/// and is not modeled here (handled at the `Event` level).
///
/// <https://github.com/nostr-protocol/nips/blob/master/52.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeBasedCalendarEvent {
    /// Identifier (`d` tag)
    pub id: String,
    /// Title
    pub title: String,
    /// Start timestamp (Unix)
    pub start: Timestamp,
    /// End timestamp (Unix)
    pub end: Option<Timestamp>,
    /// Start timezone (IANA)
    pub start_tzid: Option<String>,
    /// End timezone (IANA)
    pub end_tzid: Option<String>,
    /// Summary
    pub summary: Option<String>,
    /// Image
    pub image: Option<(Url, Option<ImageDimensions>)>,
    /// Locations (repeatable)
    pub locations: Vec<String>,
    /// Geohash
    pub geohash: Option<String>,
    /// Participants (pubkey, optional relay URL, optional role)
    pub participants: Vec<(PublicKey, Option<RelayUrl>, Option<String>)>,
    /// Hashtags
    pub hashtags: Vec<String>,
    /// References
    pub references: Vec<String>,
    /// Coordinates (`a` tags, optional) — references to kind:31924 calendars requesting inclusion
    pub coordinates: Vec<(Coordinate, Option<RelayUrl>)>,
}

impl TimeBasedCalendarEvent {
    /// Create a new time-based calendar event
    pub fn new<S1, S2>(id: S1, title: S2, start: Timestamp) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            id: id.into(),
            title: title.into(),
            start,
            end: None,
            start_tzid: None,
            end_tzid: None,
            summary: None,
            image: None,
            locations: Vec::new(),
            geohash: None,
            participants: Vec::new(),
            hashtags: Vec::new(),
            references: Vec::new(),
            coordinates: Vec::new(),
        }
    }
}

impl From<TimeBasedCalendarEvent> for Vec<Tag> {
    fn from(event: TimeBasedCalendarEvent) -> Self {
        let TimeBasedCalendarEvent {
            id,
            title,
            start,
            end,
            start_tzid,
            end_tzid,
            summary,
            image,
            locations,
            geohash,
            participants,
            hashtags,
            references,
            coordinates,
        } = event;

        let mut tags = Vec::new();

        tags.push(Tag::identifier(id));
        tags.push(Tag::from_standardized_without_cell(TagStandard::Title(
            title,
        )));
        tags.push(Tag::custom(TagKind::Start, [start.to_string()]));

        if let Some(end) = end {
            tags.push(Tag::custom(TagKind::End, [end.to_string()]));
        }

        // D tags: day-granularity timestamps for relay filtering
        const SECONDS_PER_DAY: u64 = 86400;
        let d_upper = TagKind::SingleLetter(SingleLetterTag {
            character: Alphabet::D,
            uppercase: true,
        });
        let start_day = start.as_secs() / SECONDS_PER_DAY;
        let end_day = end
            .map(|e| e.as_secs() / SECONDS_PER_DAY)
            .unwrap_or(start_day);
        for day in start_day..=end_day {
            tags.push(Tag::custom(d_upper.clone(), [day.to_string()]));
        }

        if let Some(tzid) = start_tzid {
            tags.push(Tag::custom(TagKind::custom(START_TZID_STR), [tzid]));
        }

        if let Some(tzid) = end_tzid {
            tags.push(Tag::custom(TagKind::custom(END_TZID_STR), [tzid]));
        }

        if let Some(summary) = summary {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Summary(
                summary,
            )));
        }

        if let Some((image, dim)) = image {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Image(
                image, dim,
            )));
        }

        for location in locations {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Location(
                location,
            )));
        }

        if let Some(geohash) = geohash {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Geohash(
                geohash,
            )));
        }

        for (pubkey, relay_url, role) in participants {
            tags.push(Tag::from_standardized_without_cell(TagStandard::PublicKey {
                public_key: pubkey,
                relay_url,
                alias: role,
                uppercase: false,
            }));
        }

        for hashtag in hashtags {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Hashtag(
                hashtag,
            )));
        }

        for reference in references {
            tags.push(Tag::from_standardized_without_cell(
                TagStandard::Reference(reference),
            ));
        }

        for (coordinate, relay_url) in coordinates {
            tags.push(Tag::from_standardized_without_cell(
                TagStandard::Coordinate {
                    coordinate,
                    relay_url,
                    uppercase: false,
                },
            ));
        }

        tags
    }
}

impl TryFrom<Vec<Tag>> for TimeBasedCalendarEvent {
    type Error = Error;

    fn try_from(tags: Vec<Tag>) -> Result<Self, Self::Error> {
        let id: &str = tags
            .iter()
            .find(|t| t.kind() == TagKind::d())
            .and_then(|t| t.content())
            .ok_or(Error::IdentifierMissing)?;

        let mut event = TimeBasedCalendarEvent {
            id: id.to_string(),
            title: String::new(),
            start: Timestamp::from(0),
            end: None,
            start_tzid: None,
            end_tzid: None,
            summary: None,
            image: None,
            locations: Vec::new(),
            geohash: None,
            participants: Vec::new(),
            hashtags: Vec::new(),
            references: Vec::new(),
            coordinates: Vec::new(),
        };

        let mut has_title = false;
        let mut has_start = false;

        for tag in tags.into_iter() {
            match tag.kind() {
                TagKind::Start => {
                    if let Some(content) = tag.content() {
                        match Timestamp::from_str(content) {
                            Ok(ts) => {
                                event.start = ts;
                                has_start = true;
                            }
                            Err(_) => return Err(Error::InvalidTimestamp(content.to_string())),
                        }
                    }
                }
                TagKind::End => {
                    if let Some(content) = tag.content() {
                        match Timestamp::from_str(content) {
                            Ok(ts) => event.end = Some(ts),
                            Err(_) => return Err(Error::InvalidTimestamp(content.to_string())),
                        }
                    }
                }
                TagKind::Custom(ref s) if s.as_ref() == START_TZID_STR => {
                    if let Some(content) = tag.content() {
                        event.start_tzid = Some(content.to_string());
                    }
                }
                TagKind::Custom(ref s) if s.as_ref() == END_TZID_STR => {
                    if let Some(content) = tag.content() {
                        event.end_tzid = Some(content.to_string());
                    }
                }
                _ => {
                    if let Some(std_tag) = tag.to_standardized() {
                        match std_tag {
                            TagStandard::Title(title) => {
                                event.title = title;
                                has_title = true;
                            }
                            TagStandard::Summary(summary) => event.summary = Some(summary),
                            TagStandard::Image(url, dim) => event.image = Some((url, dim)),
                            TagStandard::Location(loc) => event.locations.push(loc),
                            TagStandard::Geohash(g) => event.geohash = Some(g),
                            TagStandard::PublicKey {
                                public_key,
                                relay_url,
                                alias,
                                uppercase: false,
                            } => {
                                event.participants.push((public_key, relay_url, alias));
                            }
                            TagStandard::Hashtag(h) => event.hashtags.push(h),
                            TagStandard::Reference(r) => event.references.push(r),
                            TagStandard::Coordinate {
                                coordinate,
                                relay_url,
                                uppercase: false,
                            } => {
                                event.coordinates.push((coordinate, relay_url));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        if !has_title {
            return Err(Error::TitleMissing);
        }
        if !has_start {
            return Err(Error::StartMissing);
        }

        Ok(event)
    }
}

/// Calendar (kind 31924)
///
/// The calendar description is stored in the Nostr event's `content` field
/// and is not modeled here (handled at the `Event` level).
///
/// <https://github.com/nostr-protocol/nips/blob/master/52.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Calendar {
    /// Identifier (`d` tag)
    pub id: String,
    /// Title
    pub title: String,
    /// Calendar event coordinates (`a` tags)
    pub coordinates: Vec<(Coordinate, Option<RelayUrl>)>,
}

impl Calendar {
    /// Create a new calendar
    pub fn new<S1, S2>(id: S1, title: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            id: id.into(),
            title: title.into(),
            coordinates: Vec::new(),
        }
    }
}

impl From<Calendar> for Vec<Tag> {
    fn from(calendar: Calendar) -> Self {
        let Calendar {
            id,
            title,
            coordinates,
        } = calendar;

        let mut tags = Vec::new();

        tags.push(Tag::identifier(id));
        tags.push(Tag::from_standardized_without_cell(TagStandard::Title(
            title,
        )));

        for (coordinate, relay_url) in coordinates {
            tags.push(Tag::from_standardized_without_cell(
                TagStandard::Coordinate {
                    coordinate,
                    relay_url,
                    uppercase: false,
                },
            ));
        }

        tags
    }
}

impl TryFrom<Vec<Tag>> for Calendar {
    type Error = Error;

    fn try_from(tags: Vec<Tag>) -> Result<Self, Self::Error> {
        let id: &str = tags
            .iter()
            .find(|t| t.kind() == TagKind::d())
            .and_then(|t| t.content())
            .ok_or(Error::IdentifierMissing)?;

        let mut calendar = Calendar {
            id: id.to_string(),
            title: String::new(),
            coordinates: Vec::new(),
        };

        let mut has_title = false;

        for tag in tags.into_iter() {
            if let Some(std_tag) = tag.to_standardized() {
                match std_tag {
                    TagStandard::Title(title) => {
                        calendar.title = title;
                        has_title = true;
                    }
                    TagStandard::Coordinate {
                        coordinate,
                        relay_url,
                        uppercase: false,
                    } => {
                        calendar.coordinates.push((coordinate, relay_url));
                    }
                    _ => {}
                }
            }
        }

        if !has_title {
            return Err(Error::TitleMissing);
        }

        Ok(calendar)
    }
}

/// Calendar Event RSVP (kind 31925)
///
/// The RSVP note is stored in the Nostr event's `content` field
/// and is not modeled here (handled at the `Event` level).
///
/// <https://github.com/nostr-protocol/nips/blob/master/52.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CalendarEventRsvp {
    /// Identifier (`d` tag)
    pub id: String,
    /// Coordinate of the calendar event (`a` tag, required)
    pub coordinate: (Coordinate, Option<RelayUrl>),
    /// RSVP status
    pub status: CalendarEventRsvpStatus,
    /// Calendar event author pubkey (`p` tag, optional)
    pub author: Option<PublicKey>,
    /// Optional event ID (`e` tag, specific revision)
    pub event_id: Option<EventId>,
    /// Free/busy indicator
    pub free_busy: Option<FreeBusy>,
}

impl CalendarEventRsvp {
    /// Create a new calendar event RSVP
    pub fn new<S>(
        id: S,
        coordinate: Coordinate,
        status: CalendarEventRsvpStatus,
    ) -> Self
    where
        S: Into<String>,
    {
        Self {
            id: id.into(),
            coordinate: (coordinate, None),
            status,
            author: None,
            event_id: None,
            free_busy: None,
        }
    }
}

impl From<CalendarEventRsvp> for Vec<Tag> {
    fn from(rsvp: CalendarEventRsvp) -> Self {
        let CalendarEventRsvp {
            id,
            coordinate,
            status,
            author,
            event_id,
            free_busy,
        } = rsvp;

        let mut tags = Vec::new();

        tags.push(Tag::identifier(id));

        let (coord, relay_url) = coordinate;
        tags.push(Tag::from_standardized_without_cell(
            TagStandard::Coordinate {
                coordinate: coord,
                relay_url,
                uppercase: false,
            },
        ));

        tags.push(Tag::custom(
            TagKind::Status,
            [status.as_str()],
        ));

        if let Some(author) = author {
            tags.push(Tag::from_standardized_without_cell(TagStandard::PublicKey {
                public_key: author,
                relay_url: None,
                alias: None,
                uppercase: false,
            }));
        }

        if let Some(event_id) = event_id {
            tags.push(Tag::from_standardized_without_cell(TagStandard::event(
                event_id,
            )));
        }

        if let Some(fb) = free_busy {
            tags.push(Tag::custom(TagKind::custom(FB_STR), [fb.as_str()]));
        }

        tags
    }
}

impl TryFrom<Vec<Tag>> for CalendarEventRsvp {
    type Error = Error;

    fn try_from(tags: Vec<Tag>) -> Result<Self, Self::Error> {
        let id: String = tags
            .iter()
            .find(|t| t.kind() == TagKind::d())
            .and_then(|t| t.content())
            .ok_or(Error::IdentifierMissing)?
            .to_string();

        let mut coordinate: Option<(Coordinate, Option<RelayUrl>)> = None;
        let mut status: Option<CalendarEventRsvpStatus> = None;
        let mut author: Option<PublicKey> = None;
        let mut event_id: Option<EventId> = None;
        let mut free_busy: Option<FreeBusy> = None;

        for tag in tags.into_iter() {
            match tag.kind() {
                TagKind::Status => {
                    if let Some(content) = tag.content() {
                        if let Ok(s) = CalendarEventRsvpStatus::from_str(content) {
                            status = Some(s);
                        }
                    }
                }
                TagKind::Custom(ref s) if s.as_ref() == FB_STR => {
                    if let Some(content) = tag.content() {
                        if let Ok(fb) = FreeBusy::from_str(content) {
                            free_busy = Some(fb);
                        }
                    }
                }
                _ => {
                    if let Some(std_tag) = tag.to_standardized() {
                        match std_tag {
                            TagStandard::Coordinate {
                                coordinate: coord,
                                relay_url,
                                uppercase: false,
                            } => {
                                if coordinate.is_none() {
                                    coordinate = Some((coord, relay_url));
                                }
                            }
                            TagStandard::PublicKey {
                                public_key,
                                uppercase: false,
                                ..
                            } => {
                                if author.is_none() {
                                    author = Some(public_key);
                                }
                            }
                            TagStandard::Event {
                                event_id: eid,
                                uppercase: false,
                                ..
                            } => {
                                event_id = Some(eid);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        let coordinate = coordinate.ok_or(Error::CoordinateMissing)?;
        let status = status.ok_or(Error::StatusMissing)?;

        Ok(CalendarEventRsvp {
            id,
            coordinate,
            status,
            author,
            event_id,
            free_busy,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Kind;

    fn test_pubkey() -> PublicKey {
        PublicKey::from_str("32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245")
            .unwrap()
    }

    #[test]
    fn test_date_based_calendar_event_round_trip() {
        let event = DateBasedCalendarEvent {
            id: "poker-night".to_string(),
            title: "Poker Night".to_string(),
            start: "2023-12-25".to_string(),
            end: Some("2023-12-26".to_string()),
            summary: Some("A fun poker night".to_string()),
            image: None,
            locations: vec!["The Pub".to_string()],
            geohash: Some("u4pruydqqvj".to_string()),
            participants: vec![(test_pubkey(), None, Some("dealer".to_string()))],
            hashtags: vec!["poker".to_string()],
            references: vec!["https://example.com".to_string()],
            coordinates: Vec::new(),
        };

        let tags: Vec<Tag> = event.clone().into();

        // d, title, start, end, summary, location, geohash, p, t, r = 10
        assert_eq!(tags.len(), 10);
        assert_eq!(tags[0].kind(), TagKind::d());
        assert_eq!(tags[0].content(), Some("poker-night"));
        assert_eq!(tags[1].kind(), TagKind::Title);
        assert_eq!(tags[1].content(), Some("Poker Night"));
        assert_eq!(tags[2].kind(), TagKind::Start);
        assert_eq!(tags[2].content(), Some("2023-12-25"));
        assert_eq!(tags[3].kind(), TagKind::End);
        assert_eq!(tags[3].content(), Some("2023-12-26"));

        let parsed = DateBasedCalendarEvent::try_from(tags).unwrap();
        assert_eq!(parsed, event);
    }

    #[test]
    fn test_date_based_calendar_event_minimal() {
        let event = DateBasedCalendarEvent::new("meeting", "Team Meeting", "2024-01-15");

        let tags: Vec<Tag> = event.clone().into();
        let parsed = DateBasedCalendarEvent::try_from(tags).unwrap();

        assert_eq!(parsed, event);
    }

    #[test]
    fn test_date_based_calendar_event_missing_title() {
        let tags = vec![
            Tag::identifier("test"),
            Tag::custom(TagKind::Start, ["2024-01-01"]),
        ];
        let result = DateBasedCalendarEvent::try_from(tags);
        assert_eq!(result.unwrap_err(), Error::TitleMissing);
    }

    #[test]
    fn test_date_based_calendar_event_missing_start() {
        let tags = vec![
            Tag::identifier("test"),
            Tag::from_standardized_without_cell(TagStandard::Title("Test".to_string())),
        ];
        let result = DateBasedCalendarEvent::try_from(tags);
        assert_eq!(result.unwrap_err(), Error::StartMissing);
    }

    #[test]
    fn test_time_based_calendar_event_round_trip() {
        let event = TimeBasedCalendarEvent {
            id: "meetup-123".to_string(),
            title: "Nostr Meetup".to_string(),
            start: Timestamp::from(1700000000),
            end: Some(Timestamp::from(1700003600)),
            start_tzid: Some("America/New_York".to_string()),
            end_tzid: Some("America/New_York".to_string()),
            summary: Some("Monthly nostr meetup".to_string()),
            image: None,
            locations: vec!["NYC Hackerspace".to_string()],
            geohash: None,
            participants: vec![(test_pubkey(), None, Some("organizer".to_string()))],
            hashtags: vec!["nostr".to_string()],
            references: Vec::new(),
            coordinates: Vec::new(),
        };

        let tags: Vec<Tag> = event.clone().into();

        // d, title, start, end, D (1 day), start_tzid, end_tzid, summary, location, p, t = 11
        assert_eq!(tags.len(), 11);
        assert_eq!(tags[0].kind(), TagKind::d());
        assert_eq!(tags[0].content(), Some("meetup-123"));
        assert_eq!(tags[1].kind(), TagKind::Title);
        assert_eq!(tags[1].content(), Some("Nostr Meetup"));
        assert_eq!(tags[2].kind(), TagKind::Start);
        assert_eq!(tags[2].content(), Some("1700000000"));
        assert_eq!(tags[3].kind(), TagKind::End);
        assert_eq!(tags[3].content(), Some("1700003600"));

        let parsed = TimeBasedCalendarEvent::try_from(tags).unwrap();
        assert_eq!(parsed, event);
    }

    #[test]
    fn test_time_based_calendar_event_minimal() {
        let event = TimeBasedCalendarEvent::new("event-1", "Quick Chat", Timestamp::from(1700000000));

        let tags: Vec<Tag> = event.clone().into();

        // d, title, start, D = 4 tags
        assert_eq!(tags.len(), 4);
        assert_eq!(tags[0].kind(), TagKind::d());
        assert_eq!(tags[0].content(), Some("event-1"));
        assert_eq!(tags[1].kind(), TagKind::Title);
        assert_eq!(tags[1].content(), Some("Quick Chat"));
        assert_eq!(tags[2].kind(), TagKind::Start);
        assert_eq!(tags[2].content(), Some("1700000000"));
        // D tag with day-granularity timestamp
        let d_upper = TagKind::single_letter(Alphabet::D, true);
        assert_eq!(tags[3].kind(), d_upper);
        let expected_day = (1700000000u64 / 86400).to_string();
        assert_eq!(tags[3].content(), Some(expected_day.as_str()));

        let parsed = TimeBasedCalendarEvent::try_from(tags).unwrap();
        assert_eq!(parsed, event);
    }

    #[test]
    fn test_calendar_round_trip() {
        let coord = Coordinate {
            kind: Kind::DateBasedCalendarEvent,
            public_key: test_pubkey(),
            identifier: "poker-night".to_string(),
        };

        let calendar = Calendar {
            id: "my-calendar".to_string(),
            title: "My Calendar".to_string(),
            coordinates: vec![(coord, None)],
        };

        let tags: Vec<Tag> = calendar.clone().into();
        let parsed = Calendar::try_from(tags).unwrap();

        assert_eq!(parsed, calendar);
    }

    #[test]
    fn test_calendar_missing_title() {
        let tags = vec![Tag::identifier("test")];
        let result = Calendar::try_from(tags);
        assert_eq!(result.unwrap_err(), Error::TitleMissing);
    }

    #[test]
    fn test_rsvp_round_trip() {
        let coord = Coordinate {
            kind: Kind::TimeBasedCalendarEvent,
            public_key: test_pubkey(),
            identifier: "meetup-123".to_string(),
        };

        let rsvp = CalendarEventRsvp {
            id: "31923:32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245:meetup-123".to_string(),
            coordinate: (coord, None),
            status: CalendarEventRsvpStatus::Accepted,
            author: None,
            event_id: None,
            free_busy: Some(FreeBusy::Busy),
        };

        let tags: Vec<Tag> = rsvp.clone().into();
        let parsed = CalendarEventRsvp::try_from(tags).unwrap();

        assert_eq!(parsed, rsvp);
    }

    #[test]
    fn test_rsvp_missing_coordinate() {
        let tags = vec![
            Tag::identifier("test"),
            Tag::custom(TagKind::Status, ["accepted"]),
        ];
        let result = CalendarEventRsvp::try_from(tags);
        assert_eq!(result.unwrap_err(), Error::CoordinateMissing);
    }

    #[test]
    fn test_rsvp_missing_status() {
        let coord = Coordinate {
            kind: Kind::TimeBasedCalendarEvent,
            public_key: test_pubkey(),
            identifier: "event".to_string(),
        };
        let tags = vec![
            Tag::identifier("test"),
            Tag::from_standardized_without_cell(TagStandard::Coordinate {
                coordinate: coord,
                relay_url: None,
                uppercase: false,
            }),
        ];
        let result = CalendarEventRsvp::try_from(tags);
        assert_eq!(result.unwrap_err(), Error::StatusMissing);
    }

    #[test]
    fn test_rsvp_status_parsing() {
        assert_eq!(
            CalendarEventRsvpStatus::from_str("accepted").unwrap(),
            CalendarEventRsvpStatus::Accepted
        );
        assert_eq!(
            CalendarEventRsvpStatus::from_str("declined").unwrap(),
            CalendarEventRsvpStatus::Declined
        );
        assert_eq!(
            CalendarEventRsvpStatus::from_str("tentative").unwrap(),
            CalendarEventRsvpStatus::Tentative
        );
        assert!(CalendarEventRsvpStatus::from_str("unknown").is_err());
    }

    #[test]
    fn test_free_busy_parsing() {
        assert_eq!(FreeBusy::from_str("free").unwrap(), FreeBusy::Free);
        assert_eq!(FreeBusy::from_str("busy").unwrap(), FreeBusy::Busy);
        assert!(FreeBusy::from_str("unknown").is_err());
    }

    #[test]
    fn test_rsvp_status_serialized_values() {
        let coord = Coordinate {
            kind: Kind::TimeBasedCalendarEvent,
            public_key: test_pubkey(),
            identifier: "event".to_string(),
        };

        for (status, expected_str) in [
            (CalendarEventRsvpStatus::Accepted, "accepted"),
            (CalendarEventRsvpStatus::Declined, "declined"),
            (CalendarEventRsvpStatus::Tentative, "tentative"),
        ] {
            let rsvp = CalendarEventRsvp::new("test", coord.clone(), status);
            let tags: Vec<Tag> = rsvp.into();

            let status_tag = tags
                .iter()
                .find(|t| t.kind() == TagKind::Status)
                .expect("status tag must exist");
            assert_eq!(status_tag.content(), Some(expected_str));
        }
    }

    #[test]
    fn test_rsvp_round_trip_free() {
        let coord = Coordinate {
            kind: Kind::TimeBasedCalendarEvent,
            public_key: test_pubkey(),
            identifier: "meetup-123".to_string(),
        };

        let rsvp = CalendarEventRsvp {
            id: "rsvp-free".to_string(),
            coordinate: (coord, None),
            status: CalendarEventRsvpStatus::Accepted,
            author: None,
            event_id: None,
            free_busy: Some(FreeBusy::Free),
        };

        let tags: Vec<Tag> = rsvp.clone().into();
        let parsed = CalendarEventRsvp::try_from(tags).unwrap();
        assert_eq!(parsed, rsvp);
    }

    #[test]
    fn test_date_based_missing_identifier() {
        let tags = vec![
            Tag::from_standardized_without_cell(TagStandard::Title("Test".to_string())),
            Tag::custom(TagKind::Start, ["2024-01-01"]),
        ];
        assert_eq!(
            DateBasedCalendarEvent::try_from(tags).unwrap_err(),
            Error::IdentifierMissing
        );
    }

    #[test]
    fn test_time_based_missing_identifier() {
        let tags = vec![
            Tag::from_standardized_without_cell(TagStandard::Title("Test".to_string())),
            Tag::custom(TagKind::Start, ["1700000000"]),
        ];
        assert_eq!(
            TimeBasedCalendarEvent::try_from(tags).unwrap_err(),
            Error::IdentifierMissing
        );
    }

    #[test]
    fn test_time_based_missing_title() {
        let tags = vec![
            Tag::identifier("test"),
            Tag::custom(TagKind::Start, ["1700000000"]),
        ];
        assert_eq!(
            TimeBasedCalendarEvent::try_from(tags).unwrap_err(),
            Error::TitleMissing
        );
    }

    #[test]
    fn test_time_based_missing_start() {
        let tags = vec![
            Tag::identifier("test"),
            Tag::from_standardized_without_cell(TagStandard::Title("Test".to_string())),
        ];
        assert_eq!(
            TimeBasedCalendarEvent::try_from(tags).unwrap_err(),
            Error::StartMissing
        );
    }

    #[test]
    fn test_time_based_unparseable_start() {
        let tags = vec![
            Tag::identifier("test"),
            Tag::from_standardized_without_cell(TagStandard::Title("Test".to_string())),
            Tag::custom(TagKind::Start, ["not-a-timestamp"]),
        ];
        assert_eq!(
            TimeBasedCalendarEvent::try_from(tags).unwrap_err(),
            Error::InvalidTimestamp("not-a-timestamp".to_string())
        );
    }

    #[test]
    fn test_date_based_ignores_unknown_tags() {
        let tags = vec![
            Tag::identifier("test"),
            Tag::from_standardized_without_cell(TagStandard::Title("Test".to_string())),
            Tag::custom(TagKind::Start, ["2024-01-01"]),
            Tag::custom(TagKind::custom("unknown_tag"), ["some_value"]),
            Tag::custom(TagKind::custom("another"), ["a", "b", "c"]),
        ];
        let event = DateBasedCalendarEvent::try_from(tags).unwrap();
        assert_eq!(event.id, "test");
        assert_eq!(event.title, "Test");
        assert_eq!(event.start, "2024-01-01");
    }

    #[test]
    fn test_time_based_ignores_unknown_tags() {
        let tags = vec![
            Tag::identifier("test"),
            Tag::from_standardized_without_cell(TagStandard::Title("Test".to_string())),
            Tag::custom(TagKind::Start, ["1700000000"]),
            Tag::custom(TagKind::custom("foo"), ["bar"]),
        ];
        let event = TimeBasedCalendarEvent::try_from(tags).unwrap();
        assert_eq!(event.id, "test");
        assert_eq!(event.start, Timestamp::from(1700000000));
    }

    // M2: Date format validation for DateBasedCalendarEvent

    #[test]
    fn test_date_based_invalid_start_format() {
        let tags = vec![
            Tag::identifier("test"),
            Tag::from_standardized_without_cell(TagStandard::Title("Test".to_string())),
            Tag::custom(TagKind::Start, ["not-a-date"]),
        ];
        assert_eq!(
            DateBasedCalendarEvent::try_from(tags).unwrap_err(),
            Error::InvalidDateFormat("not-a-date".to_string())
        );
    }

    #[test]
    fn test_date_based_invalid_start_wrong_separator() {
        let tags = vec![
            Tag::identifier("test"),
            Tag::from_standardized_without_cell(TagStandard::Title("Test".to_string())),
            Tag::custom(TagKind::Start, ["2024/01/15"]),
        ];
        assert_eq!(
            DateBasedCalendarEvent::try_from(tags).unwrap_err(),
            Error::InvalidDateFormat("2024/01/15".to_string())
        );
    }

    #[test]
    fn test_date_based_invalid_end_format() {
        let tags = vec![
            Tag::identifier("test"),
            Tag::from_standardized_without_cell(TagStandard::Title("Test".to_string())),
            Tag::custom(TagKind::Start, ["2024-01-15"]),
            Tag::custom(TagKind::End, ["bad-end"]),
        ];
        assert_eq!(
            DateBasedCalendarEvent::try_from(tags).unwrap_err(),
            Error::InvalidDateFormat("bad-end".to_string())
        );
    }

    #[test]
    fn test_date_based_valid_date_format() {
        let tags = vec![
            Tag::identifier("test"),
            Tag::from_standardized_without_cell(TagStandard::Title("Test".to_string())),
            Tag::custom(TagKind::Start, ["2024-01-15"]),
            Tag::custom(TagKind::End, ["2024-01-16"]),
        ];
        let event = DateBasedCalendarEvent::try_from(tags).unwrap();
        assert_eq!(event.start, "2024-01-15");
        assert_eq!(event.end, Some("2024-01-16".to_string()));
    }

    // M4: Invalid timestamp should return InvalidTimestamp, not StartMissing

    #[test]
    fn test_time_based_invalid_start_timestamp() {
        let tags = vec![
            Tag::identifier("test"),
            Tag::from_standardized_without_cell(TagStandard::Title("Test".to_string())),
            Tag::custom(TagKind::Start, ["not-a-timestamp"]),
        ];
        assert_eq!(
            TimeBasedCalendarEvent::try_from(tags).unwrap_err(),
            Error::InvalidTimestamp("not-a-timestamp".to_string())
        );
    }

    #[test]
    fn test_time_based_invalid_end_timestamp() {
        let tags = vec![
            Tag::identifier("test"),
            Tag::from_standardized_without_cell(TagStandard::Title("Test".to_string())),
            Tag::custom(TagKind::Start, ["1700000000"]),
            Tag::custom(TagKind::End, ["garbage"]),
        ];
        assert_eq!(
            TimeBasedCalendarEvent::try_from(tags).unwrap_err(),
            Error::InvalidTimestamp("garbage".to_string())
        );
    }

    #[test]
    fn test_calendar_missing_identifier() {
        let tags = vec![Tag::from_standardized_without_cell(TagStandard::Title(
            "Test".to_string(),
        ))];
        assert_eq!(
            Calendar::try_from(tags).unwrap_err(),
            Error::IdentifierMissing
        );
    }

    #[test]
    fn test_rsvp_missing_identifier() {
        let coord = Coordinate {
            kind: Kind::TimeBasedCalendarEvent,
            public_key: test_pubkey(),
            identifier: "event".to_string(),
        };
        let tags = vec![
            Tag::from_standardized_without_cell(TagStandard::Coordinate {
                coordinate: coord,
                relay_url: None,
                uppercase: false,
            }),
            Tag::custom(TagKind::Status, ["accepted"]),
        ];
        assert_eq!(
            CalendarEventRsvp::try_from(tags).unwrap_err(),
            Error::IdentifierMissing
        );
    }
}
