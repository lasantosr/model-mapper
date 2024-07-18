#![allow(dead_code)]

use model_mapper::Mapper;

#[derive(Debug, PartialEq, Eq)]
struct Foo {
    field1: String,
    field2: i64,
    field3: i64,
    field4: Option<String>,
}

// Additional fields doesn't affect from derives, they're just ignored
#[derive(Debug, PartialEq, Eq, Mapper)]
#[mapper(from, ty = Foo, add(field = field3), add(field = field4))]
struct Bar1 {
    field1: String,
    field2: i64,
}

// For into derives
#[derive(Mapper)]
#[mapper(
    into,
    ty = Foo,
    // We can populate fields using other values
    add(field = field3, default(value = field2 / 2)),
    // Or just with the default, None in this case
    add(field = field4, default)
)]
struct Bar2 {
    field1: String,
    field2: i64,
}

#[derive(Mapper)]
#[mapper(
    // We can also implement a custom function that will require the additional fields at runtime, providing types
    into(custom = "custom_into_foo"), 
    ty = Foo,
    add(field = field3, ty = i64),
    add(field = field4, ty = "Option<String>"), // we might need to quote types
)]
struct Bar3 {
    field1: String,
    field2: i64,
}

#[derive(Mapper)]
#[mapper(
    // Or even mix both options
    into(custom),
    ty = Foo,
    add(field = field3, default(value = field2 / 2)),
    add(field = field4, ty = "Option<String>"),
)]
struct Bar4 {
    field1: String,
    field2: i64,
}

enum FooEnum {
    One,
    Two,
    Three,
    Four,
}

// For enums, the into doesn't require anything but the from requires to provide default values or to implement the
// default trait
#[derive(Default, Mapper)]
#[mapper(
    from,
    ty = FooEnum,
    // We can populate fields using some value
    add(field = Three, default(value = BarEnum::Two)),
    // Or just with the default, One in this case
    add(field = Four, default)
)]
enum BarEnum {
    #[default]
    One,
    Two,
}

fn main() {
    let expected = Foo {
        field1: "val".into(),
        field2: 2,
        field3: 1,
        field4: None,
    };

    let bar = Bar2 {
        field1: "val".into(),
        field2: 2,
    };
    let mapped = bar.into();
    assert_eq!(expected, mapped);

    let bar = Bar3 {
        field1: "val".into(),
        field2: 2,
    };
    let mapped = bar.custom_into_foo(1, None);
    assert_eq!(expected, mapped);

    let bar = Bar4 {
        field1: "val".into(),
        field2: 2,
    };
    let mapped = bar.into_foo(None);
    assert_eq!(expected, mapped);
}
