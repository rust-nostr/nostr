//! Blocking trait

use std::future::IntoFuture;

/// Trait for blocking execution.
pub trait Blocking: IntoFuture + Sized {
    /// Execute the code synchronously.
    ///
    /// This method will block the current thread until the operation is complete.
    ///
    /// # Important
    ///
    /// Avoid calling this from within an `async` context,
    /// as it may lead to "resource starvation" or a deadlock.
    /// Use `.await` instead whenever possible.
    ///
    /// This implementation is runtime-agnostic and uses a local executor to
    /// drive the future to completion.
    #[inline]
    fn blocking(self) -> Self::Output {
        futures::executor::block_on(self.into_future())
    }
}
