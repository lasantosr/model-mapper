use std::{collections::HashMap, error::Error, hash::Hash};

use super::*;

impl<KI, VI, KF, VF> ExtraInto<HashMap<KI, VI>> for HashMap<KF, VF>
where
    KI: Eq + Hash,
    KF: Into<KI> + Eq + Hash,
    VF: Into<VI>,
{
    fn into_extra(self) -> HashMap<KI, VI> {
        self.into_iter().map(|(k, v)| (k.into(), v.into())).collect()
    }
}

/// Mapper for [HashMap]
pub fn hashmap<KI, VI, KF, VF>(from: HashMap<KF, VF>) -> HashMap<KI, VI>
where
    KI: Eq + Hash,
    KF: Eq + Hash,
    KF: Into<KI>,
    VF: Into<VI>,
{
    from.into_iter().map(|(k, v)| (k.into(), v.into())).collect()
}

/// Mapper for [HashMap]
pub fn hashmap_extra<KI, VI, KF, VF>(from: HashMap<KF, VF>) -> HashMap<KI, VI>
where
    KI: Eq + Hash,
    KF: Eq + Hash,
    KF: ExtraInto<KI>,
    VF: ExtraInto<VI>,
{
    from.into_iter()
        .map(|(k, v)| (k.into_extra(), v.into_extra()))
        .collect()
}

impl<KI, VI, KF, VF> TryExtraInto<HashMap<KI, VI>> for HashMap<KF, VF>
where
    KI: Eq + Hash,
    KF: TryInto<KI> + Eq + Hash,
    <KF as TryInto<KI>>::Error: Error + Send + Sync + 'static,
    VF: TryInto<VI>,
    <VF as TryInto<VI>>::Error: Error + Send + Sync + 'static,
{
    type Error = anyhow::Error;

    fn try_into_extra(self) -> Result<HashMap<KI, VI>, Self::Error> {
        let mut ret = HashMap::with_capacity(self.len());
        for (k, v) in self.into_iter().map(|(k, v)| (k.try_into(), v.try_into())) {
            ret.insert(k?, v?);
        }
        Ok(ret)
    }
}

/// Mapper for [HashMap]
pub fn try_hashmap<KI, VI, KF, VF>(from: HashMap<KF, VF>) -> Result<HashMap<KI, VI>, anyhow::Error>
where
    KI: Eq + Hash,
    KF: Eq + Hash,
    KF: TryInto<KI>,
    <KF as TryInto<KI>>::Error: Error + Send + Sync + 'static,
    VF: TryInto<VI>,
    <VF as TryInto<VI>>::Error: Error + Send + Sync + 'static,
{
    let mut ret = HashMap::with_capacity(from.len());
    for (k, v) in from.into_iter().map(|(k, v)| (k.try_into(), v.try_into())) {
        ret.insert(k?, v?);
    }
    Ok(ret)
}

/// Mapper for [HashMap]
pub fn try_hashmap_extra<KI, VI, KF, VF>(from: HashMap<KF, VF>) -> Result<HashMap<KI, VI>, anyhow::Error>
where
    KI: Eq + Hash,
    KF: Eq + Hash,
    KF: TryExtraInto<KI>,
    <KF as TryExtraInto<KI>>::Error: Error + Send + Sync + 'static,
    VF: TryExtraInto<VI>,
    <VF as TryExtraInto<VI>>::Error: Error + Send + Sync + 'static,
{
    let mut ret = HashMap::with_capacity(from.len());
    for (k, v) in from.into_iter().map(|(k, v)| (k.try_into_extra(), v.try_into_extra())) {
        ret.insert(k?, v?);
    }
    Ok(ret)
}
