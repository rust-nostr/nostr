#![allow(unused_variables)]

use nostr::Result;

pub mod event;
pub mod keys;
mod nip_utilities;

fn main() -> Result<()> {
    keys::keys()?;
    nip_utilities::nip_utilities()?;
    Ok(())
}
