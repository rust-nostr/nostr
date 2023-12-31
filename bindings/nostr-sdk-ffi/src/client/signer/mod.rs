// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr_ffi::Keys;
use nostr_sdk::client::signer;
use uniffi::Enum;

pub mod nip46;

#[derive(Enum)]
pub enum ClientSigner {
    Keys { signer: Arc<Keys> },
    NIP46 { signer: Arc<nip46::Nip46Signer> },
}

impl From<ClientSigner> for signer::ClientSigner {
    fn from(value: ClientSigner) -> Self {
        match value {
            ClientSigner::Keys { signer } => Self::Keys(signer.as_ref().deref().clone()),
            ClientSigner::NIP46 { signer } => Self::NIP46(signer.as_ref().deref().clone()),
        }
    }
}

impl From<signer::ClientSigner> for ClientSigner {
    fn from(value: signer::ClientSigner) -> Self {
        match value {
            signer::ClientSigner::Keys(keys) => Self::Keys {
                signer: Arc::new(keys.into()),
            },
            signer::ClientSigner::NIP46(signer) => Self::NIP46 {
                signer: Arc::new(signer.into()),
            },
        }
    }
}
