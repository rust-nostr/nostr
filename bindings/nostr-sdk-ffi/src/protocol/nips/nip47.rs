// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use nostr::nips::nip47;
use nostr::{JsonUtil, Url};
use uniffi::{Enum, Object, Record};

use crate::error::Result;
use crate::protocol::{JsonValue, PublicKey, SecretKey, Timestamp};

/// NIP47 Response Error codes
#[derive(Enum)]
pub enum ErrorCode {
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

impl From<nip47::ErrorCode> for ErrorCode {
    fn from(value: nip47::ErrorCode) -> Self {
        match value {
            nip47::ErrorCode::RateLimited => Self::RateLimited,
            nip47::ErrorCode::NotImplemented => Self::NotImplemented,
            nip47::ErrorCode::InsufficientBalance => Self::InsufficientBalance,
            nip47::ErrorCode::PaymentFailed => Self::PaymentFailed,
            nip47::ErrorCode::NotFound => Self::NotFound,
            nip47::ErrorCode::QuotaExceeded => Self::QuotaExceeded,
            nip47::ErrorCode::Restricted => Self::Restricted,
            nip47::ErrorCode::Unauthorized => Self::Unauthorized,
            nip47::ErrorCode::Internal => Self::Internal,
            nip47::ErrorCode::Other => Self::Other,
        }
    }
}

impl From<ErrorCode> for nip47::ErrorCode {
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

/// NIP47 Error message
#[derive(Record)]
pub struct NIP47Error {
    /// Error Code
    pub code: ErrorCode,
    /// Human Readable error message
    pub message: String,
}

impl From<nip47::NIP47Error> for NIP47Error {
    fn from(value: nip47::NIP47Error) -> Self {
        Self {
            code: value.code.into(),
            message: value.message,
        }
    }
}

impl From<NIP47Error> for nip47::NIP47Error {
    fn from(value: NIP47Error) -> Self {
        Self {
            code: value.code.into(),
            message: value.message,
        }
    }
}

/// Method
#[derive(Enum)]
pub enum Method {
    /// Pay Invoice
    PayInvoice,
    /// Multi Pay Invoice
    MultiPayInvoice,
    /// Pay Keysend
    PayKeysend,
    /// Multi Pay Keysend
    MultiPayKeysend,
    /// Make Invoice
    MakeInvoice,
    /// Lookup Invoice
    LookupInvoice,
    /// List transactions
    ListTransactions,
    /// Get Balance
    GetBalance,
    /// Get Info
    GetInfo,
}

impl From<nip47::Method> for Method {
    fn from(value: nip47::Method) -> Self {
        match value {
            nip47::Method::PayInvoice => Self::PayInvoice,
            nip47::Method::MultiPayInvoice => Self::MultiPayInvoice,
            nip47::Method::PayKeysend => Self::PayKeysend,
            nip47::Method::MultiPayKeysend => Self::MultiPayKeysend,
            nip47::Method::MakeInvoice => Self::MakeInvoice,
            nip47::Method::LookupInvoice => Self::LookupInvoice,
            nip47::Method::ListTransactions => Self::ListTransactions,
            nip47::Method::GetBalance => Self::GetBalance,
            nip47::Method::GetInfo => Self::GetInfo,
        }
    }
}

impl From<Method> for nip47::Method {
    fn from(value: Method) -> Self {
        match value {
            Method::PayInvoice => Self::PayInvoice,
            Method::MultiPayInvoice => Self::MultiPayInvoice,
            Method::PayKeysend => Self::PayKeysend,
            Method::MultiPayKeysend => Self::MultiPayKeysend,
            Method::MakeInvoice => Self::MakeInvoice,
            Method::LookupInvoice => Self::LookupInvoice,
            Method::ListTransactions => Self::ListTransactions,
            Method::GetBalance => Self::GetBalance,
            Method::GetInfo => Self::GetInfo,
        }
    }
}

/// Nostr Wallet Connect Request Params
#[derive(Enum)]
pub enum RequestParams {
    /// Pay Invoice
    PayInvoice {
        pay_invoice: PayInvoiceRequestParams,
    },
    /// Multi Pay Invoice
    MultiPayInvoice {
        multi_pay_invoice: MultiPayInvoiceRequestParams,
    },
    /// Pay Keysend
    PayKeysend {
        pay_keysend: PayKeysendRequestParams,
    },
    /// Multi Pay Keysend
    MultiPayKeysend {
        multi_pay_keysend: MultiPayKeysendRequestParams,
    },
    /// Make Invoice
    MakeInvoice {
        make_invoice: MakeInvoiceRequestParams,
    },
    /// Lookup Invoice
    LookupInvoice {
        lookup_invoice: LookupInvoiceRequestParams,
    },
    /// List Transactions
    ListTransactions {
        list_transactions: ListTransactionsRequestParams,
    },
    /// Get Balance
    GetBalance,
    /// Get Info
    GetInfo,
}

impl From<nip47::RequestParams> for RequestParams {
    fn from(value: nip47::RequestParams) -> Self {
        match value {
            nip47::RequestParams::PayInvoice(pay_invoice) => Self::PayInvoice {
                pay_invoice: pay_invoice.into(),
            },
            nip47::RequestParams::MultiPayInvoice(multi_pay_invoice) => Self::MultiPayInvoice {
                multi_pay_invoice: multi_pay_invoice.into(),
            },
            nip47::RequestParams::PayKeysend(pay_keysend) => Self::PayKeysend {
                pay_keysend: pay_keysend.into(),
            },
            nip47::RequestParams::MultiPayKeysend(multi_pay_keysend) => Self::MultiPayKeysend {
                multi_pay_keysend: multi_pay_keysend.into(),
            },
            nip47::RequestParams::MakeInvoice(make_invoice) => Self::MakeInvoice {
                make_invoice: make_invoice.into(),
            },
            nip47::RequestParams::LookupInvoice(lookup_invoice) => Self::LookupInvoice {
                lookup_invoice: lookup_invoice.into(),
            },
            nip47::RequestParams::ListTransactions(list_transactions) => Self::ListTransactions {
                list_transactions: list_transactions.into(),
            },
            nip47::RequestParams::GetBalance => Self::GetBalance,
            nip47::RequestParams::GetInfo => Self::GetInfo,
        }
    }
}

impl From<RequestParams> for nip47::RequestParams {
    fn from(value: RequestParams) -> Self {
        match value {
            RequestParams::PayInvoice { pay_invoice } => Self::PayInvoice(pay_invoice.into()),
            RequestParams::MultiPayInvoice { multi_pay_invoice } => {
                Self::MultiPayInvoice(multi_pay_invoice.into())
            }
            RequestParams::PayKeysend { pay_keysend } => Self::PayKeysend(pay_keysend.into()),
            RequestParams::MultiPayKeysend { multi_pay_keysend } => {
                Self::MultiPayKeysend(multi_pay_keysend.into())
            }
            RequestParams::MakeInvoice { make_invoice } => Self::MakeInvoice(make_invoice.into()),
            RequestParams::LookupInvoice { lookup_invoice } => {
                Self::LookupInvoice(lookup_invoice.into())
            }
            RequestParams::ListTransactions { list_transactions } => {
                Self::ListTransactions(list_transactions.into())
            }
            RequestParams::GetBalance => Self::GetBalance,
            RequestParams::GetInfo => Self::GetInfo,
        }
    }
}

/// Pay Invoice Request Params
#[derive(Record)]
pub struct PayInvoiceRequestParams {
    /// Optional id
    pub id: Option<String>,
    /// Request invoice
    pub invoice: String,
    /// Optional amount in millisatoshis
    pub amount: Option<u64>,
}

impl From<nip47::PayInvoiceRequestParams> for PayInvoiceRequestParams {
    fn from(value: nip47::PayInvoiceRequestParams) -> Self {
        Self {
            id: value.id,
            invoice: value.invoice,
            amount: value.amount,
        }
    }
}

impl From<PayInvoiceRequestParams> for nip47::PayInvoiceRequestParams {
    fn from(value: PayInvoiceRequestParams) -> Self {
        Self {
            id: value.id,
            invoice: value.invoice,
            amount: value.amount,
        }
    }
}

/// Multi Pay Invoice Request Params
#[derive(Record)]
pub struct MultiPayInvoiceRequestParams {
    /// Invoices to pay
    pub invoices: Vec<PayInvoiceRequestParams>,
}

impl From<nip47::MultiPayInvoiceRequestParams> for MultiPayInvoiceRequestParams {
    fn from(value: nip47::MultiPayInvoiceRequestParams) -> Self {
        Self {
            invoices: value.invoices.into_iter().map(|i| i.into()).collect(),
        }
    }
}

impl From<MultiPayInvoiceRequestParams> for nip47::MultiPayInvoiceRequestParams {
    fn from(value: MultiPayInvoiceRequestParams) -> Self {
        Self {
            invoices: value.invoices.into_iter().map(|i| i.into()).collect(),
        }
    }
}

/// TLVs to be added to the keysend payment
#[derive(Record)]
pub struct KeysendTLVRecord {
    /// TLV type
    pub tlv_type: u64,
    /// TLV value
    pub value: String,
}

impl From<nip47::KeysendTLVRecord> for KeysendTLVRecord {
    fn from(value: nip47::KeysendTLVRecord) -> Self {
        Self {
            tlv_type: value.tlv_type,
            value: value.value,
        }
    }
}

impl From<KeysendTLVRecord> for nip47::KeysendTLVRecord {
    fn from(value: KeysendTLVRecord) -> Self {
        Self {
            tlv_type: value.tlv_type,
            value: value.value,
        }
    }
}

/// Pay Invoice Request Params
#[derive(Record)]
pub struct PayKeysendRequestParams {
    /// Optional id
    pub id: Option<String>,
    /// Amount in millisatoshis
    pub amount: u64,
    /// Receiver's node id
    pub pubkey: String,
    /// Optional preimage
    pub preimage: Option<String>,
    /// Optional TLVs to be added to the keysend payment
    pub tlv_records: Vec<KeysendTLVRecord>,
}

impl From<nip47::PayKeysendRequestParams> for PayKeysendRequestParams {
    fn from(value: nip47::PayKeysendRequestParams) -> Self {
        Self {
            id: value.id,
            amount: value.amount,
            pubkey: value.pubkey,
            preimage: value.preimage,
            tlv_records: value.tlv_records.into_iter().map(|t| t.into()).collect(),
        }
    }
}

impl From<PayKeysendRequestParams> for nip47::PayKeysendRequestParams {
    fn from(value: PayKeysendRequestParams) -> Self {
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
#[derive(Record)]
pub struct MultiPayKeysendRequestParams {
    /// Keysends
    pub keysends: Vec<PayKeysendRequestParams>,
}

impl From<nip47::MultiPayKeysendRequestParams> for MultiPayKeysendRequestParams {
    fn from(value: nip47::MultiPayKeysendRequestParams) -> Self {
        Self {
            keysends: value.keysends.into_iter().map(|i| i.into()).collect(),
        }
    }
}

impl From<MultiPayKeysendRequestParams> for nip47::MultiPayKeysendRequestParams {
    fn from(value: MultiPayKeysendRequestParams) -> Self {
        Self {
            keysends: value.keysends.into_iter().map(|i| i.into()).collect(),
        }
    }
}

/// Transaction Type
#[derive(Enum)]
pub enum TransactionType {
    /// Incoming payments
    Incoming,
    /// Outgoing payments
    Outgoing,
}

impl From<TransactionType> for nip47::TransactionType {
    fn from(value: TransactionType) -> Self {
        match value {
            TransactionType::Incoming => Self::Incoming,
            TransactionType::Outgoing => Self::Outgoing,
        }
    }
}

impl From<nip47::TransactionType> for TransactionType {
    fn from(value: nip47::TransactionType) -> Self {
        match value {
            nip47::TransactionType::Incoming => Self::Incoming,
            nip47::TransactionType::Outgoing => Self::Outgoing,
        }
    }
}

/// Make Invoice Request Params
#[derive(Record)]
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

impl From<nip47::MakeInvoiceRequestParams> for MakeInvoiceRequestParams {
    fn from(value: nip47::MakeInvoiceRequestParams) -> Self {
        Self {
            amount: value.amount,
            description: value.description,
            description_hash: value.description_hash,
            expiry: value.expiry,
        }
    }
}

impl From<MakeInvoiceRequestParams> for nip47::MakeInvoiceRequestParams {
    fn from(value: MakeInvoiceRequestParams) -> Self {
        Self {
            amount: value.amount,
            description: value.description,
            description_hash: value.description_hash,
            expiry: value.expiry,
        }
    }
}

/// Lookup Invoice Request Params
#[derive(Record)]
pub struct LookupInvoiceRequestParams {
    /// Payment hash of invoice
    pub payment_hash: Option<String>,
    /// Bolt11 invoice
    pub invoice: Option<String>,
}

impl From<nip47::LookupInvoiceRequestParams> for LookupInvoiceRequestParams {
    fn from(value: nip47::LookupInvoiceRequestParams) -> Self {
        Self {
            payment_hash: value.payment_hash,
            invoice: value.invoice,
        }
    }
}

impl From<LookupInvoiceRequestParams> for nip47::LookupInvoiceRequestParams {
    fn from(value: LookupInvoiceRequestParams) -> Self {
        Self {
            payment_hash: value.payment_hash,
            invoice: value.invoice,
        }
    }
}

/// List Invoice Request Params
#[derive(Record)]
pub struct ListTransactionsRequestParams {
    /// Starting timestamp in seconds since epoch
    pub from: Option<Arc<Timestamp>>,
    /// Ending timestamp in seconds since epoch
    pub until: Option<Arc<Timestamp>>,
    /// Number of invoices to return
    pub limit: Option<u64>,
    /// Offset of the first invoice to return
    pub offset: Option<u64>,
    /// If true, include unpaid invoices
    pub unpaid: Option<bool>,
    /// [`TransactionType::Incoming`] for invoices, [`TransactionType::Outgoing`] for payments, [`None`] for both
    pub transaction_type: Option<TransactionType>,
}

impl From<nip47::ListTransactionsRequestParams> for ListTransactionsRequestParams {
    fn from(value: nip47::ListTransactionsRequestParams) -> Self {
        Self {
            from: value.from.map(|t| Arc::new(t.into())),
            until: value.until.map(|t| Arc::new(t.into())),
            limit: value.limit,
            offset: value.offset,
            unpaid: value.unpaid,
            transaction_type: value.transaction_type.map(|t| t.into()),
        }
    }
}

impl From<ListTransactionsRequestParams> for nip47::ListTransactionsRequestParams {
    fn from(value: ListTransactionsRequestParams) -> Self {
        Self {
            from: value.from.map(|t| **t),
            until: value.until.map(|t| **t),
            limit: value.limit,
            offset: value.offset,
            unpaid: value.unpaid,
            transaction_type: value.transaction_type.map(|t| t.into()),
        }
    }
}

/// NIP47 Request
#[derive(Debug, PartialEq, Eq, Hash, Object)]
#[uniffi::export(Debug, Eq, Hash)]
pub struct Request {
    inner: nip47::Request,
}

impl From<nip47::Request> for Request {
    fn from(inner: nip47::Request) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl Request {
    #[uniffi::constructor]
    pub fn new(method: Method, params: RequestParams) -> Self {
        Self {
            inner: nip47::Request {
                method: method.into(),
                params: params.into(),
            },
        }
    }

    #[uniffi::constructor]
    pub fn parse(json: String) -> Result<Self> {
        Ok(nip47::Request::from_json(json)?.into())
    }

    pub fn method(&self) -> Method {
        self.inner.method.into()
    }

    pub fn params(&self) -> RequestParams {
        self.inner.params.clone().into()
    }
}

/// NIP47 Response Result
#[derive(Record)]
pub struct PayInvoiceResponseResult {
    /// Response preimage
    pub preimage: String,
}

impl From<nip47::PayInvoiceResponseResult> for PayInvoiceResponseResult {
    fn from(value: nip47::PayInvoiceResponseResult) -> Self {
        Self {
            preimage: value.preimage,
        }
    }
}

impl From<PayInvoiceResponseResult> for nip47::PayInvoiceResponseResult {
    fn from(value: PayInvoiceResponseResult) -> Self {
        Self {
            preimage: value.preimage,
        }
    }
}

/// NIP47 Response Result
#[derive(Record)]
pub struct PayKeysendResponseResult {
    /// Response preimage
    pub preimage: String,
}

impl From<nip47::PayKeysendResponseResult> for PayKeysendResponseResult {
    fn from(value: nip47::PayKeysendResponseResult) -> Self {
        Self {
            preimage: value.preimage,
        }
    }
}

impl From<PayKeysendResponseResult> for nip47::PayKeysendResponseResult {
    fn from(value: PayKeysendResponseResult) -> Self {
        Self {
            preimage: value.preimage,
        }
    }
}

/// NIP47 Response Result
#[derive(Record)]
pub struct MakeInvoiceResponseResult {
    /// Bolt 11 invoice
    pub invoice: String,
    /// Invoice's payment hash
    pub payment_hash: String,
}

impl From<nip47::MakeInvoiceResponseResult> for MakeInvoiceResponseResult {
    fn from(value: nip47::MakeInvoiceResponseResult) -> Self {
        Self {
            invoice: value.invoice,
            payment_hash: value.payment_hash,
        }
    }
}

impl From<MakeInvoiceResponseResult> for nip47::MakeInvoiceResponseResult {
    fn from(value: MakeInvoiceResponseResult) -> Self {
        Self {
            invoice: value.invoice,
            payment_hash: value.payment_hash,
        }
    }
}

/// NIP47 Response Result
#[derive(Record)]
pub struct LookupInvoiceResponseResult {
    /// Transaction type
    pub transaction_type: Option<TransactionType>,
    /// Bolt11 invoice
    pub invoice: Option<String>,
    /// Invoice's description
    pub description: Option<String>,
    /// Invoice's description hash
    pub description_hash: Option<String>,
    /// Payment preimage
    pub preimage: Option<String>,
    /// Payment hash
    pub payment_hash: String,
    /// Amount in millisatoshis
    pub amount: u64,
    /// Fees paid in millisatoshis
    pub fees_paid: u64,
    /// Creation timestamp in seconds since epoch
    pub created_at: Arc<Timestamp>,
    /// Expiration timestamp in seconds since epoch
    pub expires_at: Option<Arc<Timestamp>>,
    /// Settled timestamp in seconds since epoch
    pub settled_at: Option<Arc<Timestamp>>,
    /// Optional metadata about the payment
    pub metadata: Option<JsonValue>,
}

impl From<nip47::LookupInvoiceResponseResult> for LookupInvoiceResponseResult {
    fn from(value: nip47::LookupInvoiceResponseResult) -> Self {
        Self {
            transaction_type: value.transaction_type.map(|t| t.into()),
            invoice: value.invoice,
            description: value.description,
            description_hash: value.description_hash,
            preimage: value.preimage,
            payment_hash: value.payment_hash,
            amount: value.amount,
            fees_paid: value.fees_paid,
            created_at: Arc::new(value.created_at.into()),
            expires_at: value.expires_at.map(|t| Arc::new(t.into())),
            settled_at: value.settled_at.map(|t| Arc::new(t.into())),
            metadata: value.metadata.and_then(|m| m.try_into().ok()),
        }
    }
}

impl From<LookupInvoiceResponseResult> for nip47::LookupInvoiceResponseResult {
    fn from(value: LookupInvoiceResponseResult) -> Self {
        Self {
            transaction_type: value.transaction_type.map(|t| t.into()),
            invoice: value.invoice,
            description: value.description,
            description_hash: value.description_hash,
            preimage: value.preimage,
            payment_hash: value.payment_hash,
            amount: value.amount,
            fees_paid: value.fees_paid,
            created_at: **value.created_at,
            expires_at: value.expires_at.map(|t| **t),
            settled_at: value.settled_at.map(|t| **t),
            metadata: value.metadata.and_then(|m| m.try_into().ok()),
        }
    }
}

/// NIP47 Response Result
#[derive(Record)]
pub struct GetBalanceResponseResult {
    /// Balance amount in msats
    pub balance: u64,
}

impl From<nip47::GetBalanceResponseResult> for GetBalanceResponseResult {
    fn from(value: nip47::GetBalanceResponseResult) -> Self {
        Self {
            balance: value.balance,
        }
    }
}

impl From<GetBalanceResponseResult> for nip47::GetBalanceResponseResult {
    fn from(value: GetBalanceResponseResult) -> Self {
        Self {
            balance: value.balance,
        }
    }
}

/// NIP47 Response Result
#[derive(Record)]
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

impl From<nip47::GetInfoResponseResult> for GetInfoResponseResult {
    fn from(value: nip47::GetInfoResponseResult) -> Self {
        Self {
            alias: value.alias,
            color: value.color,
            pubkey: value.pubkey,
            network: value.network,
            block_height: value.block_height,
            block_hash: value.block_hash,
            methods: value.methods,
        }
    }
}

impl From<GetInfoResponseResult> for nip47::GetInfoResponseResult {
    fn from(value: GetInfoResponseResult) -> Self {
        Self {
            alias: value.alias,
            color: value.color,
            pubkey: value.pubkey,
            network: value.network,
            block_height: value.block_height,
            block_hash: value.block_hash,
            methods: value.methods,
        }
    }
}

/// NIP47 Response Result
#[derive(Enum)]
pub enum ResponseResult {
    /// Pay Invoice
    PayInvoice {
        pay_invoice: PayInvoiceResponseResult,
    },
    /// Multi Pay Invoice
    MultiPayInvoice {
        pay_invoice: PayInvoiceResponseResult,
    },
    /// Pay Keysend
    PayKeysend {
        pay_keysend: PayKeysendResponseResult,
    },
    /// Multi Pay Keysend
    MultiPayKeysend {
        pay_keysend: PayKeysendResponseResult,
    },
    /// Make Invoice
    MakeInvoice {
        make_invoice: MakeInvoiceResponseResult,
    },
    /// Lookup Invoice
    LookupInvoice {
        lookup_invoice: LookupInvoiceResponseResult,
    },
    /// List Transactions
    ListTransactions {
        list_transactions: Vec<LookupInvoiceResponseResult>,
    },
    /// Get Balance
    GetBalance {
        get_balance: GetBalanceResponseResult,
    },
    /// Get Info
    GetInfo { get_info: GetInfoResponseResult },
}

impl From<nip47::ResponseResult> for ResponseResult {
    fn from(value: nip47::ResponseResult) -> Self {
        match value {
            nip47::ResponseResult::PayInvoice(pay_invoice) => Self::PayInvoice {
                pay_invoice: pay_invoice.into(),
            },
            nip47::ResponseResult::MultiPayInvoice(multi_pay_invoice) => Self::MultiPayInvoice {
                pay_invoice: multi_pay_invoice.into(),
            },
            nip47::ResponseResult::PayKeysend(pay_keysend) => Self::PayKeysend {
                pay_keysend: pay_keysend.into(),
            },
            nip47::ResponseResult::MultiPayKeysend(multi_pay_keysend) => Self::MultiPayKeysend {
                pay_keysend: multi_pay_keysend.into(),
            },
            nip47::ResponseResult::MakeInvoice(make_invoice) => Self::MakeInvoice {
                make_invoice: make_invoice.into(),
            },
            nip47::ResponseResult::LookupInvoice(lookup_invoice) => Self::LookupInvoice {
                lookup_invoice: lookup_invoice.into(),
            },
            nip47::ResponseResult::ListTransactions(list_transactions) => Self::ListTransactions {
                list_transactions: list_transactions.into_iter().map(|i| i.into()).collect(),
            },
            nip47::ResponseResult::GetBalance(get_balance) => Self::GetBalance {
                get_balance: get_balance.into(),
            },
            nip47::ResponseResult::GetInfo(get_info) => Self::GetInfo {
                get_info: get_info.into(),
            },
        }
    }
}

impl From<ResponseResult> for nip47::ResponseResult {
    fn from(value: ResponseResult) -> Self {
        match value {
            ResponseResult::PayInvoice { pay_invoice } => Self::PayInvoice(pay_invoice.into()),
            ResponseResult::MultiPayInvoice { pay_invoice } => {
                Self::MultiPayInvoice(pay_invoice.into())
            }
            ResponseResult::PayKeysend { pay_keysend } => Self::PayKeysend(pay_keysend.into()),
            ResponseResult::MultiPayKeysend { pay_keysend } => {
                Self::MultiPayKeysend(pay_keysend.into())
            }
            ResponseResult::MakeInvoice { make_invoice } => Self::MakeInvoice(make_invoice.into()),
            ResponseResult::LookupInvoice { lookup_invoice } => {
                Self::LookupInvoice(lookup_invoice.into())
            }
            ResponseResult::ListTransactions { list_transactions } => {
                Self::ListTransactions(list_transactions.into_iter().map(|i| i.into()).collect())
            }
            ResponseResult::GetBalance { get_balance } => Self::GetBalance(get_balance.into()),
            ResponseResult::GetInfo { get_info } => Self::GetInfo(get_info.into()),
        }
    }
}

/// NIP47 Response
#[derive(Debug, PartialEq, Eq, Object)]
#[uniffi::export(Debug, Eq)]
pub struct Response {
    inner: nip47::Response,
}

impl From<nip47::Response> for Response {
    fn from(inner: nip47::Response) -> Self {
        Self { inner }
    }
}

impl Response {
    #[uniffi::constructor]
    pub fn new(
        result_type: Method,
        result: Option<ResponseResult>,
        error: Option<NIP47Error>,
    ) -> Self {
        Self {
            inner: nip47::Response {
                result_type: result_type.into(),
                error: error.map(|e| e.into()),
                result: result.map(|r| r.into()),
            },
        }
    }

    /// Deserialize from JSON string
    #[uniffi::constructor]
    pub fn parse(json: String) -> Result<Self> {
        Ok(nip47::Response::from_json(json)?.into())
    }

    pub fn result_type(&self) -> Method {
        self.inner.result_type.into()
    }

    pub fn result(&self) -> Option<ResponseResult> {
        self.inner.result.clone().map(|i| i.into())
    }

    pub fn error(&self) -> Option<NIP47Error> {
        self.inner.error.clone().map(|i| i.into())
    }
}

/// Nostr Connect URI
#[derive(Debug, PartialEq, Eq, Object)]
#[uniffi::export(Debug, Eq)]
pub struct NostrWalletConnectURI {
    inner: nip47::NostrWalletConnectURI,
}

impl Deref for NostrWalletConnectURI {
    type Target = nip47::NostrWalletConnectURI;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<nip47::NostrWalletConnectURI> for NostrWalletConnectURI {
    fn from(inner: nip47::NostrWalletConnectURI) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl NostrWalletConnectURI {
    /// Create new Nostr Wallet Connect URI
    #[uniffi::constructor]
    pub fn new(
        public_key: &PublicKey,
        relay_url: String,
        random_secret_key: &SecretKey,
        lud16: Option<String>,
    ) -> Result<Self> {
        Ok(nip47::NostrWalletConnectURI::new(
            **public_key,
            Url::parse(&relay_url)?,
            random_secret_key.deref().clone(),
            lud16,
        )
        .into())
    }

    #[uniffi::constructor]
    pub fn parse(uri: String) -> Result<Self> {
        Ok(nip47::NostrWalletConnectURI::from_str(&uri)?.into())
    }

    /// App Pubkey
    pub fn public_key(&self) -> Arc<PublicKey> {
        Arc::new(self.inner.public_key.into())
    }

    /// URL of the relay of choice where the `App` is connected and the `Signer` must send and listen for messages.
    pub fn relay_url(&self) -> String {
        self.inner.relay_url.to_string()
    }

    /// 32-byte randomly generated hex encoded string
    pub fn secret(&self) -> Arc<SecretKey> {
        Arc::new(self.inner.secret.clone().into())
    }

    /// A lightning address that clients can use to automatically setup the lud16 field on the user's profile if they have none configured.
    pub fn lud16(&self) -> Option<String> {
        self.inner.lud16.clone()
    }
}
