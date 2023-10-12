use std::error::Error;

use super::*;

impl<F, I> ExtraInto<Option<I>> for Option<F>
where
    F: Into<I>,
{
    fn into_extra(self) -> Option<I> {
        self.map(Into::into)
    }
}

/// Mapper for [Option]
pub fn option<F, I>(from: Option<F>) -> Option<I>
where
    F: Into<I>,
{
    from.map(Into::into)
}

/// Mapper for [Option]
pub fn option_extra<F, I>(from: Option<F>) -> Option<I>
where
    F: ExtraInto<I>,
{
    from.map(ExtraInto::into_extra)
}

/// Mapper to wrap into [Some]
pub fn add_option<F, I>(from: F) -> Option<I>
where
    F: Into<I>,
{
    Some(from.into())
}

/// Mapper to wrap into [Some]
pub fn add_option_extra<F, I>(from: F) -> Option<I>
where
    F: ExtraInto<I>,
{
    Some(from.into_extra())
}

impl<F, I> TryExtraInto<Option<I>> for Option<F>
where
    F: TryInto<I>,
{
    type Error = F::Error;

    fn try_into_extra(self) -> Result<Option<I>, Self::Error> {
        self.map(TryInto::try_into).transpose()
    }
}

/// Mapper for [Option]
pub fn try_option<F, I>(from: Option<F>) -> Result<Option<I>, <F as TryInto<I>>::Error>
where
    F: TryInto<I>,
{
    from.map(TryInto::try_into).transpose()
}

/// Mapper for [Option]
pub fn try_option_extra<F, I>(from: Option<F>) -> Result<Option<I>, <F as TryExtraInto<I>>::Error>
where
    F: TryExtraInto<I>,
{
    from.map(TryExtraInto::try_into_extra).transpose()
}

/// Mapper to wrap into [Some]
pub fn try_add_option<F, I>(from: F) -> Result<Option<I>, <F as TryInto<I>>::Error>
where
    F: TryInto<I>,
{
    Ok(Some(from.try_into()?))
}

/// Mapper to wrap into [Some]
pub fn try_add_option_extra<F, I>(from: F) -> Result<Option<I>, <F as TryExtraInto<I>>::Error>
where
    F: TryExtraInto<I>,
{
    Ok(Some(from.try_into_extra()?))
}

/// Mapper to unwrap an [Option]
pub fn try_remove_option<F, I>(from: Option<F>) -> Result<I, anyhow::Error>
where
    F: TryInto<I>,
    <F as TryInto<I>>::Error: Error + Send + Sync + 'static,
{
    match from {
        Some(f) => Ok(f.try_into()?),
        None => Err(anyhow::anyhow!("The value was required but not present")),
    }
}

/// Mapper to unwrap an [Option]
pub fn try_remove_option_extra<F, I>(from: Option<F>) -> Result<I, anyhow::Error>
where
    F: TryExtraInto<I>,
    <F as TryExtraInto<I>>::Error: Error + Send + Sync + 'static,
{
    match from {
        Some(f) => Ok(f.try_into_extra()?),
        None => Err(anyhow::anyhow!("The value was required but not present")),
    }
}
