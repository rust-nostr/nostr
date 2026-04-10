use alloc::boxed::Box;
use core::fmt;
use core::num::NonZeroU8;

use crate::event::unsigned::{AsyncBuildUnsignedEvent, BuildUnsignedEvent};
use crate::nips::nip13::{AsyncPowAdapter, PowAdapter};
use crate::util::BoxedFuture;
use crate::{PublicKey, UnsignedEvent};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PowBuildError<BE, PE> {
    Builder(BE),
    Pow(PE),
}

impl<BE, PE> core::error::Error for PowBuildError<BE, PE>
where
    BE: core::error::Error + 'static,
    PE: core::error::Error + 'static,
{
}

impl<BE, PE> fmt::Display for PowBuildError<BE, PE>
where
    BE: fmt::Display,
    PE: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Builder(e) => write!(f, "builder error: {e}"),
            Self::Pow(e) => write!(f, "pow error: {e}"),
        }
    }
}

/// Adapter for building events with proof-of-work
pub struct Pow<Builder, Adapter> {
    builder: Builder,
    adapter: Adapter,
    difficulty: NonZeroU8,
}

impl<Builder, Adapter> Pow<Builder, Adapter> {
    #[inline]
    pub(crate) fn new(builder: Builder, adapter: Adapter, difficulty: NonZeroU8) -> Self {
        Self {
            builder,
            adapter,
            difficulty,
        }
    }
}

impl<Builder, Adapter> BuildUnsignedEvent for Pow<Builder, Adapter>
where
    Builder: BuildUnsignedEvent,
    Adapter: PowAdapter,
{
    type Error = PowBuildError<Builder::Error, Adapter::Error>;

    fn build(self, public_key: PublicKey) -> Result<UnsignedEvent, Self::Error> {
        let event: UnsignedEvent = self
            .builder
            .build(public_key)
            .map_err(PowBuildError::Builder)?;

        self.adapter
            .compute(event, self.difficulty)
            .map_err(PowBuildError::Pow)
    }
}

impl<Builder, Adapter> AsyncBuildUnsignedEvent for Pow<Builder, Adapter>
where
    Builder: AsyncBuildUnsignedEvent + Send + 'static,
    Adapter: AsyncPowAdapter,
{
    type Error = PowBuildError<Builder::Error, Adapter::Error>;

    fn build_async(
        self,
        public_key: PublicKey,
    ) -> BoxedFuture<'static, Result<UnsignedEvent, Self::Error>> {
        Box::pin(async move {
            let event: UnsignedEvent = self
                .builder
                .build_async(public_key)
                .await
                .map_err(PowBuildError::Builder)?;

            self.adapter
                .compute_async(event, self.difficulty)
                .await
                .map_err(PowBuildError::Pow)
        })
    }
}
