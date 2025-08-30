use anyhow::{Context, Result};
use nostr::{Event, JsonUtil, PublicKey, UnsignedEvent};
use rsbinder::{self, hub, ProcessState, Strong, Tokio};

mod aidl_signer;

use self::aidl_signer::com::nostr::signer::ISigner::ISignerAsync;

// Define the name of the service to be registered in the HUB(service manager).
const SERVICE_NAME: &str = "nostr_nip55_signer";

/// Android signer client (NIP-55)
pub struct AndroidSigner {
    signer: Strong<dyn ISignerAsync<Tokio>>,
}

impl AndroidSigner {
    pub fn new() -> Result<Self> {
        ProcessState::init_default();

        let service = hub::get_interface(SERVICE_NAME).context("getting signer service")?;

        Ok(Self { signer: service })
    }

    pub async fn get_public_key(&self) -> Result<PublicKey> {
        #[allow(non_snake_case)]
        let pk: String = self.signer.getPublicKey().await?;

        Ok(PublicKey::from_hex(&pk)?)
    }

    pub async fn sign_event(&self, event: &UnsignedEvent) -> Result<Event> {
        let json: String = event.as_json();

        #[allow(non_snake_case)]
        let event: String = self.signer.signEvent(&json).await?;

        let event: Event = Event::from_json(&event)?;

        event.verify()?;

        Ok(event)
    }
}
