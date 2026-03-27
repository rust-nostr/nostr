// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::hint::black_box;
use std::sync::LazyLock;

use criterion::Criterion;
use nostr_sdk::relay::inner::create_relay;
use tokio::runtime::Runtime;

static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().unwrap());

fn handle_relay_msg_event(c: &mut Criterion) {
    let relay = create_relay();

    let msg = r#"["EVENT", "random_string", {"id":"70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5","pubkey":"379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe","created_at":1612809991,"kind":1,"tags":[],"content":"test","sig":"273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502"}]"#;

    c.bench_function("handle_relay_msg_event", |bh| {
        bh.iter(|| {
            RUNTIME.block_on(async {
                black_box(relay.handle_raw_relay_message(msg).await).unwrap();
            });
        })
    });
}

fn handle_relay_msg_invalid_event(c: &mut Criterion) {
    let relay = create_relay();

    let msg = r#"["EVENT", "random_string", {"id":"70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5","pubkey":"379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe","created_at":1612809991,"kind":1,"tags":[],"content":"test","sig":"fa163f5cfb75d77d9b6269011872ee22b34fb48d23251e9879bb1e4ccbdd8aaaf4b6dc5f5084a65ef42c52fbcde8f3178bac3ba207de827ec513a6aa39fa684c"}]"#;

    c.bench_function("handle_relay_msg_invalid_event", |bh| {
        bh.iter(|| {
            RUNTIME.block_on(async {
                _ = black_box(relay.handle_raw_relay_message(msg).await);
            });
        })
    });
}

pub fn benches(c: &mut Criterion) {
    handle_relay_msg_event(c);
    handle_relay_msg_invalid_event(c);
}
