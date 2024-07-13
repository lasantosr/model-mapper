//! Collection of utilities to be used in [Mapper](crate::Mapper) `with` and `try_with` arguments

use std::{collections::HashMap, convert::Infallible, hash::Hash, str::FromStr};

#[cfg(feature = "chrono")]
pub mod chrono;

#[cfg(feature = "serde")]
pub mod serde;

/// This type abstracts different kinds of wrappers like [Option], [Vec], etc.
pub trait Wrapper<T> {
    type Wrapper<U>;
    fn map_inner<Z: Fn(T) -> U, U>(self, f: Z) -> Self::Wrapper<U>;
    fn try_map_inner<Z: Fn(T) -> Result<U, E>, U, E>(self, f: Z) -> Result<Self::Wrapper<U>, E>;
}
impl<T> Wrapper<T> for Option<T> {
    type Wrapper<U> = Option<U>;

    fn map_inner<Z: Fn(T) -> U, U>(self, f: Z) -> Self::Wrapper<U> {
        self.map(f)
    }

    fn try_map_inner<Z: Fn(T) -> Result<U, E>, U, E>(self, f: Z) -> Result<Self::Wrapper<U>, E> {
        self.map(f).transpose()
    }
}
impl<T> Wrapper<T> for Vec<T> {
    type Wrapper<U> = Vec<U>;

    fn map_inner<Z: Fn(T) -> U, U>(self, f: Z) -> Self::Wrapper<U> {
        self.into_iter().map(f).collect()
    }

    fn try_map_inner<Z: Fn(T) -> Result<U, E>, U, E>(self, f: Z) -> Result<Self::Wrapper<U>, E> {
        self.into_iter().map(f).collect()
    }
}
impl<T, K: Eq + Hash> Wrapper<T> for HashMap<K, T> {
    type Wrapper<U> = HashMap<K, U>;

    fn map_inner<Z: Fn(T) -> U, U>(self, f: Z) -> Self::Wrapper<U> {
        self.into_iter().map(|(k, v)| (k, f(v))).collect()
    }

    fn try_map_inner<Z: Fn(T) -> Result<U, E>, U, E>(self, f: Z) -> Result<Self::Wrapper<U>, E> {
        self.into_iter().map(|(k, v)| f(v).map(|v| (k, v))).collect()
    }
}

/// This type abstracts different kinds of wrappers like [Option], [Vec], etc. when its value is another [Wrapper]
pub trait NestedWrapper<W: Wrapper<T>, T> {
    type NestedWrapper<U>;
    fn map_wrapper<Z: Fn(T) -> U, U>(self, f: Z) -> Self::NestedWrapper<U>;
    fn try_map_wrapper<Z: Fn(T) -> Result<U, E>, U, E>(self, f: Z) -> Result<Self::NestedWrapper<U>, E>;
}
impl<W: Wrapper<T>, T> NestedWrapper<W, T> for Option<W> {
    type NestedWrapper<U> = Option<W::Wrapper<U>>;

    fn map_wrapper<Z: Fn(T) -> U, U>(self, f: Z) -> Self::NestedWrapper<U> {
        self.map(|i| i.map_inner(f))
    }

    fn try_map_wrapper<Z: Fn(T) -> Result<U, E>, U, E>(self, f: Z) -> Result<Self::NestedWrapper<U>, E> {
        self.map(|i| i.try_map_inner(f)).transpose()
    }
}
impl<W: Wrapper<T>, T> NestedWrapper<W, T> for Vec<W> {
    type NestedWrapper<U> = Vec<W::Wrapper<U>>;

    fn map_wrapper<Z: Fn(T) -> U, U>(self, f: Z) -> Self::NestedWrapper<U> {
        self.into_iter().map(|i| i.map_inner(&f)).collect()
    }

    fn try_map_wrapper<Z: Fn(T) -> Result<U, E>, U, E>(self, f: Z) -> Result<Self::NestedWrapper<U>, E> {
        self.into_iter().map(|i| i.try_map_inner(&f)).collect()
    }
}
impl<W: Wrapper<T>, T, K: Eq + Hash> NestedWrapper<W, T> for HashMap<K, W> {
    type NestedWrapper<U> = HashMap<K, W::Wrapper<U>>;

    fn map_wrapper<Z: Fn(T) -> U, U>(self, f: Z) -> Self::NestedWrapper<U> {
        self.into_iter().map(|(k, v)| (k, v.map_inner(&f))).collect()
    }

    fn try_map_wrapper<Z: Fn(T) -> Result<U, E>, U, E>(self, f: Z) -> Result<Self::NestedWrapper<U>, E> {
        self.into_iter()
            .map(|(k, v)| v.try_map_inner(&f).map(|v| (k, v)))
            .collect()
    }
}

/// Trait to map between types
pub trait TypeMapper<F, I> {
    /// Maps between types
    fn map(from: F) -> I;
    /// Maps between types
    fn try_map<E>(from: F) -> Result<I, E> {
        Ok(Self::map(from))
    }
    /// Maps between [Wrapper] types
    fn map_wrapped<W>(from: W) -> <W as Wrapper<F>>::Wrapper<I>
    where
        W: Wrapper<F>,
    {
        from.map_inner(Self::map)
    }
    /// Maps between [Wrapper] types
    fn try_map_wrapped<W>(from: W) -> Result<<W as Wrapper<F>>::Wrapper<I>, Infallible>
    where
        W: Wrapper<F>,
    {
        Ok(Self::map_wrapped(from))
    }
    /// Maps between [Wrapper] types
    fn map_nested_wrapped<NW, W>(from: NW) -> <NW as NestedWrapper<W, F>>::NestedWrapper<I>
    where
        NW: NestedWrapper<W, F>,
        W: Wrapper<F>,
    {
        from.map_wrapper(Self::map)
    }
    /// Maps between [Wrapper] types
    fn try_map_nested_wrapped<NW, W, E>(from: NW) -> Result<<NW as NestedWrapper<W, F>>::NestedWrapper<I>, Infallible>
    where
        NW: NestedWrapper<W, F>,
        W: Wrapper<F>,
    {
        Ok(Self::map_nested_wrapped(from))
    }
    /// Maps encapsulating into an [Option]
    fn map_into_option(from: F) -> Option<I> {
        Some(Self::map(from))
    }
    /// Maps encapsulating into an [Option]
    fn try_map_into_option<E>(from: F) -> Result<Option<I>, Infallible> {
        Ok(Self::map_into_option(from))
    }
    /// Maps a [Wrapper] encapsulating into an [Option]
    fn map_wrapped_into_option<W>(from: W) -> Option<<W as Wrapper<F>>::Wrapper<I>>
    where
        W: Wrapper<F>,
    {
        Some(Self::map_wrapped(from))
    }
    /// Maps a [Wrapper] encapsulating into an [Option]
    fn try_map_wrapped_into_option<W, E>(from: W) -> Result<Option<<W as Wrapper<F>>::Wrapper<I>>, Infallible>
    where
        W: Wrapper<F>,
    {
        Ok(Self::map_wrapped_into_option(from))
    }
    /// Maps a nested [Wrapper] encapsulating into an [Option]
    fn map_nested_wrapped_into_option<NW, W>(from: NW) -> Option<<NW as NestedWrapper<W, F>>::NestedWrapper<I>>
    where
        NW: NestedWrapper<W, F>,
        W: Wrapper<F>,
    {
        Some(Self::map_nested_wrapped(from))
    }
    /// Maps a nested [Wrapper] encapsulating into an [Option]
    fn try_map_nested_wrapped_into_option<NW, W, Infallible>(
        from: NW,
    ) -> Result<Option<<NW as NestedWrapper<W, F>>::NestedWrapper<I>>, Infallible>
    where
        NW: NestedWrapper<W, F>,
        W: Wrapper<F>,
    {
        Ok(Self::map_nested_wrapped_into_option(from))
    }
    /// Maps removing an [Option]
    fn try_map_removing_option(from: Option<F>) -> Result<I, anyhow::Error> {
        from.map(Self::map)
            .ok_or_else(|| anyhow::anyhow!("The value was required but not present"))
    }
    /// Maps a wrapped type removing an [Option]
    fn try_map_wrapped_removing_option<W>(from: Option<W>) -> Result<<W as Wrapper<F>>::Wrapper<I>, anyhow::Error>
    where
        W: Wrapper<F>,
    {
        from.map(Self::map_wrapped)
            .ok_or_else(|| anyhow::anyhow!("The value was required but not present"))
    }
    /// Maps a nested wrapped type removing an [Option]
    fn try_map_nested_wrapped_removing_option<NW, W>(
        from: Option<NW>,
    ) -> Result<<NW as NestedWrapper<W, F>>::NestedWrapper<I>, anyhow::Error>
    where
        NW: NestedWrapper<W, F>,
        W: Wrapper<F>,
    {
        from.map(Self::map_nested_wrapped)
            .ok_or_else(|| anyhow::anyhow!("The value was required but not present"))
    }
}

/// Trait to try to map between types
pub trait TypeFallibleMapper<F, I> {
    type Error;
    /// Maps between types
    fn try_map(from: F) -> Result<I, Self::Error>;
    /// Maps between [Wrapper] types
    fn try_map_wrapped<W>(from: W) -> Result<<W as Wrapper<F>>::Wrapper<I>, Self::Error>
    where
        W: Wrapper<F>,
    {
        from.try_map_inner(Self::try_map)
    }
    /// Maps between [Wrapper] types
    fn try_map_nested_wrapped<NW, W>(from: NW) -> Result<<NW as NestedWrapper<W, F>>::NestedWrapper<I>, Self::Error>
    where
        NW: NestedWrapper<W, F>,
        W: Wrapper<F>,
    {
        from.try_map_wrapper(Self::try_map)
    }
    /// Maps encapsulating into an [Option]
    fn try_map_into_option<E>(from: F) -> Result<Option<I>, Self::Error> {
        Self::try_map(from).map(Some)
    }
    /// Maps a [Wrapper] encapsulating into an [Option]
    fn try_map_wrapped_into_option<W>(from: W) -> Result<Option<<W as Wrapper<F>>::Wrapper<I>>, Self::Error>
    where
        W: Wrapper<F>,
    {
        Self::try_map_wrapped(from).map(Some)
    }
    /// Maps a nested [Wrapper] encapsulating into an [Option]
    #[allow(clippy::type_complexity)] // Can't extract generics into type
    fn try_map_nested_wrapped_into_option<NW, W>(
        from: NW,
    ) -> Result<Option<<NW as NestedWrapper<W, F>>::NestedWrapper<I>>, Self::Error>
    where
        NW: NestedWrapper<W, F>,
        W: Wrapper<F>,
    {
        Self::try_map_nested_wrapped(from).map(Some)
    }
    /// Maps removing an [Option]
    fn try_map_removing_option(from: Option<F>) -> Result<I, anyhow::Error>
    where
        Self::Error: std::error::Error + Send + Sync + 'static,
    {
        from.map(Self::try_map)
            .ok_or_else(|| anyhow::anyhow!("The value was required but not present"))?
            .map_err(Into::into)
    }
    /// Maps a wrapped type removing an [Option]
    fn try_map_wrapped_removing_option<W>(from: Option<W>) -> Result<<W as Wrapper<F>>::Wrapper<I>, anyhow::Error>
    where
        W: Wrapper<F>,
        Self::Error: std::error::Error + Send + Sync + 'static,
    {
        from.map(Self::try_map_wrapped)
            .ok_or_else(|| anyhow::anyhow!("The value was required but not present"))?
            .map_err(Into::into)
    }
    /// Maps a nested wrapped type removing an [Option]
    fn try_map_nested_wrapped_removing_option<NW, W>(
        from: Option<NW>,
    ) -> Result<<NW as NestedWrapper<W, F>>::NestedWrapper<I>, anyhow::Error>
    where
        NW: NestedWrapper<W, F>,
        W: Wrapper<F>,
        Self::Error: std::error::Error + Send + Sync + 'static,
    {
        from.map(Self::try_map_nested_wrapped)
            .ok_or_else(|| anyhow::anyhow!("The value was required but not present"))?
            .map_err(Into::into)
    }
}

/// Mapper that uses the [Into] trait
pub struct IntoMapper;
impl<F, I> TypeMapper<F, I> for IntoMapper
where
    F: Into<I>,
{
    fn map(from: F) -> I {
        from.into()
    }
}

/// Mapper that uses the [TryInto] trait
pub struct TryIntoMapper;
impl<F, I> TypeFallibleMapper<F, I> for TryIntoMapper
where
    F: TryInto<I>,
{
    type Error = F::Error;

    fn try_map(from: F) -> Result<I, Self::Error> {
        from.try_into()
    }
}

/// Maps from any type to its display string
pub struct ToStringMapper;
impl<T: ToString> TypeMapper<T, String> for ToStringMapper {
    fn map(from: T) -> String {
        from.to_string()
    }
}

/// Maps from any [str] to any type that implements [FromStr]
struct TryFromStringMapper;
impl<F: AsRef<str>, I: FromStr> TypeFallibleMapper<F, I> for TryFromStringMapper {
    type Error = I::Err;

    fn try_map(from: F) -> Result<I, Self::Error> {
        from.as_ref().parse()
    }
}
