use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::sqlite::SqliteTypeInfo;
use sqlx::{Database, Encode, Sqlite, Type};

pub(crate) struct Flags(u16);

impl Flags {
    pub(crate) const NONE: Self = Self(0); // 0

    pub(crate) const READ: Self = Self(1 << 0); // 1

    pub(crate) const WRITE: Self = Self(1 << 1); // 2

    pub(crate) const PRIVATE_MESSAGE: Self = Self(1 << 2); // 4

    pub(crate) const HINT: Self = Self(1 << 3); // 8

    pub(crate) const RECEIVED: Self = Self(1 << 4); // 16

    /// New empty flags.
    #[inline]
    pub(crate) const fn new() -> Self {
        Self::NONE
    }

    /// New flags with [`READ`] and [`WRITE`].
    pub(crate) const fn read_write() -> Self {
        let mut flags: Self = Self::new();
        flags.add(Self::READ);
        flags.add(Self::WRITE);
        flags
    }

    /// Add flag.
    #[inline]
    pub(crate) const fn add(&mut self, other: Self) {
        self.0 |= other.0;
    }

    // /// Remove flag.
    // #[inline]
    // pub(crate) const fn remove(&mut self, other: Self) {
    //     self.0 ^= other.0;
    // }

    #[inline]
    const fn as_u16(&self) -> u16 {
        self.0
    }
}

impl Type<Sqlite> for Flags {
    fn type_info() -> SqliteTypeInfo {
        <u16 as Type<Sqlite>>::type_info()
    }
}

impl<'a> Encode<'a, Sqlite> for Flags {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as Database>::ArgumentBuffer<'a>,
    ) -> Result<IsNull, BoxDynError> {
        let val: u16 = self.as_u16();
        <u16 as Encode<Sqlite>>::encode_by_ref(&val, buf)
    }
}
