// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashMap;
use std::ops::Deref;

use nostr::serde_json::{Number, Value};
use nostr::util;
use uniffi::Enum;

use crate::error::Result;
use crate::{NostrError, PublicKey, SecretKey};

/// Generate shared key
///
/// **Important: use of a strong cryptographic hash function may be critical to security! Do NOT use
/// unless you understand cryptographical implications.**
#[uniffi::export]
pub fn generate_shared_key(secret_key: &SecretKey, public_key: &PublicKey) -> Vec<u8> {
    util::generate_shared_key(secret_key.deref(), public_key.deref()).to_vec()
}

#[derive(Enum, o2o::o2o)]
#[try_map_owned(Value, NostrError)]
pub enum JsonValue {
    #[o2o(repeat)]
    #[type_hint(as ())]
    
    Bool { bool: bool },

    #[ghost({Value::Number(Number::from(number))})] 
    NumberPosInt { #[into(Number::from(~))] number: u64 },

    #[ghost({Value::Number(Number::from(number))})]
    NumberNegInt { #[into(Number::from(~))] number: i64 },

    #[into(Number)]
    #[from(Number,
        match f0.as_u64() {
            Some(f0) => Self::NumberPosInt { number: f0 },
            None => match f0.as_i64() {
                Some(f0) => Self::NumberNegInt { number: f0 },
                None => match f0.as_f64() {
                    Some(f0) => Self::NumberFloat { number: f0 },
                    None => Err(NostrError::Generic(String::from(
                        "Impossible to convert number",
                    )))?
                },
            },
        }
    )]
    NumberFloat {
        #[into(
            Number::from_f64(~).ok_or(NostrError::Generic(String::from(
                "Impossible to convert finite f64 to number",
            )))?
        )]
        number: f64 
    },

    #[map(String)] Str { s: String },

    Array {
        #[map(~.into_iter().filter_map(|v| v.try_into().ok()).collect())] 
        array: Vec<JsonValue> 
    },
    Object { 
        #[map(~.into_iter().filter_map(|(k, v)| Some((k, v.try_into().ok()?))).collect())] 
        map: HashMap<String, JsonValue> 
    },

    #[o2o(stop_repeat)] 

    Null,
}