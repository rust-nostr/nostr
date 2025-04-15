//! Nostr MLS Groups

use std::str;

use nostr::nips::nip44;
use nostr::util::hex;
use nostr::{
    Event, EventBuilder, JsonUtil, Keys, Kind, PublicKey, RelayUrl, SecretKey, Tag, TagKind,
    UnsignedEvent,
};
use nostr_mls_storage::NostrMlsStorageProvider;
use openmls::group::GroupId;
use openmls::prelude::*;
use openmls_basic_credential::SignatureKeyPair;
use tls_codec::{Deserialize as TlsDeserialize, Serialize as TlsSerialize};

use super::extension::NostrGroupDataExtension;
use super::NostrMls;
use crate::error::Error;

#[allow(missing_docs)]
#[derive(Debug)]
pub struct CreateGroupResult {
    pub mls_group: MlsGroup,
    pub serialized_welcome_message: Vec<u8>,
    pub nostr_group_data: NostrGroupDataExtension,
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct SelfUpdate {
    pub serialized_message: Vec<u8>,
    pub current_secret: ExportSecret,
    pub new_secret: ExportSecret,
}

/// Export secret
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportSecret {
    /// Secret key
    pub secret_key: SecretKey,
    /// Epoch
    pub epoch: u64,
}

/// Create message result
pub struct CreateMessage {
    /// The event to publish to group relays
    pub event: Event,
    /// The secret for the current epoch
    pub secret: ExportSecret,
}

impl<Storage> NostrMls<Storage>
where
    Storage: NostrMlsStorageProvider,
{
    #[inline]
    fn get_own_leaf<'a>(&self, group: &'a MlsGroup) -> Result<&'a LeafNode, Error> {
        group.own_leaf().ok_or(Error::OwnLeafNotFound)
    }

    fn load_signer(&self, group: &MlsGroup) -> Result<SignatureKeyPair, Error> {
        let own_leaf: &LeafNode = self.get_own_leaf(group)?;
        let public_key: &[u8] = own_leaf.signature_key().as_slice();

        SignatureKeyPair::read(
            self.provider.storage(),
            public_key,
            group.ciphersuite().signature_algorithm(),
        )
        .ok_or(Error::CantLoadSigner)
    }

    /// Load group by ID
    pub fn load_group(&self, group_id: &GroupId) -> Result<Option<MlsGroup>, Error> {
        MlsGroup::load(self.provider.storage(), group_id)
            .map_err(|e| Error::Provider(e.to_string()))
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
    /// Returns a `Error` if:
    /// - Credential generation fails
    /// - Group creation fails
    /// - Adding members fails
    /// - Message serialization fails
    pub fn create_group<S1, S2>(
        &self,
        name: S1,
        description: S2,
        creator_public_key: &PublicKey,
        member_key_packages: Vec<KeyPackage>,
        admins: Vec<PublicKey>,
        group_relays: Vec<RelayUrl>,
    ) -> Result<CreateGroupResult, Error>
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let (credential, signer) = self.generate_credential_with_key(creator_public_key)?;

        tracing::debug!(
            target: "nostr_mls::groups::create_mls_group",
            "Credential and signer created, {:?}",
            credential
        );

        let group_data = NostrGroupDataExtension::new(name, description, admins, group_relays);

        tracing::debug!(
            target: "nostr_mls::groups::create_mls_group",
            "Group data created, {:?}",
            group_data
        );

        let serialized_group_data = group_data
            .as_raw()
            .tls_serialize_detached()
            .expect("Failed to serialize group data");

        let extensions = vec![Extension::Unknown(
            group_data.extension_type(),
            UnknownExtension(serialized_group_data),
        )];
        let extensions =
            Extensions::from_vec(extensions).expect("Couldn't convert extensions vec to Object");

        tracing::debug!(
            target: "nostr_mls::groups::create_mls_group",
            "Group config extensions created, {:?}",
            extensions
        );

        // Build the group config
        let capabilities = self.capabilities();
        let group_config = MlsGroupCreateConfig::builder()
            .ciphersuite(self.ciphersuite)
            .use_ratchet_tree_extension(true)
            .capabilities(capabilities)
            .with_group_context_extensions(extensions)?
            .build();

        tracing::debug!(
            target: "nostr_mls::groups::create_mls_group",
            "Group config built, {:?}",
            group_config
        );

        let mut group = MlsGroup::new(&self.provider, &signer, &group_config, credential.clone())?;

        // Add members to the group
        let (_, welcome_out, _group_info) =
            group.add_members(&self.provider, &signer, member_key_packages.as_slice())?;

        // Merge the pending commit adding the memebers
        group.merge_pending_commit(&self.provider)?;

        // Serialize the welcome message and send it to the members
        let serialized_welcome_message = welcome_out.tls_serialize_detached()?;

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
    /// A serialized encrypted MLS message as a byte vector on success, or a Error on failure.
    ///
    /// # Errors
    ///
    /// Returns a Error if:
    /// - The group cannot be loaded from storage
    /// - The signing keys cannot be loaded
    /// - Message creation fails
    /// - Message serialization fails
    pub fn create_message_for_event(
        &self,
        group: &mut MlsGroup,
        mut rumor: UnsignedEvent,
    ) -> Result<Vec<u8>, Error> {
        // Load signer
        let signer: SignatureKeyPair = self.load_signer(group)?;

        // Ensure rumor ID and serialize as JSON
        rumor.ensure_id();
        let json: String = rumor.as_json();

        // Create message
        let message_out = group.create_message(&self.provider, &signer, json.as_bytes())?;

        let serialized_message = message_out.tls_serialize_detached()?;

        Ok(serialized_message)
    }

    /// Create message [`Event`]
    pub fn create_message(
        &self,
        group_id: &GroupId,
        nostr_group_id: [u8; 32],
        rumor: UnsignedEvent,
    ) -> Result<CreateMessage, Error> {
        // Load group
        let mut group = self.load_group(group_id)?.ok_or(Error::GroupNotFound)?;

        // Create message
        let message: Vec<u8> = self.create_message_for_event(&mut group, rumor)?;

        // Export secret
        let secret: ExportSecret = self.export_secret(group_id)?;

        // Convert that secret to nostr keys
        let export_nostr_keys: Keys = Keys::new(secret.secret_key.clone());

        // Encrypt the message content
        let encrypted_content: String = nip44::encrypt(
            export_nostr_keys.secret_key(),
            &export_nostr_keys.public_key,
            &message,
            nip44::Version::default(),
        )?;

        // Generate ephemeral key
        let ephemeral_nostr_keys: Keys = Keys::generate();

        let tag: Tag = Tag::custom(TagKind::h(), [hex::encode(nostr_group_id)]);
        let event = EventBuilder::new(Kind::MlsGroupMessage, encrypted_content)
            .tag(tag)
            .sign_with_keys(&ephemeral_nostr_keys)?;

        Ok(CreateMessage { event, secret })
    }

    /// Exports a secret key from the MLS group for the **current** epoch.
    ///
    /// This secret is used for NIP-44 encrypting the content field of Group Message Events (kind:445).
    ///
    /// In real usage you would want to do this once per epoch, per group, and cache it.
    pub fn export_secret(&self, group_id: &GroupId) -> Result<ExportSecret, Error> {
        let group = self.load_group(group_id)?.ok_or(Error::GroupNotFound)?;

        let export_secret: Vec<u8> = group.export_secret(&self.provider, "nostr", b"nostr", 32)?;

        Ok(ExportSecret {
            secret_key: SecretKey::from_slice(&export_secret)?,
            epoch: group.epoch().as_u64(),
        })
    }

    /// Processes an incoming MLS message for a group.
    ///
    /// This function loads the specified MLS group, processes the incoming message according to the MLS protocol,
    /// and handles the resulting processed message content appropriately.
    ///
    /// # Arguments
    ///
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
    /// Returns a Error if:
    /// - The group cannot be loaded from storage
    /// - The specified group is not found
    /// - The message cannot be deserialized
    /// - The message's group ID doesn't match the loaded group
    /// - There is an error processing the message
    pub fn process_message_for_group(
        &self,
        group: &mut MlsGroup,
        message: &[u8],
        // TODO: return another enum instead of Option
    ) -> Result<Option<UnsignedEvent>, Error> {
        let mls_message = MlsMessageIn::tls_deserialize_exact(message)?;

        tracing::debug!(target: "nostr_openmls::groups::process_message_for_group", "Received message: {:?}", mls_message);
        let protocol_message = mls_message.try_into_protocol_message()?;

        // Return error if group ID doesn't match
        if protocol_message.group_id() != group.group_id() {
            return Err(Error::ProtocolGroupIdMismatch);
        }

        if protocol_message.content_type() == ContentType::Commit {
            tracing::debug!(target: "nostr_openmls::groups::process_message_for_group", "ABOUT TO PROCESS COMMIT MESSAGE");
        }

        let processed_message = group.process_message(&self.provider, protocol_message)?;

        tracing::debug!(
            target: "nostr_openmls::groups::process_message_for_group",
            "Processed message: {:?}",
            processed_message
        );
        // Handle the processed message based on its type
        match processed_message.into_content() {
            ProcessedMessageContent::ApplicationMessage(application_message) => {
                // This is a message from a group member
                let bytes = application_message.into_bytes();
                let rumor: UnsignedEvent = UnsignedEvent::from_json(bytes)?;
                Ok(Some(rumor))
            }
            ProcessedMessageContent::ProposalMessage(staged_proposal) => {
                // This is a proposal message
                tracing::debug!(target: "nostr_openmls::groups::process_message_for_group", "Received proposal message: {:?}", staged_proposal);
                // TODO: Handle proposal message
                Ok(None)
            }
            ProcessedMessageContent::StagedCommitMessage(staged_commit) => {
                // This is a commit message
                tracing::debug!(target: "nostr_openmls::groups::process_message_for_group", "Received commit message: {:?}", staged_commit);
                // TODO: Handle commit message
                Ok(None)
            }
            ProcessedMessageContent::ExternalJoinProposalMessage(external_join_proposal) => {
                tracing::debug!(target: "nostr_openmls::groups::process_message_for_group", "Received external join proposal message: {:?}", external_join_proposal);
                // TODO: Handle external join proposal
                Ok(None)
            }
        }
    }

    /// Process message
    pub fn process_message(
        &self,
        group_id: &GroupId,
        secret: ExportSecret,
        event: &Event,
    ) -> Result<Option<UnsignedEvent>, Error> {
        if event.kind != Kind::MlsGroupMessage {
            return Err(Error::UnexpectedEvent {
                expected: Kind::MlsGroupMessage,
                received: event.kind,
            });
        }

        // Load group
        let mut group = self.load_group(group_id)?.ok_or(Error::GroupNotFound)?;

        let export_nostr_keys = Keys::new(secret.secret_key);

        // Decrypt message
        let message: Vec<u8> = nip44::decrypt_to_bytes(
            export_nostr_keys.secret_key(),
            &export_nostr_keys.public_key,
            &event.content,
        )?;

        // The resulting serialized message is the MLS encrypted message that Bob sent
        // Now Bob can process the MLS message content and do what's needed with it
        self.process_message_for_group(&mut group, &message)
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
    /// Returns a Error if:
    /// - The group cannot be loaded from storage
    /// - The specified group is not found
    /// - A member's credential cannot be parsed
    /// - A member's identity bytes cannot be converted to a string
    pub fn member_public_keys(&self, group: &MlsGroup) -> Result<Vec<PublicKey>, Error> {
        // Store members in a variable to extend its lifetime
        let mut members = group.members();
        members.try_fold(Vec::new(), |mut acc, m| {
            let credentials: BasicCredential = BasicCredential::try_from(m.credential)?;
            let hex_bytes: &[u8] = credentials.identity();
            let hex_str: &str = str::from_utf8(hex_bytes)?;
            let public_key = PublicKey::from_hex(hex_str)?;
            acc.push(public_key);
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
    /// Returns a Error if:
    /// - The group cannot be loaded from storage
    /// - The specified group is not found
    /// - Failed to generate or store signature keypair
    /// - Failed to perform self-update operation
    pub fn self_update(&self, group_id: &GroupId) -> Result<SelfUpdate, Error> {
        let current_secret: ExportSecret = self.export_secret(group_id)?;

        tracing::debug!(target: "nostr_openmls::groups::self_update", "Current epoch: {:?}", current_secret.epoch);

        // Load group
        let mut group = self.load_group(group_id)?.ok_or(Error::GroupNotFound)?;

        // Load current signer
        let current_signer: SignatureKeyPair = self.load_signer(&group)?;

        // Get own leaf
        let own_leaf = self.get_own_leaf(&group)?;

        let new_signature_keypair = SignatureKeyPair::new(self.ciphersuite.signature_algorithm())?;

        new_signature_keypair
            .store(self.provider.storage())
            .map_err(|e| Error::Provider(e.to_string()))?;

        let pubkey = BasicCredential::try_from(own_leaf.credential().clone())?
            .identity()
            .to_vec();

        let new_credential: BasicCredential = BasicCredential::new(pubkey);
        let new_credential_with_key = CredentialWithKey {
            credential: new_credential.into(),
            signature_key: new_signature_keypair.public().into(),
        };

        let leaf_node_params = LeafNodeParameters::builder()
            .with_credential_with_key(new_credential_with_key)
            .with_capabilities(own_leaf.capabilities().clone())
            .with_extensions(own_leaf.extensions().clone())
            .build();

        let commit_message_bundle = group.self_update_with_new_signer(
            &self.provider,
            &current_signer,
            &new_signature_keypair,
            leaf_node_params,
        )?;

        // Merge the commit
        group.merge_pending_commit(&self.provider)?;

        // Export the new epoch's exporter secret
        let new_secret = self.export_secret(group_id)?;

        tracing::debug!(target: "nostr_openmls::groups::self_update", "New epoch: {:?}", new_secret.epoch);

        // Serialize the message
        let serialized_message = commit_message_bundle.commit().tls_serialize_detached()?;

        Ok(SelfUpdate {
            serialized_message,
            current_secret,
            new_secret,
        })
    }

    /// Validates the members and admins of a group during creation
    ///
    /// # Arguments
    /// * `creator_pubkey` - The public key of the group creator
    /// * `member_pubkeys` - List of public keys for group members
    /// * `admin_pubkeys` - List of public keys for group admins
    ///
    /// # Returns
    /// * `Ok(true)` if validation passes
    /// * `Err(GroupError::InvalidParameters)` if validation fails
    ///
    /// # Validation Rules
    /// - Creator must be an admin but not included in member list
    /// - Creator must have a valid public key
    /// - All member public keys must be valid
    /// - All admin public keys must be valid
    /// - All admins must also be members (except creator)
    ///
    /// # Errors
    /// Returns `GroupError::InvalidParameters` with descriptive message if:
    /// - Creator is not an admin
    /// - Creator is in member list
    /// - Creator has invalid public key
    /// - Any member has invalid public key
    /// - Any admin has invalid public key
    /// - Any admin is not a member
    pub fn validate_group_members(
        creator_pubkey: &String,
        member_pubkeys: &[String],
        admin_pubkeys: &[String],
    ) -> Result<bool, Error> {
        // Creator must be an admin
        if !admin_pubkeys.contains(creator_pubkey) {
            return Err(Error::Group("Creator must be an admin".to_string()));
        }

        // Creator must not be included as a member
        if member_pubkeys.contains(creator_pubkey) {
            return Err(Error::Group(
                "Creator must not be included as a member".to_string(),
            ));
        }

        // Creator must be valid pubkey
        if let Err(e) = PublicKey::parse(creator_pubkey) {
            return Err(Error::Group(format!(
                "{} is not a valid creator pubkey: {}",
                creator_pubkey, e
            )));
        }

        // Check that members are valid pubkeys
        for pubkey in member_pubkeys.iter() {
            if let Err(e) = PublicKey::parse(pubkey) {
                return Err(Error::Group(format!(
                    "{} is not a valid member pubkey: {}",
                    pubkey, e
                )));
            }
        }

        // Check that admins are valid pubkeys and are members
        for pubkey in admin_pubkeys.iter() {
            if let Err(e) = PublicKey::parse(pubkey) {
                return Err(Error::Group(format!(
                    "{} is not a valid admin pubkey: {}",
                    pubkey, e
                )));
            }
            if !member_pubkeys.contains(pubkey) && creator_pubkey != pubkey {
                return Err(Error::Group("Admin must be a member".to_string()));
            }
        }
        Ok(true)
    }

    // TODO: Create proposal
    // TODO: Send commit
}
