use std::borrow::Cow;

use anyhow::{Context, Result};
use nostr::prelude::{BoxedFuture, SignerBackend};
use nostr::{Event, JsonUtil, NostrSigner, PublicKey, SignerError, UnsignedEvent};
use rsbinder::{self, hub, ProcessState, Strong, Tokio};

mod aidl;

use self::aidl::com::nostr::signer::INostrSigner::INostrSignerAsync;

// Define the name of the service to be registered in the HUB(service manager).
const SERVICE_NAME: &str = "nostr_nip55_signer";

/// Android signer client (NIP-55)
#[derive(Debug)]
pub struct AndroidSigner {
    signer: Strong<dyn INostrSignerAsync<Tokio>>,
    // TODO: cache public key of current user in a OnceCell
}

impl AndroidSigner {
    pub fn new() -> Result<Self> {
        ProcessState::init_default();

        let service = hub::get_interface(SERVICE_NAME).context("getting signer service")?;

        Ok(Self { signer: service })
    }
}

impl NostrSigner for AndroidSigner {
    fn backend(&self) -> SignerBackend {
        SignerBackend::Custom(Cow::Borrowed("android"))
    }

    fn get_public_key(&self) -> BoxedFuture<Result<PublicKey, SignerError>> {
        Box::pin(async move {
            #[allow(non_snake_case)]
            let pk: String = self
                .signer
                .getPublicKey()
                .await
                .map_err(SignerError::backend)?;

            PublicKey::from_hex(&pk).map_err(SignerError::backend)
        })
    }

    fn sign_event(&self, unsigned: UnsignedEvent) -> BoxedFuture<Result<Event, SignerError>> {
        Box::pin(async move {
            let json: String = unsigned.as_json();

            #[allow(non_snake_case)]
            let event: String = self
                .signer
                .signEvent(&json)
                .await
                .map_err(SignerError::backend)?;

            let event: Event = Event::from_json(&event).map_err(SignerError::backend)?;

            event.verify().map_err(SignerError::backend)?;

            Ok(event)
        })
    }

    fn nip04_encrypt<'a>(
        &'a self,
        _public_key: &'a PublicKey,
        _content: &'a str,
    ) -> BoxedFuture<'a, Result<String, SignerError>> {
        todo!()
    }

    fn nip04_decrypt<'a>(
        &'a self,
        _public_key: &'a PublicKey,
        _encrypted_content: &'a str,
    ) -> BoxedFuture<'a, Result<String, SignerError>> {
        todo!()
    }

    fn nip44_encrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        content: &'a str,
    ) -> BoxedFuture<'a, Result<String, SignerError>> {
        Box::pin(async move {
            #[allow(non_snake_case)]
            let current_user_public_key: String = self
                .signer
                .getPublicKey()
                .await
                .map_err(SignerError::backend)?;

            let public_key = public_key.to_hex();

            #[allow(non_snake_case)]
            self.signer
                .nip44Encrypt(&current_user_public_key, &public_key, content)
                .await
                .map_err(SignerError::backend)
        })
    }

    fn nip44_decrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        payload: &'a str,
    ) -> BoxedFuture<'a, Result<String, SignerError>> {
        Box::pin(async move {
            #[allow(non_snake_case)]
            let current_user_public_key: String = self
                .signer
                .getPublicKey()
                .await
                .map_err(SignerError::backend)?;

            let public_key = public_key.to_hex();

            #[allow(non_snake_case)]
            self.signer
                .nip44Decrypt(&current_user_public_key, &public_key, payload)
                .await
                .map_err(SignerError::backend)
        })
    }
}
