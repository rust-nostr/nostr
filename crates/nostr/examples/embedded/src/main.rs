// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

#![feature(alloc_error_handler)]
#![no_std]
#![no_main]

extern crate alloc;
extern crate nostr;

use alloc::sync::Arc;
use core::panic::PanicInfo;

use alloc_cortex_m::CortexMHeap;
use cortex_m_rt::entry;
use cortex_m_semihosting::{debug, hprintln};
use nostr::secp256k1::Secp256k1;
use nostr::prelude::*;

// this is the allocator the application will use
#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

const HEAP_SIZE: usize = 1024 * 256; // 256 KB

#[derive(Debug)]
struct FakeTime;

impl TimeProvider for FakeTime {
    fn now(&self) -> Timestamp {
        Timestamp::from_secs(16777216)
    }
}

#[derive(Debug)]
struct FakeRng;

impl SecureRandom for FakeRng {
    fn fill(&self, dest: &mut [u8]) {
        for i in dest {
            *i = 57;
        }
    }
}

#[entry]
fn main() -> ! {
    hprintln!("heap size {}\n", HEAP_SIZE).unwrap();

    unsafe { ALLOCATOR.init(cortex_m_rt::heap_start() as usize, HEAP_SIZE) }

    // Install provider
    NostrProvider {
        secp: Secp256k1::new(),
        time: Arc::new(FakeTime),
        rng: Arc::new(FakeRng),
    }.install();

    // Parse secret key
    let keys = Keys::parse("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99").unwrap();
    hprintln!("Restored keys from bech32:").unwrap();
    print_keys(&keys);

    // Restore from mnemonic
    let mnemonic: &str = "equal dragon fabric refuse stable cherry smoke allow alley easy never medal attend together lumber movie what sad siege weather matrix buffalo state shoot";
    let keys = Keys::from_mnemonic(mnemonic, None).unwrap();
    hprintln!("\nRestore keys from mnemonic:").unwrap();
    print_keys(&keys);

    // Generate new random keys
    let keys = Keys::generate();
    hprintln!("\nRandom keys (using FakeRng):").unwrap();
    print_keys(&keys);

    // exit QEMU
    // NOTE do not run this on hardware; it can corrupt OpenOCD state
    debug::exit(debug::EXIT_SUCCESS);

    loop {}
}

fn print_keys(keys: &Keys) {
    hprintln!(
        "- Secret Key (hex): {}",
        keys.secret_key().to_secret_hex()
    )
    .unwrap();
    hprintln!("- Public Key (hex): {}", keys.public_key()).unwrap();
    hprintln!(
        "- Secret Key (bech32): {}",
        keys.secret_key().to_bech32().unwrap()
    )
    .unwrap();
    hprintln!(
        "- Public Key (bech32): {}",
        keys.public_key().to_bech32().unwrap()
    )
    .unwrap();
}

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    hprintln!("panic {:?}", info.message()).unwrap();
    debug::exit(debug::EXIT_FAILURE);
    loop {}
}
