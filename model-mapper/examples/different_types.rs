#![allow(dead_code)]

use std::collections::HashMap;

use model_mapper::{with::*, Mapper};

mod fallible_conversion {
    use super::*;

    struct Foo {
        field1: String,
        field2: i64,
    }

    #[derive(Mapper)]
    #[mapper(try_from, into, ty = Foo)]
    struct Bar {
        /// This field needs a mapping function to convert between [String] and [i32]
        #[mapper(from_with = TryFromStringMapper::try_map, into_with = ToStringMapper::map)]
        field1: i32,
        /// This field doesn't because [i32] implements [TryFrom<i64>]
        field2: i32,
    }
}

mod wrapped_types {

    use super::*;

    struct Foo {
        field1: String,
        field2: Vec<i32>,
        field3: HashMap<String, i32>,
        field4: Vec<Option<i32>>,
    }

    #[derive(Mapper)]
    #[mapper(from, ty = Foo)]
    struct Bar {
        field1: String,
        /// [i64] implements [From<i34>], but the vec doesn't so we need a helper function
        #[mapper(with = IntoMapper::map_wrapped)]
        field2: Vec<i64>,
        /// the same helper works with other types like [HashMap] and [Option]
        #[mapper(with = IntoMapper::map_wrapped)]
        field3: HashMap<String, i64>,
        /// and we can even map nested wrappers
        #[mapper(with = IntoMapper::map_nested_wrapped)]
        field4: Vec<Option<i64>>,
    }
}

mod remove_option {
    use super::*;

    struct Foo {
        field1: String,
        field2: i64,
    }

    #[derive(Mapper)]
    #[mapper(from, try_into, ty = Foo)]
    struct Bar {
        /// This field needs a mapping function to try to remove the [Option], failing if [None]
        #[mapper(into_with = IntoMapper::try_map_removing_option)]
        field1: Option<String>,
        field2: i64,
    }
}

mod serde_types {
    use super::*;

    #[derive(serde::Serialize)]
    struct FooProps {
        prop1: String,
        prop2: bool,
    }

    struct Foo {
        field1: String,
        field2: FooProps,
        field3: FooProps,
    }

    #[derive(serde::Deserialize)]
    struct BarProps {
        prop1: String,
        prop2: bool,
    }

    #[derive(Mapper)]
    #[mapper(try_from, ty = Foo)]
    struct Bar {
        field1: String,
        /// We can map between [serde] types
        #[mapper(with = with_serde::Mapper::try_map)]
        field2: BarProps,
        /// Or directly to [serde_json::Value]
        #[mapper(with = with_serde::ToJsonMapper::try_map)]
        field3: serde_json::Value,
    }
}

mod custom_mapper {
    use super::*;

    struct Foo {
        full_name: String,
        age: i64,
    }

    struct Name {
        first_name: String,
        last_name: Option<String>,
    }

    #[derive(Mapper)]
    #[mapper(from, into, ty = Foo)]
    struct Bar {
        // This field needs a custom mapping function
        #[mapper(rename = full_name, from_with = parse_name, into_with = split_name)]
        name: Name,
        age: i64,
    }

    fn parse_name(full_name: String) -> Name {
        full_name
            .split_once(' ')
            .map(|(first_name, last_name)| Name {
                first_name: first_name.into(),
                last_name: Some(last_name.into()),
            })
            .unwrap_or_else(|| Name {
                first_name: full_name,
                last_name: None,
            })
    }

    fn split_name(name: Name) -> String {
        match name.last_name {
            Some(last_name) => format!("{} {}", name.first_name, last_name),
            None => name.first_name,
        }
    }
}

fn main() {}
