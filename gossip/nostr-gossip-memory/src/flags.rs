#[derive(Clone, Copy, Default)]
pub(crate) struct Flags(u16);

impl Flags {
    //pub(crate) const NONE: Self = Self(0); // 0

    pub(crate) const READ: Self = Self(1 << 0); // 1

    pub(crate) const WRITE: Self = Self(1 << 1); // 2

    pub(crate) const PRIVATE_MESSAGE: Self = Self(1 << 2); // 4

    pub(crate) const HINT: Self = Self(1 << 3); // 8

    pub(crate) const RECEIVED: Self = Self(1 << 4); // 16

    // /// New empty flags.
    // #[inline]
    // pub(crate) const fn new() -> Self {
    //     Self::NONE
    // }

    /// Add flag.
    #[inline]
    pub(crate) const fn add(&mut self, other: Self) {
        self.0 |= other.0;
    }

    /// Remove flag.
    #[inline]
    pub(crate) const fn remove(&mut self, other: Self) {
        self.0 ^= other.0;
    }

    #[inline]
    pub(crate) const fn has(&self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    // #[inline]
    // pub(crate) const fn as_u16(&self) -> u16 {
    //     self.0
    // }
}
