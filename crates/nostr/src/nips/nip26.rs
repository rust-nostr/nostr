// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP26: Delegated Event Signing
//!
//! <https://github.com/nostr-protocol/nips/blob/master/26.md>

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::num::ParseIntError;
use core::str::FromStr;

use hashes::sha256::Hash as Sha256Hash;
use hashes::Hash;
#[cfg(feature = "std")]
use secp256k1::rand::rngs::OsRng;
use secp256k1::rand::{CryptoRng, Rng};
use secp256k1::schnorr::Signature;
use secp256k1::{self, Message, Secp256k1, Signing, Verification, XOnlyPublicKey};
use serde::de::Error as DeserializerError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{json, Value};

use super::nip21;
use crate::event::Event;
use crate::key::{self, Keys};
use crate::PublicKey;
#[cfg(feature = "std")]
use crate::SECP256K1;

const DELEGATION_KEYWORD: &str = "delegation";

/// `NIP26` error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Key error
    Key(key::Error),
    /// Secp256k1 error
    Secp256k1(secp256k1::Error),
    /// Invalid condition, cannot parse expected number
    ConditionsParseNumeric(ParseIntError),
    /// Conditions not satisfied
    ConditionsValidation(ValidationError),
    /// Invalid condition in conditions string
    ConditionsParseInvalidCondition,
    /// Delegation tag parse error
    DelegationTagParse,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key(e) => write!(f, "Key: {e}"),
            Self::Secp256k1(e) => write!(f, "Secp256k1: {e}"),
            Self::ConditionsParseNumeric(_) => {
                write!(f, "Invalid condition, cannot parse expected number")
            }
            Self::ConditionsValidation(_) => write!(f, "Conditions not satisfied"),
            Self::ConditionsParseInvalidCondition => {
                write!(f, "Invalid condition in conditions string")
            }
            Self::DelegationTagParse => write!(f, "Delegation tag parse error"),
        }
    }
}

impl From<key::Error> for Error {
    fn from(e: key::Error) -> Self {
        Self::Key(e)
    }
}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Self {
        Self::Secp256k1(e)
    }
}

impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Self {
        Self::ConditionsParseNumeric(e)
    }
}

impl From<ValidationError> for Error {
    fn from(e: ValidationError) -> Self {
        Self::ConditionsValidation(e)
    }
}

/// Tag validation errors
#[derive(Debug, PartialEq, Eq)]
pub enum ValidationError {
    /// Signature does not match
    InvalidSignature,
    /// Event kind does not match
    InvalidKind,
    /// Creation time is earlier than validity period
    CreatedTooEarly,
    /// Creation time is later than validity period
    CreatedTooLate,
}

#[cfg(feature = "std")]
impl std::error::Error for ValidationError {}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSignature => write!(f, "Signature does not match"),
            Self::InvalidKind => write!(f, "Event kind does not match"),
            Self::CreatedTooEarly => write!(f, "Creation time is earlier than validity period"),
            Self::CreatedTooLate => write!(f, "Creation time is later than validity period"),
        }
    }
}

/// Sign delegation.
/// See `create_delegation_tag` for more complete functionality.
#[inline]
#[cfg(feature = "std")]
pub fn sign_delegation(
    delegator_keys: &Keys,
    delegatee_pk: &PublicKey,
    conditions: &Conditions,
) -> Signature {
    sign_delegation_with_ctx(
        SECP256K1,
        &mut OsRng,
        delegator_keys,
        delegatee_pk,
        conditions,
    )
}

/// Sign delegation.
/// See `create_delegation_tag` for more complete functionality.
pub fn sign_delegation_with_ctx<C, R>(
    secp: &Secp256k1<C>,
    rng: &mut R,
    delegator_keys: &Keys,
    delegatee_pk: &PublicKey,
    conditions: &Conditions,
) -> Signature
where
    C: Signing,
    R: Rng + CryptoRng,
{
    let unhashed_token = DelegationToken::new(delegatee_pk, conditions);
    let hashed_token = Sha256Hash::hash(unhashed_token.as_bytes());
    let message: Message = Message::from_digest(hashed_token.to_byte_array());
    delegator_keys.sign_schnorr_with_ctx(secp, &message, rng)
}

/// Verify delegation signature
#[inline]
#[cfg(feature = "std")]
pub fn verify_delegation_signature(
    delegator_public_key: &PublicKey,
    signature: Signature,
    delegatee_public_key: &PublicKey,
    conditions: &Conditions,
) -> Result<(), Error> {
    verify_delegation_signature_with_ctx(
        SECP256K1,
        delegator_public_key,
        signature,
        delegatee_public_key,
        conditions,
    )
}

/// Verify delegation signature
pub fn verify_delegation_signature_with_ctx<C>(
    secp: &Secp256k1<C>,
    delegator_public_key: &PublicKey,
    signature: Signature,
    delegatee_public_key: &PublicKey,
    conditions: &Conditions,
) -> Result<(), Error>
where
    C: Verification,
{
    let unhashed_token = DelegationToken::new(delegatee_public_key, conditions);
    let hashed_token = Sha256Hash::hash(unhashed_token.as_bytes());
    let message = Message::from_digest(hashed_token.to_byte_array());
    let public_key: XOnlyPublicKey = delegator_public_key.xonly()?;
    secp.verify_schnorr(&signature, &message, &public_key)?;
    Ok(())
}

/// Delegation token
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DelegationToken(String);

impl DelegationToken {
    /// Generate [`DelegationToken`]
    #[inline]
    pub fn new(delegatee_pk: &PublicKey, conditions: &Conditions) -> Self {
        Self(format!(
            "{}:{DELEGATION_KEYWORD}:{delegatee_pk}:{conditions}",
            nip21::SCHEME
        ))
    }

    /// Get as bytes
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl fmt::Display for DelegationToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Delegation tag, as defined in NIP26
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DelegationTag {
    delegator_pubkey: PublicKey,
    conditions: Conditions,
    signature: Signature,
}

impl DelegationTag {
    /// Create a delegation tag (including the signature).
    #[inline]
    #[cfg(feature = "std")]
    pub fn new(
        delegator_keys: &Keys,
        delegatee_pubkey: &PublicKey,
        conditions: Conditions,
    ) -> Self {
        Self::new_with_ctx(
            SECP256K1,
            &mut OsRng,
            delegator_keys,
            delegatee_pubkey,
            conditions,
        )
    }

    /// Create a delegation tag (including the signature).
    pub fn new_with_ctx<C, R>(
        secp: &Secp256k1<C>,
        rng: &mut R,
        delegator_keys: &Keys,
        delegatee_pubkey: &PublicKey,
        conditions: Conditions,
    ) -> Self
    where
        C: Signing,
        R: Rng + CryptoRng,
    {
        let signature: Signature =
            sign_delegation_with_ctx(secp, rng, delegator_keys, delegatee_pubkey, &conditions);
        Self {
            delegator_pubkey: delegator_keys.public_key(),
            conditions,
            signature,
        }
    }

    /// Get delegator public key
    #[inline]
    pub fn delegator_pubkey(&self) -> PublicKey {
        self.delegator_pubkey
    }

    /// Get conditions
    #[inline]
    pub fn conditions(&self) -> Conditions {
        self.conditions.clone()
    }

    /// Get signature
    #[inline]
    pub fn signature(&self) -> Signature {
        self.signature
    }

    /// Validate a delegation tag, check signature and conditions.
    #[inline]
    #[cfg(feature = "std")]
    pub fn validate(
        &self,
        delegatee_pubkey: &PublicKey,
        event_properties: &EventProperties,
    ) -> Result<(), Error> {
        self.validate_with_ctx(SECP256K1, delegatee_pubkey, event_properties)
    }

    /// Validate a delegation tag, check signature and conditions.
    pub fn validate_with_ctx<C>(
        &self,
        secp: &Secp256k1<C>,
        delegatee_pubkey: &PublicKey,
        event_properties: &EventProperties,
    ) -> Result<(), Error>
    where
        C: Verification,
    {
        // Verify signature
        verify_delegation_signature_with_ctx(
            secp,
            &self.delegator_pubkey,
            self.signature,
            delegatee_pubkey,
            &self.conditions,
        )
        .map_err(|_| Error::ConditionsValidation(ValidationError::InvalidSignature))?;

        // Validate conditions
        self.conditions.evaluate(event_properties)?;

        Ok(())
    }

    /// Convert to JSON string.
    pub fn as_json(&self) -> String {
        let tag = json!([
            DELEGATION_KEYWORD,
            self.delegator_pubkey.to_string(),
            self.conditions.to_string(),
            self.signature.to_string(),
        ]);
        tag.to_string()
    }

    /// Parse from a JSON string
    pub fn from_json(s: &str) -> Result<Self, Error> {
        let tag: Vec<String> = serde_json::from_str(s).map_err(|_| Error::DelegationTagParse)?;
        Self::try_from(tag)
    }
}

impl TryFrom<Vec<String>> for DelegationTag {
    type Error = Error;

    fn try_from(tag: Vec<String>) -> Result<Self, Self::Error> {
        if tag.len() != 4 {
            return Err(Error::DelegationTagParse);
        }
        if tag[0] != DELEGATION_KEYWORD {
            return Err(Error::DelegationTagParse);
        }
        Ok(Self {
            delegator_pubkey: PublicKey::from_str(&tag[1])?,
            conditions: Conditions::from_str(&tag[2])?,
            signature: Signature::from_str(&tag[3])?,
        })
    }
}

impl fmt::Display for DelegationTag {
    /// Return tag in JSON string format
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_json())
    }
}

impl FromStr for DelegationTag {
    type Err = Error;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_json(s)
    }
}

/// A condition from the delegation conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Condition {
    /// Event kind, e.g. kind=1
    Kind(u16),
    /// Creation time before, e.g. created_at<1679000000
    CreatedBefore(u64),
    /// Creation time after, e.g. created_at>1676000000
    CreatedAfter(u64),
}

/// Represents properties of an event, relevant for delegation
pub struct EventProperties {
    /// Event kind. For simplicity/flexibility, numeric type is used.
    kind: u16,
    /// Creation time, as unix timestamp
    created_time: u64,
}

impl Condition {
    /// Evaluate whether an event satisfies this condition
    pub(crate) fn evaluate(&self, ep: &EventProperties) -> Result<(), ValidationError> {
        match self {
            Self::Kind(k) => {
                if ep.kind != *k {
                    return Err(ValidationError::InvalidKind);
                }
            }
            Self::CreatedBefore(t) => {
                if ep.created_time >= *t {
                    return Err(ValidationError::CreatedTooLate);
                }
            }
            Self::CreatedAfter(t) => {
                if ep.created_time <= *t {
                    return Err(ValidationError::CreatedTooEarly);
                }
            }
        }
        Ok(())
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Kind(k) => write!(f, "kind={k}"),
            Self::CreatedBefore(t) => write!(f, "created_at<{t}"),
            Self::CreatedAfter(t) => write!(f, "created_at>{t}"),
        }
    }
}

impl FromStr for Condition {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(kind) = s.strip_prefix("kind=") {
            let n = u16::from_str(kind)?;
            return Ok(Self::Kind(n));
        }
        if let Some(created_before) = s.strip_prefix("created_at<") {
            let n = u64::from_str(created_before)?;
            return Ok(Self::CreatedBefore(n));
        }
        if let Some(created_after) = s.strip_prefix("created_at>") {
            let n = u64::from_str(created_after)?;
            return Ok(Self::CreatedAfter(n));
        }
        Err(Error::ConditionsParseInvalidCondition)
    }
}

/// Set of conditions of a delegation.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Conditions(Vec<Condition>);

impl Conditions {
    /// New empty [`Conditions`]
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add [`Condition`]
    #[inline]
    pub fn add(&mut self, cond: Condition) {
        self.0.push(cond);
    }

    /// Evaluate whether an event satisfies all these conditions
    fn evaluate(&self, ep: &EventProperties) -> Result<(), ValidationError> {
        for c in &self.0 {
            c.evaluate(ep)?;
        }
        Ok(())
    }

    /// Get [`Vec<Condition>`]
    #[inline]
    pub fn inner(&self) -> Vec<Condition> {
        self.0.clone()
    }
}

impl fmt::Display for Conditions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Convert parts, join
        let conditions: String = self
            .0
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .join("&");
        write!(f, "{conditions}")
    }
}

impl FromStr for Conditions {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(Self::new());
        }
        let cond = s
            .split('&')
            .map(Condition::from_str)
            .collect::<Result<Vec<Condition>, Self::Err>>()?;
        Ok(Self(cond))
    }
}

impl Serialize for Conditions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Conditions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json_value = Value::deserialize(deserializer)?;
        let conditions: String =
            serde_json::from_value(json_value).map_err(DeserializerError::custom)?;
        Self::from_str(&conditions).map_err(DeserializerError::custom)
    }
}

impl EventProperties {
    /// Create new with values
    #[inline]
    pub fn new(event_kind: u16, created_time: u64) -> Self {
        Self {
            kind: event_kind,
            created_time,
        }
    }

    /// Create from an Event
    pub fn from_event(event: &Event) -> Self {
        Self {
            kind: event.kind.as_u16(),
            created_time: event.created_at.as_u64(),
        }
    }
}
