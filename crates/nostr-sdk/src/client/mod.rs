// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Client

#[cfg(not(target_arch = "wasm32"))]
pub mod native;
pub mod options;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(not(target_arch = "wasm32"))]
pub use self::native::*;
#[cfg(target_arch = "wasm32")]
pub use self::wasm::*;
