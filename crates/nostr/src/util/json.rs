use serde::de;
use serde_json::Value;

use crate::error::Error;

#[inline]
pub(crate) fn parse_json<'a, T, V>(bytes: &'a V) -> Result<T, Error>
where
    T: de::Deserialize<'a>,
    V: AsRef<[u8]> + 'a,
{
    Ok(serde_json::from_slice(bytes.as_ref())?)
}

#[inline]
pub(crate) fn parse_json_from_value<T>(value: Value) -> Result<T, Error>
where
    T: de::DeserializeOwned,
{
    Ok(serde_json::from_value(value)?)
}

macro_rules! impl_json_methods {
    ($ty:ty) => {
        impl_json_methods! {
            $ty,
            from_json(json) {
                Ok(serde_json::from_slice(json.as_ref())?)
            }
        }
    };

    ($ty:ty, from_json($json:ident) $from_json:block) => {
        impl $ty {
            /// Deserialize from JSON.
            #[inline]
            pub fn from_json<T>($json: T) -> Result<Self, crate::error::Error>
            where
                T: AsRef<[u8]>,
            {
                $from_json
            }

            /// Serialize as JSON string.
            ///
            /// This method could panic. Use `try_as_json` for error propagation.
            #[inline]
            pub fn as_json(&self) -> alloc::string::String {
                self.try_as_json().unwrap()
            }

            /// Serialize as JSON string.
            #[inline]
            pub fn try_as_json(&self) -> Result<alloc::string::String, crate::error::Error> {
                Ok(serde_json::to_string(self)?)
            }

            /// Serialize as pretty JSON string.
            ///
            /// This method could panic. Use `try_as_pretty_json` for error propagation.
            #[inline]
            pub fn as_pretty_json(&self) -> alloc::string::String {
                self.try_as_pretty_json().unwrap()
            }

            /// Serialize as pretty JSON string.
            #[inline]
            pub fn try_as_pretty_json(&self) -> Result<alloc::string::String, crate::error::Error> {
                Ok(serde_json::to_string_pretty(self)?)
            }
        }
    };
}

pub(crate) use impl_json_methods;
