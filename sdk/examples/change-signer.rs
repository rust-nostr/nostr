use std::borrow::Cow;
use std::sync::Arc;

use nostr_sdk::prelude::*;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct MySignerSwitcher {
    signer: RwLock<Arc<dyn AsyncNostrSigner>>,
}

impl MySignerSwitcher {
    pub fn new<T>(signer: T) -> Self
    where
        T: IntoAsyncNostrSigner,
    {
        Self {
            signer: RwLock::new(signer.into_nostr_signer()),
        }
    }

    async fn get(&self) -> Arc<dyn AsyncNostrSigner> {
        self.signer.read().await.clone()
    }

    pub async fn switch<T>(&self, new: T)
    where
        T: IntoAsyncNostrSigner,
    {
        let mut signer = self.signer.write().await;
        *signer = new.into_nostr_signer();
    }
}

impl AsyncGetPublicKey for MySignerSwitcher {
    fn get_public_key(&self) -> BoxedFuture<'_, Result<PublicKey, SignerError>> {
        Box::pin(async move { self.get().await.get_public_key().await })
    }
}

impl AsyncSignEvent for MySignerSwitcher {
    fn sign_event(&self, unsigned: UnsignedEvent) -> BoxedFuture<'_, Result<Event, SignerError>> {
        Box::pin(async move { self.get().await.sign_event(unsigned).await })
    }
}

impl AsyncNip04 for MySignerSwitcher {
    type Error = SignerError;

    fn nip04_encrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        content: &'a str,
    ) -> BoxedFuture<'a, Result<String, Self::Error>> {
        Box::pin(async move { self.get().await.nip04_encrypt(public_key, content).await })
    }

    fn nip04_decrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        encrypted_content: &'a str,
    ) -> BoxedFuture<'a, Result<String, Self::Error>> {
        Box::pin(async move {
            self.get()
                .await
                .nip04_decrypt(public_key, encrypted_content)
                .await
        })
    }
}

impl AsyncNip44 for MySignerSwitcher {
    type Error = SignerError;

    fn nip44_encrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        content: &'a str,
    ) -> BoxedFuture<'a, Result<String, Self::Error>> {
        Box::pin(async move { self.get().await.nip44_encrypt(public_key, content).await })
    }

    fn nip44_decrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        payload: &'a str,
    ) -> BoxedFuture<'a, Result<String, Self::Error>> {
        Box::pin(async move { self.get().await.nip44_decrypt(public_key, payload).await })
    }
}

impl AsyncNostrSigner for MySignerSwitcher {
    fn backend(&self) -> SignerBackend<'_> {
        SignerBackend::Custom(Cow::Borrowed("custom"))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let keys = Keys::parse("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")?;
    let signer = Arc::new(MySignerSwitcher::new(keys));

    let client = Client::builder().signer(signer.clone()).build();

    let pk = client.signer().unwrap().get_public_key().await?;
    println!("Public Key: {}", pk.to_bech32()?);

    let new_keys = Keys::generate();
    signer.switch(new_keys).await;

    let pk = client.signer().unwrap().get_public_key().await?;
    println!("Public Key: {}", pk.to_bech32()?);

    Ok(())
}
