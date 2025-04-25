//! Database utilities for SQLite storage.

use std::collections::BTreeSet;
use std::io::{Error as IoError, ErrorKind};
use std::str::FromStr;

use nostr::{EventId, JsonUtil, Kind, PublicKey, RelayUrl, Tags, Timestamp, UnsignedEvent};
use nostr_mls_storage::groups::types::{
    Group, GroupExporterSecret, GroupRelay, GroupState, GroupType,
};
use nostr_mls_storage::messages::types::{Message, ProcessedMessage, ProcessedMessageState};
use nostr_mls_storage::welcomes::types::{
    ProcessedWelcome, ProcessedWelcomeState, Welcome, WelcomeState,
};
use rusqlite::types::Type;
use rusqlite::{Error, Result as SqliteResult, Row};

#[inline]
fn map_to_text_boxed_error<T>(e: T) -> Error
where
    T: std::error::Error + Send + Sync + 'static,
{
    Error::FromSqlConversionFailure(0, Type::Text, Box::new(e))
}

#[inline]
fn map_invalid_text_data(msg: &str) -> Error {
    Error::FromSqlConversionFailure(
        0,
        Type::Text,
        Box::new(IoError::new(ErrorKind::InvalidData, msg)),
    )
}

#[inline]
fn map_invalid_blob_data(msg: &str) -> Error {
    Error::FromSqlConversionFailure(
        0,
        Type::Blob,
        Box::new(IoError::new(ErrorKind::InvalidData, msg)),
    )
}

/// Convert a row to a Group struct
pub fn row_to_group(row: &Row) -> SqliteResult<Group> {
    let mls_group_id: Vec<u8> = row.get("mls_group_id")?;
    let nostr_group_id: String = row.get("nostr_group_id")?;
    let name: String = row.get("name")?;
    let description: String = row.get("description")?;

    // Parse admin pubkeys from JSON
    let admin_pubkeys_json: &str = row.get_ref("admin_pubkeys")?.as_str()?;
    let admin_pubkeys: BTreeSet<PublicKey> =
        serde_json::from_str(admin_pubkeys_json).map_err(map_to_text_boxed_error)?;

    let last_message_id: Option<&[u8]> = row.get_ref("last_message_id")?.as_blob_or_null()?;
    let last_message_at: Option<u64> = row.get("last_message_at")?;
    let last_message_id: Option<EventId> =
        last_message_id.and_then(|id| EventId::from_slice(id).ok());
    let last_message_at: Option<Timestamp> = last_message_at.map(Timestamp::from_secs);

    let group_type: &str = row.get_ref("group_type")?.as_str()?;
    let group_type: GroupType =
        GroupType::from_str(group_type).map_err(|_| map_invalid_text_data("Invalid group type"))?;

    // Convert group_type and state to GroupType and GroupState
    let state: &str = row.get_ref("state")?.as_str()?;
    let state: GroupState =
        GroupState::from_str(state).map_err(|_| map_invalid_text_data("Invalid group state"))?;

    let epoch: u64 = row.get("epoch")?;

    Ok(Group {
        mls_group_id,
        nostr_group_id,
        name,
        description,
        admin_pubkeys,
        last_message_id,
        last_message_at,
        group_type,
        epoch,
        state,
    })
}

/// Convert a row to a GroupRelay struct
pub fn row_to_group_relay(row: &Row) -> SqliteResult<GroupRelay> {
    let mls_group_id: Vec<u8> = row.get("mls_group_id")?;
    let relay_url: &str = row.get_ref("relay_url")?.as_str()?;

    // Parse relay URL
    let relay_url: RelayUrl =
        RelayUrl::parse(relay_url).map_err(|_| map_invalid_text_data("Invalid relay URL"))?;

    Ok(GroupRelay {
        mls_group_id,
        relay_url,
    })
}

/// Convert a row to a GroupExporterSecret struct
pub fn row_to_group_exporter_secret(row: &Row) -> SqliteResult<GroupExporterSecret> {
    let mls_group_id: Vec<u8> = row.get("mls_group_id")?;
    let epoch: u64 = row.get("epoch")?;
    let secret: Vec<u8> = row.get("secret")?;

    Ok(GroupExporterSecret {
        mls_group_id,
        epoch,
        secret,
    })
}

/// Convert a row to a Message struct
pub fn row_to_message(row: &Row) -> SqliteResult<Message> {
    let id_blob: &[u8] = row.get_ref("id")?.as_blob()?;
    let pubkey_blob: &[u8] = row.get_ref("pubkey")?.as_blob()?;
    let kind_value: u16 = row.get("kind")?;
    let mls_group_id: Vec<u8> = row.get("mls_group_id")?;
    let created_at_value: u64 = row.get("created_at")?;
    let content: String = row.get("content")?;
    let tags_json: &str = row.get_ref("tags")?.as_str()?;
    let event_json: &str = row.get_ref("event")?.as_str()?;
    let wrapper_event_id_blob: &[u8] = row.get_ref("wrapper_event_id")?.as_blob()?;

    // Parse values
    let id: EventId =
        EventId::from_slice(id_blob).map_err(|_| map_invalid_blob_data("Invalid event ID"))?;

    let pubkey: PublicKey = PublicKey::from_slice(pubkey_blob)
        .map_err(|_| map_invalid_blob_data("Invalid public key"))?;

    let kind: Kind = Kind::from(kind_value);
    let created_at: Timestamp = Timestamp::from(created_at_value);

    let tags: Tags = serde_json::from_str(tags_json).map_err(map_to_text_boxed_error)?;

    let event: UnsignedEvent =
        UnsignedEvent::from_json(event_json).map_err(map_to_text_boxed_error)?;

    let wrapper_event_id: EventId = EventId::from_slice(wrapper_event_id_blob)
        .map_err(|_| map_invalid_blob_data("Invalid wrapper event ID"))?;

    Ok(Message {
        id,
        pubkey,
        kind,
        mls_group_id,
        created_at,
        content,
        tags,
        event,
        wrapper_event_id,
    })
}

/// Convert a row to a ProcessedMessage struct
pub fn row_to_processed_message(row: &Row) -> SqliteResult<ProcessedMessage> {
    let wrapper_event_id_blob: &[u8] = row.get_ref("wrapper_event_id")?.as_blob()?;
    let message_event_id_blob: Option<&[u8]> =
        row.get_ref("message_event_id")?.as_blob_or_null()?;
    let processed_at_value: u64 = row.get("processed_at")?;
    let state_str: &str = row.get_ref("state")?.as_str()?;
    let failure_reason: Option<String> = row.get("failure_reason")?;

    // Parse values
    let wrapper_event_id: EventId = EventId::from_slice(wrapper_event_id_blob)
        .map_err(|_| map_invalid_blob_data("Invalid wrapper event ID"))?;

    let message_event_id: Option<EventId> = match message_event_id_blob {
        Some(id_blob) => Some(
            EventId::from_slice(id_blob)
                .map_err(|_| map_invalid_blob_data("Invalid message event ID"))?,
        ),
        None => None,
    };

    let processed_at: Timestamp = Timestamp::from_secs(processed_at_value);
    let state: ProcessedMessageState = ProcessedMessageState::from_str(state_str)
        .map_err(|_| map_invalid_text_data("Invalid state"))?;

    Ok(ProcessedMessage {
        wrapper_event_id,
        message_event_id,
        processed_at,
        state,
        failure_reason,
    })
}

/// Convert a row to a Welcome struct
pub fn row_to_welcome(row: &Row) -> SqliteResult<Welcome> {
    let id_blob: &[u8] = row.get_ref("id")?.as_blob()?;
    let event_json: &str = row.get_ref("event")?.as_str()?;
    let mls_group_id: Vec<u8> = row.get("mls_group_id")?;
    let nostr_group_id: String = row.get("nostr_group_id")?;
    let group_name: String = row.get("group_name")?;
    let group_description: String = row.get("group_description")?;
    let group_admin_pubkeys_json: &str = row.get_ref("group_admin_pubkeys")?.as_str()?;
    let group_relays_json: &str = row.get_ref("group_relays")?.as_str()?;
    let welcomer_blob: &[u8] = row.get_ref("welcomer")?.as_blob()?;
    let member_count: u64 = row.get("member_count")?;
    let state_str: &str = row.get_ref("state")?.as_str()?;
    let wrapper_event_id_blob: &[u8] = row.get_ref("wrapper_event_id")?.as_blob()?;

    // Parse values
    let id: EventId =
        EventId::from_slice(id_blob).map_err(|_| map_invalid_blob_data("Invalid event ID"))?;

    let event: UnsignedEvent =
        UnsignedEvent::from_json(event_json).map_err(map_to_text_boxed_error)?;

    let group_admin_pubkeys: BTreeSet<PublicKey> =
        serde_json::from_str(group_admin_pubkeys_json).map_err(map_to_text_boxed_error)?;

    let group_relays: BTreeSet<RelayUrl> =
        serde_json::from_str(group_relays_json).map_err(map_to_text_boxed_error)?;

    let welcomer: PublicKey = PublicKey::from_slice(welcomer_blob)
        .map_err(|_| map_invalid_blob_data("Invalid welcomer public key"))?;

    let wrapper_event_id: EventId = EventId::from_slice(wrapper_event_id_blob)
        .map_err(|_| map_invalid_blob_data("Invalid wrapper event ID"))?;

    let state: WelcomeState =
        WelcomeState::from_str(state_str).map_err(|_| map_invalid_text_data("Invalid state"))?;

    Ok(Welcome {
        id,
        event,
        mls_group_id,
        nostr_group_id,
        group_name,
        group_description,
        group_admin_pubkeys,
        group_relays,
        welcomer,
        member_count: member_count as u32,
        state,
        wrapper_event_id,
    })
}

/// Convert a row to a ProcessedWelcome struct
pub fn row_to_processed_welcome(row: &Row) -> SqliteResult<ProcessedWelcome> {
    let wrapper_event_id_blob: &[u8] = row.get_ref("wrapper_event_id")?.as_blob()?;
    let welcome_event_id_blob: Option<&[u8]> =
        row.get_ref("welcome_event_id")?.as_blob_or_null()?;
    let processed_at_value: u64 = row.get("processed_at")?;
    let state_str: &str = row.get_ref("state")?.as_str()?;
    let failure_reason: Option<String> = row.get("failure_reason")?;

    // Parse values
    let wrapper_event_id: EventId = EventId::from_slice(wrapper_event_id_blob)
        .map_err(|_| map_invalid_blob_data("Invalid wrapper event ID"))?;

    let welcome_event_id: Option<EventId> = match welcome_event_id_blob {
        Some(id_blob) => Some(
            EventId::from_slice(id_blob)
                .map_err(|_| map_invalid_blob_data("Invalid welcome event ID"))?,
        ),
        None => None,
    };

    let processed_at: Timestamp = Timestamp::from_secs(processed_at_value);
    let state: ProcessedWelcomeState = ProcessedWelcomeState::from_str(state_str)
        .map_err(|_| map_invalid_text_data("Invalid state"))?;

    Ok(ProcessedWelcome {
        wrapper_event_id,
        welcome_event_id,
        processed_at,
        state,
        failure_reason,
    })
}
