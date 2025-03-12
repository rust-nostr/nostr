// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;
use core::str::FromStr;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::protocol::key::{JsPublicKey, JsSecretKey};
use crate::protocol::types::time::JsTimestamp;

/// NIP47 Response Error codes
#[wasm_bindgen(js_name = Nip47ErrorCode)]
pub enum JsErrorCode {
    ///  The client is sending commands too fast.
    RateLimited,
    /// The command is not known of is intentionally not implemented
    NotImplemented,
    /// The wallet does not have enough funds to cover a fee reserve or the payment amount
    InsufficientBalance,
    /// The payment failed. This may be due to a timeout, exhausting all routes, insufficient capacity or similar.
    PaymentFailed,
    /// The invoice could not be found by the given parameters.
    NotFound,
    /// The wallet has exceeded its spending quota
    QuotaExceeded,
    /// This public key is not allowed to do this operation
    Restricted,
    /// This public key has no wallet connected
    Unauthorized,
    /// An internal error
    Internal,
    /// Other error
    Other,
}

impl From<ErrorCode> for JsErrorCode {
    fn from(value: ErrorCode) -> Self {
        match value {
            ErrorCode::RateLimited => Self::RateLimited,
            ErrorCode::NotImplemented => Self::NotImplemented,
            ErrorCode::InsufficientBalance => Self::InsufficientBalance,
            ErrorCode::PaymentFailed => Self::PaymentFailed,
            ErrorCode::NotFound => Self::NotFound,
            ErrorCode::QuotaExceeded => Self::QuotaExceeded,
            ErrorCode::Restricted => Self::Restricted,
            ErrorCode::Unauthorized => Self::Unauthorized,
            ErrorCode::Internal => Self::Internal,
            ErrorCode::Other => Self::Other,
        }
    }
}

impl From<JsErrorCode> for ErrorCode {
    fn from(value: JsErrorCode) -> Self {
        match value {
            JsErrorCode::RateLimited => Self::RateLimited,
            JsErrorCode::NotImplemented => Self::NotImplemented,
            JsErrorCode::InsufficientBalance => Self::InsufficientBalance,
            JsErrorCode::PaymentFailed => Self::PaymentFailed,
            JsErrorCode::NotFound => Self::NotFound,
            JsErrorCode::QuotaExceeded => Self::QuotaExceeded,
            JsErrorCode::Restricted => Self::Restricted,
            JsErrorCode::Unauthorized => Self::Unauthorized,
            JsErrorCode::Internal => Self::Internal,
            JsErrorCode::Other => Self::Other,
        }
    }
}

/// Pay Invoice Request
#[derive(Clone)]
#[wasm_bindgen(js_name = PayInvoiceRequest)]
pub struct JsPayInvoiceRequest {
    /// Optional id
    #[wasm_bindgen(getter_with_clone)]
    pub id: Option<String>,
    /// Request invoice
    #[wasm_bindgen(getter_with_clone)]
    pub invoice: String,
    /// Optional amount in millisatoshis
    pub amount: Option<u64>,
}

impl From<PayInvoiceRequest> for JsPayInvoiceRequest {
    fn from(value: PayInvoiceRequest) -> Self {
        Self {
            id: value.id,
            invoice: value.invoice,
            amount: value.amount,
        }
    }
}

impl From<JsPayInvoiceRequest> for PayInvoiceRequest {
    fn from(value: JsPayInvoiceRequest) -> Self {
        Self {
            id: value.id,
            invoice: value.invoice,
            amount: value.amount,
        }
    }
}

/// Multi Pay Invoice Request Params
#[wasm_bindgen(js_name = MultiPayInvoiceRequest)]
pub struct JsMultiPayInvoiceRequest {
    /// Invoices to pay
    #[wasm_bindgen(getter_with_clone)]
    pub invoices: Vec<JsPayInvoiceRequest>,
}

impl From<MultiPayInvoiceRequest> for JsMultiPayInvoiceRequest {
    fn from(value: MultiPayInvoiceRequest) -> Self {
        Self {
            invoices: value.invoices.into_iter().map(|i| i.into()).collect(),
        }
    }
}

impl From<JsMultiPayInvoiceRequest> for MultiPayInvoiceRequest {
    fn from(value: JsMultiPayInvoiceRequest) -> Self {
        Self {
            invoices: value.invoices.into_iter().map(|i| i.into()).collect(),
        }
    }
}

/// TLVs to be added to the keysend payment
#[derive(Clone)]
#[wasm_bindgen(js_name = KeysendTLVRecord)]
pub struct JsKeysendTLVRecord {
    /// TLV type
    pub tlv_type: u64,
    /// TLV value
    #[wasm_bindgen(getter_with_clone)]
    pub value: String,
}

impl From<KeysendTLVRecord> for JsKeysendTLVRecord {
    fn from(value: KeysendTLVRecord) -> Self {
        Self {
            tlv_type: value.tlv_type,
            value: value.value,
        }
    }
}

impl From<JsKeysendTLVRecord> for KeysendTLVRecord {
    fn from(value: JsKeysendTLVRecord) -> Self {
        Self {
            tlv_type: value.tlv_type,
            value: value.value,
        }
    }
}

/// Pay Invoice Request Params
#[derive(Clone)]
#[wasm_bindgen(js_name = PayKeysendRequest)]
pub struct JsPayKeysendRequest {
    /// Optional id
    #[wasm_bindgen(getter_with_clone)]
    pub id: Option<String>,
    /// Amount in millisatoshis
    pub amount: u64,
    /// Receiver's node id
    #[wasm_bindgen(getter_with_clone)]
    pub pubkey: String,
    /// Optional preimage
    #[wasm_bindgen(getter_with_clone)]
    pub preimage: Option<String>,
    /// Optional TLVs to be added to the keysend payment
    #[wasm_bindgen(getter_with_clone)]
    pub tlv_records: Vec<JsKeysendTLVRecord>,
}

impl From<PayKeysendRequest> for JsPayKeysendRequest {
    fn from(value: PayKeysendRequest) -> Self {
        Self {
            id: value.id,
            amount: value.amount,
            pubkey: value.pubkey,
            preimage: value.preimage,
            tlv_records: value.tlv_records.into_iter().map(|t| t.into()).collect(),
        }
    }
}

impl From<JsPayKeysendRequest> for PayKeysendRequest {
    fn from(value: JsPayKeysendRequest) -> Self {
        Self {
            id: value.id,
            amount: value.amount,
            pubkey: value.pubkey,
            preimage: value.preimage,
            tlv_records: value.tlv_records.into_iter().map(|t| t.into()).collect(),
        }
    }
}

/// Multi Pay Keysend Request Params
#[derive(Clone)]
#[wasm_bindgen(js_name = MultiPayKeysendRequest)]
pub struct JsMultiPayKeysendRequest {
    /// Keysends
    #[wasm_bindgen(getter_with_clone)]
    pub keysends: Vec<JsPayKeysendRequest>,
}

impl From<MultiPayKeysendRequest> for JsMultiPayKeysendRequest {
    fn from(value: MultiPayKeysendRequest) -> Self {
        Self {
            keysends: value.keysends.into_iter().map(|i| i.into()).collect(),
        }
    }
}

impl From<JsMultiPayKeysendRequest> for MultiPayKeysendRequest {
    fn from(value: JsMultiPayKeysendRequest) -> Self {
        Self {
            keysends: value.keysends.into_iter().map(|i| i.into()).collect(),
        }
    }
}

/// Transaction Type
#[derive(Clone, Copy)]
#[wasm_bindgen(js_name = TransactionType)]
pub enum JsTransactionType {
    /// Incoming payments
    Incoming,
    /// Outgoing payments
    Outgoing,
}

impl From<JsTransactionType> for TransactionType {
    fn from(value: JsTransactionType) -> Self {
        match value {
            JsTransactionType::Incoming => Self::Incoming,
            JsTransactionType::Outgoing => Self::Outgoing,
        }
    }
}

impl From<TransactionType> for JsTransactionType {
    fn from(value: TransactionType) -> Self {
        match value {
            TransactionType::Incoming => Self::Incoming,
            TransactionType::Outgoing => Self::Outgoing,
        }
    }
}

/// Make Invoice Request Params
#[derive(Clone)]
#[wasm_bindgen(js_name = MakeInvoiceRequest)]
pub struct JsMakeInvoiceRequest {
    /// Amount in millisatoshis
    pub amount: u64,
    /// Invoice description
    #[wasm_bindgen(getter_with_clone)]
    pub description: Option<String>,
    /// Invoice description hash
    #[wasm_bindgen(getter_with_clone)]
    pub description_hash: Option<String>,
    /// Invoice expiry in seconds
    pub expiry: Option<u64>,
}

impl From<MakeInvoiceRequest> for JsMakeInvoiceRequest {
    fn from(value: MakeInvoiceRequest) -> Self {
        Self {
            amount: value.amount,
            description: value.description,
            description_hash: value.description_hash,
            expiry: value.expiry,
        }
    }
}

impl From<JsMakeInvoiceRequest> for MakeInvoiceRequest {
    fn from(value: JsMakeInvoiceRequest) -> Self {
        Self {
            amount: value.amount,
            description: value.description,
            description_hash: value.description_hash,
            expiry: value.expiry,
        }
    }
}

/// Lookup Invoice Request Params
#[derive(Clone)]
#[wasm_bindgen(js_name = LookupInvoiceRequest)]
pub struct JsLookupInvoiceRequest {
    /// Payment hash of invoice
    #[wasm_bindgen(getter_with_clone)]
    pub payment_hash: Option<String>,
    /// Bolt11 invoice
    #[wasm_bindgen(getter_with_clone)]
    pub invoice: Option<String>,
}

impl From<LookupInvoiceRequest> for JsLookupInvoiceRequest {
    fn from(value: LookupInvoiceRequest) -> Self {
        Self {
            payment_hash: value.payment_hash,
            invoice: value.invoice,
        }
    }
}

impl From<JsLookupInvoiceRequest> for LookupInvoiceRequest {
    fn from(value: JsLookupInvoiceRequest) -> Self {
        Self {
            payment_hash: value.payment_hash,
            invoice: value.invoice,
        }
    }
}

/// List Invoice Request Params
#[derive(Clone)]
#[wasm_bindgen(js_name = ListTransactionsRequest)]
pub struct JsListTransactionsRequest {
    /// Starting timestamp in seconds since epoch
    pub from: Option<JsTimestamp>,
    /// Ending timestamp in seconds since epoch
    pub until: Option<JsTimestamp>,
    /// Number of invoices to return
    pub limit: Option<u64>,
    /// Offset of the first invoice to return
    pub offset: Option<u64>,
    /// If true, include unpaid invoices
    pub unpaid: Option<bool>,
    /// [`TransactionType::Incoming`] for invoices, [`TransactionType::Outgoing`] for payments, [`None`] for both
    pub transaction_type: Option<JsTransactionType>,
}

impl From<ListTransactionsRequest> for JsListTransactionsRequest {
    fn from(value: ListTransactionsRequest) -> Self {
        Self {
            from: value.from.map(|t| t.into()),
            until: value.until.map(|t| t.into()),
            limit: value.limit,
            offset: value.offset,
            unpaid: value.unpaid,
            transaction_type: value.transaction_type.map(|t| t.into()),
        }
    }
}

impl From<JsListTransactionsRequest> for ListTransactionsRequest {
    fn from(value: JsListTransactionsRequest) -> Self {
        Self {
            from: value.from.map(|t| *t),
            until: value.until.map(|t| *t),
            limit: value.limit,
            offset: value.offset,
            unpaid: value.unpaid,
            transaction_type: value.transaction_type.map(|t| t.into()),
        }
    }
}

#[wasm_bindgen(js_name = PayInvoiceResponse)]
pub struct JsPayInvoiceResponse {
    /// Response preimage
    #[wasm_bindgen(getter_with_clone)]
    pub preimage: String,
}

impl From<PayInvoiceResponse> for JsPayInvoiceResponse {
    fn from(value: PayInvoiceResponse) -> Self {
        Self {
            preimage: value.preimage,
        }
    }
}

impl From<JsPayInvoiceResponse> for PayInvoiceResponse {
    fn from(value: JsPayInvoiceResponse) -> Self {
        Self {
            preimage: value.preimage,
        }
    }
}

#[wasm_bindgen(js_name = PayKeysendResponse)]
pub struct JsPayKeysendResponse {
    /// Response preimage
    #[wasm_bindgen(getter_with_clone)]
    pub preimage: String,
}

impl From<PayKeysendResponse> for JsPayKeysendResponse {
    fn from(value: PayKeysendResponse) -> Self {
        Self {
            preimage: value.preimage,
        }
    }
}

impl From<JsPayKeysendResponse> for PayKeysendResponse {
    fn from(value: JsPayKeysendResponse) -> Self {
        Self {
            preimage: value.preimage,
        }
    }
}

#[wasm_bindgen(js_name = MakeInvoiceResponse)]
pub struct JsMakeInvoiceResponse {
    /// Bolt 11 invoice
    #[wasm_bindgen(getter_with_clone)]
    pub invoice: String,
    /// Invoice's payment hash
    #[wasm_bindgen(getter_with_clone)]
    pub payment_hash: String,
}

impl From<MakeInvoiceResponse> for JsMakeInvoiceResponse {
    fn from(value: MakeInvoiceResponse) -> Self {
        Self {
            invoice: value.invoice,
            payment_hash: value.payment_hash,
        }
    }
}

impl From<JsMakeInvoiceResponse> for MakeInvoiceResponse {
    fn from(value: JsMakeInvoiceResponse) -> Self {
        Self {
            invoice: value.invoice,
            payment_hash: value.payment_hash,
        }
    }
}

#[wasm_bindgen(js_name = LookupInvoiceResponse)]
pub struct JsLookupInvoiceResponse {
    /// Transaction type
    pub transaction_type: Option<JsTransactionType>,
    /// Bolt11 invoice
    #[wasm_bindgen(getter_with_clone)]
    pub invoice: Option<String>,
    /// Invoice's description
    #[wasm_bindgen(getter_with_clone)]
    pub description: Option<String>,
    /// Invoice's description hash
    #[wasm_bindgen(getter_with_clone)]
    pub description_hash: Option<String>,
    /// Payment preimage
    #[wasm_bindgen(getter_with_clone)]
    pub preimage: Option<String>,
    /// Payment hash
    #[wasm_bindgen(getter_with_clone)]
    pub payment_hash: String,
    /// Amount in millisatoshis
    pub amount: u64,
    /// Fees paid in millisatoshis
    pub fees_paid: u64,
    /// Creation timestamp in seconds since epoch
    pub created_at: JsTimestamp,
    /// Expiration timestamp in seconds since epoch
    pub expires_at: Option<JsTimestamp>,
    /// Settled timestamp in seconds since epoch
    pub settled_at: Option<JsTimestamp>,
    // /// Optional metadata about the payment
    // #[wasm_bindgen(getter_with_clone)]
    // pub metadata: String, // TODO: this is not a string
}

impl From<LookupInvoiceResponse> for JsLookupInvoiceResponse {
    fn from(value: LookupInvoiceResponse) -> Self {
        Self {
            transaction_type: value.transaction_type.map(|t| t.into()),
            invoice: value.invoice,
            description: value.description,
            description_hash: value.description_hash,
            preimage: value.preimage,
            payment_hash: value.payment_hash,
            amount: value.amount,
            fees_paid: value.fees_paid,
            created_at: value.created_at.into(),
            expires_at: value.expires_at.map(|t| t.into()),
            settled_at: value.settled_at.map(|t| t.into()),
            // metadata: value.metadata.to_string(),
        }
    }
}

impl From<JsLookupInvoiceResponse> for LookupInvoiceResponse {
    fn from(value: JsLookupInvoiceResponse) -> Self {
        Self {
            transaction_type: value.transaction_type.map(|t| t.into()),
            invoice: value.invoice,
            description: value.description,
            description_hash: value.description_hash,
            preimage: value.preimage,
            payment_hash: value.payment_hash,
            amount: value.amount,
            fees_paid: value.fees_paid,
            created_at: *value.created_at,
            expires_at: value.expires_at.map(|t| *t),
            settled_at: value.settled_at.map(|t| *t),
            metadata: None,
        }
    }
}

#[wasm_bindgen(js_name = GetBalanceResponse)]
pub struct JsGetBalanceResponse {
    /// Balance amount in msats
    pub balance: u64,
}

impl From<GetBalanceResponse> for JsGetBalanceResponse {
    fn from(value: GetBalanceResponse) -> Self {
        Self {
            balance: value.balance,
        }
    }
}

impl From<JsGetBalanceResponse> for GetBalanceResponse {
    fn from(value: JsGetBalanceResponse) -> Self {
        Self {
            balance: value.balance,
        }
    }
}

#[wasm_bindgen(js_name = GetInfoResponse)]
pub struct JsGetInfoResponse {
    /// The alias of the lightning node
    #[wasm_bindgen(getter_with_clone)]
    pub alias: Option<String>,
    /// The color of the current node in hex code format
    #[wasm_bindgen(getter_with_clone)]
    pub color: Option<String>,
    /// Lightning Node's public key
    #[wasm_bindgen(getter_with_clone)]
    pub pubkey: Option<String>,
    /// Active network
    #[wasm_bindgen(getter_with_clone)]
    pub network: Option<String>,
    /// Current block height
    pub block_height: Option<u32>,
    /// Most Recent Block Hash
    #[wasm_bindgen(getter_with_clone)]
    pub block_hash: Option<String>,
    /// Available methods for this connection
    #[wasm_bindgen(getter_with_clone)]
    pub methods: Vec<String>,
    /// List of supported notifications for this connection (optional)
    #[wasm_bindgen(getter_with_clone)]
    pub notifications: Vec<String>,
}

impl From<GetInfoResponse> for JsGetInfoResponse {
    fn from(value: GetInfoResponse) -> Self {
        Self {
            alias: value.alias,
            color: value.color,
            pubkey: value.pubkey.map(|p| p.to_string()),
            network: value.network,
            block_height: value.block_height,
            block_hash: value.block_hash,
            methods: value.methods,
            notifications: value.notifications,
        }
    }
}

impl From<JsGetInfoResponse> for GetInfoResponse {
    fn from(value: JsGetInfoResponse) -> Self {
        Self {
            alias: value.alias,
            color: value.color,
            pubkey: value.pubkey.and_then(|p| p.parse().ok()),
            network: value.network,
            block_height: value.block_height,
            block_hash: value.block_hash,
            methods: value.methods,
            notifications: value.notifications,
        }
    }
}

#[wasm_bindgen(js_name = NostrWalletConnectURI)]
pub struct JsNostrWalletConnectURI {
    inner: NostrWalletConnectURI,
}

impl Deref for JsNostrWalletConnectURI {
    type Target = NostrWalletConnectURI;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[wasm_bindgen(js_class = NostrWalletConnectURI)]
impl JsNostrWalletConnectURI {
    /// Create new Nostr Wallet Connect URI
    #[wasm_bindgen(constructor)]
    pub fn new(
        public_key: &JsPublicKey,
        relays: Vec<String>,
        random_secret_key: &JsSecretKey,
        lud16: Option<String>,
    ) -> Result<JsNostrWalletConnectURI> {
        Ok(Self {
            inner: NostrWalletConnectURI::new(
                **public_key,
                relays
                    .into_iter()
                    .map(|r| RelayUrl::parse(&r))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(into_err)?,
                random_secret_key.deref().clone(),
                lud16,
            ),
        })
    }

    /// Parse
    pub fn parse(uri: &str) -> Result<JsNostrWalletConnectURI> {
        Ok(Self {
            inner: NostrWalletConnectURI::from_str(uri).map_err(into_err)?,
        })
    }

    /// App Pubkey
    #[wasm_bindgen(js_name = publicKey)]
    pub fn public_key(&self) -> JsPublicKey {
        self.inner.public_key.into()
    }

    /// URLs of the relays of choice where the `App` is connected and the `Signer` must send and listen for messages.
    pub fn relays(&self) -> Vec<String> {
        self.inner.relays.iter().map(|r| r.to_string()).collect()
    }

    /// 32-byte randomly generated hex encoded string
    pub fn secret(&self) -> JsSecretKey {
        self.inner.secret.clone().into()
    }

    /// A lightning address that clients can use to automatically setup the lud16 field on the user's profile if they have none configured.
    pub fn lud16(&self) -> Option<String> {
        self.inner.lud16.clone()
    }

    #[wasm_bindgen(js_name = asString)]
    pub fn as_string(&self) -> String {
        self.inner.to_string()
    }
}
