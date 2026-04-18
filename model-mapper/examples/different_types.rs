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
        field1: Option<i64>,
        field2: VecDeque<i64>,
        field3: HashMap<String, i64>,
        field4: Box<i64>,
        field5: Box<i64>,
        field6: i64,
        field7: Vec<Option<i64>>,
        field8: Option<Vec<Option<i64>>>,
        field9: Option<HashMap<String, Option<Box<i64>>>>,
        field10: Option<Vec<String>>,
    }

    #[derive(Mapper)]
    #[mapper(into, ty = Foo)]
    struct Bar {
        /// [i32] implements [Into<i64>], but [Option<i32>] doesn't implement [Into<Option<i64>>]
        /// we need to give some hint to the macro
        #[mapper(opt)]
        field1: Option<i32>,
        /// there are more hints like for iterators, that works with any [IntoIterator] and [FromIterator] combination
        #[mapper(iter)]
        field2: HashSet<i32>,
        /// or for [HashMap], which are just iterators of 2-element tuples
        #[mapper(map)]
        field3: HashMap<String, i32>,
        /// also for [Box]ed values
        #[mapper(boxed)]
        field4: Box<i32>,
        /// even mapping to a boxed value allocating it on the heap
        #[mapper(box)]
        field5: i32,
        /// or moving the value out of the heap
        #[mapper(unbox)]
        field6: Box<i32>,
        /// and they can be nested
        #[mapper(iter(opt))]
        field7: Vec<Option<i32>>,
        /// at any level
        #[mapper(opt(iter(opt)))]
        field8: Option<Vec<Option<i32>>>,
        /// mixing any of them
        #[mapper(opt(map(opt(boxed))))]
        field9: Option<HashMap<String, Option<Box<i32>>>>,
        /// or even with custom functions
        /// `field10` will contain the inner field, on this case the [i32], not the outer [Option]
        #[mapper(opt(iter(with = field10.to_string())))]
        field10: Option<Vec<i32>>,
    }
}

fn main() {}
