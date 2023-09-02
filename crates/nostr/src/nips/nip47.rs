// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIP47
//!
//! <https://github.com/nostr-protocol/nips/blob/master/47.md>

use alloc::string::String;
use alloc::{borrow::Cow, string::ToString};
use core::fmt;
use core::str::FromStr;

use bitcoin::secp256k1::{self, SecretKey, XOnlyPublicKey};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{json, Value};
use url_fork::form_urlencoded::byte_serialize;
use url_fork::{ParseError, Url};

use super::nip04;

/// NIP47 error
#[derive(Debug)]
pub enum Error {
    /// JSON error
    JSON(serde_json::Error),
    /// Url parse error
    Url(ParseError),
    /// Secp256k1 error
    Secp256k1(secp256k1::Error),
    /// NIP04 error
    NIP04(nip04::Error),
    /// Unsigned event error
    UnsignedEvent(crate::event::unsigned::Error),
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
            Self::Secp256k1(e) => write!(f, "Secp256k1: {e}"),
            Self::NIP04(e) => write!(f, "NIP04: {e}"),
            Self::UnsignedEvent(e) => write!(f, "Unsigned event: {e}"),
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

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Self {
        Self::Secp256k1(e)
    }
}

/// NIP47 Response Error codes
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// NIP47 Error message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NIP47Error {
    /// Error Code
    pub code: ErrorCode,
    /// Human Readable error message
    pub message: String,
}

/// Method
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Method {
    /// Pay Invoice
    #[serde(rename = "pay_invoice")]
    PayInvoice,
    /// Make Invoice
    #[serde(rename = "make_invoice")]
    MakeInvoice,
    /// Lookup Invoice
    #[serde(rename = "lookup_invoice")]
    LookupInvoice,
    /// Get Balance
    #[serde(rename = "get_balance")]
    GetBalance,
}

/// Nostr Wallet Connect Request Params
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestParams {
    /// Pay Invoice
    PayInvoice(PayInvoiceRequestParams),
    /// Make Invoice
    MakeInvoice(MakeInvoiceRequestParams),
    /// Lookup Invoice
    LookupInvoice(LookupInvoiceRequestParams),
    /// Get Balance
    GetBalance,
}

impl Serialize for RequestParams {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            RequestParams::PayInvoice(p) => p.serialize(serializer),
            RequestParams::MakeInvoice(p) => p.serialize(serializer),
            RequestParams::LookupInvoice(p) => p.serialize(serializer),
            RequestParams::GetBalance => serializer.serialize_none(),
        }
    }
}

/// Pay Invoice Request Params
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PayInvoiceRequestParams {
    /// Request invoice
    pub invoice: String,
}

/// Make Invoice Request Params
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MakeInvoiceRequestParams {
    /// Amount in millisatoshis
    pub amount: i64,
    /// Invoice description
    pub description: Option<String>,
    /// Invoice description hash
    pub description_hash: Option<String>,
    /// Invoice expiry in seconds
    pub expiry: Option<i64>,
}

/// Lookup Invoice Request Params
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LookupInvoiceRequestParams {
    /// Payment hash of invoice
    pub payment_hash: Option<String>,
    /// Bolt11 invoice
    pub bolt11: Option<String>,
}

/// NIP47 Request
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
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
    params: Value,
}

impl Request {
    /// Serialize [`Request`] as JSON string
    pub fn as_json(&self) -> String {
        json!(self).to_string()
    }

    /// Deserialize from JSON string
    pub fn from_json<S>(json: S) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        let template: RequestTemplate = match serde_json::from_str(json.as_ref()) {
            Ok(response) => response,
            Err(_err) => {
                let json = json.as_ref().replace('\\', "");
                serde_json::from_str(&json)?
            }
        };

        let params = match template.method {
            Method::PayInvoice => {
                let params: PayInvoiceRequestParams = serde_json::from_value(template.params)?;
                RequestParams::PayInvoice(params)
            }
            Method::MakeInvoice => {
                let params: MakeInvoiceRequestParams = serde_json::from_value(template.params)?;
                RequestParams::MakeInvoice(params)
            }
            Method::LookupInvoice => {
                let params: LookupInvoiceRequestParams = serde_json::from_value(template.params)?;
                RequestParams::LookupInvoice(params)
            }
            Method::GetBalance => RequestParams::GetBalance,
        };

        Ok(Self {
            method: template.method,
            params,
        })
    }
}

impl<'de> Deserialize<'de> for Request {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::from_json(
            Value::deserialize(deserializer)
                .map_err(serde::de::Error::custom)?
                .to_string()
                .as_str(),
        )
        .map_err(serde::de::Error::custom)
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
pub struct MakeInvoiceResponseResult {
    /// Bolt 11 invoice
    pub invoice: String,
    /// Invoice's payment hash
    pub payment_hash: String,
}

/// NIP47 Response Result
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct LookupInvoiceResponseResult {
    /// Bolt11 invoice
    pub invoice: String,
    /// If the invoice has been paid
    pub paid: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// Budget renewal type
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

/// NIP47 Response Result
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct GetBalanceResponseResult {
    /// Balance amount in sats
    pub balance: u64,
    /// Max amount payable within current budget
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_amount: Option<u64>,
    /// Budget renewal type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_renewal: Option<BudgetType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// NIP47 Response Result
pub enum ResponseResult {
    /// Pay Invoice
    PayInvoice(PayInvoiceResponseResult),
    /// Make Invoice
    MakeInvoice(MakeInvoiceResponseResult),
    /// Lookup Invoice
    LookupInvoice(LookupInvoiceResponseResult),
    /// Get Balance
    GetBalance(GetBalanceResponseResult),
}

impl Serialize for ResponseResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ResponseResult::PayInvoice(p) => p.serialize(serializer),
            ResponseResult::MakeInvoice(p) => p.serialize(serializer),
            ResponseResult::LookupInvoice(p) => p.serialize(serializer),
            ResponseResult::GetBalance(p) => p.serialize(serializer),
        }
    }
}

/// NIP47 Response
#[derive(Debug, Clone, Serialize)]
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
    /// Serialize [`Response`] as JSON string
    pub fn as_json(&self) -> String {
        json!(self).to_string()
    }

    /// Deserialize from JSON string
    pub fn from_json<S>(json: S) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        let template: ResponseTemplate = serde_json::from_str(json.as_ref())?;

        if let Some(result) = template.result {
            let result = match template.result_type {
                Method::PayInvoice => {
                    let result: PayInvoiceResponseResult = serde_json::from_value(result)?;
                    ResponseResult::PayInvoice(result)
                }
                Method::MakeInvoice => {
                    let result: MakeInvoiceResponseResult = serde_json::from_value(result)?;
                    ResponseResult::MakeInvoice(result)
                }
                Method::LookupInvoice => {
                    let result: LookupInvoiceResponseResult = serde_json::from_value(result)?;
                    ResponseResult::LookupInvoice(result)
                }
                Method::GetBalance => {
                    let result: GetBalanceResponseResult = serde_json::from_value(result)?;
                    ResponseResult::GetBalance(result)
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
}

impl<'de> Deserialize<'de> for Response {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::from_json(
            Value::deserialize(deserializer)
                .map_err(serde::de::Error::custom)?
                .to_string()
                .as_str(),
        )
        .map_err(serde::de::Error::custom)
    }
}

/// Nostr Wallet Connect Info
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NostrWalletConnectInfo {}

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
    pub public_key: XOnlyPublicKey,
    /// URL of the relay of choice where the `App` is connected and the `Signer` must send and listen for messages.
    pub relay_url: Url,
    /// 32-byte randomly generated hex encoded string
    pub secret: SecretKey,
    /// A lightning address that clients can use to automatically setup the lud16 field on the user's profile if they have none configured.
    pub lud16: Option<String>,
}

impl NostrWalletConnectURI {
    /// Create new [`NostrWalletConnectURI`]
    pub fn new(
        public_key: XOnlyPublicKey,
        relay_url: Url,
        random_secret_key: SecretKey,
        lud16: Option<String>,
    ) -> Result<Self, Error> {
        Ok(Self {
            public_key,
            relay_url,
            secret: random_secret_key,
            lud16,
        })
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
            let public_key = XOnlyPublicKey::from_str(pubkey)?;

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
        write!(
            f,
            "{NOSTR_WALLET_CONNECT_URI_SCHEME}://{}?relay={}&secret={}",
            self.public_key,
            url_encode(self.relay_url.to_string()),
            url_encode(self.secret.display_secret().to_string())
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
mod test {
    use core::str::FromStr;

    use super::*;

    #[test]
    fn test_uri() {
        let pubkey = XOnlyPublicKey::from_str(
            "b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4",
        )
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
        )
        .unwrap();
        assert_eq!(
            uri.to_string(),
            "nostr+walletconnect://b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4?relay=wss%3A%2F%2Frelay.damus.io%2F&secret=71a8c14c1407c113601079c4302dab36460f0ccd0ad506f1f2dc73b5100e4f3c&lud16=nostr%40nostr.com".to_string()
        );
    }

    #[test]
    fn test_parse_uri() {
        let uri = "nostr+walletconnect://b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4?relay=wss%3A%2F%2Frelay.damus.io%2F&secret=71a8c14c1407c113601079c4302dab36460f0ccd0ad506f1f2dc73b5100e4f3c&lud16=nostr%40nostr.com";
        let uri = NostrWalletConnectURI::from_str(uri).unwrap();

        let pubkey = XOnlyPublicKey::from_str(
            "b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4",
        )
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
            .unwrap()
        );
    }

    #[test]
    fn seralize_request() {
        let request = Request {
            method: Method::PayInvoice,
            params: RequestParams::PayInvoice(PayInvoiceRequestParams { invoice: "lnbc210n1pj99rx0pp5ehevgz9nf7d97h05fgkdeqxzytm6yuxd7048axru03fpzxxvzt7shp5gv7ef0s26pw5gy5dpwvsh6qgc8se8x2lmz2ev90l9vjqzcns6u6scqzzsxqyz5vqsp".to_string() }),
        };

        assert_eq!(Request::from_json(request.as_json()).unwrap(), request);

        assert_eq!(request.as_json(), "{\"method\":\"pay_invoice\",\"params\":{\"invoice\":\"lnbc210n1pj99rx0pp5ehevgz9nf7d97h05fgkdeqxzytm6yuxd7048axru03fpzxxvzt7shp5gv7ef0s26pw5gy5dpwvsh6qgc8se8x2lmz2ev90l9vjqzcns6u6scqzzsxqyz5vqsp\"}}");
    }

    #[test]
    fn test_parse_request() {
        let request = "{\\\"params\\\":{\\\"invoice\\\":\\\"lnbc210n1pj99rx0pp5ehevgz9nf7d97h05fgkdeqxzytm6yuxd7048axru03fpzxxvzt7shp5gv7ef0s26pw5gy5dpwvsh6qgc8se8x2lmz2ev90l9vjqzcns6u6scqzzsxqyz5vqsp5rdjyt9jr2avv2runy330766avkweqp30ndnyt9x6dp5juzn7q0nq9qyyssq2mykpgu04q0hlga228kx9v95meaqzk8a9cnvya305l4c353u3h04azuh9hsmd503x6jlzjrsqzark5dxx30s46vuatwzjhzmkt3j4tgqu35rms\\\"},\\\"method\\\":\\\"pay_invoice\\\"}";

        let request = Request::from_json(request).unwrap();

        assert_eq!(request.method, Method::PayInvoice);

        if let RequestParams::PayInvoice(pay) = request.params {
            assert_eq!(pay.invoice, "lnbc210n1pj99rx0pp5ehevgz9nf7d97h05fgkdeqxzytm6yuxd7048axru03fpzxxvzt7shp5gv7ef0s26pw5gy5dpwvsh6qgc8se8x2lmz2ev90l9vjqzcns6u6scqzzsxqyz5vqsp5rdjyt9jr2avv2runy330766avkweqp30ndnyt9x6dp5juzn7q0nq9qyyssq2mykpgu04q0hlga228kx9v95meaqzk8a9cnvya305l4c353u3h04azuh9hsmd503x6jlzjrsqzark5dxx30s46vuatwzjhzmkt3j4tgqu35rms".to_string());
        } else {
            panic!("Invalid request params");
        }
    }
}
