// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashMap;
use std::ops::Deref;

use nostr::serde_json::{Number, Value};
use nostr::util;
use uniffi::Enum;

use super::key::{PublicKey, SecretKey};
use crate::error::{NostrSdkError, Result};

/// Generate shared key
///
/// **Important: use of a strong cryptographic hash function may be critical to security! Do NOT use
/// unless you understand cryptographical implications.**
#[uniffi::export]
pub fn generate_shared_key(secret_key: &SecretKey, public_key: &PublicKey) -> Result<Vec<u8>> {
    Ok(util::generate_shared_key(secret_key.deref(), public_key.deref())?.to_vec())
}

#[derive(Enum)]
pub enum JsonValue {
    Bool { bool: bool },
    NumberPosInt { number: u64 },
    NumberNegInt { number: i64 },
    NumberFloat { number: f64 },
    Str { s: String },
    Array { array: Vec<JsonValue> },
    Object { map: HashMap<String, JsonValue> },
    Null,
}

impl TryFrom<JsonValue> for Value {
    type Error = NostrSdkError;

    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        Ok(match value {
            JsonValue::Bool { bool } => Self::Bool(bool),
            JsonValue::NumberPosInt { number } => Self::Number(Number::from(number)),
            JsonValue::NumberNegInt { number } => Self::Number(Number::from(number)),
            JsonValue::NumberFloat { number } => {
                let float = Number::from_f64(number).ok_or(NostrSdkError::Generic(
                    String::from("Impossible to convert finite f64 to number"),
                ))?;
                Self::Number(float)
            }
            JsonValue::Str { s } => Self::String(s),
            JsonValue::Array { array } => Self::Array(
                array
                    .into_iter()
                    .filter_map(|v| v.try_into().ok())
                    .collect(),
            ),
            JsonValue::Object { map } => Self::Object(
                map.into_iter()
                    .filter_map(|(k, v)| Some((k, v.try_into().ok()?)))
                    .collect(),
            ),
            JsonValue::Null => Self::Null,
        })
    }
}

impl TryFrom<Value> for JsonValue {
    type Error = NostrSdkError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(match value {
            Value::Bool(bool) => Self::Bool { bool },
            Value::Number(number) => match number.as_u64() {
                Some(number) => Self::NumberPosInt { number },
                None => match number.as_i64() {
                    Some(number) => Self::NumberNegInt { number },
                    None => match number.as_f64() {
                        Some(number) => Self::NumberFloat { number },
                        None => {
                            return Err(NostrSdkError::Generic(String::from(
                                "Impossible to convert number",
                            )))
                        }
                    },
                },
            },
            Value::String(s) => Self::Str { s },
            Value::Array(array) => Self::Array {
                array: array
                    .into_iter()
                    .filter_map(|v| v.try_into().ok())
                    .collect(),
            },
            Value::Object(map) => Self::Object {
                map: map
                    .into_iter()
                    .filter_map(|(k, v)| Some((k, v.try_into().ok()?)))
                    .collect(),
            },
            Value::Null => Self::Null,
        })
    }
}
