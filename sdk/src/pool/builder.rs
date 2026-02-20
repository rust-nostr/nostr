use std::num::NonZeroUsize;
use std::sync::Arc;

use nostr::NostrSigner;
use nostr_database::NostrDatabase;

use super::RelayPool;
use crate::monitor::Monitor;
use crate::policy::AdmitPolicy;
use crate::transport::websocket::WebSocketTransport;

pub(crate) struct RelayPoolBuilder {
    pub(crate) websocket_transport: Arc<dyn WebSocketTransport>,
    pub(crate) admit_policy: Option<Arc<dyn AdmitPolicy>>,
    pub(crate) monitor: Option<Monitor>,
    pub(crate) database: Arc<dyn NostrDatabase>,
    pub(crate) signer: Option<Arc<dyn NostrSigner>>,
    pub(crate) max_relays: Option<NonZeroUsize>,
    pub(crate) nip42_auto_authentication: bool,
    pub(crate) notification_channel_size: NonZeroUsize,
}

impl RelayPoolBuilder {
    #[inline]
    pub(crate) fn build(self) -> RelayPool {
        RelayPool::from_builder(self)
    }
}
