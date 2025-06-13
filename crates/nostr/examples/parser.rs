// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::prelude::*;

fn main() -> Result<()> {
    let parser = NostrParser::new();

    let text: &str = "I have never been very active in discussions but working on rust-nostr (at the time called nostr-rs-sdk) since September 2022 🦀 \n\nIf I remember correctly there were also nostr:nprofile1qqsqfyvdlsmvj0nakmxq6c8n0c2j9uwrddjd8a95ynzn9479jhlth3gpvemhxue69uhkv6tvw3jhytnwdaehgu3wwa5kuef0dec82c33w94xwcmdd3cxketedsux6ertwecrgues0pk8xdrew33h27pkd4unvvpkw3nkv7pe0p68gat58ycrw6ps0fenwdnvva48w0mzwfhkzerrv9ehg0t5wf6k2qgnwaehxw309ac82unsd3jhqct89ejhxtcpz4mhxue69uhhyetvv9ujuerpd46hxtnfduhsh8njvk and nostr:nprofile1qqswuyd9ml6qcxd92h6pleptfrcqucvvjy39vg4wx7mv9wm8kakyujgpypmhxue69uhkx6r0wf6hxtndd94k2erfd3nk2u3wvdhk6w35xs6z7qgwwaehxw309ahx7uewd3hkctcpypmhxue69uhkummnw3ezuetfde6kuer6wasku7nfvuh8xurpvdjj7a0nq40";

    for token in parser.parse(text) {
        println!("{token:?}");
    }

    for token in parser.parse("Check this: https://example.com/foo/bar.html") {
        println!("{token:?}");
    }

    let opts = NostrParserOptions::disable_all().urls(true);

    for token in parser.parse("Check this: https://example.com").opts(opts) {
        println!("{token:?}");
    }

    Ok(())
}
