#![allow(dead_code)]

use model_mapper::Mapper;

#[derive(Debug, Clone, PartialEq, Eq)]
struct FooTuple(String, bool);

#[derive(Mapper)]
#[mapper(from, into, ty = FooTuple)]
struct BarTuple(String, bool);

#[derive(Debug, Clone, PartialEq, Eq)]
struct FooNamed {
    field1: String,
    field2: bool,
}

#[derive(Mapper)]
#[mapper(from, into, ty = FooNamed)]
struct BarNamed {
    field1: String,
    field2: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FooEnumSimple {
    One,
    Two,
    Three,
}

#[derive(Mapper)]
#[mapper(from, into, ty = FooEnumSimple)]
enum BarEnumSimple {
    One,
    Two,
    Three,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FooEnumComplex {
    One,
    Two(i32, FooTuple),
    Three { field1: String, field2: bool },
}

#[derive(Mapper)]
#[mapper(from, into, ty = FooEnumComplex)]
enum BarEnumComplex {
    One,
    Two(i32, BarTuple),
    Three { field1: String, field2: bool },
}

fn main() {
    let source = FooEnumComplex::Two(1, FooTuple("val".to_owned(), true));
    let bar = BarEnumComplex::from(source.clone());
    let mapped: FooEnumComplex = bar.into();

    assert_eq!(source, mapped);
}
