//! NIP-60: Cashu Wallets
//!
//! <https://github.com/nostr-protocol/nips/blob/master/60.md>

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;

use serde::{Deserialize, Serialize};

#[cfg(feature = "nip44")]
use crate::event::{EventBuilder, Kind, Tag};
#[cfg(feature = "nip44")]
use crate::key::PublicKey;
#[cfg(feature = "nip44")]
use crate::prelude::nip44;
use crate::types::Url;
use crate::types::url::ParseError;
use crate::Timestamp;

/// NIP60 error
#[derive(Debug)]
pub enum Error {
    /// NIP44 error
    #[cfg(feature = "nip44")]
    Nip44(nip44::Error),
    /// JSON error
    Json(serde_json::Error),
    /// Tag error
    Tag(crate::event::tag::Error),
    /// URL error
    Url(ParseError),
    /// Invalid direction
    InvalidDirection,
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
            #[cfg(feature = "nip44")]
            Self::Nip44(e) => write!(f, "{e}"),
            Self::Json(e) => write!(f, "{e}"),
            Self::Tag(e) => write!(f, "{e}"),
            Self::Url(e) => write!(f, "{e}"),
            Self::InvalidDirection => write!(f, "Invalid direction"),
            Self::MissingField(field) => write!(f, "Missing required field: {field}"),
            Self::InvalidAmount => write!(f, "Invalid amount"),
            Self::MissingMintTag => write!(f, "Missing mint tag"),
            Self::InvalidMintUrl => write!(f, "Invalid mint URL"),
        }
    }
}

#[cfg(feature = "nip44")]
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

impl From<crate::event::tag::Error> for Error {
    fn from(e: crate::event::tag::Error) -> Self {
        Self::Tag(e)
    }
}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Self::Url(e)
    }
}

/// Cashu proof
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// Wallet event (kind: 17375)
#[derive(Debug, Clone, PartialEq, Eq)]
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

    /// Add mint URL
    pub fn mint(mut self, mint: Url) -> Self {
        self.mints.push(mint);
        self
    }

    /// Convert to encrypted content for event
    #[cfg(feature = "nip44")]
    pub fn to_encrypted_content(
        &self,
        secret_key: &crate::key::SecretKey,
        public_key: &PublicKey,
    ) -> Result<String, Error> {
        let mut wallet_data = vec![vec!["privkey".to_string(), self.privkey.clone()]];

        // Add each mint URL as a separate entry
        for mint in &self.mints {
            wallet_data.push(vec!["mint".to_string(), mint.to_string()]);
        }

        let json = serde_json::to_string(&wallet_data)?;
        Ok(nip44::encrypt(
            secret_key,
            public_key,
            json,
            nip44::Version::V2,
        )?)
    }

    /// Parse from wallet event
    #[cfg(feature = "nip44")]
    pub fn from_wallet_event(
        event: &crate::event::Event,
        secret_key: &crate::key::SecretKey,
        public_key: &PublicKey,
    ) -> Result<Self, Error> {
        let decrypted = nip44::decrypt(secret_key, public_key, &event.content)?;
        let wallet_data: Vec<Vec<String>> = serde_json::from_str(&decrypted)?;

        let mut privkey = String::new();
        let mut mints = Vec::new();

        for item in wallet_data {
            if item.len() >= 2 {
                match item[0].as_str() {
                    "privkey" => privkey = item[1].clone(),
                    "mint" => {
                        if let Ok(mint_url) = Url::parse(&item[1]) {
                            mints.push(mint_url);
                        }
                    }
                    _ => {}
                }
            }
        }

        if privkey.is_empty() {
            return Err(Error::MissingField("privkey".to_string()));
        }

        if mints.is_empty() {
            return Err(Error::MissingField("mints".to_string()));
        }

        Ok(Self { privkey, mints })
    }

    /// Convert to event builder
    #[cfg(feature = "nip44")]
    pub fn to_event_builder(
        &self,
        secret_key: &crate::key::SecretKey,
        public_key: &PublicKey,
    ) -> Result<EventBuilder, Error> {
        let content = self.to_encrypted_content(secret_key, public_key)?;
        let mut tags = Vec::new();

        // Add mint tags
        for mint in &self.mints {
            tags.push(Tag::parse(["mint", &mint.to_string()])?);
        }

        Ok(EventBuilder::new(Kind::Custom(17375), content).tags(tags))
    }
}

/// Token event data (kind: 7375)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenEventData {
    /// Mint URL
    pub mint: Url,
    /// Unspent proofs
    pub proofs: Vec<CashuProof>,
    /// Token event IDs that were destroyed
    pub del: Vec<String>,
}

impl TokenEventData {
    /// Create new token event data
    pub fn new(mint: Url, proofs: Vec<CashuProof>) -> Self {
        Self {
            mint,
            proofs,
            del: Vec::new(),
        }
    }

    /// Add destroyed token event ID
    pub fn destroyed(mut self, event_id: String) -> Self {
        self.del.push(event_id);
        self
    }

    /// Convert to encrypted content for event
    #[cfg(feature = "nip44")]
    pub fn to_encrypted_content(
        &self,
        secret_key: &crate::key::SecretKey,
        public_key: &PublicKey,
    ) -> Result<String, Error> {
        let json = serde_json::to_string(self)?;
        Ok(nip44::encrypt(
            secret_key,
            public_key,
            json,
            nip44::Version::V2,
        )?)
    }

    /// Parse from token event
    #[cfg(feature = "nip44")]
    pub fn from_token_event(
        event: &crate::event::Event,
        secret_key: &crate::key::SecretKey,
        public_key: &PublicKey,
    ) -> Result<Self, Error> {
        let decrypted = nip44::decrypt(secret_key, public_key, &event.content)?;
        Ok(serde_json::from_str(&decrypted)?)
    }

    /// Convert to event builder
    #[cfg(feature = "nip44")]
    pub fn to_event_builder(
        &self,
        secret_key: &crate::key::SecretKey,
        public_key: &PublicKey,
    ) -> Result<EventBuilder, Error> {
        let content = self.to_encrypted_content(secret_key, public_key)?;
        let mut tags = Vec::new();

        // Add mint tag
        tags.push(Tag::parse(["mint", &self.mint.to_string()])?);

        Ok(EventBuilder::new(Kind::Custom(7375), content).tags(tags))
    }
}

/// Transaction direction
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionDirection {
    /// Received funds
    In,
    /// Sent funds
    Out,
}

impl fmt::Display for TransactionDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::In => write!(f, "in"),
            Self::Out => write!(f, "out"),
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

/// Event reference marker
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventMarker {
    /// A new token event was created
    Created,
    /// A token event was destroyed
    Destroyed,
    /// A NIP-61 nutzap was redeemed
    Redeemed,
}

impl fmt::Display for EventMarker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Created => write!(f, "created"),
            Self::Destroyed => write!(f, "destroyed"),
            Self::Redeemed => write!(f, "redeemed"),
        }
    }
}

impl FromStr for EventMarker {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "created" => Ok(Self::Created),
            "destroyed" => Ok(Self::Destroyed),
            "redeemed" => Ok(Self::Redeemed),
            _ => Err(Error::InvalidDirection), // Reuse existing error type
        }
    }
}

/// Event reference
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventReference {
    /// Event ID
    pub event_id: String,
    /// Marker indicating the meaning of the reference
    pub marker: EventMarker,
}

/// Spending history event data (kind: 7376)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpendingHistoryData {
    /// Transaction direction
    pub direction: TransactionDirection,
    /// Amount in sats
    pub amount: u64,
    /// Created event IDs
    pub created: Vec<String>,
    /// Destroyed event IDs
    pub destroyed: Vec<String>,
    /// Redeemed event IDs
    pub redeemed: Vec<String>,
}

impl SpendingHistoryData {
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

    /// Add created event ID
    pub fn add_created(mut self, event_id: String) -> Self {
        self.created.push(event_id);
        self
    }

    /// Add destroyed event ID
    pub fn add_destroyed(mut self, event_id: String) -> Self {
        self.destroyed.push(event_id);
        self
    }

    /// Add redeemed event ID
    pub fn add_redeemed(mut self, event_id: String) -> Self {
        self.redeemed.push(event_id);
        self
    }

    /// Convert to encrypted content for event
    #[cfg(feature = "nip44")]
    pub fn to_encrypted_content(
        &self,
        secret_key: &crate::key::SecretKey,
        public_key: &PublicKey,
    ) -> Result<String, Error> {
        let mut data = vec![
            vec!["direction".to_string(), self.direction.to_string()],
            vec!["amount".to_string(), self.amount.to_string()],
        ];

        // Add created event references (encrypted)
        for event_id in &self.created {
            data.push(vec![
                "e".to_string(),
                event_id.clone(),
                "".to_string(),
                "created".to_string(),
            ]);
        }

        // Add destroyed event references (encrypted)
        for event_id in &self.destroyed {
            data.push(vec![
                "e".to_string(),
                event_id.clone(),
                "".to_string(),
                "destroyed".to_string(),
            ]);
        }

        let json = serde_json::to_string(&data)?;
        Ok(nip44::encrypt(
            secret_key,
            public_key,
            json,
            nip44::Version::V2,
        )?)
    }

    /// Parse from spending history event
    #[cfg(feature = "nip44")]
    pub fn from_spending_history_event(
        event: &crate::event::Event,
        secret_key: &crate::key::SecretKey,
        public_key: &PublicKey,
    ) -> Result<Self, Error> {
        let decrypted = nip44::decrypt(secret_key, public_key, &event.content)?;
        let data: Vec<Vec<String>> = serde_json::from_str(&decrypted)?;

        let mut direction = None;
        let mut amount = None;
        let mut created = Vec::new();
        let mut destroyed = Vec::new();
        let mut redeemed = Vec::new();

        // Parse encrypted content for created/destroyed events
        for item in data {
            if item.len() >= 2 {
                match item[0].as_str() {
                    "direction" => {
                        direction = Some(TransactionDirection::from_str(&item[1])?);
                    }
                    "amount" => {
                        amount = Some(item[1].parse().map_err(|_| Error::InvalidAmount)?);
                    }
                    "e" if item.len() >= 4 => {
                        let event_id = item[1].clone();
                        let marker = item[3].clone();
                        match marker.as_str() {
                            "created" => created.push(event_id),
                            "destroyed" => destroyed.push(event_id),
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
            if slice.len() >= 4 && slice[0] == "e" && slice[3] == "redeemed" {
                redeemed.push(slice[1].clone());
            }
        }

        let direction = direction.ok_or_else(|| Error::MissingField("direction".to_string()))?;
        let amount = amount.ok_or_else(|| Error::MissingField("amount".to_string()))?;

        Ok(Self {
            direction,
            amount,
            created,
            destroyed,
            redeemed,
        })
    }

    /// Convert to event builder
    #[cfg(feature = "nip44")]
    pub fn to_event_builder(
        &self,
        secret_key: &crate::key::SecretKey,
        public_key: &PublicKey,
    ) -> Result<EventBuilder, Error> {
        let content = self.to_encrypted_content(secret_key, public_key)?;
        let mut tags = Vec::new();

        // Add redeemed event tags (unencrypted)
        for event_id in &self.redeemed {
            tags.push(Tag::parse(["e", event_id, "", "redeemed"])?);
        }

        Ok(EventBuilder::new(Kind::Custom(7376), content).tags(tags))
    }
}

/// Quote event data (kind: 7374)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuoteEventData {
    /// Quote ID
    pub quote_id: String,
    /// Mint URL
    pub mint: Url,
    /// Expiration timestamp
    pub expiration: Option<Timestamp>,
}

impl QuoteEventData {
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

    /// Set expiration timestamp
    pub fn expiration(mut self, expiration: Timestamp) -> Self {
        self.expiration = Some(expiration);
        self
    }

    /// Convert to encrypted content for event
    #[cfg(feature = "nip44")]
    pub fn to_encrypted_content(
        &self,
        secret_key: &crate::key::SecretKey,
        public_key: &PublicKey,
    ) -> Result<String, Error> {
        Ok(nip44::encrypt(
            secret_key,
            public_key,
            &self.quote_id,
            nip44::Version::V2,
        )?)
    }

    /// Parse from quote event
    #[cfg(feature = "nip44")]
    pub fn from_quote_event(
        event: &crate::event::Event,
        secret_key: &crate::key::SecretKey,
        public_key: &PublicKey,
    ) -> Result<Self, Error> {
        let quote_id = nip44::decrypt(secret_key, public_key, &event.content)?;

        // Extract mint URL from tags
        let mint = event
            .tags
            .iter()
            .find(|tag| {
                let slice = tag.as_slice();
                slice.len() >= 2 && slice[0] == "mint"
            })
            .and_then(|tag| tag.as_slice().get(1))
            .ok_or(Error::MissingMintTag)?
            .parse()
            .map_err(|_| Error::InvalidMintUrl)?;

        // Extract NIP-40 expiration from tags if present
        let expiration = event.tags.expiration().copied();

        Ok(Self {
            quote_id,
            mint,
            expiration,
        })
    }

    /// Convert to event builder
    #[cfg(feature = "nip44")]
    pub fn to_event_builder(
        &self,
        secret_key: &crate::key::SecretKey,
        public_key: &PublicKey,
    ) -> Result<EventBuilder, Error> {
        let content = self.to_encrypted_content(secret_key, public_key)?;
        let mut tags = Vec::new();

        // Add mint tag
        tags.push(Tag::parse(["mint", &self.mint.to_string()])?);

        // Add NIP-40 expiration tag (current time + 2 weeks)
        let expiration = Timestamp::now() + 14 * 24 * 60 * 60; // 2 weeks in seconds
        tags.push(Tag::expiration(expiration));

        Ok(EventBuilder::new(Kind::Custom(7374), content).tags(tags))
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

        let token_data = TokenEventData::new(mint_url.clone(), vec![proof.clone()]);

        assert_eq!(token_data.mint, mint_url);
        assert_eq!(token_data.proofs.len(), 1);
        assert_eq!(token_data.proofs[0], proof);
        assert!(token_data.del.is_empty());
    }

    #[test]
    fn test_spending_history_data() {
        let history = SpendingHistoryData::new(TransactionDirection::In, 50);

        assert_eq!(history.direction, TransactionDirection::In);
        assert_eq!(history.amount, 50);
        assert!(history.created.is_empty());
        assert!(history.destroyed.is_empty());
        assert!(history.redeemed.is_empty());
    }

    #[test]
    fn test_event_marker() {
        assert_eq!(EventMarker::Created.to_string(), "created");
        assert_eq!(EventMarker::Destroyed.to_string(), "destroyed");
        assert_eq!(EventMarker::Redeemed.to_string(), "redeemed");

        assert_eq!(
            EventMarker::from_str("created").unwrap(),
            EventMarker::Created
        );
        assert_eq!(
            EventMarker::from_str("destroyed").unwrap(),
            EventMarker::Destroyed
        );
        assert_eq!(
            EventMarker::from_str("redeemed").unwrap(),
            EventMarker::Redeemed
        );
        assert!(EventMarker::from_str("invalid").is_err());
    }

    #[test]
    fn test_spending_history_with_references() {
        let history = SpendingHistoryData::new(TransactionDirection::In, 50)
            .add_created("event1".to_string())
            .add_destroyed("event2".to_string())
            .add_redeemed("event3".to_string());

        assert_eq!(history.direction, TransactionDirection::In);
        assert_eq!(history.amount, 50);
        assert_eq!(history.created.len(), 1);
        assert_eq!(history.destroyed.len(), 1);
        assert_eq!(history.redeemed.len(), 1);
        assert_eq!(history.created[0], "event1");
        assert_eq!(history.destroyed[0], "event2");
        assert_eq!(history.redeemed[0], "event3");
    }

    #[test]
    fn test_quote_event_data() {
        let mint_url = Url::parse("https://example.com").unwrap();
        let quote = QuoteEventData::new("test_quote_id", mint_url.clone());

        assert_eq!(quote.quote_id, "test_quote_id");
        assert_eq!(quote.mint, mint_url);
        assert!(quote.expiration.is_none());
    }
}
