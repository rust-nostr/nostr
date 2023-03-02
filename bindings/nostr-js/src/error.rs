use wasm_bindgen::JsValue;

pub type Result<T> = std::result::Result<T, JsValue>;

/// Helper to replace the `E` to `Error` to `napi::Error` conversion.
#[inline]
pub fn into_err<E>(error: E) -> JsValue
where
    E: std::error::Error,
{
    JsValue::from_str(&error.to_string())
}
