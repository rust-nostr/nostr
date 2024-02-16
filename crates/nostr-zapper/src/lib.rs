// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Zapper

#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![allow(unknown_lints)]

use core::fmt;
use std::sync::Arc;

pub extern crate nostr;

pub use async_trait::async_trait;
use nostr::prelude::*;

pub mod error;
pub mod prelude;

pub use self::error::ZapperError;

/// Backend
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZapperBackend {
    /// WebLN
    WebLN,
    /// Nostr Wallet Connect
    NWC,
    /// Custom
    Custom(String),
}

/// A type-erased [`NostrZapper`].
pub type DynNostrZapper = dyn NostrZapper<Err = ZapperError>;

/// A type that can be type-erased into `Arc<dyn NostrZapper>`.
pub trait IntoNostrZapper {
    #[doc(hidden)]
    fn into_nostr_zapper(self) -> Arc<DynNostrZapper>;
}

impl IntoNostrZapper for Arc<DynNostrZapper> {
    fn into_nostr_zapper(self) -> Arc<DynNostrZapper> {
        self
    }
}

impl<T> IntoNostrZapper for T
where
    T: NostrZapper + Sized + 'static,
{
    fn into_nostr_zapper(self) -> Arc<DynNostrZapper> {
        Arc::new(EraseNostrZapperError(self))
    }
}

// Turns a given `Arc<T>` into `Arc<DynNostrZapper>` by attaching the
// NostrZapper impl vtable of `EraseNostrZapperError<T>`.
impl<T> IntoNostrZapper for Arc<T>
where
    T: NostrZapper + 'static,
{
    fn into_nostr_zapper(self) -> Arc<DynNostrZapper> {
        let ptr: *const T = Arc::into_raw(self);
        let ptr_erased = ptr as *const EraseNostrZapperError<T>;
        // SAFETY: EraseNostrZapperError is repr(transparent) so T and
        //         EraseNostrZapperError<T> have the same layout and ABI
        unsafe { Arc::from_raw(ptr_erased) }
    }
}

/// Nostr Database
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait NostrZapper: AsyncTraitDeps {
    /// Error
    type Err: From<ZapperError> + Into<ZapperError>;

    /// Name of the backend zapper used (ex. WebLN, NWC, ...)
    fn backend(&self) -> ZapperBackend;

    /// Pay invoice
    async fn pay(&self, invoice: String) -> Result<(), Self::Err>;
}

#[repr(transparent)]
struct EraseNostrZapperError<T>(T);

impl<T: fmt::Debug> fmt::Debug for EraseNostrZapperError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<T: NostrZapper> NostrZapper for EraseNostrZapperError<T> {
    type Err = ZapperError;

    fn backend(&self) -> ZapperBackend {
        self.0.backend()
    }

    async fn pay(&self, invoice: String) -> Result<(), Self::Err> {
        self.0.pay(invoice).await.map_err(Into::into)
    }
}

/// Alias for `Send` on non-wasm, empty trait (implemented by everything) on
/// wasm.
#[cfg(not(target_arch = "wasm32"))]
pub trait SendOutsideWasm: Send {}
#[cfg(not(target_arch = "wasm32"))]
impl<T: Send> SendOutsideWasm for T {}

/// Alias for `Send` on non-wasm, empty trait (implemented by everything) on
/// wasm.
#[cfg(target_arch = "wasm32")]
pub trait SendOutsideWasm {}
#[cfg(target_arch = "wasm32")]
impl<T> SendOutsideWasm for T {}

/// Alias for `Sync` on non-wasm, empty trait (implemented by everything) on
/// wasm.
#[cfg(not(target_arch = "wasm32"))]
pub trait SyncOutsideWasm: Sync {}
#[cfg(not(target_arch = "wasm32"))]
impl<T: Sync> SyncOutsideWasm for T {}

/// Alias for `Sync` on non-wasm, empty trait (implemented by everything) on
/// wasm.
#[cfg(target_arch = "wasm32")]
pub trait SyncOutsideWasm {}
#[cfg(target_arch = "wasm32")]
impl<T> SyncOutsideWasm for T {}

/// Super trait that is used for our store traits, this trait will differ if
/// it's used on WASM. WASM targets will not require `Send` and `Sync` to have
/// implemented, while other targets will.
pub trait AsyncTraitDeps: std::fmt::Debug + SendOutsideWasm + SyncOutsideWasm {}
impl<T: std::fmt::Debug + SendOutsideWasm + SyncOutsideWasm> AsyncTraitDeps for T {}
