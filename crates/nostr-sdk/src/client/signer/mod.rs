// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Client Signers

use std::fmt;

use nostr::prelude::*;

#[cfg(feature = "nip46")]
pub mod nip46;

#[cfg(feature = "nip46")]
use self::nip46::Nip46Signer;
use super::Error;

/// Client Signer Type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ClientSignerType {
    /// Keys
    Keys,
    /// NIP07
    #[cfg(all(feature = "nip07", target_arch = "wasm32"))]
    NIP07,
    /// NIP46
    #[cfg(feature = "nip46")]
    NIP46,
}

// TODO: better display
impl fmt::Display for ClientSignerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Keys => write!(f, "Keys"),
            #[cfg(all(feature = "nip07", target_arch = "wasm32"))]
            Self::NIP07 => write!(f, "NIP07"),
            #[cfg(feature = "nip46")]
            Self::NIP46 => write!(f, "NIP46"),
        }
    }
}

/// Client signer
#[derive(Debug, Clone)]
pub enum ClientSigner {
    /// Private Keys
    Keys(Keys),
    /// NIP07 signer
    #[cfg(all(feature = "nip07", target_arch = "wasm32"))]
    NIP07(Nip07Signer),
    /// NIP46 signer
    #[cfg(feature = "nip46")]
    NIP46(Box<Nip46Signer>),
}

impl ClientSigner {
    /// Create a new [NIP07] instance and compose [ClientSigner]
    #[cfg(all(feature = "nip07", target_arch = "wasm32"))]
    pub fn nip07() -> Result<Self, Error> {
        let instance = Nip07Signer::new()?;
        Ok(Self::NIP07(instance))
    }

    /// Get Client Signer Type
    pub fn r#type(&self) -> ClientSignerType {
        match self {
            Self::Keys(..) => ClientSignerType::Keys,
            #[cfg(all(feature = "nip07", target_arch = "wasm32"))]
            Self::NIP07(..) => ClientSignerType::NIP07,
            #[cfg(feature = "nip46")]
            Self::NIP46(..) => ClientSignerType::NIP46,
        }
    }

    /// Get signer public key
    pub async fn public_key(&self) -> Result<XOnlyPublicKey, Error> {
        match self {
            Self::Keys(keys) => Ok(keys.public_key()),
            #[cfg(all(feature = "nip07", target_arch = "wasm32"))]
            Self::NIP07(s) => Ok(s.get_public_key().await?),
            #[cfg(feature = "nip46")]
            Self::NIP46(s) => Ok(s.signer_public_key()),
        }
    }

    /// Sign an [UnsignedEvent]
    pub async fn sign_event(&self, unsigned: UnsignedEvent) -> Result<Event, Error> {
        match self {
            ClientSigner::Keys(keys) => Ok(unsigned.sign(keys)?),
            #[cfg(all(feature = "nip07", target_arch = "wasm32"))]
            ClientSigner::NIP07(nip07) => Ok(nip07.sign_event(unsigned).await?),
            #[cfg(feature = "nip46")]
            ClientSigner::NIP46(nip46) => {
                let res = nip46
                    .send_req_to_signer(nostr::nips::nip46::Request::SignEvent(unsigned), None)
                    .await?;
                if let nostr::nips::nip46::Response::SignEvent(event) = res {
                    Ok(event)
                } else {
                    Err(Error::ResponseNotMatchRequest)
                }
            }
        }
    }

    /// NIP04 encrypt
    #[cfg(feature = "nip04")]
    pub async fn nip04_encrypt<T>(
        &self,
        public_key: XOnlyPublicKey,
        content: T,
    ) -> Result<String, Error>
    where
        T: AsRef<str>,
    {
        let content: &str = content.as_ref();
        match self {
            ClientSigner::Keys(keys) => {
                Ok(nip04::encrypt(&keys.secret_key()?, &public_key, content)?)
            }
            #[cfg(all(feature = "nip07", target_arch = "wasm32"))]
            ClientSigner::NIP07(signer) => Ok(signer.nip04_encrypt(public_key, content).await?),
            #[cfg(feature = "nip46")]
            ClientSigner::NIP46(signer) => {
                let req = nostr::nips::nip46::Request::Nip04Encrypt {
                    public_key,
                    text: content.to_string(),
                };
                let res: nostr::nips::nip46::Response =
                    signer.send_req_to_signer(req, None).await?;
                if let nostr::nips::nip46::Response::Nip04Encrypt(ciphertext) = res {
                    Ok(ciphertext)
                } else {
                    Err(Error::ResponseNotMatchRequest)
                }
            }
        }
    }

    /// NIP04 decrypt
    #[cfg(feature = "nip04")]
    pub async fn nip04_decrypt<T>(
        &self,
        public_key: XOnlyPublicKey,
        encrypted_content: T,
    ) -> Result<String, Error>
    where
        T: AsRef<str>,
    {
        let encrypted_content: &str = encrypted_content.as_ref();
        match self {
            ClientSigner::Keys(keys) => Ok(nip04::decrypt(
                &keys.secret_key()?,
                &public_key,
                encrypted_content,
            )?),
            #[cfg(all(feature = "nip07", target_arch = "wasm32"))]
            ClientSigner::NIP07(signer) => {
                Ok(signer.nip04_decrypt(public_key, encrypted_content).await?)
            }
            #[cfg(feature = "nip46")]
            ClientSigner::NIP46(signer) => {
                let req = nostr::nips::nip46::Request::Nip04Decrypt {
                    public_key,
                    text: encrypted_content.to_string(),
                };
                let res: nostr::nips::nip46::Response =
                    signer.send_req_to_signer(req, None).await?;
                if let nostr::nips::nip46::Response::Nip04Decrypt(content) = res {
                    Ok(content)
                } else {
                    Err(Error::ResponseNotMatchRequest)
                }
            }
        }
    }

    /// NIP44 encryption with [ClientSigner]
    #[cfg(feature = "nip44")]
    pub async fn nip44_encrypt<T>(
        &self,
        public_key: XOnlyPublicKey,
        content: T,
    ) -> Result<String, Error>
    where
        T: AsRef<[u8]>,
    {
        match self {
            ClientSigner::Keys(keys) => Ok(nip44::encrypt(
                &keys.secret_key()?,
                &public_key,
                content,
                nip44::Version::default(),
            )?),
            #[cfg(all(feature = "nip07", target_arch = "wasm32"))]
            ClientSigner::NIP07(..) => Err(Error::unsupported(
                "NIP44 encryption not supported with NIP07 signer yet!",
            )),
            #[cfg(feature = "nip46")]
            ClientSigner::NIP46(..) => Err(Error::unsupported(
                "NIP44 encryption not supported with NIP46 signer yet!",
            )),
        }
    }

    /// NIP44 decryption with [ClientSigner]
    #[cfg(feature = "nip44")]
    pub async fn nip44_decrypt<T>(
        &self,
        public_key: XOnlyPublicKey,
        payload: T,
    ) -> Result<String, Error>
    where
        T: AsRef<[u8]>,
    {
        match self {
            ClientSigner::Keys(keys) => {
                Ok(nip44::decrypt(&keys.secret_key()?, &public_key, payload)?)
            }
            #[cfg(all(feature = "nip07", target_arch = "wasm32"))]
            ClientSigner::NIP07(..) => Err(Error::unsupported(
                "NIP44 decryption not supported with NIP07 signer yet!",
            )),
            #[cfg(feature = "nip46")]
            ClientSigner::NIP46(..) => Err(Error::unsupported(
                "NIP44 decryption not supported with NIP46 signer yet!",
            )),
        }
    }
}

impl From<Keys> for ClientSigner {
    fn from(keys: Keys) -> Self {
        Self::Keys(keys)
    }
}

impl From<&Keys> for ClientSigner {
    fn from(keys: &Keys) -> Self {
        Self::Keys(keys.clone())
    }
}

#[cfg(all(feature = "nip07", target_arch = "wasm32"))]
impl From<Nip07Signer> for ClientSigner {
    fn from(nip07: Nip07Signer) -> Self {
        Self::NIP07(nip07)
    }
}

#[cfg(feature = "nip46")]
impl From<Nip46Signer> for ClientSigner {
    fn from(nip46: Nip46Signer) -> Self {
        Self::NIP46(Box::new(nip46))
    }
}
