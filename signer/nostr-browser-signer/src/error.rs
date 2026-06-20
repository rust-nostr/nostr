//! Error

use wasm_bindgen::JsValue;

opaquerr::define_kind! {
    /// NIP-07 error kind.
    pub ErrorKind {
        /// Nostr protocol error.
        Protocol => "nostr protocol error",
        /// Browser/Extension-related error.
        Extension => "browser extension error",
        /// Anything not covered by the stable categories above.
        Other => "other error",
    }
}

opaquerr::define_error! {
    /// NIP-07 error.
    pub Error(ErrorKind)

    from {
        nostr::error::Error => ErrorKind::Protocol,
    }
}

impl From<JsValue> for Error {
    fn from(e: JsValue) -> Self {
        Self::new(ErrorKind::Extension, format!("{e:?}"))
    }
}

impl Error {
    pub(super) fn no_global_window_object() -> Self {
        Self::with_static_message(ErrorKind::Extension, "no global `window` object")
    }

    pub(super) fn namespace_not_found(namespace: &str) -> Self {
        Self::new(
            ErrorKind::Extension,
            format!("namespace `{namespace}` not found"),
        )
    }

    pub(super) fn object_key_not_found(key: &str) -> Self {
        Self::new(
            ErrorKind::Extension,
            format!("key `{key}` not found in object"),
        )
    }

    pub(super) fn type_mismatch() -> Self {
        Self::new(ErrorKind::Extension, "type mismatch")
    }
}
