use super::*;

impl<F, I> ExtraInto<Vec<I>> for Vec<F>
where
    F: Into<I>,
{
    fn into_extra(self) -> Vec<I> {
        self.into_iter().map(Into::into).collect()
    }
}

/// Mapper for [Vec]
pub fn vec<F, I>(from: Vec<F>) -> Vec<I>
where
    F: Into<I>,
{
    from.into_iter().map(Into::into).collect()
}

/// Mapper for [Vec]
pub fn vec_extra<F, I>(from: Vec<F>) -> Vec<I>
where
    F: ExtraInto<I>,
{
    from.into_iter().map(ExtraInto::into_extra).collect()
}

impl<F, I> TryExtraInto<Vec<I>> for Vec<F>
where
    F: TryInto<I>,
{
    type Error = F::Error;

    fn try_into_extra(self) -> Result<Vec<I>, Self::Error> {
        let mut ret = Vec::with_capacity(self.len());
        for i in self.into_iter().map(TryInto::try_into) {
            ret.push(i?);
        }
        Ok(ret)
    }
}

/// Mapper for [Vec]
pub fn try_vec<F, I>(from: Vec<F>) -> Result<Vec<I>, <F as TryInto<I>>::Error>
where
    F: TryInto<I>,
{
    let mut ret = Vec::with_capacity(from.len());
    for i in from.into_iter().map(TryInto::try_into) {
        ret.push(i?);
    }
    Ok(ret)
}

/// Mapper for [Vec]
pub fn try_vec_extra<F, I>(from: Vec<F>) -> Result<Vec<I>, <F as TryExtraInto<I>>::Error>
where
    F: TryExtraInto<I>,
{
    let mut ret = Vec::with_capacity(from.len());
    for i in from.into_iter().map(TryExtraInto::try_into_extra) {
        ret.push(i?);
    }
    Ok(ret)
}
