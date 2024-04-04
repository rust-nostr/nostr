#![allow(unused_variables)]

use nostr::Result;

mod event;
mod keys;
mod messages;
mod nip01;
mod nip44;
mod nip59;
mod vanity;

fn main() -> Result<()> {
    keys::keys()?;

    event::builder::event()?;
    event::json::event()?;

    messages::relay::relay_message()?;

    nip01::nip01()?;
    nip44::run()?;
    nip59::run()?;
    
    vanity::run()?;

    Ok(())
}
