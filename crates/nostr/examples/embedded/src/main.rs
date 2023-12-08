// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![no_std]
#![no_main]

extern crate alloc;
extern crate nostr;

use core::panic::PanicInfo;

use alloc_cortex_m::CortexMHeap;
use cortex_m_rt::entry;
use cortex_m_semihosting::{debug, hprintln};
use nostr::secp256k1::rand::{self, RngCore};
use nostr::secp256k1::{Secp256k1, SecretKey};
use nostr::{FromBech32, Keys, ToBech32};
use nostr::nips::nip06::FromMnemonic;

// this is the allocator the application will use
#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

const HEAP_SIZE: usize = 1024 * 256; // 256 KB

struct FakeRng;

impl RngCore for FakeRng {
    fn next_u32(&mut self) -> u32 {
        57
    }

    fn next_u64(&mut self) -> u64 {
        57
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        for i in dest {
            *i = 57;
        }
        Ok(())
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.try_fill_bytes(dest).unwrap();
    }
}

#[entry]
fn main() -> ! {
    hprintln!("heap size {}\n", HEAP_SIZE).unwrap();

    unsafe { ALLOCATOR.init(cortex_m_rt::heap_start() as usize, HEAP_SIZE) }

    let secp = Secp256k1::new();

    // Restore from bech32 secret key
    let secret_key =
        SecretKey::from_bech32("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")
            .unwrap();
    let keys = Keys::new_with_ctx(&secp, secret_key);
    hprintln!("Restored keys from bech32:").unwrap();
    print_keys(&keys);

    // Restore from menmonic
    let mnemonic: &str = "equal dragon fabric refuse stable cherry smoke allow alley easy never medal attend together lumber movie what sad siege weather matrix buffalo state shoot";
    let keys = Keys::from_mnemonic_with_ctx(&secp, mnemonic, None).unwrap();
    hprintln!("\nRestore keys from mnemonic:").unwrap();
    print_keys(&keys);

    // Generate new random keys
    let keys = Keys::generate_with_ctx(&secp, &mut FakeRng);
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
        keys.secret_key().unwrap().display_secret()
    )
    .unwrap();
    hprintln!("- Public Key (hex): {}", keys.public_key()).unwrap();
    hprintln!(
        "- Secret Key (bech32): {}",
        keys.secret_key().unwrap().to_bech32().unwrap()
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
