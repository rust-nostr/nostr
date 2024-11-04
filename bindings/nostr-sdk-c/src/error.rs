// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

// pub enum CResult {
//     code: i32,
//     message: *const c_char,
//     value: *const c_char, // Change type as needed for different results
// }

#[repr(C)]
pub struct CError {
    message: String,
}

pub type Result<T> = core::result::Result<T, CError>;

#[inline(always)]
pub fn into_err<E>(error: E) -> CError
where
    E: std::error::Error,
{
    CError {
        message: error.to_string(),
    }
}
