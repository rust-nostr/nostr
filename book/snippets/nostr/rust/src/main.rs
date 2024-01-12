#![allow(unused_variables)]

use nostr::Result;

pub mod event;
pub mod keys;

fn main() -> Result<()> {
    keys::keys()?;
    Ok(())
}
