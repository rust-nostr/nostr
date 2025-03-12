use openmls::prelude::*;
use thiserror::Error;
use tls_codec::Deserialize as TlsDeserialize;

use crate::nostr_group_data_extension::NostrGroupDataExtension;
use crate::NostrMls;

#[derive(Debug, Error, Eq, PartialEq, Clone)]
pub enum WelcomeError {
    #[error("Error creating the welcome: {0}")]
    CreateWelcomeError(String),

    #[error("Error parsing the welcome: {0}")]
    ParseWelcomeError(String),

    #[error("Error processing the welcome: {0}")]
    ProcessWelcomeError(String),

    #[error("Error joining the group: {0}")]
    JoinGroupError(String),

    #[error("Error deserializing the welcome: {0}")]
    DeserializeWelcomeError(String),
}

#[derive(Debug)]
pub struct WelcomePreview {
    pub staged_welcome: StagedWelcome,
    pub nostr_group_data: NostrGroupDataExtension,
}

#[derive(Debug)]
pub struct JoinedGroupResult {
    pub mls_group: MlsGroup,
    pub nostr_group_data: NostrGroupDataExtension,
}

/// Parses a welcome message and extracts group information.
///
/// This function takes a serialized welcome message and processes it to extract both the staged welcome
/// and the Nostr-specific group data. This is a lower-level function used by both `preview_welcome_event`
/// and `join_group_from_welcome`.
///
/// # Arguments
///
/// * `nostr_mls` - The NostrMls instance containing MLS configuration and provider
/// * `welcome_message` - The serialized welcome message as a byte vector
///
/// # Returns
///
/// A tuple containing:
/// - The `StagedWelcome` which can be used to join the group
/// - The `NostrGroupDataExtension` containing Nostr-specific group metadata
///
/// # Errors
///
/// Returns a `WelcomeError` if:
/// - The welcome message cannot be deserialized
/// - The message is not a valid welcome message
/// - The welcome message cannot be processed
/// - The group data extension cannot be extracted
pub fn parse_welcome_message(
    nostr_mls: &NostrMls,
    welcome_message: Vec<u8>,
) -> Result<(StagedWelcome, NostrGroupDataExtension), WelcomeError> {
    let welcome_message_in = MlsMessageIn::tls_deserialize(&mut welcome_message.as_slice())
        .map_err(|e| WelcomeError::DeserializeWelcomeError(e.to_string()))?;
    let welcome = match welcome_message_in.extract() {
        MlsMessageBodyIn::Welcome(welcome) => welcome,
        _ => {
            return Err(WelcomeError::ParseWelcomeError(
                "Invalid welcome message".to_string(),
            ))
        }
    };

    let mls_group_config = MlsGroupJoinConfig::builder()
        .use_ratchet_tree_extension(true)
        .build();

    let staged_welcome =
        StagedWelcome::new_from_welcome(&nostr_mls.provider, &mls_group_config, welcome, None)
            .map_err(|e| WelcomeError::ProcessWelcomeError(e.to_string()))?;

    let nostr_group_data =
        NostrGroupDataExtension::from_group_context(staged_welcome.group_context())
            .map_err(|e| WelcomeError::ProcessWelcomeError(e.to_string()))?;

    Ok((staged_welcome, nostr_group_data))
}

/// Previews a welcome message without joining the group.
///
/// This function parses and validates a welcome message, returning information about the group
/// that can be used to decide whether to join it. Unlike `join_group_from_welcome`, this does
/// not actually join the group.
///
/// # Arguments
///
/// * `nostr_mls` - The NostrMls instance containing MLS configuration and provider
/// * `welcome_message` - The serialized welcome message as a byte vector
///
/// # Returns
///
/// A `WelcomePreview` containing the staged welcome and group data on success,
/// or a `WelcomeError` on failure.
///
/// # Errors
///
/// Returns a `WelcomeError` if:
/// - The welcome message cannot be parsed
/// - The welcome message is invalid
pub fn preview_welcome_event(
    nostr_mls: &NostrMls,
    welcome_message: Vec<u8>,
) -> Result<WelcomePreview, WelcomeError> {
    let (staged_welcome, nostr_group_data) = parse_welcome_message(nostr_mls, welcome_message)?;

    Ok(WelcomePreview {
        staged_welcome,
        nostr_group_data,
    })
}

/// Joins an MLS group using a welcome message.
///
/// This function processes a welcome message and joins the corresponding MLS group. It first parses and validates
/// the welcome message, then uses it to create a new MLS group instance that the user can participate in.
///
/// # Arguments
///
/// * `nostr_mls` - The NostrMls instance containing MLS configuration and provider
/// * `welcome_message` - The serialized welcome message as a byte vector
///
/// # Returns
///
/// A `JoinedGroupResult` containing the joined MLS group and associated group data on success,
/// or a `WelcomeError` on failure.
///
/// # Errors
///
/// Returns a `WelcomeError` if:
/// - The welcome message cannot be parsed
/// - The group cannot be joined with the provided welcome message
pub fn join_group_from_welcome(
    nostr_mls: &NostrMls,
    welcome_message: Vec<u8>,
) -> Result<JoinedGroupResult, WelcomeError> {
    let (staged_welcome, nostr_group_data) = parse_welcome_message(nostr_mls, welcome_message)?;

    let mls_group = staged_welcome
        .into_group(&nostr_mls.provider)
        .map_err(|e| WelcomeError::JoinGroupError(format!("Error joining group: {:?}", e)))?;

    Ok(JoinedGroupResult {
        mls_group,
        nostr_group_data,
    })
}
