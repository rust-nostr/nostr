#![allow(unused_variables)]

use nostr::Result;

mod event;
mod keys;
mod messages;
mod nip_01;

fn main() -> Result<()> {
    keys::keys()?;

    event::builder::event()?;
    event::json::event()?;

    messages::relay::relay_message()?;

    nip_01::nip_01()?;

    Ok(())
}
