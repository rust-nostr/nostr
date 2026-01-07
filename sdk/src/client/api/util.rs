use std::collections::HashMap;

use nostr::types::url;
use nostr::{Filter, RelayUrl, RelayUrlArg};

use super::filters_arg::FiltersArg;
use crate::pool::RelayPool;

pub(super) async fn convert_filters_arg_to_targets(
    pool: &RelayPool,
    target: FiltersArg<'_>,
) -> Result<HashMap<RelayUrl, Vec<Filter>>, url::Error> {
    match target {
        FiltersArg::Broadcast(filters) => {
            let urls: Vec<RelayUrl> = pool.read_relay_urls().await;
            Ok(urls.into_iter().map(|u| (u, filters.clone())).collect())
        }
        FiltersArg::Targeted(targets) => convert_filters_arg_vec_to_map(targets),
    }
}

pub(super) fn convert_filters_arg_vec_to_map(
    targeted: Vec<(RelayUrlArg<'_>, Vec<Filter>)>,
) -> Result<HashMap<RelayUrl, Vec<Filter>>, url::Error> {
    let mut map: HashMap<RelayUrl, Vec<Filter>> = HashMap::with_capacity(targeted.len());
    for (url_arg, filters) in targeted {
        let url: RelayUrl = url_arg.try_into_relay_url()?.into_owned();
        map.insert(url, filters);
    }
    Ok(map)
}
