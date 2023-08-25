// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use nostr::nips::nip19::FromBech32;
use nostr::secp256k1::SecretKey;
use nostr::{Event, EventBuilder, EventId, Keys, Result};

const MY_BECH32_SK: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";

fn main() -> Result<()> {
    let secret_key = SecretKey::from_bech32(MY_BECH32_SK)?;
    let my_keys = Keys::new(secret_key);

    let event_id =
        EventId::from_hex("7469af3be8c8e06e1b50ef1caceba30392ddc0b6614507398b7d7daa4c218e96")?;

    let event: Event = EventBuilder::delete(
        vec![event_id],
        Some("these posts were published by accident"),
    )
    .to_event(&my_keys)?;

    println!("{}", event.as_json());

    Ok(())
}
