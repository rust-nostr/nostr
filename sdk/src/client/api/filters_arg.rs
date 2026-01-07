use std::collections::{BTreeMap, HashMap};

use nostr::{Filter, RelayUrlArg};

/// Filters argument
///
/// Defines where to apply filters: broadcast to all relays or targeted to specific relays.
pub enum FiltersArg<'url> {
    /// Broadcast filters to all connected relays
    Broadcast(Vec<Filter>),
    /// Target specific relays with their own filters
    Targeted(Vec<(RelayUrlArg<'url>, Vec<Filter>)>),
}

impl<'url> FiltersArg<'url> {
    /// Create a targeted subscription from an iterator.
    ///
    /// This method accepts any iterator of tuples where:
    /// - The first element can be converted into a [`RelayUrlArg`] (e.g., `&str`, `String`, `RelayUrl`)
    /// - The second element can be converted into a `Vec<Filter>` (e.g., `Filter`, `Vec<Filter>`)
    fn targeted_from_iter<I, U, F>(iter: I) -> Self
    where
        I: IntoIterator<Item = (U, F)>,
        U: Into<RelayUrlArg<'url>>,
        F: Into<Vec<Filter>>,
    {
        Self::Targeted(
            iter.into_iter()
                .map(|(url, filters)| (url.into(), filters.into()))
                .collect(),
        )
    }
}

impl From<Filter> for FiltersArg<'_> {
    fn from(f: Filter) -> Self {
        Self::Broadcast(vec![f])
    }
}

impl From<Vec<Filter>> for FiltersArg<'_> {
    fn from(filters: Vec<Filter>) -> Self {
        Self::Broadcast(filters)
    }
}

impl<const N: usize> From<[Filter; N]> for FiltersArg<'_> {
    fn from(filters: [Filter; N]) -> Self {
        Self::Broadcast(filters.into())
    }
}

impl<'url, T> From<Vec<(T, Vec<Filter>)>> for FiltersArg<'url>
where
    T: Into<RelayUrlArg<'url>>,
{
    fn from(targets: Vec<(T, Vec<Filter>)>) -> Self {
        Self::targeted_from_iter(targets)
    }
}

impl<'url, T> From<HashMap<T, Vec<Filter>>> for FiltersArg<'url>
where
    T: Into<RelayUrlArg<'url>>,
{
    fn from(map: HashMap<T, Vec<Filter>>) -> Self {
        Self::targeted_from_iter(map)
    }
}

impl<'url, T> From<HashMap<T, Filter>> for FiltersArg<'url>
where
    T: Into<RelayUrlArg<'url>>,
{
    fn from(map: HashMap<T, Filter>) -> Self {
        Self::targeted_from_iter(map)
    }
}

impl<'url, T> From<BTreeMap<T, Vec<Filter>>> for FiltersArg<'url>
where
    T: Into<RelayUrlArg<'url>>,
{
    fn from(map: BTreeMap<T, Vec<Filter>>) -> Self {
        Self::targeted_from_iter(map)
    }
}

impl<'url, T> From<BTreeMap<T, Filter>> for FiltersArg<'url>
where
    T: Into<RelayUrlArg<'url>>,
{
    fn from(map: BTreeMap<T, Filter>) -> Self {
        Self::targeted_from_iter(map)
    }
}
