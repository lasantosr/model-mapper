#![allow(dead_code)]

use model_mapper::Mapper;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Foo {
    field1: String,
    field2: bool,
}

#[derive(Mapper)]
#[mapper(from, into, ty = Foo)]
struct Bar {
    #[mapper(rename = field1)]
    first_field: String,
    #[mapper(rename = field2)]
    second_field: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FooEnum {
    One,
    Two(i32, Foo),
    Three { field1: String, field2: bool },
}

#[derive(Mapper)]
#[mapper(from, into, ty = FooEnum)]
enum BarEnum {
    #[mapper(rename = One)]
    First,
    #[mapper(rename = Two)]
    Second(i32, Bar),
    #[mapper(rename = Three)]
    Third {
        #[mapper(rename = field1)]
        first_field: String,
        #[mapper(rename = field2)]
        second_field: bool,
    },
}

fn main() {
    let source = FooEnum::Three {
        field1: "val".into(),
        field2: true,
    };
    let bar = BarEnum::from(source.clone());
    let mapped: FooEnum = bar.into();

    assert_eq!(source, mapped);
}
