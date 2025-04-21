use std::str::FromStr;

/// Database utilities for SQLite storage.
use nostr::{EventId, Kind, PublicKey, RelayUrl, Tags, Timestamp, UnsignedEvent};
use nostr_mls_storage::groups::types::{Group, GroupRelay, GroupState, GroupType};
use nostr_mls_storage::messages::types::{Message, ProcessedMessage, ProcessedMessageState};
use nostr_mls_storage::welcomes::types::{
    ProcessedWelcome, ProcessedWelcomeState, Welcome, WelcomeState,
};
use rusqlite::{Result as SqliteResult, Row};

/// Convert a row to a Group struct
pub fn row_to_group(row: &Row) -> SqliteResult<Group> {
    let mls_group_id: Vec<u8> = row.get("mls_group_id")?;
    let nostr_group_id: String = row.get("nostr_group_id")?;
    let name: String = row.get("name")?;
    let description: String = row.get("description")?;

    // Parse admin pubkeys from JSON
    let admin_pubkeys_json: String = row.get("admin_pubkeys")?;
    let admin_pubkeys: Vec<String> = serde_json::from_str(&admin_pubkeys_json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;

    // Convert string pubkeys to PublicKey type
    let admin_pubkeys: Vec<PublicKey> = admin_pubkeys
        .iter()
        .filter_map(|pk| PublicKey::parse(pk).ok())
        .collect();

    let last_message_id: Option<Vec<u8>> = row.get("last_message_id")?;
    let last_message_at: Option<i64> = row.get("last_message_at")?;
    let group_type: String = row.get("group_type")?;
    let epoch: u64 = row.get::<_, i64>("epoch")? as u64;
    let state: String = row.get("state")?;

    let last_message_id = match last_message_id {
        Some(id) => EventId::from_slice(&id).ok(),
        None => None,
    };

    let last_message_at = last_message_at.map(|ts| Timestamp::from(ts as u64));

    // Convert group_type and state to GroupType and GroupState
    let group_type = GroupType::from_str(&group_type).map_err(|_| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid group type",
            )),
        )
    })?;
    let state = GroupState::from_str(&state).map_err(|_| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid group state",
            )),
        )
    })?;
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
    let relay_url: String = row.get("relay_url")?;

    // Parse relay URL
    let relay_url = RelayUrl::from_str(&relay_url).map_err(|_| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid relay URL",
            )),
        )
    })?;

    Ok(GroupRelay {
        mls_group_id,
        relay_url,
    })
}

/// Convert a row to a Message struct
pub fn row_to_message(row: &Row) -> SqliteResult<Message> {
    let id_blob: Vec<u8> = row.get("id")?;
    let pubkey_blob: Vec<u8> = row.get("pubkey")?;
    let kind_value: u16 = row.get("kind")?;
    let mls_group_id: Vec<u8> = row.get("mls_group_id")?;
    let created_at_value: u64 = row.get("created_at")?;
    let content: String = row.get("content")?;
    let tags_json: String = row.get("tags")?;
    let event_json: String = row.get("event")?;
    let wrapper_event_id_blob: Vec<u8> = row.get("wrapper_event_id")?;

    // Parse values
    let id = EventId::from_slice(&id_blob).map_err(|_| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Blob,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid event ID",
            )),
        )
    })?;

    let pubkey = PublicKey::from_slice(&pubkey_blob).map_err(|_| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Blob,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid public key",
            )),
        )
    })?;

    let kind = Kind::from(kind_value);
    let created_at = Timestamp::from(created_at_value);

    let tags: Tags = serde_json::from_str(&tags_json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;

    let event: UnsignedEvent = serde_json::from_str(&event_json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;

    let wrapper_event_id = EventId::from_slice(&wrapper_event_id_blob).map_err(|_| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Blob,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid wrapper event ID",
            )),
        )
    })?;

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
    let wrapper_event_id_blob: Vec<u8> = row.get("wrapper_event_id")?;
    let message_event_id_blob: Option<Vec<u8>> = row.get("message_event_id")?;
    let processed_at_value: i64 = row.get("processed_at")?;
    let state_str: String = row.get("state")?;
    let failure_reason: String = row.get("failure_reason")?;

    // Parse values
    let wrapper_event_id = EventId::from_slice(&wrapper_event_id_blob).map_err(|_| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Blob,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid wrapper event ID",
            )),
        )
    })?;

    let message_event_id = match message_event_id_blob {
        Some(id_blob) => Some(EventId::from_slice(&id_blob).map_err(|_| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Blob,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid message event ID",
                )),
            )
        })?),
        None => None,
    };

    let processed_at = Timestamp::from(processed_at_value as u64);
    let state = ProcessedMessageState::from_str(&state_str).map_err(|_| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid state",
            )),
        )
    })?;

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
    let id_blob: Vec<u8> = row.get("id")?;
    let event_json: String = row.get("event")?;
    let mls_group_id: Vec<u8> = row.get("mls_group_id")?;
    let nostr_group_id: String = row.get("nostr_group_id")?;
    let group_name: String = row.get("group_name")?;
    let group_description: String = row.get("group_description")?;
    let group_admin_pubkeys_json: String = row.get("group_admin_pubkeys")?;
    let group_relays_json: String = row.get("group_relays")?;
    let welcomer_blob: Vec<u8> = row.get("welcomer")?;
    let member_count: i64 = row.get("member_count")?;
    let state_str: String = row.get("state")?;
    let wrapper_event_id_blob: Vec<u8> = row.get("wrapper_event_id")?;

    // Parse values
    let id = EventId::from_slice(&id_blob).map_err(|_| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Blob,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid event ID",
            )),
        )
    })?;

    let event: UnsignedEvent = serde_json::from_str(&event_json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;

    let group_admin_pubkeys: Vec<String> = serde_json::from_str(&group_admin_pubkeys_json)
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?;

    let group_relays: Vec<String> = serde_json::from_str(&group_relays_json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;

    let welcomer = PublicKey::from_slice(&welcomer_blob).map_err(|_| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Blob,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid welcomer public key",
            )),
        )
    })?;

    let wrapper_event_id = EventId::from_slice(&wrapper_event_id_blob).map_err(|_| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Blob,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid wrapper event ID",
            )),
        )
    })?;

    let state = WelcomeState::from_str(&state_str).map_err(|_| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid state",
            )),
        )
    })?;

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
    let wrapper_event_id_blob: Vec<u8> = row.get("wrapper_event_id")?;
    let welcome_event_id_blob: Option<Vec<u8>> = row.get("welcome_event_id")?;
    let processed_at_value: i64 = row.get("processed_at")?;
    let state_str: String = row.get("state")?;
    let failure_reason: String = row.get("failure_reason")?;

    // Parse values
    let wrapper_event_id = EventId::from_slice(&wrapper_event_id_blob).map_err(|_| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Blob,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid wrapper event ID",
            )),
        )
    })?;

    let welcome_event_id = match welcome_event_id_blob {
        Some(id_blob) => Some(EventId::from_slice(&id_blob).map_err(|_| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Blob,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid welcome event ID",
                )),
            )
        })?),
        None => None,
    };

    let processed_at = Timestamp::from(processed_at_value as u64);
    let state = ProcessedWelcomeState::from_str(&state_str).map_err(|_| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid state",
            )),
        )
    })?;

    Ok(ProcessedWelcome {
        wrapper_event_id,
        welcome_event_id,
        processed_at,
        state,
        failure_reason,
    })
}
