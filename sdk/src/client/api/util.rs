use std::collections::HashMap;

use nostr::types::url;
use nostr::{Filter, RelayUrl, RelayUrlArg};

use super::filters_arg::FiltersArg;
use crate::client::{Client, Error};
use crate::pool::RelayPool;

// Build the targets for a REQ
pub(super) async fn build_targets(
    client: &Client,
    target: FiltersArg<'_>,
) -> Result<HashMap<RelayUrl, Vec<Filter>>, Error> {
    // Build targets
    match &client.gossip {
        Some(gossip) => match target {
            // Gossip is configured and we need to break down filters before subscribing
            FiltersArg::Broadcast(filters) => client.break_down_filters(gossip, filters).await,
            // The request is already targeted, skip gossip
            FiltersArg::Targeted(target) => Ok(convert_filters_arg_vec_to_map(target)?),
        },
        // No gossip configured: directly use the target
        None => Ok(convert_filters_arg_to_targets(&client.pool, target).await?),
    }
}

async fn convert_filters_arg_to_targets(
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

fn convert_filters_arg_vec_to_map(
    targeted: Vec<(RelayUrlArg<'_>, Vec<Filter>)>,
) -> Result<HashMap<RelayUrl, Vec<Filter>>, url::Error> {
    let mut map: HashMap<RelayUrl, Vec<Filter>> = HashMap::with_capacity(targeted.len());
    for (url_arg, filters) in targeted {
        let url: RelayUrl = url_arg.try_into_relay_url()?.into_owned();
        map.insert(url, filters);
    }
    Ok(map)
}
