//! Collection of utilities to be used in [Mapper](crate::Mapper) `with` and `try_with` arguments

#[cfg(feature = "chrono")]
mod chrono;
#[cfg(feature = "chrono")]
pub use chrono::*;

mod hashmap;
mod option;
mod vec;
pub use hashmap::*;
pub use option::*;
pub use vec::*;

/// Mapper for types implementing [ExtraInto]
pub fn extra<F, I>(from: F) -> I
where
    F: ExtraInto<I>,
{
    ExtraInto::into_extra(from)
}

/// Owned trait to implement [Into] on foreign types
pub trait ExtraInto<I> {
    fn into_extra(self) -> I;
}

/// Owned trait to implement [TryInto] on foreign types
pub trait TryExtraInto<I> {
    type Error;
    fn try_into_extra(self) -> Result<I, Self::Error>;
}
