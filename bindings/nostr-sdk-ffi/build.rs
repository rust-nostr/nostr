// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    if let Ok(output) = Command::new("git").args(["rev-parse", "HEAD"]).output() {
        if let Ok(git_hash) = String::from_utf8(output.stdout) {
            println!("cargo:rerun-if-changed={git_hash}");
            println!("cargo:rustc-env=GIT_HASH={git_hash}");
        }
    }

    setup_x86_64_android_workaround();
}

/// Adds a temporary workaround for an issue with the Rust compiler and Android
/// in x86_64 devices: https://github.com/rust-lang/rust/issues/109717.
/// The workaround comes from: https://github.com/mozilla/application-services/pull/5442
/// This below code is inspired by: https://github.com/matrix-org/matrix-rust-sdk/blob/f18e0b18a1ea757921bcae07c29600a9839fdc97/bindings/matrix-sdk-ffi/build.rs
fn setup_x86_64_android_workaround() {
    // Get target OS and arch
    let target_os: String = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set");
    let target_arch: String =
        env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH not set");

    // Check if match x86_64-android
    if target_arch == "x86_64" && target_os == "android" {
        // Configure rust to statically link against the `libclang_rt.builtins` supplied
        // with clang.

        // cargo-ndk sets CC_x86_64-linux-android to the path to `clang`, within the
        // Android NDK.
        let clang_path = PathBuf::from(
            env::var("CC_x86_64-linux-android").expect("CC_x86_64-linux-android not set"),
        );

        // clang_path should now look something like
        // `.../sdk/ndk/28.0.12674087/toolchains/llvm/prebuilt/linux-x86_64/bin/clang`.
        // We strip `/bin/clang` from the end to get the toolchain path.
        let toolchain_path = clang_path
            .ancestors()
            .nth(2)
            .expect("could not find NDK toolchain path")
            .to_str()
            .expect("NDK toolchain path is not valid UTF-8");

        let clang_version = get_clang_major_version(&clang_path);

        println!("cargo:rustc-link-search={toolchain_path}/lib/clang/{clang_version}/lib/linux/");
        println!("cargo:rustc-link-lib=static=clang_rt.builtins-x86_64-android");
    }
}

/// Run the clang binary at `clang_path`, and return its major version number
fn get_clang_major_version(clang_path: &Path) -> String {
    let clang_output = Command::new(clang_path)
        .arg("-dumpversion")
        .output()
        .expect("failed to start clang");

    if !clang_output.status.success() {
        panic!(
            "failed to run clang: {}",
            String::from_utf8_lossy(&clang_output.stderr)
        );
    }

    let clang_version = String::from_utf8(clang_output.stdout).expect("clang output is not utf8");
    clang_version
        .split('.')
        .next()
        .expect("could not parse clang output")
        .to_owned()
}
