// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

extern crate nostr;

use std::error::Error;

use nostr::key::{FromSeedPhrase, Keys, ToBech32};

// WORK ONLY WITH 24-WORD MNEMONICS
const SEED_PHRASE: &str = "equal dragon fabric refuse stable cherry smoke allow alley easy never medal attend together lumber movie what sad siege weather matrix buffalo state shoot";

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let keys = Keys::from_seed(SEED_PHRASE)?;

    println!("{}", keys.secret_key()?.to_bech32()?);

    Ok(())
}
