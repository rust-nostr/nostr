// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::str::FromStr;
use std::sync::Arc;

use nostr::nips::nip47;
use nostr::{JsonUtil, Url};
use uniffi::{Enum, Record};

use crate::error::Result;
use crate::{PublicKey, SecretKey};

/// NIP47 Response Error codes
#[derive(Enum)]
pub enum ErrorCode {
    ///  The client is sending commands too fast.
    RateLimited,
    /// The command is not known of is intentionally not implemented
    NotImplemented,
    /// The wallet does not have enough funds to cover a fee reserve or the payment amount
    InsufficientBalance,
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
            nip47::ErrorCode::QuotaExceeded => Self::QuotaExceeded,
            nip47::ErrorCode::Restricted => Self::Restricted,
            nip47::ErrorCode::Unauthorized => Self::Unauthorized,
            nip47::ErrorCode::Internal => Self::Internal,
            nip47::ErrorCode::Other => Self::Other,
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

/// Method
#[derive(Enum)]
pub enum Method {
    /// Pay Invoice
    PayInvoice,
    /// Pay Keysend
    PayKeysend,
    /// Make Invoice
    MakeInvoice,
    /// Lookup Invoice
    LookupInvoice,
    /// List Invoices
    ListInvoices,
    /// List Payments
    ListPayments,
    /// Get Balance
    GetBalance,
}

impl From<nip47::Method> for Method {
    fn from(value: nip47::Method) -> Self {
        match value {
            nip47::Method::PayInvoice => Self::PayInvoice,
            nip47::Method::PayKeysend => Self::PayKeysend,
            nip47::Method::MakeInvoice => Self::MakeInvoice,
            nip47::Method::LookupInvoice => Self::LookupInvoice,
            nip47::Method::ListInvoices => Self::ListInvoices,
            nip47::Method::ListPayments => Self::ListPayments,
            nip47::Method::GetBalance => Self::GetBalance,
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
    /// Pay Keysend
    PayKeysend {
        pay_keysend: PayKeysendRequestParams,
    },
    /// Make Invoice
    MakeInvoice {
        make_invoice: MakeInvoiceRequestParams,
    },
    /// Lookup Invoice
    LookupInvoice {
        lookup_invoice: LookupInvoiceRequestParams,
    },
    /// List Invoices
    ListInvoices {
        list_invoice: ListInvoicesRequestParams,
    },
    /// List Payments
    ListPayments {
        list_payments: ListPaymentsRequestParams,
    },
    /// Get Balance
    GetBalance,
}

impl From<nip47::RequestParams> for RequestParams {
    fn from(value: nip47::RequestParams) -> Self {
        match value {
            nip47::RequestParams::PayInvoice(pay_invoice) => Self::PayInvoice {
                pay_invoice: pay_invoice.into(),
            },
            nip47::RequestParams::PayKeysend(pay_keysend) => Self::PayKeysend {
                pay_keysend: pay_keysend.into(),
            },
            nip47::RequestParams::MakeInvoice(make_invoice) => Self::MakeInvoice {
                make_invoice: make_invoice.into(),
            },
            nip47::RequestParams::LookupInvoice(lookup_invoice) => Self::LookupInvoice {
                lookup_invoice: lookup_invoice.into(),
            },
            nip47::RequestParams::ListInvoices(list_invoice) => Self::ListInvoices {
                list_invoice: list_invoice.into(),
            },
            nip47::RequestParams::ListPayments(list_payments) => Self::ListPayments {
                list_payments: list_payments.into(),
            },
            nip47::RequestParams::GetBalance => Self::GetBalance,
        }
    }
}

/// Pay Invoice Request Params
#[derive(Record)]
pub struct PayInvoiceRequestParams {
    /// Request invoice
    pub invoice: String,
}

impl From<nip47::PayInvoiceRequestParams> for PayInvoiceRequestParams {
    fn from(value: nip47::PayInvoiceRequestParams) -> Self {
        Self {
            invoice: value.invoice,
        }
    }
}

/// TLVs to be added to the keysend payment
#[derive(Record)]
pub struct KeysendTLVRecord {
    /// TLV type
    pub type_: u64,
    /// TLV value
    pub value: String,
}

impl From<nip47::KeysendTLVRecord> for KeysendTLVRecord {
    fn from(value: nip47::KeysendTLVRecord) -> Self {
        Self {
            type_: value.type_,
            value: value.value,
        }
    }
}

/// Pay Invoice Request Params
#[derive(Record)]
pub struct PayKeysendRequestParams {
    /// Amount in millisatoshis
    pub amount: i64,
    /// Receiver's node id
    pub pubkey: String,
    /// Optional message
    pub message: Option<String>,
    /// Optional preimage
    pub preimage: Option<String>,
    /// Optional TLVs to be added to the keysend payment
    pub tlv_records: Vec<KeysendTLVRecord>,
}

impl From<nip47::PayKeysendRequestParams> for PayKeysendRequestParams {
    fn from(value: nip47::PayKeysendRequestParams) -> Self {
        Self {
            amount: value.amount,
            pubkey: value.pubkey,
            message: value.message,
            preimage: value.preimage,
            tlv_records: value.tlv_records.into_iter().map(|t| t.into()).collect(),
        }
    }
}

/// Make Invoice Request Params
#[derive(Record)]
pub struct MakeInvoiceRequestParams {
    /// Amount in millisatoshis
    pub amount: i64,
    /// Invoice description
    pub description: Option<String>,
    /// Invoice description hash
    pub description_hash: Option<String>,
    /// Preimage to be used for the invoice
    pub preimage: Option<String>,
    /// Invoice expiry in seconds
    pub expiry: Option<i64>,
}

impl From<nip47::MakeInvoiceRequestParams> for MakeInvoiceRequestParams {
    fn from(value: nip47::MakeInvoiceRequestParams) -> Self {
        Self {
            amount: value.amount,
            description: value.description,
            description_hash: value.description_hash,
            preimage: value.preimage,
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
    pub bolt11: Option<String>,
}

impl From<nip47::LookupInvoiceRequestParams> for LookupInvoiceRequestParams {
    fn from(value: nip47::LookupInvoiceRequestParams) -> Self {
        Self {
            payment_hash: value.payment_hash,
            bolt11: value.bolt11,
        }
    }
}

/// List Invoice Request Params
#[derive(Record)]
pub struct ListInvoicesRequestParams {
    /// Starting timestamp in seconds since epoch
    pub from: Option<u64>,
    /// Ending timestamp in seconds since epoch
    pub until: Option<u64>,
    /// Number of invoices to return
    pub limit: Option<u64>,
    /// Offset of the first invoice to return
    pub offset: Option<u64>,
    /// If true, include unpaid invoices
    pub unpaid: Option<bool>,
}

impl From<nip47::ListInvoicesRequestParams> for ListInvoicesRequestParams {
    fn from(value: nip47::ListInvoicesRequestParams) -> Self {
        Self {
            from: value.from,
            until: value.until,
            limit: value.limit,
            offset: value.offset,
            unpaid: value.unpaid,
        }
    }
}

/// List Payments Request Params
#[derive(Record)]
pub struct ListPaymentsRequestParams {
    /// Starting timestamp in seconds since epoch
    pub from: Option<u64>,
    /// Ending timestamp in seconds since epoch
    pub until: Option<u64>,
    /// Number of invoices to return
    pub limit: Option<u64>,
    /// Offset of the first invoice to return
    pub offset: Option<u64>,
}

impl From<nip47::ListPaymentsRequestParams> for ListPaymentsRequestParams {
    fn from(value: nip47::ListPaymentsRequestParams) -> Self {
        Self {
            from: value.from,
            until: value.until,
            limit: value.limit,
            offset: value.offset,
        }
    }
}

/// NIP47 Request
#[derive(Record)]
pub struct Request {
    /// Request method
    pub method: Method,
    /// Params
    pub params: RequestParams,
}

impl From<nip47::Request> for Request {
    fn from(value: nip47::Request) -> Self {
        Self {
            method: value.method.into(),
            params: value.params.into(),
        }
    }
}

#[uniffi::export]
impl Request {
    #[uniffi::constructor]
    pub fn parse(json: String) -> Result<Self> {
        Ok(nip47::Request::from_json(json)?.into())
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

/// NIP47 Response Result
#[derive(Record)]
pub struct PayKeysendResponseResult {
    /// Response preimage
    pub preimage: String,
    /// Payment hash
    pub payment_hash: String,
}

impl From<nip47::PayKeysendResponseResult> for PayKeysendResponseResult {
    fn from(value: nip47::PayKeysendResponseResult) -> Self {
        Self {
            preimage: value.preimage,
            payment_hash: value.payment_hash,
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

/// NIP47 Response Result
#[derive(Record)]
pub struct LookupInvoiceResponseResult {
    /// Bolt11 invoice
    pub invoice: String,
    /// If the invoice has been paid
    pub paid: bool,
}

impl From<nip47::LookupInvoiceResponseResult> for LookupInvoiceResponseResult {
    fn from(value: nip47::LookupInvoiceResponseResult) -> Self {
        Self {
            invoice: value.invoice,
            paid: value.paid,
        }
    }
}

/// NIP47 Response Result
#[derive(Record)]
pub struct ListPaymentResponseResult {
    /// Bolt11 invoice
    pub invoice: String,
    /// Preimage for the payment
    pub preimage: Option<String>,
}

impl From<nip47::ListPaymentResponseResult> for ListPaymentResponseResult {
    fn from(value: nip47::ListPaymentResponseResult) -> Self {
        Self {
            invoice: value.invoice,
            preimage: value.preimage,
        }
    }
}

/// Budget renewal type
#[derive(Enum)]
pub enum BudgetType {
    /// Daily
    Daily,
    /// Weekly
    Weekly,
    /// Monthly
    Monthly,
    /// Yearly
    Yearly,
}

impl From<nip47::BudgetType> for BudgetType {
    fn from(value: nip47::BudgetType) -> Self {
        match value {
            nip47::BudgetType::Daily => Self::Daily,
            nip47::BudgetType::Weekly => Self::Weekly,
            nip47::BudgetType::Monthly => Self::Monthly,
            nip47::BudgetType::Yearly => Self::Yearly,
        }
    }
}

/// NIP47 Response Result
#[derive(Record)]
pub struct GetBalanceResponseResult {
    /// Balance amount in sats
    pub balance: u64,
    /// Max amount payable within current budget
    pub max_amount: Option<u64>,
    /// Budget renewal type
    pub budget_renewal: Option<BudgetType>,
}

impl From<nip47::GetBalanceResponseResult> for GetBalanceResponseResult {
    fn from(value: nip47::GetBalanceResponseResult) -> Self {
        Self {
            balance: value.balance,
            max_amount: value.max_amount,
            budget_renewal: value.budget_renewal.map(|b| b.into()),
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
    /// Pay Keysend
    PayKeysend {
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
    /// List Invoices
    ListInvoices {
        list_invoices: Vec<LookupInvoiceResponseResult>,
    },
    /// List Payments
    ListPayments {
        list_payments: Vec<ListPaymentResponseResult>,
    },
    /// Get Balance
    GetBalance {
        get_balance: GetBalanceResponseResult,
    },
}

impl From<nip47::ResponseResult> for ResponseResult {
    fn from(value: nip47::ResponseResult) -> Self {
        match value {
            nip47::ResponseResult::PayInvoice(pay_invoice) => Self::PayInvoice {
                pay_invoice: pay_invoice.into(),
            },
            nip47::ResponseResult::PayKeysend(pay_keysend) => Self::PayKeysend {
                pay_keysend: pay_keysend.into(),
            },
            nip47::ResponseResult::MakeInvoice(make_invoice) => Self::MakeInvoice {
                make_invoice: make_invoice.into(),
            },
            nip47::ResponseResult::LookupInvoice(lookup_invoice) => Self::LookupInvoice {
                lookup_invoice: lookup_invoice.into(),
            },
            nip47::ResponseResult::ListInvoices(list_invoices) => Self::ListInvoices {
                list_invoices: list_invoices.into_iter().map(|i| i.into()).collect(),
            },
            nip47::ResponseResult::ListPayments(list_payments) => Self::ListPayments {
                list_payments: list_payments.into_iter().map(|p| p.into()).collect(),
            },
            nip47::ResponseResult::GetBalance(get_balance) => Self::GetBalance {
                get_balance: get_balance.into(),
            },
        }
    }
}

/// NIP47 Response
#[derive(Record)]
pub struct Response {
    /// Request Method
    pub result_type: Method,
    /// NIP47 Error
    pub error: Option<NIP47Error>,
    /// NIP47 Result
    pub result: Option<ResponseResult>,
}

impl From<nip47::Response> for Response {
    fn from(value: nip47::Response) -> Self {
        Self {
            result_type: value.result_type.into(),
            error: value.error.map(|e| e.into()),
            result: value.result.map(|r| r.into()),
        }
    }
}

impl Response {
    /// Deserialize from JSON string
    #[uniffi::constructor]
    pub fn parse(json: String) -> Result<Self> {
        Ok(nip47::Response::from_json(json)?.into())
    }
}

/// Nostr Connect URI
#[derive(Record)]
pub struct NostrWalletConnectURI {
    /// App Pubkey
    pub public_key: Arc<PublicKey>,
    /// URL of the relay of choice where the `App` is connected and the `Signer` must send and listen for messages.
    pub relay_url: String,
    /// 32-byte randomly generated hex encoded string
    pub secret: Arc<SecretKey>,
    /// A lightning address that clients can use to automatically setup the lud16 field on the user's profile if they have none configured.
    pub lud16: Option<String>,
}

impl From<nip47::NostrWalletConnectURI> for NostrWalletConnectURI {
    fn from(value: nip47::NostrWalletConnectURI) -> Self {
        Self {
            public_key: Arc::new(value.public_key.into()),
            relay_url: value.relay_url.into(),
            secret: Arc::new(value.secret.into()),
            lud16: value.lud16,
        }
    }
}

#[uniffi::export]
impl NostrWalletConnectURI {
    #[uniffi::constructor]
    /// Create new [`NostrWalletConnectURI`]
    pub fn new(
        public_key: Arc<PublicKey>,
        relay_url: String,
        random_secret_key: Arc<SecretKey>,
        lud16: Option<String>,
    ) -> Result<Self> {
        Ok(nip47::NostrWalletConnectURI::new(
            **public_key,
            Url::parse(&relay_url)?,
            **random_secret_key,
            lud16,
        )?
        .into())
    }

    #[uniffi::constructor]
    pub fn from_string(uri: String) -> Result<Self> {
        Ok(nip47::NostrWalletConnectURI::from_str(&uri)?.into())
    }
}
