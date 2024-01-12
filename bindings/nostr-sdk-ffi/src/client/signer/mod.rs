// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr_ffi::Keys;
use nostr_sdk::client::signer;
use uniffi::Object;

pub mod nip46;

use self::nip46::Nip46Signer;

#[derive(Object)]
pub struct ClientSigner {
    inner: signer::ClientSigner,
}

impl Deref for ClientSigner {
    type Target = signer::ClientSigner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<signer::ClientSigner> for ClientSigner {
    fn from(inner: signer::ClientSigner) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl ClientSigner {
    #[uniffi::constructor]
    pub fn keys(keys: Arc<Keys>) -> Self {
        Self {
            inner: signer::ClientSigner::Keys(keys.as_ref().deref().clone()),
        }
    }

    #[uniffi::constructor]
    pub fn nip46(nip46: Arc<Nip46Signer>) -> Self {
        Self {
            inner: signer::ClientSigner::NIP46(nip46.as_ref().deref().clone()),
        }
    }
}
