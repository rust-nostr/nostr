// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use crate::error::Result;
use crate::protocol::signer::{NostrSigner, NostrSignerFFI2Rust};
use crate::protocol::{Event, PublicKey, Tag};

/// Private Direct message
///
/// <https://github.com/nostr-protocol/nips/blob/master/17.md>
#[uniffi::export(async_runtime = "tokio", default(rumor_extra_tags = []))]
pub async fn make_private_msg(
    signer: Arc<dyn NostrSigner>,
    receiver: &PublicKey,
    message: &str,
    rumor_extra_tags: Vec<Arc<Tag>>,
) -> Result<Event> {
    let signer = NostrSignerFFI2Rust::new(signer);
    Ok(nostr::EventBuilder::private_msg(
        &signer,
        **receiver,
        message,
        rumor_extra_tags
            .into_iter()
            .map(|t| t.as_ref().deref().clone()),
    )
    .await?
    .into())
}
