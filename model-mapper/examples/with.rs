#![allow(dead_code, clippy::restriction, clippy::disallowed_names, reason = "example")]

use model_mapper::Mapper;

#[derive(Debug, PartialEq, Eq, Clone)]
struct Foo {
    field1: i32,
    field2: String,
    field3: i32,
}

#[derive(Mapper, Debug, PartialEq)]
#[mapper(from, ty = Foo)]
struct Bar {
    // We can use any expression using the fields of the other type
    #[mapper(with = field1 + 1)]
    field1: i32,
    // Or we can pass a function that takes a reference to the field
    #[mapper(with = String::len)]
    field2: usize,
    // Or one that takes ownership of the field
    #[mapper(with = i32::abs)]
    field3: i32,
}

#[derive(Mapper, Debug, PartialEq)]
#[mapper(from, into, ty = Foo)]
struct BarMultiple {
    // When deriving both from and into, we can specify different custom logic for each direction
    #[mapper(from_with = field1 + 1, into_with = field1 - 1)]
    field1: i32,
    field2: String,
    // And mix with a single expression for both ways as well
    #[mapper(with = 0 - field3)]
    field3: i32,
}

fn main() {
    let foo = Foo {
        field1: 1,
        field2: "hello".to_string(),
        field3: -10,
    };

    let bar = Bar::from(foo.clone());

    assert_eq!(bar.field1, 2);
    assert_eq!(bar.field2, 5);
    assert_eq!(bar.field3, 10);

    let multiple = BarMultiple::from(foo.clone());
    assert_eq!(multiple.field1, 2);
    assert_eq!(multiple.field2, "hello");
    assert_eq!(multiple.field3, 10);

    let back_to_foo: Foo = multiple.into();
    assert_eq!(back_to_foo, foo);
}
