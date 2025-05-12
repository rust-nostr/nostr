use std::sync::Arc;

use nostr::prelude::BoxedFuture;
use nostr::{Event, EventBuilder, NostrSigner, RelayUrl};
use nostr_relay_pool::policy::{AuthenticationMiddleware, PolicyError};
use tokio::sync::RwLock;

use super::error::Error;

#[derive(Debug)]
pub(crate) struct DefaultAuthMiddleware {
    pub(crate) signer: Arc<RwLock<Option<Arc<dyn NostrSigner>>>>,
}

impl AuthenticationMiddleware for DefaultAuthMiddleware {
    fn is_ready(&self) -> BoxedFuture<'_, bool> {
        Box::pin(async move { self.signer.read().await.is_some() })
    }

    fn authenticate<'a>(
        &'a self,
        _relay_url: &'a RelayUrl,
        builder: EventBuilder,
    ) -> BoxedFuture<'a, Result<Event, PolicyError>> {
        Box::pin(async move {
            let signer = self.signer.read().await;
            let signer: &Arc<dyn NostrSigner> = signer
                .as_ref()
                .ok_or(PolicyError::backend(Error::SignerNotConfigured))?;
            builder.sign(signer).await.map_err(PolicyError::backend)
        })
    }
}
