// Copyright (c) 2024-2025 Jeff Gardner
// Copyright (c) 2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr MLS errors

use std::string::FromUtf8Error;
use std::{fmt, str};

use nostr::nips::nip44;
use nostr::types::url;
use nostr::{event, key, Kind, SignerError};
use openmls::credentials::errors::BasicCredentialError;
use openmls::error::LibraryError;
use openmls::extensions::errors::InvalidExtensionError;
use openmls::framing::errors::ProtocolMessageError;
use openmls::group::{
    AddMembersError, CommitToPendingProposalsError, CreateGroupContextExtProposalError,
    CreateMessageError, ExportSecretError, MergePendingCommitError, NewGroupError,
    ProcessMessageError, SelfUpdateError, WelcomeError,
};
use openmls::key_packages::errors::{KeyPackageNewError, KeyPackageVerifyError};
use openmls_traits::types::CryptoError;

/// Nostr MLS error
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Hex error
    Hex(hex::FromHexError),
    /// Keys error
    Keys(key::Error),
    /// Event error
    Event(event::Error),
    /// Event Builder error
    EventBuilder(event::builder::Error),
    /// Nostr Signer error
    Signer(SignerError),
    /// NIP44 error
    NIP44(nip44::Error),
    /// Relay URL error
    RelayUrl(url::Error),
    /// TLS error
    Tls(tls_codec::Error),
    /// UTF8 error
    Utf8(str::Utf8Error),
    /// Crypto error
    Crypto(CryptoError),
    /// Generic OpenMLS error
    OpenMlsGeneric(LibraryError),
    /// Invalid extension error
    InvalidExtension(InvalidExtensionError),
    /// Create message error
    CreateMessage(CreateMessageError),
    /// Export secret error
    ExportSecret(ExportSecretError),
    /// Basic credential error
    BasicCredential(BasicCredentialError),
    /// Process message error
    ProcessMessage(ProcessMessageError),
    /// Protocol message error
    ProtocolMessage(String),
    /// Key package error
    KeyPackage(String),
    /// Group error
    Group(String),
    /// Group exporter secret not found
    GroupExporterSecretNotFound,
    /// Message error
    Message(String),
    /// Cannot decrypt own message
    CannotDecryptOwnMessage,
    /// Merge pending commit error
    MergePendingCommit(String),
    /// Commit to pending proposal
    CommitToPendingProposalsError,
    /// Self update error
    SelfUpdate(String),
    /// Welcome error
    Welcome(String),
    /// We're missing a Welcome for an existing ProcessedWelcome
    MissingWelcomeForProcessedWelcome,
    /// Processed welcome not found
    ProcessedWelcomeNotFound,
    /// Provider error
    Provider(String),
    /// Group not found
    GroupNotFound,
    /// Protocol message group ID doesn't match the current group ID
    ProtocolGroupIdMismatch,
    /// Own leaf not found
    OwnLeafNotFound,
    /// Failed to load signer
    CantLoadSigner,
    /// Invalid Welcome message
    InvalidWelcomeMessage,
    /// Unexpected event
    UnexpectedEvent {
        /// Expected event kind
        expected: Kind,
        /// Received event kind
        received: Kind,
    },
    /// Unexpected extension type
    UnexpectedExtensionType,
    /// Nostr group data extension not found
    NostrGroupDataExtensionNotFound,
    /// Message from a non-member of a group
    MessageFromNonMember,
    /// Code path is not yet implemented
    NotImplemented(String),
    /// Stored message not found
    MessageNotFound,
    /// Proposal message received from a non-admin
    ProposalFromNonAdmin,
    /// Commit message received from a non-admin
    CommitFromNonAdmin,
    /// Error when updating group context extensions
    UpdateGroupContextExts(String),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Hex(e) => write!(f, "{e}"),
            Self::Keys(e) => write!(f, "{e}"),
            Self::Event(e) => write!(f, "{e}"),
            Self::EventBuilder(e) => write!(f, "{e}"),
            Self::Signer(e) => write!(f, "{e}"),
            Self::NIP44(e) => write!(f, "{e}"),
            Self::RelayUrl(e) => write!(f, "{e}"),
            Self::Tls(e) => write!(f, "{e}"),
            Self::Utf8(e) => write!(f, "{e}"),
            Self::Crypto(e) => write!(f, "{e}"),
            Self::OpenMlsGeneric(e) => write!(f, "{e}"),
            Self::InvalidExtension(e) => write!(f, "{e}"),
            Self::CreateMessage(e) => write!(f, "{e}"),
            Self::ExportSecret(e) => write!(f, "{e}"),
            Self::BasicCredential(e) => write!(f, "{e}"),
            Self::ProcessMessage(e) => write!(f, "{e}"),
            Self::ProtocolMessage(e) => write!(f, "{e}"),
            Self::KeyPackage(e) => write!(f, "{e}"),
            Self::Group(e) => write!(f, "{e}"),
            Self::GroupExporterSecretNotFound => write!(f, "group exporter secret not found"),
            Self::Message(e) => write!(f, "{e}"),
            Self::CannotDecryptOwnMessage => write!(f, "cannot decrypt own message"),
            Self::Welcome(e) => write!(f, "{e}"),
            Self::MissingWelcomeForProcessedWelcome => {
                write!(f, "missing welcome for processed welcome")
            }
            Self::ProcessedWelcomeNotFound => write!(f, "processed welcome not found"),
            Self::MergePendingCommit(e) => write!(f, "{e}"),
            Self::CommitToPendingProposalsError => {
                write!(f, "unable to commit to pending proposal")
            }
            Self::SelfUpdate(e) => write!(f, "{e}"),
            Self::Provider(e) => write!(f, "{e}"),
            Self::GroupNotFound => write!(f, "group not found"),
            Self::ProtocolGroupIdMismatch => write!(
                f,
                "protocol message group ID doesn't match the current group ID"
            ),
            Self::OwnLeafNotFound => write!(f, "own leaf not found"),
            Self::CantLoadSigner => write!(f, "can't load signer"),
            Self::InvalidWelcomeMessage => write!(f, "invalid welcome message"),
            Self::UnexpectedEvent { expected, received } => write!(
                f,
                "unexpected event kind: expected={expected}, received={received}"
            ),
            Self::UnexpectedExtensionType => {
                write!(f, "Unexpected extension type")
            }
            Self::NostrGroupDataExtensionNotFound => {
                write!(f, "Nostr group data extension not found")
            }
            Self::MessageFromNonMember => {
                write!(f, "Message recieved from non-member")
            }
            Self::NotImplemented(e) => {
                write!(f, "{e}")
            }
            Self::MessageNotFound => write!(f, "stored message not found"),
            Self::ProposalFromNonAdmin => write!(f, "not processing proposal from non-admin"),
            Self::CommitFromNonAdmin => write!(f, "not processing commit from non-admin"),
            Self::UpdateGroupContextExts(e) => {
                write!(f, "Error when updating group context extensions {e}")
            }
        }
    }
}

impl From<hex::FromHexError> for Error {
    fn from(e: hex::FromHexError) -> Self {
        Self::Hex(e)
    }
}

impl From<key::Error> for Error {
    fn from(e: key::Error) -> Self {
        Self::Keys(e)
    }
}

impl From<event::Error> for Error {
    fn from(e: event::Error) -> Self {
        Self::Event(e)
    }
}

impl From<event::builder::Error> for Error {
    fn from(e: event::builder::Error) -> Self {
        Self::EventBuilder(e)
    }
}

impl From<SignerError> for Error {
    fn from(e: SignerError) -> Self {
        Self::Signer(e)
    }
}

impl From<nip44::Error> for Error {
    fn from(e: nip44::Error) -> Self {
        Self::NIP44(e)
    }
}

impl From<url::Error> for Error {
    fn from(e: url::Error) -> Self {
        Self::RelayUrl(e)
    }
}

impl From<tls_codec::Error> for Error {
    fn from(e: tls_codec::Error) -> Self {
        Self::Tls(e)
    }
}

impl From<str::Utf8Error> for Error {
    fn from(e: str::Utf8Error) -> Self {
        Self::Utf8(e)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(e: FromUtf8Error) -> Self {
        Self::Utf8(e.utf8_error())
    }
}

impl From<CryptoError> for Error {
    fn from(e: CryptoError) -> Self {
        Self::Crypto(e)
    }
}

impl From<LibraryError> for Error {
    fn from(e: LibraryError) -> Self {
        Self::OpenMlsGeneric(e)
    }
}

impl From<InvalidExtensionError> for Error {
    fn from(e: InvalidExtensionError) -> Self {
        Self::InvalidExtension(e)
    }
}

impl From<CreateMessageError> for Error {
    fn from(e: CreateMessageError) -> Self {
        Self::CreateMessage(e)
    }
}

impl From<ExportSecretError> for Error {
    fn from(e: ExportSecretError) -> Self {
        Self::ExportSecret(e)
    }
}

impl From<BasicCredentialError> for Error {
    fn from(e: BasicCredentialError) -> Self {
        Self::BasicCredential(e)
    }
}

impl From<ProcessMessageError> for Error {
    fn from(e: ProcessMessageError) -> Self {
        Self::ProcessMessage(e)
    }
}

impl From<ProtocolMessageError> for Error {
    fn from(e: ProtocolMessageError) -> Self {
        Self::ProtocolMessage(e.to_string())
    }
}

impl From<KeyPackageNewError> for Error {
    fn from(e: KeyPackageNewError) -> Self {
        Self::KeyPackage(e.to_string())
    }
}

impl From<KeyPackageVerifyError> for Error {
    fn from(e: KeyPackageVerifyError) -> Self {
        Self::KeyPackage(e.to_string())
    }
}

impl<T> From<NewGroupError<T>> for Error
where
    T: fmt::Display,
{
    fn from(e: NewGroupError<T>) -> Self {
        Self::Group(e.to_string())
    }
}

impl<T> From<AddMembersError<T>> for Error
where
    T: fmt::Display,
{
    fn from(e: AddMembersError<T>) -> Self {
        Self::Group(e.to_string())
    }
}

impl<T> From<MergePendingCommitError<T>> for Error
where
    T: fmt::Display,
{
    fn from(e: MergePendingCommitError<T>) -> Self {
        Self::MergePendingCommit(e.to_string())
    }
}

impl<T> From<CommitToPendingProposalsError<T>> for Error
where
    T: fmt::Display,
{
    fn from(_e: CommitToPendingProposalsError<T>) -> Self {
        Self::CommitToPendingProposalsError
    }
}

impl<T> From<SelfUpdateError<T>> for Error
where
    T: fmt::Display,
{
    fn from(e: SelfUpdateError<T>) -> Self {
        Self::SelfUpdate(e.to_string())
    }
}

impl<T> From<WelcomeError<T>> for Error
where
    T: fmt::Display,
{
    fn from(e: WelcomeError<T>) -> Self {
        Self::Welcome(e.to_string())
    }
}

impl<T> From<CreateGroupContextExtProposalError<T>> for Error
where
    T: fmt::Display,
{
    fn from(e: CreateGroupContextExtProposalError<T>) -> Self {
        Self::UpdateGroupContextExts(e.to_string())
    }
}
