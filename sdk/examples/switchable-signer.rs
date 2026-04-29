use std::sync::Arc;

use nostr_sdk::prelude::*;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct MySignerSwitcher {
    signer: Arc<RwLock<Arc<dyn AsyncNostrSigner>>>,
}

impl MySignerSwitcher {
    pub fn new<T>(signer: T) -> Self
    where
        T: AsyncNostrSigner,
    {
        Self {
            signer: Arc::new(RwLock::new(Arc::new(signer))),
        }
    }

    async fn get(&self) -> Arc<dyn AsyncNostrSigner> {
        self.signer.read().await.clone()
    }

    pub async fn switch<T>(&self, new: T)
    where
        T: AsyncNostrSigner,
    {
        let mut signer = self.signer.write().await;
        *signer = Arc::new(new);
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt::init();

    let url = RelayUrl::parse("wss://relay.damus.io")?;

    let keys = Keys::parse("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")?;
    let signer = MySignerSwitcher::new(keys.clone());

    let authenticator = SignerAuthenticator::new(signer.clone());

    let pk = signer.get_public_key().await?;
    assert_eq!(pk, keys.public_key);
    println!("Public Key: {}", pk.to_bech32()?);

    // The authenticator uses the same signer
    let auth_event = authenticator.make_auth_event(&url, "test").await?;
    assert_eq!(auth_event.pubkey, pk);

    let new_keys = Keys::generate();
    signer.switch(new_keys.clone()).await;

    let pk = signer.get_public_key().await?;
    assert_eq!(pk, new_keys.public_key);
    println!("Public Key: {}", pk.to_bech32()?);

    // The authenticator uses the same signer, so the pk must switch as well
    let auth_event = authenticator.make_auth_event(&url, "test2").await?;
    assert_eq!(auth_event.pubkey, pk);

    Ok(())
}
