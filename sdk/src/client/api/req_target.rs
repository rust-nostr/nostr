use std::collections::{BTreeMap, HashMap};

use nostr::{Filter, RelayUrlArg};

// Keep this enum private, so if we change it in the future, we'll not cause breaking changes
// (i.e., Vec to HashMap)
pub(super) enum InnerReqTarget<'url> {
    Auto(Vec<Filter>),
    Manual(Vec<(RelayUrlArg<'url>, Vec<Filter>)>),
}

/// Request target
pub struct ReqTarget<'url>(InnerReqTarget<'url>);

impl<'url> ReqTarget<'url> {
    /// Automatic relay selection.
    ///
    /// Uses all relays with read permission.
    /// With gossip enabled, also queries relays discovered from public keys in filters.
    pub fn auto<I>(filters: I) -> Self
    where
        I: IntoIterator<Item = Filter>,
    {
        Self(InnerReqTarget::Auto(filters.into_iter().collect()))
    }

    /// Target a specific relay.
    pub fn single<T, I>(url: T, filters: I) -> Self
    where
        T: Into<RelayUrlArg<'url>>,
        I: IntoIterator<Item = Filter>,
    {
        Self(InnerReqTarget::Manual(vec![(
            url.into(),
            filters.into_iter().collect(),
        )]))
    }

    /// Target specific relays with their own filters.
    pub fn manual<I, U, F>(filters: I) -> Self
    where
        I: IntoIterator<Item = (U, F)>,
        U: Into<RelayUrlArg<'url>>,
        F: Into<Vec<Filter>>,
    {
        Self(InnerReqTarget::Manual(
            filters
                .into_iter()
                .map(|(url, filters)| (url.into(), filters.into()))
                .collect(),
        ))
    }

    #[inline]
    pub(super) fn into_inner(self) -> InnerReqTarget<'url> {
        self.0
    }
}

impl From<Filter> for ReqTarget<'_> {
    fn from(f: Filter) -> Self {
        Self::auto(vec![f])
    }
}

impl From<Vec<Filter>> for ReqTarget<'_> {
    fn from(filters: Vec<Filter>) -> Self {
        Self::auto(filters)
    }
}

impl<const N: usize> From<[Filter; N]> for ReqTarget<'_> {
    fn from(filters: [Filter; N]) -> Self {
        Self::auto(filters)
    }
}

impl<'url, T> From<Vec<(T, Vec<Filter>)>> for ReqTarget<'url>
where
    T: Into<RelayUrlArg<'url>>,
{
    fn from(targets: Vec<(T, Vec<Filter>)>) -> Self {
        Self::manual(targets)
    }
}

impl<'url, T> From<HashMap<T, Vec<Filter>>> for ReqTarget<'url>
where
    T: Into<RelayUrlArg<'url>>,
{
    fn from(map: HashMap<T, Vec<Filter>>) -> Self {
        Self::manual(map)
    }
}

impl<'url, T> From<HashMap<T, Filter>> for ReqTarget<'url>
where
    T: Into<RelayUrlArg<'url>>,
{
    fn from(map: HashMap<T, Filter>) -> Self {
        Self::manual(map)
    }
}

impl<'url, T> From<BTreeMap<T, Vec<Filter>>> for ReqTarget<'url>
where
    T: Into<RelayUrlArg<'url>>,
{
    fn from(map: BTreeMap<T, Vec<Filter>>) -> Self {
        Self::manual(map)
    }
}

impl<'url, T> From<BTreeMap<T, Filter>> for ReqTarget<'url>
where
    T: Into<RelayUrlArg<'url>>,
{
    fn from(map: BTreeMap<T, Filter>) -> Self {
        Self::manual(map)
    }
}
