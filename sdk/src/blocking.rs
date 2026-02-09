/// Implements an inherent `blocking(self)` method for a type that already implements `IntoFuture`.
///
/// - On `wasm32`, the method is not generated (blocking the thread isn't supported).
/// - The return type is inferred as `<Self as IntoFuture>::Output`.
macro_rules! impl_blocking {
    // ---- Generic form: accepts explicit generics (lifetimes, types, consts) ----
    (for<$($gen:tt),+> $ty:ty where $($where:tt)+) => {
        impl<$($gen),+> $ty
        where
            $($where)+
        {
            /// Execute the operation synchronously.
            ///
            /// # Important
            ///
            /// Avoid calling this from within an `async` context, as it may lead to
            /// "resource starvation" or a deadlock. Prefer `.await` whenever possible.
            ///
            /// This is runtime-agnostic and uses a local executor to drive the future
            /// to completion.
            #[inline]
            #[cfg(not(target_family = "wasm"))]
            pub fn blocking(self) -> <Self as ::std::future::IntoFuture>::Output
            where
                Self: ::std::future::IntoFuture,
            {
                ::futures::executor::block_on(::std::future::IntoFuture::into_future(self))
            }
        }
    };

    (for<$($gen:tt),+> $ty:ty) => {
        impl<$($gen),+> $ty {
            /// Execute the operation synchronously.
            ///
            /// # Important
            ///
            /// Avoid calling this from within an `async` context, as it may lead to
            /// "resource starvation" or a deadlock. Prefer `.await` whenever possible.
            ///
            /// This is runtime-agnostic and uses a local executor to drive the future
            /// to completion.
            #[inline]
            #[cfg(not(target_family = "wasm"))]
            pub fn blocking(self) -> <Self as ::std::future::IntoFuture>::Output
            where
                Self: ::std::future::IntoFuture,
            {
                ::futures::executor::block_on(::std::future::IntoFuture::into_future(self))
            }
        }
    };

    // ---- Non-generic form: works for concrete types or elided lifetimes ----
    ($ty:ty where $($where:tt)+) => {
        impl $ty
        where
            $($where)+
        {
            /// Execute the operation synchronously.
            ///
            /// # Important
            ///
            /// Avoid calling this from within an `async` context, as it may lead to
            /// "resource starvation" or a deadlock. Prefer `.await` whenever possible.
            ///
            /// This is runtime-agnostic and uses a local executor to drive the future
            /// to completion.
            #[inline]
            #[cfg(not(target_family = "wasm"))]
            pub fn blocking(self) -> <Self as ::std::future::IntoFuture>::Output
            where
                Self: ::std::future::IntoFuture,
            {
                ::futures::executor::block_on(::std::future::IntoFuture::into_future(self))
            }
        }
    };

    ($ty:ty) => {
        impl $ty {
            /// Execute the operation synchronously.
            ///
            /// # Important
            ///
            /// Avoid calling this from within an `async` context, as it may lead to
            /// "resource starvation" or a deadlock. Prefer `.await` whenever possible.
            ///
            /// This is runtime-agnostic and uses a local executor to drive the future
            /// to completion.
            #[inline]
            #[cfg(not(target_family = "wasm"))]
            pub fn blocking(self) -> <Self as ::std::future::IntoFuture>::Output
            where
                Self: ::std::future::IntoFuture,
            {
                ::futures::executor::block_on(::std::future::IntoFuture::into_future(self))
            }
        }
    };
}
