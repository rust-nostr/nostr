// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIP47
//!
//! <https://github.com/nostr-protocol/nips/blob/master/47.md>

use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;

use secp256k1::{SecretKey, XOnlyPublicKey};
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::form_urlencoded::byte_serialize;
use url::Url;

use super::nip04;
use crate::{key, Keys};

/// NIP47 error
#[derive(Debug)]
pub enum Error {
    /// Key error
    Key(key::Error),
    /// JSON error
    JSON(serde_json::Error),
    /// Url parse error
    Url(url::ParseError),
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

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key(e) => write!(f, "{e}"),
            Self::JSON(e) => write!(f, "{e}"),
            Self::Url(e) => write!(f, "{e}"),
            Self::Secp256k1(e) => write!(f, "{e}"),
            Self::NIP04(e) => write!(f, "{e}"),
            Self::UnsignedEvent(e) => write!(f, "{e}"),
            Self::InvalidRequest => write!(f, "Invalid NIP47 Request"),
            Self::InvalidParamsLength => write!(f, "Invalid NIP47 Params length"),
            Self::UnsupportedMethod(e) => write!(f, "{e}"),
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

impl From<url::ParseError> for Error {
    fn from(e: url::ParseError) -> Self {
        Self::Url(e)
    }
}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Self {
        Self::Secp256k1(e)
    }
}

impl From<key::Error> for Error {
    fn from(e: key::Error) -> Self {
        Self::Key(e)
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
    InsufficantBalance,
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
}

/// Request Params
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequestParams {
    /// Request invoice
    pub invoice: String,
}

/// NIP47 Request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Request {
    /// Request method
    pub method: Method,
    /// Params
    pub params: RequestParams,
}

impl Request {
    /// Serialize [`Message`] as JSON string
    pub fn as_json(&self) -> String {
        json!(self).to_string()
    }

    /// Deserialize from JSON string
    pub fn from_json<S>(json: S) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        match serde_json::from_str(json.as_ref()) {
            Ok(response) => Ok(response),
            Err(_err) => {
                let json = json.as_ref().replace('\\', "");
                Ok(serde_json::from_str(&json)?)
            }
        }
    }
}

/// NIP47 Response Result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseResult {
    /// Response preimage
    pub preimage: String,
}

/// NIP47 Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// Request Method
    pub result_type: Method,
    /// NIP47 Error
    pub error: Option<NIP47Error>,
    /// NIP47 Result
    pub result: Option<ResponseResult>,
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
        match serde_json::from_str(json.as_ref()) {
            Ok(response) => Ok(response),
            Err(_err) => {
                let json = json.as_ref().replace('\\', "");
                Ok(serde_json::from_str(&json)?)
            }
        }
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
        secret: Option<SecretKey>,
        lud16: Option<String>,
    ) -> Result<Self, Error> {
        let secret = match secret {
            Some(secret) => secret,
            None => {
                let keys = Keys::generate();
                keys.secret_key()?
            }
        };

        Ok(Self {
            public_key,
            relay_url,
            secret,
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

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;
    use crate::{key::FromSkStr, Result};

    #[test]
    fn test_uri() -> Result<()> {
        let pubkey = XOnlyPublicKey::from_str(
            "b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4",
        )?;
        let relay_url = Url::parse("wss://relay.damus.io")?;
        let secret =
            Keys::from_sk_str("71a8c14c1407c113601079c4302dab36460f0ccd0ad506f1f2dc73b5100e4f3c")?;
        let uri = NostrWalletConnectURI::new(
            pubkey,
            relay_url,
            Some(secret.secret_key()?),
            Some("nostr@nostr.com".to_string()),
        )?;
        assert_eq!(
            uri.to_string(),
            "nostr+walletconnect://b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4?relay=wss%3A%2F%2Frelay.damus.io%2F&secret=71a8c14c1407c113601079c4302dab36460f0ccd0ad506f1f2dc73b5100e4f3c&lud16=nostr%40nostr.com".to_string()
        );
        Ok(())
    }

    #[test]
    fn test_parse_uri() -> Result<()> {
        let uri = "nostr+walletconnect://b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4?relay=wss%3A%2F%2Frelay.damus.io%2F&secret=71a8c14c1407c113601079c4302dab36460f0ccd0ad506f1f2dc73b5100e4f3c&lud16=nostr%40nostr.com";
        let uri = NostrWalletConnectURI::from_str(uri)?;

        let pubkey = XOnlyPublicKey::from_str(
            "b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4",
        )?;
        let relay_url = Url::parse("wss://relay.damus.io")?;
        let secret =
            Keys::from_sk_str("71a8c14c1407c113601079c4302dab36460f0ccd0ad506f1f2dc73b5100e4f3c")?;
        assert_eq!(
            uri,
            NostrWalletConnectURI::new(
                pubkey,
                relay_url,
                Some(secret.secret_key()?),
                Some("nostr@nostr.com".to_string())
            )
            .unwrap()
        );
        Ok(())
    }

    #[test]
    fn seralize_request() -> Result<()> {
        let request = Request {
            method: Method::PayInvoice,
            params: RequestParams { invoice: "lnbc210n1pj99rx0pp5ehevgz9nf7d97h05fgkdeqxzytm6yuxd7048axru03fpzxxvzt7shp5gv7ef0s26pw5gy5dpwvsh6qgc8se8x2lmz2ev90l9vjqzcns6u6scqzzsxqyz5vqsp".to_string() }            
        };

        assert_eq!(Request::from_json(request.as_json()).unwrap(), request);

        assert_eq!(request.as_json(), "{\"method\":\"pay_invoice\",\"params\":{\"invoice\":\"lnbc210n1pj99rx0pp5ehevgz9nf7d97h05fgkdeqxzytm6yuxd7048axru03fpzxxvzt7shp5gv7ef0s26pw5gy5dpwvsh6qgc8se8x2lmz2ev90l9vjqzcns6u6scqzzsxqyz5vqsp\"}}");
        Ok(())
    }

    #[test]
    fn test_parse_request() -> Result<()> {
        let request = "{\\\"params\\\":{\\\"invoice\\\":\\\"lnbc210n1pj99rx0pp5ehevgz9nf7d97h05fgkdeqxzytm6yuxd7048axru03fpzxxvzt7shp5gv7ef0s26pw5gy5dpwvsh6qgc8se8x2lmz2ev90l9vjqzcns6u6scqzzsxqyz5vqsp5rdjyt9jr2avv2runy330766avkweqp30ndnyt9x6dp5juzn7q0nq9qyyssq2mykpgu04q0hlga228kx9v95meaqzk8a9cnvya305l4c353u3h04azuh9hsmd503x6jlzjrsqzark5dxx30s46vuatwzjhzmkt3j4tgqu35rms\\\"},\\\"method\\\":\\\"pay_invoice\\\"}";

        let request = Request::from_json(request).unwrap();

        assert_eq!(request.method, Method::PayInvoice);
        assert_eq!(request.params.invoice, "lnbc210n1pj99rx0pp5ehevgz9nf7d97h05fgkdeqxzytm6yuxd7048axru03fpzxxvzt7shp5gv7ef0s26pw5gy5dpwvsh6qgc8se8x2lmz2ev90l9vjqzcns6u6scqzzsxqyz5vqsp5rdjyt9jr2avv2runy330766avkweqp30ndnyt9x6dp5juzn7q0nq9qyyssq2mykpgu04q0hlga228kx9v95meaqzk8a9cnvya305l4c353u3h04azuh9hsmd503x6jlzjrsqzark5dxx30s46vuatwzjhzmkt3j4tgqu35rms".to_string());
        Ok(())
    }
}
