// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP47
//!
//! <https://github.com/nostr-protocol/nips/blob/master/47.md>

use alloc::borrow::Cow;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use super::nip04;
use crate::types::url::form_urlencoded::byte_serialize;
use crate::types::url::{ParseError, Url};
use crate::{key, Event, JsonUtil, PublicKey, SecretKey};
#[cfg(feature = "std")]
use crate::{EventBuilder, Keys, Kind, Tag};

/// NIP47 error
#[derive(Debug)]
pub enum Error {
    /// JSON error
    JSON(serde_json::Error),
    /// Url parse error
    Url(ParseError),
    /// Keys error
    Keys(key::Error),
    /// NIP04 error
    NIP04(nip04::Error),
    /// Event Builder error
    #[cfg(feature = "std")]
    EventBuilder(crate::event::builder::Error),
    /// Unsigned event error
    UnsignedEvent(crate::event::unsigned::Error),
    /// Error code
    ErrorCode(NIP47Error),
    /// NIP47 Error Code
    UnexpectedResult(String),
    /// Invalid request
    InvalidRequest,
    /// Too many/few params
    InvalidParamsLength,
    /// Unsupported method
    UnsupportedMethod(String),
    /// Invalid URI
    InvalidURI,
    /// Invalid URI scheme
    InvalidURIScheme,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::JSON(e) => write!(f, "Json: {e}"),
            Self::Url(e) => write!(f, "Url: {e}"),
            Self::Keys(e) => write!(f, "Keys: {e}"),
            Self::NIP04(e) => write!(f, "NIP04: {e}"),
            #[cfg(feature = "std")]
            Self::EventBuilder(e) => write!(f, "Event Builder: {e}"),
            Self::UnsignedEvent(e) => write!(f, "Unsigned event: {e}"),
            Self::ErrorCode(e) => write!(f, "{e}"),
            Self::UnexpectedResult(json) => write!(f, "Unexpected NIP47 result: {json}"),
            Self::InvalidRequest => write!(f, "Invalid NIP47 Request"),
            Self::InvalidParamsLength => write!(f, "Invalid NIP47 Params length"),
            Self::UnsupportedMethod(e) => write!(f, "Unsupported method: {e}"),
            Self::InvalidURI => write!(f, "Invalid NIP47 URI"),
            Self::InvalidURIScheme => write!(f, "Invalid NIP47 URI Scheme"),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::JSON(e)
    }
}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Self::Url(e)
    }
}

impl From<key::Error> for Error {
    fn from(e: key::Error) -> Self {
        Self::Keys(e)
    }
}

impl From<nip04::Error> for Error {
    fn from(e: nip04::Error) -> Self {
        Self::NIP04(e)
    }
}

#[cfg(feature = "std")]
impl From<crate::event::builder::Error> for Error {
    fn from(e: crate::event::builder::Error) -> Self {
        Self::EventBuilder(e)
    }
}

/// NIP47 Response Error codes
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ErrorCode {
    ///  The client is sending commands too fast.
    #[serde(rename = "RATE_LIMITED")]
    RateLimited,
    /// The command is not known of is intentionally not implemented
    #[serde(rename = "NOT_IMPLEMENTED")]
    NotImplemented,
    /// The wallet does not have enough funds to cover a fee reserve or the payment amount
    #[serde(rename = "INSUFFICIENT_BALANCE")]
    InsufficientBalance,
    /// The payment failed. This may be due to a timeout, exhausting all routes, insufficient capacity or similar.
    #[serde(rename = "PAYMENT_FAILED")]
    PaymentFailed,
    /// The invoice could not be found by the given parameters.
    #[serde(rename = "NOT_FOUND")]
    NotFound,
    /// The wallet has exceeded its spending quota
    #[serde(rename = "QUOTA_EXCEEDED")]
    QuotaExceeded,
    /// This public key is not allowed to do this operation
    #[serde(rename = "RESTRICTED")]
    Restricted,
    /// This public key has no wallet connected
    #[serde(rename = "UNAUTHORIZED")]
    Unauthorized,
    /// An internal error
    #[serde(rename = "INTERNAL")]
    Internal,
    /// Other error
    #[serde(rename = "OTHER")]
    Other,
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Method::PayInvoice => write!(f, "pay_invoice"),
            Method::MultiPayInvoice => write!(f, "multi_pay_invoice"),
            Method::PayKeysend => write!(f, "pay_keysend"),
            Method::MultiPayKeysend => write!(f, "multi_pay_keysend"),
            Method::MakeInvoice => write!(f, "make_invoice"),
            Method::LookupInvoice => write!(f, "lookup_invoice"),
            Method::ListTransactions => write!(f, "list_transactions"),
            Method::GetBalance => write!(f, "get_balance"),
            Method::GetInfo => write!(f, "get_info"),
        }
    }
}

impl FromStr for Method {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pay_invoice" => Ok(Method::PayInvoice),
            "multi_pay_invoice" => Ok(Method::MultiPayInvoice),
            "pay_keysend" => Ok(Method::PayKeysend),
            "multi_pay_keysend" => Ok(Method::MultiPayKeysend),
            "make_invoice" => Ok(Method::MakeInvoice),
            "lookup_invoice" => Ok(Method::LookupInvoice),
            "list_transactions" => Ok(Method::ListTransactions),
            "get_balance" => Ok(Method::GetBalance),
            "get_info" => Ok(Method::GetInfo),
            _ => Err(Error::InvalidURI),
        }
    }
}

/// NIP47 Error message
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NIP47Error {
    /// Error Code
    pub code: ErrorCode,
    /// Human Readable error message
    pub message: String,
}

impl fmt::Display for NIP47Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} [{:?}]", self.message, self.code)
    }
}

/// Method
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Method {
    /// Pay Invoice
    #[serde(rename = "pay_invoice")]
    PayInvoice,
    /// Multi Pay Invoice
    #[serde(rename = "multi_pay_invoice")]
    MultiPayInvoice,
    /// Pay Keysend
    #[serde(rename = "pay_keysend")]
    PayKeysend,
    /// Multi Pay Keysend
    #[serde(rename = "multi_pay_keysend")]
    MultiPayKeysend,
    /// Make Invoice
    #[serde(rename = "make_invoice")]
    MakeInvoice,
    /// Lookup Invoice
    #[serde(rename = "lookup_invoice")]
    LookupInvoice,
    /// List transactions
    #[serde(rename = "list_transactions")]
    ListTransactions,
    /// Get Balance
    #[serde(rename = "get_balance")]
    GetBalance,
    /// Get Info
    #[serde(rename = "get_info")]
    GetInfo,
}

/// Nostr Wallet Connect Request Params
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RequestParams {
    /// Pay Invoice
    PayInvoice(PayInvoiceRequestParams),
    /// Multiple Pay Invoice
    MultiPayInvoice(MultiPayInvoiceRequestParams),
    /// Pay Keysend
    PayKeysend(PayKeysendRequestParams),
    /// Multiple Pay Keysend
    MultiPayKeysend(MultiPayKeysendRequestParams),
    /// Make Invoice
    MakeInvoice(MakeInvoiceRequestParams),
    /// Lookup Invoice
    LookupInvoice(LookupInvoiceRequestParams),
    /// List Transactions
    ListTransactions(ListTransactionsRequestParams),
    /// Get Balance
    GetBalance,
    /// Get Info
    GetInfo,
}

impl Serialize for RequestParams {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            RequestParams::PayInvoice(p) => p.serialize(serializer),
            RequestParams::MultiPayInvoice(p) => p.serialize(serializer),
            RequestParams::PayKeysend(p) => p.serialize(serializer),
            RequestParams::MultiPayKeysend(p) => p.serialize(serializer),
            RequestParams::MakeInvoice(p) => p.serialize(serializer),
            RequestParams::LookupInvoice(p) => p.serialize(serializer),
            RequestParams::ListTransactions(p) => p.serialize(serializer),
            RequestParams::GetBalance => serializer.serialize_none(),
            RequestParams::GetInfo => serializer.serialize_none(),
        }
    }
}

/// Pay Invoice Request Params
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PayInvoiceRequestParams {
    /// Optional id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Request invoice
    pub invoice: String,
    /// Optional amount in millisatoshis
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<u64>,
}

/// Multiple Pay Invoice Request Params
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MultiPayInvoiceRequestParams {
    /// Requested invoices
    pub invoices: Vec<PayInvoiceRequestParams>,
}

/// TLVs to be added to the keysend payment
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeysendTLVRecord {
    /// TLV type
    #[serde(rename = "type")]
    pub tlv_type: u64,
    /// TLV value
    pub value: String,
}

/// Pay Invoice Request Params
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PayKeysendRequestParams {
    /// Optional id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Amount in millisatoshis
    pub amount: u64,
    /// Receiver's node id
    pub pubkey: String,
    /// Optional preimage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preimage: Option<String>,
    /// Optional TLVs to be added to the keysend payment
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tlv_records: Vec<KeysendTLVRecord>,
}

/// Multiple Pay Keysend Request Params
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MultiPayKeysendRequestParams {
    /// Requested keysends
    pub keysends: Vec<PayKeysendRequestParams>,
}

/// Make Invoice Request Params
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MakeInvoiceRequestParams {
    /// Amount in millisatoshis
    pub amount: u64,
    /// Invoice description
    pub description: Option<String>,
    /// Invoice description hash
    pub description_hash: Option<String>,
    /// Invoice expiry in seconds
    pub expiry: Option<u64>,
}

/// Lookup Invoice Request Params
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LookupInvoiceRequestParams {
    /// Payment hash of invoice
    pub payment_hash: Option<String>,
    /// Bolt11 invoice
    pub invoice: Option<String>,
}

/// Transaction Type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransactionType {
    /// Incoming payments
    #[serde(rename = "incoming")]
    Incoming,
    /// Outgoing payments
    #[serde(rename = "outgoing")]
    Outgoing,
}

/// List Transactions Request Params
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ListTransactionsRequestParams {
    /// Starting timestamp in seconds since epoch
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<u64>,
    /// Ending timestamp in seconds since epoch
    #[serde(skip_serializing_if = "Option::is_none")]
    pub until: Option<u64>,
    /// Number of invoices to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
    /// Offset of the first invoice to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u64>,
    /// If true, include unpaid invoices
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unpaid: Option<bool>,
    /// [`TransactionType::Incoming`] for invoices, [`TransactionType::Outgoing`] for payments, [`None`] for both
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_type: Option<TransactionType>,
}

/// NIP47 Request
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Request {
    /// Request method
    pub method: Method,
    /// Params
    pub params: RequestParams,
}

#[derive(Serialize, Deserialize)]
struct RequestTemplate {
    /// Request method
    method: Method,
    /// Params
    #[serde(default)] // handle no params as `Value::Null`
    params: Value,
}

impl Request {
    /// Compose `pay_invoice` request
    #[inline]
    pub fn pay_invoice(params: PayInvoiceRequestParams) -> Self {
        Self {
            method: Method::PayInvoice,
            params: RequestParams::PayInvoice(params),
        }
    }

    /// Compose `pay_keysend` request
    #[inline]
    pub fn pay_keysend(params: PayKeysendRequestParams) -> Self {
        Self {
            method: Method::PayKeysend,
            params: RequestParams::PayKeysend(params),
        }
    }

    /// Compose `make_invoice` request
    #[inline]
    pub fn make_invoice(params: MakeInvoiceRequestParams) -> Self {
        Self {
            method: Method::MakeInvoice,
            params: RequestParams::MakeInvoice(params),
        }
    }

    /// Compose `lookup_invoice` request
    #[inline]
    pub fn lookup_invoice(params: LookupInvoiceRequestParams) -> Self {
        Self {
            method: Method::LookupInvoice,
            params: RequestParams::LookupInvoice(params),
        }
    }

    /// Compose `list_transactions` request
    #[inline]
    pub fn list_transactions(params: ListTransactionsRequestParams) -> Self {
        Self {
            method: Method::ListTransactions,
            params: RequestParams::ListTransactions(params),
        }
    }

    /// Compose `get_balance` request
    #[inline]
    pub fn get_balance() -> Self {
        Self {
            method: Method::GetBalance,
            params: RequestParams::GetBalance,
        }
    }

    /// Compose `get_info` request
    #[inline]
    pub fn get_info() -> Self {
        Self {
            method: Method::GetInfo,
            params: RequestParams::GetInfo,
        }
    }

    /// Deserialize from [`Value`]
    pub fn from_value(value: Value) -> Result<Self, Error> {
        let template: RequestTemplate = serde_json::from_value(value)?;

        let params = match template.method {
            Method::PayInvoice => {
                let params: PayInvoiceRequestParams = serde_json::from_value(template.params)?;
                RequestParams::PayInvoice(params)
            }
            Method::MultiPayInvoice => {
                let params: MultiPayInvoiceRequestParams = serde_json::from_value(template.params)?;
                RequestParams::MultiPayInvoice(params)
            }
            Method::PayKeysend => {
                let params: PayKeysendRequestParams = serde_json::from_value(template.params)?;
                RequestParams::PayKeysend(params)
            }
            Method::MultiPayKeysend => {
                let params: MultiPayKeysendRequestParams = serde_json::from_value(template.params)?;
                RequestParams::MultiPayKeysend(params)
            }
            Method::MakeInvoice => {
                let params: MakeInvoiceRequestParams = serde_json::from_value(template.params)?;
                RequestParams::MakeInvoice(params)
            }
            Method::LookupInvoice => {
                let params: LookupInvoiceRequestParams = serde_json::from_value(template.params)?;
                RequestParams::LookupInvoice(params)
            }
            Method::ListTransactions => {
                let params: ListTransactionsRequestParams =
                    serde_json::from_value(template.params)?;
                RequestParams::ListTransactions(params)
            }
            Method::GetBalance => RequestParams::GetBalance,
            Method::GetInfo => RequestParams::GetInfo,
        };

        Ok(Self {
            method: template.method,
            params,
        })
    }

    /// Create request [Event]
    #[cfg(feature = "std")]
    pub fn to_event(self, uri: &NostrWalletConnectURI) -> Result<Event, Error> {
        let encrypted = nip04::encrypt(&uri.secret, &uri.public_key, self.as_json())?;
        let keys: Keys = Keys::new(uri.secret.clone());
        Ok(EventBuilder::new(
            Kind::WalletConnectRequest,
            encrypted,
            [Tag::public_key(uri.public_key)],
        )
        .to_event(&keys)?)
    }
}

impl JsonUtil for Request {
    type Err = Error;
}

impl<'de> Deserialize<'de> for Request {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: Value = Value::deserialize(deserializer).map_err(serde::de::Error::custom)?;
        Self::from_value(value).map_err(serde::de::Error::custom)
    }
}

/// NIP47 Response Result
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PayInvoiceResponseResult {
    /// Response preimage
    pub preimage: String,
}

/// NIP47 Response Result
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PayKeysendResponseResult {
    /// Response preimage
    pub preimage: String,
}

/// NIP47 Response Result
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct MakeInvoiceResponseResult {
    /// Bolt 11 invoice
    pub invoice: String,
    /// Invoice's payment hash
    pub payment_hash: String,
}

/// NIP47 Response Result
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct LookupInvoiceResponseResult {
    /// Transaction type
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_type: Option<TransactionType>,
    /// Bolt11 invoice
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice: Option<String>,
    /// Invoice's description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Invoice's description hash
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description_hash: Option<String>,
    /// Payment preimage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preimage: Option<String>,
    /// Payment hash
    pub payment_hash: String,
    /// Amount in millisatoshis
    pub amount: u64,
    /// Fees paid in millisatoshis
    pub fees_paid: u64,
    /// Creation timestamp in seconds since epoch
    pub created_at: u64,
    /// Expiration timestamp in seconds since epoch
    pub expires_at: u64,
    /// Settled timestamp in seconds since epoch
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settled_at: Option<u64>,
    /// Optional metadata about the payment
    pub metadata: Value,
}

/// NIP47 `get_balance` Response Result
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct GetBalanceResponseResult {
    /// Balance amount in msats
    pub balance: u64,
}

/// NIP47 `get_info` Response Result
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct GetInfoResponseResult {
    /// The alias of the lightning node
    pub alias: String,
    /// The color of the current node in hex code format
    pub color: String,
    /// Lightning Node's public key
    pub pubkey: String,
    /// Active network
    pub network: String,
    /// Current block height
    pub block_height: u32,
    /// Most Recent Block Hash
    pub block_hash: String,
    /// Available methods for this connection
    pub methods: Vec<String>,
}

/// NIP47 Response Result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResponseResult {
    /// Pay Invoice
    PayInvoice(PayInvoiceResponseResult),
    /// Multiple Pay Invoice
    MultiPayInvoice(PayInvoiceResponseResult),
    /// Pay Keysend
    PayKeysend(PayKeysendResponseResult),
    /// Multiple Pay Keysend
    MultiPayKeysend(PayKeysendResponseResult),
    /// Make Invoice
    MakeInvoice(MakeInvoiceResponseResult),
    /// Lookup Invoice
    LookupInvoice(LookupInvoiceResponseResult),
    /// List Invoices
    ListTransactions(Vec<LookupInvoiceResponseResult>),
    /// Get Balance
    GetBalance(GetBalanceResponseResult),
    /// Get Info
    GetInfo(GetInfoResponseResult),
}

impl Serialize for ResponseResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ResponseResult::PayInvoice(p) => p.serialize(serializer),
            ResponseResult::MultiPayInvoice(p) => p.serialize(serializer),
            ResponseResult::PayKeysend(p) => p.serialize(serializer),
            ResponseResult::MultiPayKeysend(p) => p.serialize(serializer),
            ResponseResult::MakeInvoice(p) => p.serialize(serializer),
            ResponseResult::LookupInvoice(p) => p.serialize(serializer),
            ResponseResult::ListTransactions(p) => p.serialize(serializer),
            ResponseResult::GetBalance(p) => p.serialize(serializer),
            ResponseResult::GetInfo(p) => p.serialize(serializer),
        }
    }
}

/// NIP47 Response
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Response {
    /// Request Method
    pub result_type: Method,
    /// NIP47 Error
    pub error: Option<NIP47Error>,
    /// NIP47 Result
    pub result: Option<ResponseResult>,
}

/// NIP47 Response
#[derive(Debug, Clone, Deserialize)]
struct ResponseTemplate {
    /// Request Method
    pub result_type: Method,
    /// NIP47 Error
    pub error: Option<NIP47Error>,
    /// NIP47 Result
    pub result: Option<Value>,
}

impl Response {
    /// Deserialize from [Event]
    #[inline]
    pub fn from_event(uri: &NostrWalletConnectURI, event: &Event) -> Result<Self, Error> {
        let decrypt_res: String = nip04::decrypt(&uri.secret, event.author_ref(), event.content())?;
        Self::from_json(decrypt_res)
    }

    /// Deserialize from JSON string
    pub fn from_value(value: Value) -> Result<Self, Error> {
        let template: ResponseTemplate = serde_json::from_value(value)?;

        if let Some(result) = template.result {
            let result = match template.result_type {
                Method::PayInvoice => {
                    let result: PayInvoiceResponseResult = serde_json::from_value(result)?;
                    ResponseResult::PayInvoice(result)
                }
                Method::MultiPayInvoice => {
                    let result: PayInvoiceResponseResult = serde_json::from_value(result)?;
                    ResponseResult::MultiPayInvoice(result)
                }
                Method::PayKeysend => {
                    let result: PayKeysendResponseResult = serde_json::from_value(result)?;
                    ResponseResult::PayKeysend(result)
                }
                Method::MultiPayKeysend => {
                    let result: PayKeysendResponseResult = serde_json::from_value(result)?;
                    ResponseResult::MultiPayKeysend(result)
                }
                Method::MakeInvoice => {
                    let result: MakeInvoiceResponseResult = serde_json::from_value(result)?;
                    ResponseResult::MakeInvoice(result)
                }
                Method::LookupInvoice => {
                    let result: LookupInvoiceResponseResult = serde_json::from_value(result)?;
                    ResponseResult::LookupInvoice(result)
                }
                Method::ListTransactions => {
                    let transactions: Value =
                        result
                            .get("transactions")
                            .cloned()
                            .ok_or(Error::UnexpectedResult(String::from(
                                "Missing 'transactions' field",
                            )))?;
                    let result: Vec<LookupInvoiceResponseResult> =
                        serde_json::from_value(transactions)?;
                    ResponseResult::ListTransactions(result)
                }
                Method::GetBalance => {
                    let result: GetBalanceResponseResult = serde_json::from_value(result)?;
                    ResponseResult::GetBalance(result)
                }
                Method::GetInfo => {
                    let result: GetInfoResponseResult = serde_json::from_value(result)?;
                    ResponseResult::GetInfo(result)
                }
            };

            Ok(Self {
                result_type: template.result_type,
                error: template.error,
                result: Some(result),
            })
        } else {
            Ok(Self {
                result_type: template.result_type,
                error: template.error,
                result: None,
            })
        }
    }

    /// Covert [Response] to [PayInvoiceResponseResult]
    pub fn to_pay_invoice(self) -> Result<PayInvoiceResponseResult, Error> {
        if let Some(e) = self.error {
            return Err(Error::ErrorCode(e));
        }

        if let Some(ResponseResult::PayInvoice(result)) = self.result {
            return Ok(result);
        }

        Err(Error::UnexpectedResult(self.as_json()))
    }

    /// Covert [Response] to [PayKeysendResponseResult]
    pub fn to_pay_keysend(self) -> Result<PayKeysendResponseResult, Error> {
        if let Some(e) = self.error {
            return Err(Error::ErrorCode(e));
        }

        if let Some(ResponseResult::PayKeysend(result)) = self.result {
            return Ok(result);
        }

        Err(Error::UnexpectedResult(self.as_json()))
    }

    /// Covert [Response] to [MakeInvoiceResponseResult]
    pub fn to_make_invoice(self) -> Result<MakeInvoiceResponseResult, Error> {
        if let Some(e) = self.error {
            return Err(Error::ErrorCode(e));
        }

        if let Some(ResponseResult::MakeInvoice(result)) = self.result {
            return Ok(result);
        }

        Err(Error::UnexpectedResult(self.as_json()))
    }

    /// Covert [Response] to [LookupInvoiceResponseResult]
    pub fn to_lookup_invoice(self) -> Result<LookupInvoiceResponseResult, Error> {
        if let Some(e) = self.error {
            return Err(Error::ErrorCode(e));
        }

        if let Some(ResponseResult::LookupInvoice(result)) = self.result {
            return Ok(result);
        }

        Err(Error::UnexpectedResult(self.as_json()))
    }

    /// Covert [Response] to list of [LookupInvoiceResponseResult]
    pub fn to_list_transactions(self) -> Result<Vec<LookupInvoiceResponseResult>, Error> {
        if let Some(e) = self.error {
            return Err(Error::ErrorCode(e));
        }

        if let Some(ResponseResult::ListTransactions(result)) = self.result {
            return Ok(result);
        }

        Err(Error::UnexpectedResult(self.as_json()))
    }

    /// Covert [Response] to [GetBalanceResponseResult]
    pub fn to_get_balance(self) -> Result<GetBalanceResponseResult, Error> {
        if let Some(e) = self.error {
            return Err(Error::ErrorCode(e));
        }

        if let Some(ResponseResult::GetBalance(result)) = self.result {
            return Ok(result);
        }

        Err(Error::UnexpectedResult(self.as_json()))
    }

    /// Covert [Response] to [GetInfoResponseResult]
    pub fn to_get_info(self) -> Result<GetInfoResponseResult, Error> {
        if let Some(e) = self.error {
            return Err(Error::ErrorCode(e));
        }

        if let Some(ResponseResult::GetInfo(result)) = self.result {
            return Ok(result);
        }

        Err(Error::UnexpectedResult(self.as_json()))
    }
}

impl JsonUtil for Response {
    type Err = Error;
}

impl<'de> Deserialize<'de> for Response {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: Value = Value::deserialize(deserializer).map_err(serde::de::Error::custom)?;
        Self::from_value(value).map_err(serde::de::Error::custom)
    }
}

#[inline]
fn url_encode<T>(data: T) -> String
where
    T: AsRef<[u8]>,
{
    byte_serialize(data.as_ref()).collect()
}

/// NIP47 URI Scheme
pub const NOSTR_WALLET_CONNECT_URI_SCHEME: &str = "nostr+walletconnect";

/// Nostr Connect URI
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NostrWalletConnectURI {
    /// App Pubkey
    pub public_key: PublicKey,
    /// URL of the relay of choice where the `App` is connected and the `Signer` must send and listen for messages.
    pub relay_url: Url,
    /// 32-byte randomly generated hex encoded string
    pub secret: SecretKey,
    /// A lightning address that clients can use to automatically setup the lud16 field on the user's profile if they have none configured.
    pub lud16: Option<String>,
}

impl NostrWalletConnectURI {
    /// Create new [`NostrWalletConnectURI`]
    #[inline]
    pub fn new(
        public_key: PublicKey,
        relay_url: Url,
        random_secret_key: SecretKey,
        lud16: Option<String>,
    ) -> Self {
        Self {
            public_key,
            relay_url,
            secret: random_secret_key,
            lud16,
        }
    }
}

impl FromStr for NostrWalletConnectURI {
    type Err = Error;

    fn from_str(uri: &str) -> Result<Self, Self::Err> {
        let url = Url::parse(uri)?;

        if url.scheme() != NOSTR_WALLET_CONNECT_URI_SCHEME {
            return Err(Error::InvalidURIScheme);
        }

        if let Some(pubkey) = url.domain() {
            let public_key = PublicKey::from_str(pubkey)?;

            let mut relay_url: Option<Url> = None;
            let mut secret: Option<SecretKey> = None;
            let mut lud16: Option<String> = None;

            for (key, value) in url.query_pairs() {
                match key {
                    Cow::Borrowed("relay") => {
                        let value = value.to_string();
                        relay_url = Some(Url::parse(&value)?);
                    }
                    Cow::Borrowed("secret") => {
                        let value = value.to_string();
                        secret = Some(SecretKey::from_str(&value)?);
                    }
                    Cow::Borrowed("lud16") => {
                        lud16 = Some(value.to_string());
                    }
                    _ => (),
                }
            }

            if let Some(relay_url) = relay_url {
                if let Some(secret) = secret {
                    return Ok(Self {
                        public_key,
                        relay_url,
                        secret,
                        lud16,
                    });
                }
            }
        }

        Err(Error::InvalidURI)
    }
}

impl fmt::Display for NostrWalletConnectURI {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // trailing slash is removed, this breaks some clients
        let relay_url = self.relay_url.to_string();
        let relay_url = relay_url.strip_suffix('/').unwrap_or(&relay_url);
        write!(
            f,
            "{NOSTR_WALLET_CONNECT_URI_SCHEME}://{}?relay={}&secret={}",
            self.public_key,
            url_encode(relay_url),
            url_encode(self.secret.to_secret_hex())
        )?;
        if let Some(lud16) = &self.lud16 {
            write!(f, "&lud16={}", url_encode(lud16))?;
        }
        Ok(())
    }
}

impl Serialize for NostrWalletConnectURI {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'a> Deserialize<'a> for NostrWalletConnectURI {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        let uri = String::deserialize(deserializer)?;
        NostrWalletConnectURI::from_str(&uri).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use super::*;

    #[test]
    fn test_uri() {
        let pubkey =
            PublicKey::from_str("b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4")
                .unwrap();
        let relay_url = Url::parse("wss://relay.damus.io").unwrap();
        let secret =
            SecretKey::from_str("71a8c14c1407c113601079c4302dab36460f0ccd0ad506f1f2dc73b5100e4f3c")
                .unwrap();
        let uri = NostrWalletConnectURI::new(
            pubkey,
            relay_url,
            secret,
            Some("nostr@nostr.com".to_string()),
        );
        assert_eq!(
            uri.to_string(),
            "nostr+walletconnect://b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4?relay=wss%3A%2F%2Frelay.damus.io&secret=71a8c14c1407c113601079c4302dab36460f0ccd0ad506f1f2dc73b5100e4f3c&lud16=nostr%40nostr.com".to_string()
        );
    }

    #[test]
    fn test_parse_uri() {
        let uri = "nostr+walletconnect://b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4?relay=wss%3A%2F%2Frelay.damus.io%2F&secret=71a8c14c1407c113601079c4302dab36460f0ccd0ad506f1f2dc73b5100e4f3c&lud16=nostr%40nostr.com";
        let uri = NostrWalletConnectURI::from_str(uri).unwrap();

        let pubkey =
            PublicKey::from_str("b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4")
                .unwrap();
        let relay_url = Url::parse("wss://relay.damus.io").unwrap();
        let secret =
            SecretKey::from_str("71a8c14c1407c113601079c4302dab36460f0ccd0ad506f1f2dc73b5100e4f3c")
                .unwrap();
        assert_eq!(
            uri,
            NostrWalletConnectURI::new(
                pubkey,
                relay_url,
                secret,
                Some("nostr@nostr.com".to_string())
            )
        );
    }

    #[test]
    fn serialize_request() {
        let request = Request {
            method: Method::PayInvoice,
            params: RequestParams::PayInvoice(PayInvoiceRequestParams { id: None, invoice: "lnbc210n1pj99rx0pp5ehevgz9nf7d97h05fgkdeqxzytm6yuxd7048axru03fpzxxvzt7shp5gv7ef0s26pw5gy5dpwvsh6qgc8se8x2lmz2ev90l9vjqzcns6u6scqzzsxqyz5vqsp".to_string(), amount: None }),
        };

        assert_eq!(Request::from_json(request.as_json()).unwrap(), request);

        assert_eq!(request.as_json(), "{\"method\":\"pay_invoice\",\"params\":{\"invoice\":\"lnbc210n1pj99rx0pp5ehevgz9nf7d97h05fgkdeqxzytm6yuxd7048axru03fpzxxvzt7shp5gv7ef0s26pw5gy5dpwvsh6qgc8se8x2lmz2ev90l9vjqzcns6u6scqzzsxqyz5vqsp\"}}");
    }

    #[test]
    fn test_parse_request() {
        let request = "{\"params\":{\"invoice\":\"lnbc210n1pj99rx0pp5ehevgz9nf7d97h05fgkdeqxzytm6yuxd7048axru03fpzxxvzt7shp5gv7ef0s26pw5gy5dpwvsh6qgc8se8x2lmz2ev90l9vjqzcns6u6scqzzsxqyz5vqsp5rdjyt9jr2avv2runy330766avkweqp30ndnyt9x6dp5juzn7q0nq9qyyssq2mykpgu04q0hlga228kx9v95meaqzk8a9cnvya305l4c353u3h04azuh9hsmd503x6jlzjrsqzark5dxx30s46vuatwzjhzmkt3j4tgqu35rms\"},\"method\":\"pay_invoice\"}";

        let request = Request::from_json(request).unwrap();

        assert_eq!(request.method, Method::PayInvoice);

        if let RequestParams::PayInvoice(pay) = request.params {
            assert_eq!(pay.invoice, "lnbc210n1pj99rx0pp5ehevgz9nf7d97h05fgkdeqxzytm6yuxd7048axru03fpzxxvzt7shp5gv7ef0s26pw5gy5dpwvsh6qgc8se8x2lmz2ev90l9vjqzcns6u6scqzzsxqyz5vqsp5rdjyt9jr2avv2runy330766avkweqp30ndnyt9x6dp5juzn7q0nq9qyyssq2mykpgu04q0hlga228kx9v95meaqzk8a9cnvya305l4c353u3h04azuh9hsmd503x6jlzjrsqzark5dxx30s46vuatwzjhzmkt3j4tgqu35rms".to_string());
        } else {
            panic!("Invalid request params");
        }
    }
}
