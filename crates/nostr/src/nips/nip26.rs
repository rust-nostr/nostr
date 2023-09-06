// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIP26
//!
//! <https://github.com/nostr-protocol/nips/blob/master/26.md>

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::num::ParseIntError;
use core::str::FromStr;

use bitcoin::hashes::sha256::Hash as Sha256Hash;
use bitcoin::hashes::Hash;
#[cfg(feature = "std")]
use bitcoin::secp256k1::rand;
use bitcoin::secp256k1::rand::{CryptoRng, Rng};
use bitcoin::secp256k1::schnorr::Signature;
use bitcoin::secp256k1::{self, Message, Secp256k1, Signing, Verification, XOnlyPublicKey};
use serde::de::Error as DeserializerError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{json, Value};

use super::nip21;
use crate::event::Event;
use crate::key::{self, Keys};
#[cfg(feature = "std")]
use crate::SECP256K1;

const DELEGATION_KEYWORD: &str = "delegation";

/// `NIP26` error
#[derive(Debug, Eq, PartialEq)]
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
#[cfg(feature = "std")]
pub fn sign_delegation(
    delegator_keys: &Keys,
    delegatee_pk: XOnlyPublicKey,
    conditions: Conditions,
) -> Result<Signature, Error> {
    sign_delegation_with_ctx(
        &SECP256K1,
        &mut rand::thread_rng(),
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
    delegatee_pk: XOnlyPublicKey,
    conditions: Conditions,
) -> Result<Signature, Error>
where
    C: Signing,
    R: Rng + CryptoRng,
{
    let unhashed_token = DelegationToken::new(delegatee_pk, conditions);
    let hashed_token = Sha256Hash::hash(unhashed_token.as_bytes());
    let message = Message::from_slice(hashed_token.as_byte_array())?;
    Ok(delegator_keys.sign_schnorr_with_ctx(secp, &message, rng)?)
}

/// Verify delegation signature
#[cfg(feature = "std")]
pub fn verify_delegation_signature(
    delegator_public_key: XOnlyPublicKey,
    signature: Signature,
    delegatee_public_key: XOnlyPublicKey,
    conditions: Conditions,
) -> Result<(), Error> {
    verify_delegation_signature_with_ctx(
        &SECP256K1,
        delegator_public_key,
        signature,
        delegatee_public_key,
        conditions,
    )
}

/// Verify delegation signature
pub fn verify_delegation_signature_with_ctx<C>(
    secp: &Secp256k1<C>,
    delegator_public_key: XOnlyPublicKey,
    signature: Signature,
    delegatee_public_key: XOnlyPublicKey,
    conditions: Conditions,
) -> Result<(), Error>
where
    C: Verification,
{
    let unhashed_token = DelegationToken::new(delegatee_public_key, conditions);
    let hashed_token = Sha256Hash::hash(unhashed_token.as_bytes());
    let message = Message::from_slice(hashed_token.as_byte_array())?;
    secp.verify_schnorr(&signature, &message, &delegator_public_key)?;
    Ok(())
}

/// Delegation token
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DelegationToken(String);

impl DelegationToken {
    /// Generate [`DelegationToken`]
    pub fn new(delegatee_pk: XOnlyPublicKey, conditions: Conditions) -> Self {
        Self(format!(
            "{}:{DELEGATION_KEYWORD}:{delegatee_pk}:{conditions}",
            nip21::SCHEME
        ))
    }

    /// Get as bytes
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl fmt::Display for DelegationToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Delegation tag, as defined in NIP-26
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DelegationTag {
    delegator_pubkey: XOnlyPublicKey,
    conditions: Conditions,
    signature: Signature,
}

impl DelegationTag {
    /// Create a delegation tag (including the signature).
    #[cfg(feature = "std")]
    pub fn new(
        delegator_keys: &Keys,
        delegatee_pubkey: XOnlyPublicKey,
        conditions: Conditions,
    ) -> Result<Self, Error> {
        Self::new_with_ctx(
            &SECP256K1,
            &mut rand::thread_rng(),
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
        delegatee_pubkey: XOnlyPublicKey,
        conditions: Conditions,
    ) -> Result<Self, Error>
    where
        C: Signing,
        R: Rng + CryptoRng,
    {
        let signature = sign_delegation_with_ctx(
            secp,
            rng,
            delegator_keys,
            delegatee_pubkey,
            conditions.clone(),
        )?;
        Ok(Self {
            delegator_pubkey: delegator_keys.public_key(),
            conditions,
            signature,
        })
    }

    /// Get delegator public key
    pub fn delegator_pubkey(&self) -> XOnlyPublicKey {
        self.delegator_pubkey
    }

    /// Get conditions
    pub fn conditions(&self) -> Conditions {
        self.conditions.clone()
    }

    /// Get signature
    pub fn signature(&self) -> Signature {
        self.signature
    }

    /// Validate a delegation tag, check signature and conditions.
    #[cfg(feature = "std")]
    pub fn validate(
        &self,
        delegatee_pubkey: XOnlyPublicKey,
        event_properties: &EventProperties,
    ) -> Result<(), Error> {
        self.validate_with_ctx(&SECP256K1, delegatee_pubkey, event_properties)
    }

    /// Validate a delegation tag, check signature and conditions.
    pub fn validate_with_ctx<C>(
        &self,
        secp: &Secp256k1<C>,
        delegatee_pubkey: XOnlyPublicKey,
        event_properties: &EventProperties,
    ) -> Result<(), Error>
    where
        C: Verification,
    {
        // verify signature
        verify_delegation_signature_with_ctx(
            secp,
            self.delegator_pubkey,
            self.signature,
            delegatee_pubkey,
            self.conditions.clone(),
        )
        .map_err(|_| Error::ConditionsValidation(ValidationError::InvalidSignature))?;

        // validate conditions
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
            delegator_pubkey: XOnlyPublicKey::from_str(&tag[1])?,
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

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_json(s)
    }
}

/// A condition from the delegation conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Condition {
    /// Event kind, e.g. kind=1
    Kind(u64),
    /// Creation time before, e.g. created_at<1679000000
    CreatedBefore(u64),
    /// Creation time after, e.g. created_at>1676000000
    CreatedAfter(u64),
}

/// Represents properties of an event, relevant for delegation
pub struct EventProperties {
    /// Event kind. For simplicity/flexibility, numeric type is used.
    kind: u64,
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

impl ToString for Condition {
    fn to_string(&self) -> String {
        match self {
            Self::Kind(k) => format!("kind={k}"),
            Self::CreatedBefore(t) => format!("created_at<{t}"),
            Self::CreatedAfter(t) => format!("created_at>{t}"),
        }
    }
}

impl FromStr for Condition {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(kind) = s.strip_prefix("kind=") {
            let n = u64::from_str(kind)?;
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Conditions(Vec<Condition>);

impl Default for Conditions {
    fn default() -> Self {
        Self::new()
    }
}

impl Conditions {
    /// New empty [`Conditions`]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Add [`Condition`]
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

    /// Get [`Vec<Contifion>`]
    pub fn inner(&self) -> Vec<Condition> {
        self.0.clone()
    }
}

impl fmt::Display for Conditions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // convert parts, join
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
    pub fn new(event_kind: u64, created_time: u64) -> Self {
        Self {
            kind: event_kind,
            created_time,
        }
    }

    /// Create from an Event

    pub fn from_event(event: &Event) -> Self {
        Self {
            kind: event.kind.as_u64(),
            created_time: event.created_at.as_u64(),
        }
    }
}

#[cfg(test)]
mod test {
    use core::str::FromStr;

    use bitcoin::secp256k1::{Secp256k1, SecretKey};

    use super::*;

    #[test]
    fn test_serialize_conditions() {
        let mut conditions = Conditions::new();
        conditions.add(Condition::Kind(1));
        conditions.add(Condition::CreatedAfter(1676067553));
        conditions.add(Condition::CreatedBefore(1678659553));

        assert_eq!(
            json!(conditions).as_str().unwrap(),
            "kind=1&created_at>1676067553&created_at<1678659553"
        );
    }

    #[test]
    fn test_deserialize_conditions() {
        let mut conditions = Conditions::new();
        conditions.add(Condition::Kind(1));
        conditions.add(Condition::CreatedAfter(1676067553));
        conditions.add(Condition::CreatedBefore(1678659553));

        let value = json!("kind=1&created_at>1676067553&created_at<1678659553");
        assert_eq!(
            serde_json::from_value::<Conditions>(value).unwrap(),
            conditions
        );

        let value = "\"kind=1&created_at>1676067553&created_at<1678659553\"";
        assert_eq!(
            serde_json::from_str::<Conditions>(value).unwrap(),
            conditions
        );
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_create_delegation_tag() {
        let delegator_secret_key =
            SecretKey::from_str("b2f3673ee3a659283e6599080e0ab0e669a3c2640914375a9b0b357faae08b17")
                .unwrap();
        let delegator_keys = Keys::new(delegator_secret_key);
        let delegatee_pubkey = XOnlyPublicKey::from_str(
            "bea8aeb6c1657e33db5ac75a83910f77e8ec6145157e476b5b88c6e85b1fab34",
        )
        .unwrap();
        let conditions =
            Conditions::from_str("kind=1&created_at>1676067553&created_at<1678659553").unwrap();

        let tag =
            DelegationTag::new(&delegator_keys, delegatee_pubkey, conditions.clone()).unwrap();

        // verify signature (it's variable)
        let verify_result = verify_delegation_signature(
            delegator_keys.public_key(),
            tag.signature(),
            delegatee_pubkey,
            conditions,
        );
        assert!(verify_result.is_ok());

        // signature changes, cannot compare to expected constant, use signature from result
        let expected = format!(
            "[\"delegation\",\"1a459a8a6aa6441d480ba665fb8fb21a4cfe8bcacb7d87300f8046a558a3fce4\",\"kind=1&created_at>1676067553&created_at<1678659553\",\"{}\"]",
            &tag.signature.to_string());
        assert_eq!(tag.to_string(), expected);
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_validate_delegation_tag() {
        let delegator_secret_key =
            SecretKey::from_str("b2f3673ee3a659283e6599080e0ab0e669a3c2640914375a9b0b357faae08b17")
                .unwrap();
        let delegator_keys = Keys::new(delegator_secret_key);
        let delegatee_pubkey = XOnlyPublicKey::from_str(
            "bea8aeb6c1657e33db5ac75a83910f77e8ec6145157e476b5b88c6e85b1fab34",
        )
        .unwrap();
        let conditions =
            Conditions::from_str("kind=1&created_at>1676067553&created_at<1678659553").unwrap();

        let tag = DelegationTag::new(&delegator_keys, delegatee_pubkey, conditions).unwrap();

        assert!(tag
            .validate(delegatee_pubkey, &EventProperties::new(1, 1677000000))
            .is_ok());
    }

    #[test]
    fn test_delegation_tag_parse_and_validate() {
        let secp = Secp256k1::new();

        let tag_str = "[\"delegation\",\"1a459a8a6aa6441d480ba665fb8fb21a4cfe8bcacb7d87300f8046a558a3fce4\",\"kind=1&created_at>1676067553&created_at<1678659553\",\"369aed09c1ad52fceb77ecd6c16f2433eac4a3803fc41c58876a5b60f4f36b9493d5115e5ec5a0ce6c3668ffe5b58d47f2cbc97233833bb7e908f66dbbbd9d36\"]";
        let delegatee_pubkey = XOnlyPublicKey::from_str(
            "bea8aeb6c1657e33db5ac75a83910f77e8ec6145157e476b5b88c6e85b1fab34",
        )
        .unwrap();

        let tag = DelegationTag::from_str(tag_str).unwrap();

        assert!(tag
            .validate_with_ctx(
                &secp,
                delegatee_pubkey,
                &EventProperties::new(1, 1677000000)
            )
            .is_ok());

        // additional test: verify a value from inside the tag
        assert_eq!(
            tag.conditions().to_string(),
            "kind=1&created_at>1676067553&created_at<1678659553"
        );

        // additional test: try validation with invalid values, invalid event kind

        match tag
            .validate_with_ctx(
                &secp,
                delegatee_pubkey,
                &EventProperties::new(5, 1677000000),
            )
            .err()
            .unwrap()
        {
            Error::ConditionsValidation(e) => assert_eq!(e, ValidationError::InvalidKind),
            _ => panic!("Expected ConditionsValidation"),
        };
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_sign_delegation_verify_delegation_signature() {
        let delegator_secret_key =
            SecretKey::from_str("ee35e8bb71131c02c1d7e73231daa48e9953d329a4b701f7133c8f46dd21139c")
                .unwrap();
        let delegator_keys = Keys::new(delegator_secret_key);
        let delegatee_public_key = XOnlyPublicKey::from_str(
            "477318cfb5427b9cfc66a9fa376150c1ddbc62115ae27cef72417eb959691396",
        )
        .unwrap();
        let conditions =
            Conditions::from_str("kind=1&created_at>1674834236&created_at<1677426236").unwrap();

        let signature =
            sign_delegation(&delegator_keys, delegatee_public_key, conditions.clone()).unwrap();

        // signature is changing, validate by verify method
        let verify_result = verify_delegation_signature(
            delegator_keys.public_key(),
            signature,
            delegatee_public_key,
            conditions,
        );
        assert!(verify_result.is_ok());
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_sign_delegation_verify_lowlevel() {
        let delegator_secret_key =
            SecretKey::from_str("ee35e8bb71131c02c1d7e73231daa48e9953d329a4b701f7133c8f46dd21139c")
                .unwrap();
        let delegator_keys = Keys::new(delegator_secret_key);
        let delegatee_public_key = XOnlyPublicKey::from_str(
            "477318cfb5427b9cfc66a9fa376150c1ddbc62115ae27cef72417eb959691396",
        )
        .unwrap();
        let conditions =
            Conditions::from_str("kind=1&created_at>1674834236&created_at<1677426236").unwrap();

        let signature =
            sign_delegation(&delegator_keys, delegatee_public_key, conditions.clone()).unwrap();

        // signature is changing, validate by lowlevel verify
        let unhashed_token: String =
            format!("nostr:delegation:{delegatee_public_key}:{conditions}");
        let hashed_token = Sha256Hash::hash(unhashed_token.as_bytes());
        let message = Message::from_slice(hashed_token.as_byte_array()).unwrap();

        let verify_result =
            SECP256K1.verify_schnorr(&signature, &message, &delegator_keys.public_key());
        assert!(verify_result.is_ok());
    }

    #[test]
    fn test_verify_delegation_signature() {
        let secp = Secp256k1::new();

        let delegator_secret_key =
            SecretKey::from_str("ee35e8bb71131c02c1d7e73231daa48e9953d329a4b701f7133c8f46dd21139c")
                .unwrap();
        let delegator_keys = Keys::new_with_ctx(&secp, delegator_secret_key);
        // use one concrete signature
        let signature = Signature::from_str("f9f00fcf8480686d9da6dfde1187d4ba19c54f6ace4c73361a14db429c4b96eb30b29283d6ea1f06ba9e18e06e408244c689039ddadbacffc56060f3da5b04b8").unwrap();
        let delegatee_pk = XOnlyPublicKey::from_str(
            "477318cfb5427b9cfc66a9fa376150c1ddbc62115ae27cef72417eb959691396",
        )
        .unwrap();
        let conditions =
            Conditions::from_str("kind=1&created_at>1674834236&created_at<1677426236").unwrap();

        let verify_result = verify_delegation_signature_with_ctx(
            &secp,
            delegator_keys.public_key(),
            signature,
            delegatee_pk,
            conditions,
        );
        assert!(verify_result.is_ok());
    }

    #[test]
    fn test_delegation_token() {
        let delegatee_pk = XOnlyPublicKey::from_str(
            "477318cfb5427b9cfc66a9fa376150c1ddbc62115ae27cef72417eb959691396",
        )
        .unwrap();
        let conditions =
            Conditions::from_str("kind=1&created_at>1674834236&created_at<1677426236").unwrap();
        let unhashed_token = DelegationToken::new(delegatee_pk, conditions);
        assert_eq!(
            unhashed_token.to_string().as_str(),
            "nostr:delegation:477318cfb5427b9cfc66a9fa376150c1ddbc62115ae27cef72417eb959691396:kind=1&created_at>1674834236&created_at<1677426236"
        );
    }

    #[test]
    fn test_delegation_tag_to_json() {
        let secp = Secp256k1::new();

        let delegator_sk =
            SecretKey::from_str("b2f3673ee3a659283e6599080e0ab0e669a3c2640914375a9b0b357faae08b17")
                .unwrap();
        let delegator_pubkey = Keys::new_with_ctx(&secp, delegator_sk).public_key();
        let conditions = Conditions::from_str("kind=1&created_at<1678659553").unwrap();
        let signature = Signature::from_str("435091ab4c4a11e594b1a05e0fa6c2f6e3b6eaa87c53f2981a3d6980858c40fdcaffde9a4c461f352a109402a4278ff4dbf90f9ebd05f96dac5ae36a6364a976").unwrap();
        let d = DelegationTag {
            delegator_pubkey,
            conditions,
            signature,
        };
        let tag = d.as_json();
        assert_eq!(tag, "[\"delegation\",\"1a459a8a6aa6441d480ba665fb8fb21a4cfe8bcacb7d87300f8046a558a3fce4\",\"kind=1&created_at<1678659553\",\"435091ab4c4a11e594b1a05e0fa6c2f6e3b6eaa87c53f2981a3d6980858c40fdcaffde9a4c461f352a109402a4278ff4dbf90f9ebd05f96dac5ae36a6364a976\"]");
    }

    #[test]
    fn test_delegation_tag_from_str() {
        let tag_str = "[\"delegation\",\"1a459a8a6aa6441d480ba665fb8fb21a4cfe8bcacb7d87300f8046a558a3fce4\",\"kind=1&created_at>1676067553&created_at<1678659553\",\"369aed09c1ad52fceb77ecd6c16f2433eac4a3803fc41c58876a5b60f4f36b9493d5115e5ec5a0ce6c3668ffe5b58d47f2cbc97233833bb7e908f66dbbbd9d36\"]";

        let tag = DelegationTag::from_str(tag_str).unwrap();

        assert_eq!(tag.to_string(), tag_str);
        assert_eq!(
            tag.conditions().to_string(),
            "kind=1&created_at>1676067553&created_at<1678659553"
        );
        assert_eq!(
            tag.delegator_pubkey().to_string().as_str(),
            "1a459a8a6aa6441d480ba665fb8fb21a4cfe8bcacb7d87300f8046a558a3fce4"
        );
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_validate_delegation_tag_negative() {
        let delegator_secret_key =
            SecretKey::from_str("b2f3673ee3a659283e6599080e0ab0e669a3c2640914375a9b0b357faae08b17")
                .unwrap();
        let delegator_keys = Keys::new(delegator_secret_key);
        let delegatee_pubkey = XOnlyPublicKey::from_str(
            "bea8aeb6c1657e33db5ac75a83910f77e8ec6145157e476b5b88c6e85b1fab34",
        )
        .unwrap();
        let conditions =
            Conditions::from_str("kind=1&created_at>1676067553&created_at<1678659553").unwrap();

        let tag = DelegationTag::new(&delegator_keys, delegatee_pubkey, conditions).unwrap();

        // positive
        assert!(tag
            .validate(delegatee_pubkey, &EventProperties::new(1, 1677000000))
            .is_ok());

        // signature verification fails if wrong delegatee key is given
        let wrong_pubkey = XOnlyPublicKey::from_str(
            "14b91c20c0287495615210ef7772192d43eca6d2a34342e723bd237035e7955b",
        )
        .unwrap();

        // Note: Error cannot be tested simply  using equality
        match tag
            .validate(wrong_pubkey, &EventProperties::new(1, 1677000000))
            .err()
            .unwrap()
        {
            Error::ConditionsValidation(e) => assert_eq!(e, ValidationError::InvalidSignature),
            _ => panic!("Expected ConditionsValidation"),
        }

        // wrong event kind
        match tag
            .validate(delegatee_pubkey, &EventProperties::new(9, 1677000000))
            .err()
            .unwrap()
        {
            Error::ConditionsValidation(e) => assert_eq!(e, ValidationError::InvalidKind),
            _ => panic!("Expected ConditionsValidation"),
        };

        // wrong creation time
        match tag
            .validate(delegatee_pubkey, &EventProperties::new(1, 1679000000))
            .err()
            .unwrap()
        {
            Error::ConditionsValidation(e) => assert_eq!(e, ValidationError::CreatedTooLate),
            _ => panic!("Expected ConditionsValidation"),
        };
    }

    #[test]
    fn test_conditions_to_string() {
        let mut c = Conditions::new();
        c.add(Condition::Kind(1));
        assert_eq!(c.to_string(), "kind=1");
        c.add(Condition::CreatedAfter(1674834236));
        c.add(Condition::CreatedBefore(1677426236));
        assert_eq!(
            c.to_string(),
            "kind=1&created_at>1674834236&created_at<1677426236"
        );
    }

    #[test]
    fn test_conditions_parse() {
        let c = Conditions::from_str("kind=1&created_at>1674834236&created_at<1677426236").unwrap();
        assert_eq!(
            c.to_string(),
            "kind=1&created_at>1674834236&created_at<1677426236"
        );

        // special: empty string
        let c_empty = Conditions::from_str("").unwrap();
        assert_eq!(c_empty.to_string(), "");

        // one condition
        let c_one = Conditions::from_str("created_at<10000").unwrap();
        assert_eq!(c_one.to_string(), "created_at<10000");
    }

    #[test]
    fn test_conditions_parse_negative() {
        match Conditions::from_str("__invalid_condition__&kind=1")
            .err()
            .unwrap()
        {
            Error::ConditionsParseInvalidCondition => {}
            _ => panic!("Expected ConditionsParseInvalidCondition"),
        }
        match Conditions::from_str("kind=__invalid_number__")
            .err()
            .unwrap()
        {
            Error::ConditionsParseNumeric(_) => {}
            _ => panic!("Expected ConditionsParseNumeric"),
        }
    }

    #[test]
    fn test_conditions_evaluate() {
        let c_kind = Conditions::from_str("kind=3").unwrap();
        assert!(c_kind.evaluate(&EventProperties::new(3, 0)).is_ok());
        assert_eq!(
            c_kind.evaluate(&EventProperties::new(5, 0)).err().unwrap(),
            ValidationError::InvalidKind
        );

        let c_impossible = Conditions::from_str("kind=3&kind=4").unwrap();
        assert_eq!(
            c_impossible
                .evaluate(&EventProperties::new(3, 0))
                .err()
                .unwrap(),
            ValidationError::InvalidKind
        );

        let c_before = Conditions::from_str("created_at<1000").unwrap();
        assert!(c_before.evaluate(&EventProperties::new(3, 500)).is_ok());
        assert_eq!(
            c_before
                .evaluate(&EventProperties::new(3, 2000))
                .err()
                .unwrap(),
            ValidationError::CreatedTooLate
        );

        let c_after = Conditions::from_str("created_at>1000").unwrap();
        assert!(c_after.evaluate(&EventProperties::new(3, 2000)).is_ok());
        assert_eq!(
            c_after
                .evaluate(&EventProperties::new(3, 500))
                .err()
                .unwrap(),
            ValidationError::CreatedTooEarly
        );

        let c_complex =
            Conditions::from_str("kind=1&created_at>1676067553&created_at<1678659553").unwrap();
        assert!(c_complex
            .evaluate(&EventProperties::new(1, 1677000000))
            .is_ok());
        //assert_eq!(c_complex.evaluate(&EventProperties{ kind: 1, created_time: 1677000000}).err().unwrap(), ValidationError::InvalidKind);
        assert_eq!(
            c_complex
                .evaluate(&EventProperties::new(5, 1677000000))
                .err()
                .unwrap(),
            ValidationError::InvalidKind
        );
        assert_eq!(
            c_complex
                .evaluate(&EventProperties::new(1, 1674000000))
                .err()
                .unwrap(),
            ValidationError::CreatedTooEarly
        );
        assert_eq!(
            c_complex
                .evaluate(&EventProperties::new(1, 1699000000))
                .err()
                .unwrap(),
            ValidationError::CreatedTooLate
        );
    }
}
