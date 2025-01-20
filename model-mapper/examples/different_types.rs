#![allow(dead_code)]

use std::collections::{HashMap, HashSet, VecDeque};

use model_mapper::Mapper;

mod using_with {

    use super::*;

    struct Foo {
        field1: String,
        field2: i64,
    }

    #[derive(Mapper)]
    #[mapper(into, ty = Foo)]
    struct Bar {
        /// This field needs a mapping function to convert between [String] and [i32].
        /// We can specify a function to map between the types
        #[mapper(with = field1.to_string())]
        field1: i32,
        field2: i32,
    }

    #[derive(Mapper)]
    #[mapper(try_from, into, ty = Foo)]
    struct Bar2 {
        /// Or if we're deriving both 'from' and 'into', we can provide functions for both of them.
        /// We can specify a path to a function receiving the field, or provide an expression using any field
        #[mapper(from_with = parse_field_1, into_with = format!("{},{}", field1, field2))]
        field1: i32,
        /// This field doesn't because [i32] implements [TryFrom<i64>]
        field2: i32,
    }

    fn parse_field_1(value: String) -> Result<i32, std::num::ParseIntError> {
        value.split(',').next().unwrap().parse()
    }
}

mod wrapped_types {

    use super::*;

    struct Foo {
        field1: Option<i32>,
        field2: VecDeque<i32>,
        field3: HashMap<String, i32>,
        field4: Vec<Option<i32>>,
        field5: Option<Vec<Option<i32>>>,
        field6: Option<HashMap<String, Option<Vec<i32>>>>,
        field7: Option<Vec<i32>>,
    }

    #[derive(Mapper)]
    #[mapper(from, ty = Foo)]
    struct Bar {
        /// [i64] implements [From<i34>], but [Option<i64>] doesn't implement [From<Option<i34>>]
        /// we need to give some hint to the macro
        #[mapper(opt)]
        field1: Option<i64>,
        /// there are more hints like for iterators, that works with any [IntoIterator] and [FromIterator] combination
        #[mapper(iter)]
        field2: HashSet<i64>,
        /// or for [HashMap], which are just iterators of 2-element tuples
        #[mapper(map)]
        field3: HashMap<String, i64>,
        /// and they can be nested
        #[mapper(iter(opt))]
        field4: Vec<Option<i64>>,
        /// at any level
        #[mapper(opt(iter(opt)))]
        field5: Option<Vec<Option<i64>>>,
        /// mixing any of them
        #[mapper(opt(map(opt(iter))))]
        field6: Option<HashMap<String, Option<Vec<i64>>>>,
        /// or even with custom functions
        /// the field will contain the inner field, on this case the [i32], not the outer [Option]
        #[mapper(opt(iter(with = field7.to_string())))]
        field7: Option<Vec<String>>,
    }
}

fn main() {}
