//! NIP-60: Cashu Wallets
//!
//! <https://github.com/nostr-protocol/nips/blob/master/60.md>

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;

use serde::{Deserialize, Serialize};

use super::nip44;
use crate::event::{self, tag, Event, EventId, TagKind};
#[cfg(feature = "std")]
use crate::event::{EventBuilder, Kind, Tag};
use crate::key::{PublicKey, SecretKey};
use crate::types::time::Timestamp;
use crate::types::url::{ParseError, Url};

const E_TAG_STR: &str = "e";
const PRIVKEY: &str = "privkey";
const MINT: &str = "mint";
const DIRECTION: &str = "direction";
const AMOUNT: &str = "amount";
const EVENT_MARKER_CREATED: &str = "created";
const EVENT_MARKER_DESTROYED: &str = "destroyed";
const EVENT_MARKER_REDEEMED: &str = "redeemed";

/// NIP60 error
#[derive(Debug)]
pub enum Error {
    /// NIP44 error
    Nip44(nip44::Error),
    /// JSON error
    Json(serde_json::Error),
    /// Event error
    Event(event::Error),
    /// Tag error
    Tag(tag::Error),
    /// URL error
    Url(ParseError),
    /// Invalid direction
    InvalidDirection,
    /// Found multiple private keys
    FoundMultiplePrivKeys,
    /// Missing required field
    MissingField(String),
    /// Invalid amount
    InvalidAmount,
    /// Missing mint tag
    MissingMintTag,
    /// Invalid mint URL
    InvalidMintUrl,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nip44(e) => e.fmt(f),
            Self::Json(e) => e.fmt(f),
            Self::Event(e) => e.fmt(f),
            Self::Tag(e) => e.fmt(f),
            Self::Url(e) => e.fmt(f),
            Self::InvalidDirection => f.write_str("Invalid direction"),
            Self::FoundMultiplePrivKeys => f.write_str("Found multiple private keys"),
            Self::MissingField(field) => write!(f, "Missing required field: {field}"),
            Self::InvalidAmount => f.write_str("Invalid amount"),
            Self::MissingMintTag => f.write_str("Missing mint tag"),
            Self::InvalidMintUrl => f.write_str("Invalid mint URL"),
        }
    }
}

impl From<nip44::Error> for Error {
    fn from(e: nip44::Error) -> Self {
        Self::Nip44(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<event::Error> for Error {
    fn from(e: event::Error) -> Self {
        Self::Event(e)
    }
}

impl From<tag::Error> for Error {
    fn from(e: tag::Error) -> Self {
        Self::Tag(e)
    }
}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Self::Url(e)
    }
}

/// Cashu proof
///
/// <https://github.com/nostr-protocol/nips/blob/master/60.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CashuProof {
    /// Proof ID
    pub id: String,
    /// Amount in sats
    pub amount: u64,
    /// Secret
    pub secret: String,
    /// C value
    pub c: String,
}

/// Wallet event
///
/// <https://github.com/nostr-protocol/nips/blob/master/60.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WalletEvent {
    /// Private key used to unlock P2PK ecash
    pub privkey: String,
    /// Mint URLs this wallet uses
    pub mints: Vec<Url>,
}

impl WalletEvent {
    /// Create new wallet event
    pub fn new<S>(privkey: S, mints: Vec<Url>) -> Self
    where
        S: Into<String>,
    {
        Self {
            privkey: privkey.into(),
            mints,
        }
    }

    /// Parse from wallet event
    pub fn from_wallet_event(
        secret_key: &SecretKey,
        public_key: &PublicKey,
        event: &Event,
    ) -> Result<Self, Error> {
        let decrypted: String = nip44::decrypt(secret_key, public_key, &event.content)?;
        let wallet_data: Vec<Vec<String>> = serde_json::from_str(&decrypted)?;

        let mut privkey: String = String::new();
        let mut mints: Vec<Url> = Vec::new();

        for item in wallet_data.into_iter() {
            let mut iter = item.into_iter();

            if let (Some(key), Some(value)) = (iter.next(), iter.next()) {
                match key.as_str() {
                    PRIVKEY => {
                        if privkey.is_empty() {
                            privkey = value
                        } else {
                            return Err(Error::FoundMultiplePrivKeys);
                        }
                    }
                    MINT => {
                        if let Ok(mint_url) = Url::parse(&value) {
                            mints.push(mint_url);
                        }
                    }
                    _ => {}
                }
            }
        }

        if privkey.is_empty() {
            return Err(Error::MissingField(PRIVKEY.to_string()));
        }

        if mints.is_empty() {
            return Err(Error::MissingField(MINT.to_string()));
        }

        Ok(Self { privkey, mints })
    }

    #[cfg(feature = "std")]
    fn to_encrypted_content(
        &self,
        secret_key: &SecretKey,
        public_key: &PublicKey,
    ) -> Result<String, Error> {
        let mut wallet_data: Vec<Vec<&str>> = vec![vec![PRIVKEY, &self.privkey]];

        // Add each mint URL as a separate entry
        for mint in self.mints.iter() {
            wallet_data.push(vec![MINT, mint.as_str()]);
        }

        let json: String = serde_json::to_string(&wallet_data)?;

        Ok(nip44::encrypt(
            secret_key,
            public_key,
            json,
            nip44::Version::V2,
        )?)
    }

    /// Convert to [`EventBuilder`].
    #[cfg(feature = "std")]
    pub fn to_event_builder(
        &self,
        secret_key: &SecretKey,
        public_key: &PublicKey,
    ) -> Result<EventBuilder, Error> {
        // Build event content
        let content: String = self.to_encrypted_content(secret_key, public_key)?;

        // Construct event builder
        Ok(EventBuilder::new(Kind::CashuWallet, content))
    }
}

/// Token event
///
/// <https://github.com/nostr-protocol/nips/blob/master/60.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TokenEvent {
    /// Mint URL
    pub mint: Url,
    /// Unspent proofs
    pub proofs: Vec<CashuProof>,
    /// Token event IDs that were destroyed
    pub del: Vec<String>,
}

impl TokenEvent {
    /// Create new token event data
    pub fn new(mint: Url, proofs: Vec<CashuProof>) -> Self {
        Self {
            mint,
            proofs,
            del: Vec::new(),
        }
    }

    /// Parse from token event
    pub fn from_token_event(
        secret_key: &SecretKey,
        public_key: &PublicKey,
        event: &Event,
    ) -> Result<Self, Error> {
        let decrypted: String = nip44::decrypt(secret_key, public_key, &event.content)?;
        Ok(serde_json::from_str(&decrypted)?)
    }

    /// Add destroyed token event ID
    pub fn destroyed(mut self, event_id: String) -> Self {
        self.del.push(event_id);
        self
    }

    #[cfg(feature = "std")]
    fn to_encrypted_content(
        &self,
        secret_key: &SecretKey,
        public_key: &PublicKey,
    ) -> Result<String, Error> {
        let json: String = serde_json::to_string(self)?;
        Ok(nip44::encrypt(
            secret_key,
            public_key,
            json,
            nip44::Version::V2,
        )?)
    }

    /// Convert to [`EventBuilder`].
    #[cfg(feature = "std")]
    pub fn to_event_builder(
        &self,
        secret_key: &SecretKey,
        public_key: &PublicKey,
    ) -> Result<EventBuilder, Error> {
        // Build event content
        let content: String = self.to_encrypted_content(secret_key, public_key)?;

        // Construct event builder
        Ok(EventBuilder::new(Kind::CashuWalletUnspentProof, content))
    }
}

/// Transaction direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TransactionDirection {
    /// Received funds
    In,
    /// Sent funds
    Out,
}

impl fmt::Display for TransactionDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TransactionDirection {
    /// Get as `&str`.
    pub fn as_str(&self) -> &str {
        match self {
            Self::In => "in",
            Self::Out => "out",
        }
    }
}

impl FromStr for TransactionDirection {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "in" => Ok(Self::In),
            "out" => Ok(Self::Out),
            _ => Err(Error::InvalidDirection),
        }
    }
}

/// Spending history event data (kind: 7376)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpendingHistory {
    /// Transaction direction
    pub direction: TransactionDirection,
    /// Amount in sats
    pub amount: u64,
    /// Created event IDs
    pub created: Vec<EventId>,
    /// Destroyed event IDs
    pub destroyed: Vec<EventId>,
    /// Redeemed event IDs
    pub redeemed: Vec<EventId>,
}

impl SpendingHistory {
    /// Create new spending history data
    pub fn new(direction: TransactionDirection, amount: u64) -> Self {
        Self {
            direction,
            amount,
            created: Vec::new(),
            destroyed: Vec::new(),
            redeemed: Vec::new(),
        }
    }

    /// Parse from spending history event
    pub fn from_spending_history_event(
        secret_key: &SecretKey,
        public_key: &PublicKey,
        event: &Event,
    ) -> Result<Self, Error> {
        let decrypted: String = nip44::decrypt(secret_key, public_key, &event.content)?;
        let data: Vec<Vec<String>> = serde_json::from_str(&decrypted)?;

        let mut direction = None;
        let mut amount = None;
        let mut created: Vec<EventId> = Vec::new();
        let mut destroyed: Vec<EventId> = Vec::new();
        let mut redeemed: Vec<EventId> = Vec::new();

        // Parse encrypted content for created/destroyed events
        for item in data {
            if item.len() >= 2 {
                match item[0].as_str() {
                    DIRECTION => {
                        direction = Some(TransactionDirection::from_str(&item[1])?);
                    }
                    AMOUNT => {
                        amount = Some(item[1].parse().map_err(|_| Error::InvalidAmount)?);
                    }
                    E_TAG_STR if item.len() >= 4 => {
                        let event_id: EventId = EventId::from_hex(&item[1])?;
                        let marker: &str = &item[3];
                        match marker {
                            EVENT_MARKER_CREATED => created.push(event_id),
                            EVENT_MARKER_DESTROYED => destroyed.push(event_id),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }

        // Check unencrypted e tags for redeemed events
        for tag in event.tags.iter() {
            let slice = tag.as_slice();
            if slice.len() >= 4 && slice[0] == E_TAG_STR && slice[3] == EVENT_MARKER_REDEEMED {
                redeemed.push(EventId::from_hex(&slice[1])?);
            }
        }

        let direction: TransactionDirection =
            direction.ok_or_else(|| Error::MissingField(DIRECTION.to_string()))?;
        let amount: u64 = amount.ok_or_else(|| Error::MissingField(AMOUNT.to_string()))?;

        Ok(Self {
            direction,
            amount,
            created,
            destroyed,
            redeemed,
        })
    }

    /// Add created event ID
    pub fn add_created(mut self, id: EventId) -> Self {
        self.created.push(id);
        self
    }

    /// Add destroyed event ID
    pub fn add_destroyed(mut self, id: EventId) -> Self {
        self.destroyed.push(id);
        self
    }

    /// Add redeemed event ID
    pub fn add_redeemed(mut self, id: EventId) -> Self {
        self.redeemed.push(id);
        self
    }

    #[cfg(feature = "std")]
    fn to_encrypted_content(
        &self,
        secret_key: &SecretKey,
        public_key: &PublicKey,
    ) -> Result<String, Error> {
        let mut data: Vec<Vec<String>> = vec![
            vec![DIRECTION.to_string(), self.direction.to_string()],
            vec![AMOUNT.to_string(), self.amount.to_string()],
        ];

        // Add created event references (encrypted)
        for event_id in &self.created {
            let tag: Tag = Tag::custom(
                TagKind::e(),
                [
                    event_id.to_hex(),
                    String::new(),
                    EVENT_MARKER_CREATED.to_string(),
                ],
            );
            data.push(tag.to_vec());
        }

        // Add destroyed event references (encrypted)
        for event_id in &self.destroyed {
            let tag: Tag = Tag::custom(
                TagKind::e(),
                [
                    event_id.to_hex(),
                    String::new(),
                    EVENT_MARKER_DESTROYED.to_string(),
                ],
            );
            data.push(tag.to_vec());
        }

        let json: String = serde_json::to_string(&data)?;

        Ok(nip44::encrypt(
            secret_key,
            public_key,
            json,
            nip44::Version::V2,
        )?)
    }

    /// Convert to event builder
    #[cfg(feature = "std")]
    pub fn to_event_builder(
        &self,
        secret_key: &SecretKey,
        public_key: &PublicKey,
    ) -> Result<EventBuilder, Error> {
        let content: String = self.to_encrypted_content(secret_key, public_key)?;

        let mut tags: Vec<Tag> = Vec::with_capacity(self.redeemed.len());

        // Add redeemed event tags (unencrypted)
        for event_id in self.redeemed.iter() {
            tags.push(Tag::parse([
                "e",
                &event_id.to_hex(),
                "",
                EVENT_MARKER_REDEEMED,
            ])?);
        }

        Ok(EventBuilder::new(Kind::CashuWalletSpendingHistory, content).tags(tags))
    }
}

/// Quote event
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct QuoteEvent {
    /// Quote ID
    pub quote_id: String,
    /// Mint URL
    pub mint: Url,
    /// Expiration timestamp
    pub expiration: Option<Timestamp>,
}

impl QuoteEvent {
    /// Create new quote event data
    pub fn new<S>(quote_id: S, mint: Url) -> Self
    where
        S: Into<String>,
    {
        Self {
            quote_id: quote_id.into(),
            mint,
            expiration: None,
        }
    }

    /// Parse from quote event
    pub fn from_quote_event(
        secret_key: &SecretKey,
        public_key: &PublicKey,
        event: &Event,
    ) -> Result<Self, Error> {
        let quote_id: String = nip44::decrypt(secret_key, public_key, &event.content)?;

        // Extract mint URL from tags
        let mint: Url = event
            .tags
            .find(TagKind::custom(MINT))
            .and_then(|tag| tag.content())
            .ok_or(Error::MissingMintTag)?
            .parse()
            .map_err(|_| Error::InvalidMintUrl)?;

        // Extract NIP-40 expiration from tags if present
        let expiration: Option<Timestamp> = event.tags.expiration().copied();

        Ok(Self {
            quote_id,
            mint,
            expiration,
        })
    }

    /// Set expiration timestamp
    pub fn expiration(mut self, expiration: Timestamp) -> Self {
        self.expiration = Some(expiration);
        self
    }

    #[cfg(feature = "std")]
    fn to_encrypted_content(
        &self,
        secret_key: &SecretKey,
        public_key: &PublicKey,
    ) -> Result<String, Error> {
        Ok(nip44::encrypt(
            secret_key,
            public_key,
            &self.quote_id,
            nip44::Version::V2,
        )?)
    }

    /// Convert to event builder
    #[cfg(feature = "std")]
    pub fn to_event_builder(
        &self,
        secret_key: &SecretKey,
        public_key: &PublicKey,
    ) -> Result<EventBuilder, Error> {
        let content: String = self.to_encrypted_content(secret_key, public_key)?;

        let mut tags: Vec<Tag> = Vec::with_capacity(2);

        // Add mint tag
        tags.push(Tag::custom(TagKind::custom(MINT), [self.mint.as_str()]));

        // Add NIP-40 expiration tag (current time + 2 weeks)
        let expiration: Timestamp = Timestamp::now() + 14 * 24 * 60 * 60; // 2 weeks in seconds
        tags.push(Tag::expiration(expiration));

        Ok(EventBuilder::new(Kind::CashuWalletQuote, content).tags(tags))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_direction() {
        assert_eq!(TransactionDirection::In.to_string(), "in");
        assert_eq!(TransactionDirection::Out.to_string(), "out");

        assert_eq!(
            TransactionDirection::from_str("in").unwrap(),
            TransactionDirection::In
        );
        assert_eq!(
            TransactionDirection::from_str("out").unwrap(),
            TransactionDirection::Out
        );
        assert!(TransactionDirection::from_str("invalid").is_err());
    }

    #[test]
    fn test_wallet_event() {
        let mint_url = Url::parse("https://example.com").unwrap();
        let wallet = WalletEvent::new("test_privkey", vec![mint_url.clone()]);

        assert_eq!(wallet.privkey, "test_privkey");
        assert_eq!(wallet.mints.len(), 1);
        assert_eq!(wallet.mints[0], mint_url);
    }

    #[test]
    fn test_token_event_data() {
        let mint_url = Url::parse("https://example.com").unwrap();
        let proof = CashuProof {
            id: "test_id".to_string(),
            amount: 100,
            secret: "test_secret".to_string(),
            c: "test_c".to_string(),
        };

        let token_data = TokenEvent::new(mint_url.clone(), vec![proof.clone()]);

        assert_eq!(token_data.mint, mint_url);
        assert_eq!(token_data.proofs.len(), 1);
        assert_eq!(token_data.proofs[0], proof);
        assert!(token_data.del.is_empty());
    }

    #[test]
    fn test_spending_history_data() {
        let history = SpendingHistory::new(TransactionDirection::In, 50);

        assert_eq!(history.direction, TransactionDirection::In);
        assert_eq!(history.amount, 50);
        assert!(history.created.is_empty());
        assert!(history.destroyed.is_empty());
        assert!(history.redeemed.is_empty());
    }

    #[test]
    fn test_spending_history_with_references() {
        let id1 = EventId::all_zeros();
        let id2 = EventId::from_slice(&[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 1,
        ])
        .unwrap();
        let id3 = EventId::from_slice(&[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 2,
        ])
        .unwrap();

        let history = SpendingHistory::new(TransactionDirection::In, 50)
            .add_created(id1)
            .add_destroyed(id2)
            .add_redeemed(id3);

        assert_eq!(history.direction, TransactionDirection::In);
        assert_eq!(history.amount, 50);
        assert_eq!(history.created.len(), 1);
        assert_eq!(history.destroyed.len(), 1);
        assert_eq!(history.redeemed.len(), 1);
        assert_eq!(history.created[0], id1);
        assert_eq!(history.destroyed[0], id2);
        assert_eq!(history.redeemed[0], id3);
    }

    #[test]
    fn test_quote_event_data() {
        let mint_url = Url::parse("https://example.com").unwrap();
        let quote = QuoteEvent::new("test_quote_id", mint_url.clone());

        assert_eq!(quote.quote_id, "test_quote_id");
        assert_eq!(quote.mint, mint_url);
        assert!(quote.expiration.is_none());
    }
}
