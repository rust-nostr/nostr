// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use nostr_sdk_base::Keys;

fn main() {
    //  Random keys
    let keys = Keys::generate_from_os_random();

    println!("Public key: {}", keys.public_key);
    println!("Secret key: {}", keys.secret_key_as_str().unwrap());

    // Bech32 keys
    let keys =
        Keys::new_from_bech32("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")
            .unwrap();
    println!("Public key: {}", keys.public_key);

    let keys = Keys::new_pub_only_from_bech32(
        "npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy",
    )
    .unwrap();
    println!("Public key: {}", keys.public_key);
}
