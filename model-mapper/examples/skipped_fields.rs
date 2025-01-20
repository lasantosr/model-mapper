#![allow(dead_code)]

use model_mapper::Mapper;

#[derive(Clone)]
struct Foo {
    field1: String,
    field2: i64,
}

// Skipped fields doesn't affect 'into' derives, they're just ignored
#[derive(Debug, PartialEq, Eq, Mapper)]
#[mapper(into, ty = Foo)]
struct Bar1 {
    field1: String,
    field2: i64,
    #[mapper(skip)]
    field3: i64,
    #[mapper(skip)]
    field4: Option<String>,
}

// But for 'from' derives we need a way of populating its value
#[derive(Debug, PartialEq, Eq, Mapper)]
#[mapper(from, ty = Foo)]
struct Bar2 {
    field1: String,
    field2: i64,
    // Using other values
    #[mapper(skip(default(value = field2 / 2)))]
    field3: i64,
    // Or just with the default, None in this case
    #[mapper(skip(default))]
    field4: Option<String>,
}

// We can also implement a custom function instead, that will require the additional fields at runtime:
#[derive(Debug, PartialEq, Eq, Mapper)]
#[mapper(from(custom = "from_foo_custom"), ty = Foo)]
struct Bar3 {
    field1: String,
    field2: i64,
    #[mapper(skip)]
    field3: i64,
    #[mapper(skip)]
    field4: Option<String>,
}
// impl Bar3 {
//     pub fn from_foo_custom(from: Foo, field3: i64, field4: Option<String>) -> Self {
//         let Foo { field1, field2 } = from;
//         Self { field1, field2, field3, field4 }
//     }
// }

// Or even mix both options
#[derive(Debug, PartialEq, Eq, Mapper)]
#[mapper(from(custom), ty = Foo)]
struct Bar4 {
    field1: String,
    field2: i64,
    #[mapper(skip(default(value = field2 / 2)))]
    field3: i64,
    #[mapper(skip)]
    field4: Option<String>,
}
// impl Bar4 {
//     pub fn from_foo(from: Foo, field4: Option<String>) -> Self {
//         let Foo { field1, field2 } = from;
//         Self {
//             field1,
//             field2,
//             field3: field2 / 2,
//             field4,
//         }
//     }
// }

#[derive(Default)]
enum FooEnum {
    #[default]
    One,
    Two,
}

// For enums, the 'from' derive doesn't require anything but the 'into' requires to provide default values or to
// implement the default trait
#[derive(Mapper)]
#[mapper(into, ty = FooEnum)]
enum BarEnum {
    One,
    Two,
    // We can provide a value and avoid the need of the default trait
    #[mapper(skip(default(value = FooEnum::Two)))]
    Three,
    // Or rely on the default
    #[mapper(skip(default))]
    Four,
}
// impl From<BarEnum> for FooEnum {
//     fn from(other: BarEnum) -> Self {
//         match other {
//             BarEnum::One => FooEnum::One,
//             BarEnum::Two => FooEnum::Two,
//             BarEnum::Three => FooEnum::Two,
//             BarEnum::Four => FooEnum::default(),
//         }
//     }
// }

fn main() {
    let source = Foo {
        field1: "val".into(),
        field2: 2,
    };

    let mapped = Bar2::from(source.clone());
    assert_eq!(
        Bar2 {
            field1: "val".into(),
            field2: 2,
            field3: 1,
            field4: None,
        },
        mapped
    );

    let mapped = Bar3::from_foo_custom(source.clone(), 1, None);
    assert_eq!(
        Bar3 {
            field1: "val".into(),
            field2: 2,
            field3: 1,
            field4: None,
        },
        mapped
    );

    let mapped = Bar4::from_foo(source, None);
    assert_eq!(
        Bar4 {
            field1: "val".into(),
            field2: 2,
            field3: 1,
            field4: None,
        },
        mapped
    );
}
