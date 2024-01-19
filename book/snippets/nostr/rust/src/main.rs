#![allow(unused_variables)]

use nostr::Result;

pub mod event;
pub mod keys;
mod nip_01;

fn main() -> Result<()> {
    keys::keys()?;
    nip_01::nip_01()?;
    Ok(())
}
