#![allow(dead_code)]

use chrono::{DateTime, Utc};
use model_mapper::{with::*, Mapper};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Foo {
    id: i64,
    name: String,
    surname: Option<String>,
    age: i32,
    tags: Option<Vec<String>>,
    created_at: DateTime<Utc>,
}

#[derive(Mapper)]
#[mapper(
    from, 
    into(custom = "into_foo_struct"),
    ty = Foo,
    add(field = surname, default),
    add(field = age, ty = i32)
)]
struct Bar {
    id: i64,
    name: String,
    #[mapper(skip(default(value = tags.as_ref().map(|t| t.iter().any(|t| t == "admin")).unwrap_or_default())))]
    is_admin: bool,
    #[mapper(from_with = IntoMapper::map_removing_option_default)]
    tags: Vec<String>,
    #[mapper(rename = created_at)]
    created: DateTime<Utc>,
}

enum FooEnum {
    Empty,
    Id(i64),
    Custom {
        id: i64,
        name: String,
        surname: Option<String>,
        age: i32,
        tags: Option<Vec<String>>,
        created_at: DateTime<Utc>,
    },
    Unknown,
}

#[derive(Mapper)]
#[mapper(
    from, 
    into(custom),
    ty = FooEnum,
    add(field = Unknown, default(value = BarEnum::Empty)),
)]
enum BarEnum {
    Empty,
    Id(i64),
    #[mapper(
        rename = Custom,
        add(field = surname, default),
        add(field = age, ty = i32)
    )]
    New {
        id: i64,
        name: String,
        #[mapper(skip(default(value = tags.as_ref().map(|t| t.iter().any(|t| t == "admin")).unwrap_or_default())))]
        is_admin: bool,
        #[mapper(from_with = IntoMapper::map_removing_option_default)]
        tags: Vec<String>,
        #[mapper(rename = created_at)]
        created: DateTime<Utc>,
    },
    #[mapper(skip(default(value = FooEnum::Unknown)))]
    Error,
}

fn main() {}
