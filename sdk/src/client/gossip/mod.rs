use std::sync::Arc;

use nostr::{Alphabet, Filter, Kind, SingleLetterTag};
use nostr_gossip::NostrGossip;

mod refresher;
mod resolver;
mod semaphore;
mod updater;

use self::refresher::*;
pub(super) use self::resolver::*;
pub(super) use self::semaphore::*;

const P_TAG: SingleLetterTag = SingleLetterTag::lowercase(Alphabet::P);

pub(super) enum GossipFilterPattern {
    Nip65,
    Nip65AndNip17,
}

impl GossipFilterPattern {
    #[inline]
    fn has_nip17(&self) -> bool {
        matches!(self, Self::Nip65AndNip17)
    }
}

/// Use both NIP-65 and NIP-17 if:
/// - the `kinds` field contains the [`Kind::GiftWrap`];
/// - if it's set a `#p` tag and no kind is specified
pub(super) fn find_filter_pattern(filter: &Filter) -> GossipFilterPattern {
    let (are_kinds_empty, has_gift_wrap_kind): (bool, bool) = match &filter.kinds {
        Some(kinds) if kinds.is_empty() => (true, false),
        Some(kinds) => (false, kinds.contains(&Kind::GiftWrap)),
        None => (true, false),
    };
    let has_p_tags: bool = filter.generic_tags.contains_key(&P_TAG);

    // TODO: use both also if there are only IDs?

    if has_gift_wrap_kind || (has_p_tags && are_kinds_empty) {
        return GossipFilterPattern::Nip65AndNip17;
    }

    GossipFilterPattern::Nip65
}

#[derive(Debug)]
pub(super) struct Gossip {
    store: Arc<dyn NostrGossip>,
    resolver: GossipRelayResolver,
    semaphore: GossipSemaphore,
    refresher: GossipBackgroundRefresher,
}

impl Gossip {
    #[inline]
    pub(super) fn new(gossip: Arc<dyn NostrGossip>) -> Self {
        Self {
            store: gossip.clone(),
            resolver: GossipRelayResolver::new(gossip),
            semaphore: GossipSemaphore::new(),
            refresher: GossipBackgroundRefresher::new(),
        }
    }

    #[inline]
    pub(super) fn store(&self) -> &Arc<dyn NostrGossip> {
        &self.store
    }

    #[inline]
    pub(super) fn resolver(&self) -> &GossipRelayResolver {
        &self.resolver
    }

    #[inline]
    pub(super) fn semaphore(&self) -> &GossipSemaphore {
        &self.semaphore
    }

    #[inline]
    fn refresher(&self) -> &GossipBackgroundRefresher {
        &self.refresher
    }

    #[cfg(test)]
    pub(super) fn is_background_refresher_spawned(&self) -> bool {
        self.refresher.is_background_refresher_spawned()
    }
}
