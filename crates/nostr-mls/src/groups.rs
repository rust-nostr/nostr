use openmls::prelude::*;
use openmls_basic_credential::SignatureKeyPair;
use thiserror::Error;
use tls_codec::{Deserialize as TlsDeserialize, Serialize as TlsSerialize};

use crate::key_packages::generate_credential_with_key;
use crate::nostr_group_data_extension::NostrGroupDataExtension;
use crate::NostrMls;

#[derive(Debug, Error, Eq, PartialEq, Clone)]
pub enum GroupError {
    #[error("Error creating the group: {0}")]
    CreateGroupError(String),

    #[error("Error loading MLS group from storage: {0}")]
    LoadGroupError(String),

    #[error("Error creating message for group: {0}")]
    CreateMessageError(String),

    #[error("Error serializing message for group: {0}")]
    SerializeMessageError(String),

    #[error("Error exporting group secret: {0}")]
    ExportSecretError(String),

    #[error("Error processing message for group: {0}")]
    ProcessMessageError(String),

    #[error("Error with member identity: {0}")]
    MemberIdentityError(String),

    #[error("Error with signature keypair: {0}")]
    SignatureKeypairError(String),

    #[error("Error with self update: {0}")]
    SelfUpdateError(String),
}

#[derive(Debug)]
pub struct CreateGroupResult {
    pub mls_group: MlsGroup,
    pub serialized_welcome_message: Vec<u8>,
    pub nostr_group_data: NostrGroupDataExtension,
}

#[derive(Debug)]
pub struct SelfUpdateResult {
    pub serialized_message: Vec<u8>,
    pub current_exporter_secret_hex: String,
    pub new_exporter_secret_hex: String,
    pub new_epoch: u64,
}

/// Creates a new MLS group with the specified members and settings.
///
/// This function creates a new MLS group with the given name, description, members, and administrators.
/// It generates the necessary cryptographic credentials, configures the group with Nostr-specific extensions,
/// and adds the specified members.
///
/// # Arguments
///
/// * `nostr_mls` - The NostrMls instance containing MLS configuration and provider
/// * `name` - The name of the group
/// * `description` - A description of the group
/// * `member_key_packages` - A vector of KeyPackages for the initial group members
/// * `admin_pubkeys_hex` - A vector of hex-encoded Nostr public keys for group administrators
/// * `creator_pubkey_hex` - The hex-encoded Nostr public key of the group creator
/// * `group_relays` - A vector of relay URLs where group messages will be published
///
/// # Returns
///
/// A `CreateGroupResult` containing:
/// - The created MLS group
/// - A serialized welcome message for the initial members
/// - The Nostr-specific group data
///
/// # Errors
///
/// Returns a `GroupError` if:
/// - Credential generation fails
/// - Group creation fails
/// - Adding members fails
/// - Message serialization fails
pub fn create_mls_group(
    nostr_mls: &NostrMls,
    name: String,
    description: String,
    member_key_packages: Vec<KeyPackage>,
    admin_pubkeys_hex: Vec<String>,
    creator_pubkey_hex: String,
    group_relays: Vec<String>,
) -> Result<CreateGroupResult, GroupError> {
    let capabilities = nostr_mls.default_capabilities();

    let (credential, signer) = generate_credential_with_key(creator_pubkey_hex.clone(), nostr_mls)
        .map_err(|e| GroupError::CreateGroupError(e.to_string()))?;

    tracing::debug!(
        target: "nostr_mls::groups::create_mls_group",
        "Credential and signer created, {:?}",
        credential
    );

    let group_data =
        NostrGroupDataExtension::new(name, description, admin_pubkeys_hex, group_relays);

    tracing::debug!(
        target: "nostr_mls::groups::create_mls_group",
        "Group data created, {:?}",
        group_data
    );

    let serialized_group_data = group_data
        .tls_serialize_detached()
        .expect("Failed to serialize group data");

    let extensions = vec![Extension::Unknown(
        group_data.extension_type(),
        UnknownExtension(serialized_group_data),
    )];

    tracing::debug!(
        target: "nostr_mls::groups::create_mls_group",
        "Group config extensions created, {:?}",
        extensions
    );

    // Build the group config
    let group_config = MlsGroupCreateConfig::builder()
        .ciphersuite(nostr_mls.ciphersuite)
        .use_ratchet_tree_extension(true)
        .capabilities(capabilities)
        .with_group_context_extensions(
            Extensions::from_vec(extensions).expect("Couldn't convert extensions vec to Object"),
        )
        .map_err(|e| GroupError::CreateGroupError(e.to_string()))?
        .build();

    tracing::debug!(
        target: "nostr_mls::groups::create_mls_group",
        "Group config built, {:?}",
        group_config
    );

    let mut group = MlsGroup::new(
        &nostr_mls.provider,
        &signer,
        &group_config,
        credential.clone(),
    )
    .map_err(|e| GroupError::CreateGroupError(e.to_string()))?;

    // Add members to the group
    let (_, welcome_out, _group_info) = group
        .add_members(&nostr_mls.provider, &signer, member_key_packages.as_slice())
        .map_err(|e| GroupError::CreateGroupError(e.to_string()))?;

    // Merge the pending commit adding the memebers
    group
        .merge_pending_commit(&nostr_mls.provider)
        .map_err(|e| GroupError::CreateGroupError(e.to_string()))?;

    // Serialize the welcome message and send it to the members
    let serialized_welcome_message = welcome_out
        .tls_serialize_detached()
        .map_err(|e| GroupError::CreateGroupError(e.to_string()))?;

    Ok(CreateGroupResult {
        mls_group: group,
        serialized_welcome_message,
        nostr_group_data: group_data,
    })
}

/// Creates an encrypted message for an MLS group
///
/// This function loads the specified MLS group, retrieves the necessary signing keys,
/// and creates an encrypted message that can only be decrypted by other group members.
///
/// # Arguments
///
/// * `nostr_mls` - The NostrMls instance containing MLS configuration and provider
/// * `mls_group_id` - The ID of the MLS group to create the message for
/// * `message` - The message content to encrypt
///
/// # Returns
///
/// A serialized encrypted MLS message as a byte vector on success, or a GroupError on failure.
///
/// # Errors
///
/// Returns a GroupError if:
/// - The group cannot be loaded from storage
/// - The signing keys cannot be loaded
/// - Message creation fails
/// - Message serialization fails
pub fn create_message_for_group(
    nostr_mls: &NostrMls,
    mls_group_id: Vec<u8>,
    message: String,
) -> Result<Vec<u8>, GroupError> {
    let mut group = MlsGroup::load(
        nostr_mls.provider.storage(),
        &GroupId::from_slice(&mls_group_id),
    )
    .map_err(|e| GroupError::LoadGroupError(e.to_string()))?
    .ok_or_else(|| GroupError::LoadGroupError("Group not found".to_string()))?;

    let signer = SignatureKeyPair::read(
        nostr_mls.provider.storage(),
        group.own_leaf().unwrap().signature_key().clone().as_slice(),
        group.ciphersuite().signature_algorithm(),
    )
    .ok_or_else(|| GroupError::LoadGroupError("Failed to load signer".to_string()))?;

    let message_out = group
        .create_message(&nostr_mls.provider, &signer, message.as_bytes())
        .map_err(|e| GroupError::CreateMessageError(e.to_string()))?;

    let serialized_message = message_out
        .tls_serialize_detached()
        .map_err(|e| GroupError::SerializeMessageError(e.to_string()))?;

    Ok(serialized_message)
}

/// Exports a secret key from the MLS group as a hex-encoded string.
/// This secret is used for NIP-44 encrypting the content field of Group Message Events (kind:445)
///
/// # Arguments
/// * `nostr_mls` - The NostrMls instance containing the provider and storage
/// * `mls_group_id` - The ID of the MLS group to export the secret from
///
/// # Returns
/// * `Ok(String)` - The hex-encoded secret key if successful
/// * `Err(GroupError)` - If there was an error loading the group or exporting the secret
pub fn export_secret_as_hex_secret_key_and_epoch(
    nostr_mls: &NostrMls,
    mls_group_id: Vec<u8>,
) -> Result<(String, u64), GroupError> {
    let group = MlsGroup::load(
        nostr_mls.provider.storage(),
        &GroupId::from_slice(&mls_group_id),
    )
    .map_err(|e| GroupError::LoadGroupError(e.to_string()))?
    .ok_or_else(|| GroupError::LoadGroupError("Group not found".to_string()))?;

    let export_secret = group
        .export_secret(&nostr_mls.provider, "nostr", b"nostr", 32)
        .map_err(|e| GroupError::ExportSecretError(e.to_string()))?;

    Ok((hex::encode(&export_secret), group.epoch().as_u64()))
}

/// Processes an incoming MLS message for a group.
///
/// This function loads the specified MLS group, processes the incoming message according to the MLS protocol,
/// and handles the resulting processed message content appropriately.
///
/// # Arguments
///
/// * `nostr_mls` - The NostrMls instance containing MLS configuration and provider
/// * `mls_group_id` - The ID of the MLS group as a byte vector
/// * `message` - The serialized MLS message to process
///
/// # Returns
///
/// A Result containing:
/// - For application messages: The decrypted message bytes
/// - For other message types (proposals, commits, etc): An empty vector
///
/// # Errors
///
/// Returns a GroupError if:
/// - The group cannot be loaded from storage
/// - The specified group is not found
/// - The message cannot be deserialized
/// - The message's group ID doesn't match the loaded group
/// - There is an error processing the message
pub fn process_message_for_group(
    nostr_mls: &NostrMls,
    mls_group_id: Vec<u8>,
    message: Vec<u8>,
) -> Result<Vec<u8>, GroupError> {
    let mut group = MlsGroup::load(
        nostr_mls.provider.storage(),
        &GroupId::from_slice(&mls_group_id),
    )
    .map_err(|e| GroupError::LoadGroupError(e.to_string()))?
    .ok_or_else(|| GroupError::LoadGroupError("Group not found".to_string()))?;

    let mls_message = MlsMessageIn::tls_deserialize_exact(message.as_slice())
        .map_err(|e| GroupError::ProcessMessageError(e.to_string()))?;

    tracing::debug!(target: "nostr_openmls::groups::process_message_for_group", "Received message: {:?}", mls_message);
    let protocol_message = mls_message
        .try_into_protocol_message()
        .map_err(|e| GroupError::ProcessMessageError(e.to_string()))?;

    match protocol_message.group_id() == group.group_id() {
        true => {
            if protocol_message.content_type() == ContentType::Commit {
                tracing::debug!(target: "nostr_openmls::groups::process_message_for_group", "ABOUT TO PROCESS COMMIT MESSAGE");
            }
            let processed_message = group
                .process_message(&nostr_mls.provider, protocol_message)
                .map_err(|e| GroupError::ProcessMessageError(e.to_string()))?;

            tracing::debug!(
                target: "nostr_openmls::groups::process_message_for_group",
                "Processed message: {:?}",
                processed_message
            );
            // Handle the processed message based on its type
            match processed_message.into_content() {
                ProcessedMessageContent::ApplicationMessage(application_message) => {
                    // This is a message from a group member
                    Ok(application_message.into_bytes())
                }
                ProcessedMessageContent::ProposalMessage(staged_proposal) => {
                    // This is a proposal message
                    tracing::debug!(target: "nostr_openmls::groups::process_message_for_group", "Received proposal message: {:?}", staged_proposal);
                    // TODO: Handle proposal message
                    Ok(vec![])
                }
                ProcessedMessageContent::StagedCommitMessage(staged_commit) => {
                    // This is a commit message
                    tracing::debug!(target: "nostr_openmls::groups::process_message_for_group", "Received commit message: {:?}", staged_commit);
                    // TODO: Handle commit message
                    Ok(vec![])
                }
                ProcessedMessageContent::ExternalJoinProposalMessage(external_join_proposal) => {
                    tracing::debug!(target: "nostr_openmls::groups::process_message_for_group", "Received external join proposal message: {:?}", external_join_proposal);
                    // TODO: Handle external join proposal
                    Ok(vec![])
                }
            }
        }
        false => {
            tracing::error!(target: "nostr_openmls::groups::process_message_for_group", "ProtocolMessage GroupId doesn't match MlsGroup GroupId. Not processing event");
            Err(GroupError::ProcessMessageError(
                "ProtocolMessage GroupId doesn't match MlsGroup GroupId. Not processing event"
                    .to_string(),
            ))
        }
    }
}

/// Returns a list of Nostr hex-encoded public keys for all members in an MLS group.
///
/// This function loads the specified MLS group and extracts the Nostr public keys
/// of all current group members from their credentials.
///
/// # Arguments
///
/// * `nostr_mls` - The NostrMls instance containing MLS configuration and provider
/// * `mls_group_id` - The ID of the MLS group as a byte vector
///
/// # Returns
///
/// A Result containing a vector of hex-encoded Nostr public keys for all group members.
///
/// # Errors
///
/// Returns a GroupError if:
/// - The group cannot be loaded from storage
/// - The specified group is not found
/// - A member's credential cannot be parsed
/// - A member's identity bytes cannot be converted to a string
pub fn member_pubkeys(
    nostr_mls: &NostrMls,
    mls_group_id: Vec<u8>,
) -> Result<Vec<String>, GroupError> {
    let group = MlsGroup::load(
        nostr_mls.provider.storage(),
        &GroupId::from_slice(&mls_group_id),
    )
    .map_err(|e| GroupError::LoadGroupError(e.to_string()))?
    .ok_or_else(|| GroupError::LoadGroupError("Group not found".to_string()))?;

    // Store members in a variable to extend its lifetime
    let mut members = group.members();
    members.try_fold(Vec::new(), |mut acc, m| {
        let pubkey = String::from_utf8(
            BasicCredential::try_from(m.credential)
                .map_err(|e| GroupError::MemberIdentityError(e.to_string()))?
                .identity()
                .to_vec(),
        )
        .map_err(|e| GroupError::MemberIdentityError(e.to_string()))?;
        acc.push(pubkey);
        Ok(acc)
    })
}

/// Updates the current member's leaf node in an MLS group.
/// Does not currently support updating any group attributes.
///
/// This function performs a self-update operation in the specified MLS group by:
/// 1. Loading the group from storage
/// 2. Generating a new signature keypair
/// 3. Storing the keypair
/// 4. Creating and applying a self-update proposal
///
/// # Arguments
///
/// * `nostr_mls` - The NostrMls instance containing MLS configuration and provider
/// * `mls_group_id` - The ID of the MLS group as a byte vector
///
/// # Returns
///
/// A Result containing a tuple of:
/// - MlsMessageOut: The self-update message to be sent to the group
/// - Option<MlsMessageOut>: Optional welcome message if new members are added
/// - Option<GroupInfo>: Optional updated group info
///
/// # Errors
///
/// Returns a GroupError if:
/// - The group cannot be loaded from storage
/// - The specified group is not found
/// - Failed to generate or store signature keypair
/// - Failed to perform self-update operation
pub fn self_update(
    nostr_mls: &NostrMls,
    mls_group_id: Vec<u8>,
) -> Result<SelfUpdateResult, GroupError> {
    let mut group = MlsGroup::load(
        nostr_mls.provider.storage(),
        &GroupId::from_slice(&mls_group_id),
    )
    .map_err(|e| GroupError::LoadGroupError(e.to_string()))?
    .ok_or_else(|| GroupError::LoadGroupError("Group not found".to_string()))?;

    let (current_exporter_secret_hex, current_epoch) =
        nostr_mls.export_secret_as_hex_secret_key_and_epoch(mls_group_id.clone())?;
    tracing::debug!(target: "nostr_openmls::groups::self_update", "Current epoch: {:?}", current_epoch);

    let current_signature_keypair = SignatureKeyPair::read(
        nostr_mls.provider.storage(),
        group.own_leaf().unwrap().signature_key().clone().as_slice(),
        group.ciphersuite().signature_algorithm(),
    )
    .unwrap();

    let new_signature_keypair = SignatureKeyPair::new(nostr_mls.ciphersuite.signature_algorithm())
        .map_err(|e| GroupError::SignatureKeypairError(e.to_string()))?;

    new_signature_keypair
        .store(nostr_mls.provider.storage())
        .map_err(|e| GroupError::SignatureKeypairError(e.to_string()))?;

    let pubkey = BasicCredential::try_from(group.own_leaf().unwrap().credential().clone())
        .map_err(|e| GroupError::MemberIdentityError(e.to_string()))?
        .identity()
        .to_vec();

    let new_credential: BasicCredential = BasicCredential::new(pubkey);
    let new_credential_with_key = CredentialWithKey {
        credential: new_credential.into(),
        signature_key: new_signature_keypair.public().into(),
    };

    let leaf_node_params = LeafNodeParameters::builder()
        .with_credential_with_key(new_credential_with_key)
        .with_capabilities(group.own_leaf().unwrap().capabilities().clone())
        .with_extensions(group.own_leaf().unwrap().extensions().clone())
        .build();

    let (mls_message, _welcome, _group_info) = group
        .self_update(
            &nostr_mls.provider,
            &current_signature_keypair,
            leaf_node_params,
        )
        .map_err(|e| GroupError::SelfUpdateError(e.to_string()))?;

    // Merge the commit
    group
        .merge_pending_commit(&nostr_mls.provider)
        .map_err(|e| GroupError::SelfUpdateError(e.to_string()))?;

    // Export the new epoch's exporter secret
    let (new_exporter_secret_hex, new_epoch) =
        nostr_mls.export_secret_as_hex_secret_key_and_epoch(mls_group_id)?;

    tracing::debug!(target: "nostr_openmls::groups::self_update", "New epoch: {:?}", new_epoch);

    // Serialize the message
    let serialized_message = mls_message
        .tls_serialize_detached()
        .map_err(|e| GroupError::SerializeMessageError(e.to_string()))?;

    Ok(SelfUpdateResult {
        serialized_message,
        current_exporter_secret_hex,
        new_exporter_secret_hex,
        new_epoch,
    })
}

// TODO: Create proposal
// TODO: Send commit
