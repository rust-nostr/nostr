#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
pub(crate) type BoxedFuture<'a, T> = futures::future::BoxFuture<'a, T>;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub(crate) type BoxedFuture<'a, T> = futures::future::LocalBoxFuture<'a, T>;
