pub struct Flags(u16);

impl Flags {
    pub const NONE: Self = Self(0); // 0

    pub const READ: Self = Self(1 << 0); // 1

    pub const WRITE: Self = Self(1 << 1); // 2

    pub const HINT: Self = Self(1 << 2); // 4

    pub const PRIVATE_MESSAGE: Self = Self(1 << 3); // 8

    /// New empty flags.
    #[inline]
    pub const fn new() -> Self {
        Self(0)
    }

    /// Add flag.
    #[inline]
    pub const fn add(&mut self, other: Self) {
        self.0 |= other.0;
    }

    /// Remove flag.
    #[inline]
    pub const fn remove(&mut self, other: Self) {
        self.0 ^= other.0;
    }

    #[inline]
    pub const fn as_u16(&self) -> u16 {
        self.0
    }
}
