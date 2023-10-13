#![allow(dead_code)]

use model_mapper::{with, Mapper};

#[derive(Clone, Default)]
pub struct Entity {
    id: i64,
    name: String,
    fav_number: Option<i32>,
    age: Option<i32>,
    sizes: Option<Vec<i32>>,
}

#[derive(Clone, Default, Mapper)]
#[mapper(from, ty = Entity)]
pub struct ModelFrom {
    id: i64,
    name: String,
    fav_number: Option<i32>,
    #[mapper(with = with::option)]
    age: Option<i64>,
    #[mapper(with = with::option_extra)]
    sizes: Option<Vec<i64>>,
}

#[derive(Clone, Default, Mapper)]
#[mapper(into, ty = Entity)]
pub struct ModelInto {
    id: i64,
    name: String,
    fav_number: Option<i32>,
    #[mapper(with = with::option)]
    age: Option<i16>,
    #[mapper(with = with::option_extra)]
    sizes: Option<Vec<i16>>,
}

#[derive(Clone, Default, Mapper)]
#[mapper(try_from, ty = Entity)]
pub struct ModelTryFrom {
    id: i64,
    name: String,
    fav_number: Option<i32>,
    #[mapper(try_with = with::try_option)]
    age: Option<i16>,
    #[mapper(try_with = with::try_option_extra)]
    sizes: Option<Vec<i16>>,
}

#[derive(Clone, Default, Mapper)]
#[mapper(try_into, ty = Entity)]
pub struct ModelTryInto {
    id: i64,
    name: String,
    fav_number: Option<i32>,
    #[mapper(try_with = with::try_option)]
    age: Option<i64>,
    #[mapper(try_with = with::try_option_extra)]
    sizes: Option<Vec<i64>>,
}

#[derive(Clone, Default, Mapper)]
#[mapper(from, into, ty = Entity, ignore(field = name), ignore(field = age))]
pub struct ModelFull {
    id: i64,
    fav_number: Option<i32>,
    #[mapper(rename = "sizes")]
    the_sizes: Option<Vec<i32>>,
    #[mapper(skip)]
    other: String,
}

#[derive(Clone, Default)]
pub struct OtherEntity {
    id: i64,
    first_name: String,
    last_name: Option<String>,
    email: Option<String>,
}

#[derive(Clone, Default, Mapper)]
#[mapper(derive(try_from, ty = Entity, ignore(field = fav_number), ignore(field = sizes)))]
#[mapper(derive(into, ty = OtherEntity, ignore(field = email)))]
pub struct ModelMultiple {
    id: i64,
    #[mapper(when(ty = OtherEntity, rename = "first_name"))]
    name: String,
    #[mapper(when(ty = Entity, try_with = with::try_remove_option))]
    #[mapper(when(ty = OtherEntity, skip))]
    age: i32,
    #[mapper(when(ty = Entity, skip))]
    #[mapper(when(ty = OtherEntity, rename = "last_name", with = with::add_option))]
    surname: String,
    #[mapper(skip)]
    other: String,
}

pub struct EntityTuple(i32, String);

#[derive(Mapper)]
#[mapper(from, try_into, ty = EntityTuple)]
pub struct ModelTuple(i64, String);

pub enum EntityEnum {
    Empty,
    Entity(Entity),
    InPlace {
        id: i64,
        name: String,
        age: Option<i32>,
        sizes: Option<Vec<i32>>,
    },
    Unknown,
}

#[derive(Mapper)]
#[mapper(try_from, into, ty = EntityEnum, ignore(field = Unknown, default = ModelEnum::Empty))]
pub enum ModelEnum {
    Empty,
    Entity(Entity),
    #[mapper(rename = InPlace, ignore(field = sizes, default = Some(vec![5])))]
    New {
        id: i64,
        #[mapper(rename = "name")]
        first_name: String,
        #[mapper(with = with::option)]
        #[mapper(try_with = with::try_option)]
        age: Option<i16>,
        #[mapper(skip, default = true)]
        random: bool,
    },
    #[mapper(skip, default = EntityEnum::Unknown)]
    Error,
}

// It doesn't need the `#[test]` macro because it passes if it compiles.
fn test_implemented_traits() {
    let _from: ModelFrom = ModelFrom::from(Entity::default());
    let _try_from: Result<ModelTryFrom, _> = ModelTryFrom::try_from(Entity::default());
    let _into: Entity = ModelInto::default().into();
    let _try_into: Result<Entity, _> = ModelTryInto::default().try_into();
    let _from_full: ModelFull = ModelFull::from(Entity::default());
    let _into_full: Entity = ModelFull::default().into();
    let _try_from_multiple: Result<ModelMultiple, _> = ModelMultiple::try_from(Entity::default());
    let _into_multiple: OtherEntity = ModelMultiple::default().into();
    let _try_from_enum: Result<ModelEnum, _> = ModelEnum::try_from(EntityEnum::Empty);
    let _try_into_enum: Result<EntityEnum, _> = ModelEnum::Empty.try_into();
}
