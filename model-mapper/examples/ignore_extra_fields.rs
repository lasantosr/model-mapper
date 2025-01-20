#![allow(dead_code)]

use model_mapper::Mapper;

mod ignore_extra_into {
    use super::*;

    #[derive(Default)]
    struct Foo {
        field1: String,
        field2: i64,
        field3: Option<i32>,
        field4: Option<String>,
    }

    /// When deriving 'into' for structs the [Default] trait is required on the target type,
    /// in order to populate missing fields
    #[derive(Mapper)]
    #[mapper(into, ty = Foo, ignore_extra)]
    struct Bar {
        field1: String,
        field2: i64,
    }

    /// But it's not required for enums
    enum FooEnum {
        One,
        Two,
        Three,
        Four,
    }

    #[derive(Mapper)]
    #[mapper(into, ty = FooEnum, ignore_extra)]
    enum BarEnum {
        One,
        Two,
    }
}

mod ignore_extra_from {
    use super::*;

    struct Foo {
        field1: String,
        field2: i64,
        field3: Option<i32>,
        field4: Option<String>,
    }

    /// When deriving 'from' for structs the [Default] trait is not required on the target type
    #[derive(Mapper)]
    #[mapper(from, ty = Foo, ignore_extra)]
    struct Bar {
        field1: String,
        field2: i64,
    }

    enum FooEnum {
        One,
        Two,
        Three,
        Four,
    }

    /// But it's required on the source enum, in order to route missing variants
    #[derive(Mapper, Default)]
    #[mapper(from, ty = FooEnum, ignore_extra)]
    enum BarEnum {
        #[default]
        One,
        Two,
    }
}

mod ignore_extra_explicit {
    use super::*;

    struct Foo {
        field1: String,
        field2: i64,
        field3: Option<i32>,
        field4: Option<String>,
    }

    /// The [Default] is not required for the target struct if the additional fields are explicitly set.
    /// For example if [Foo] resides in other crate and we can't modify it
    #[derive(Mapper)]
    #[mapper(
        into, from,
        ty = Foo,
        // By providing an explicit value
        add(field = field3, default(value = Some(1))),
        // Or explicitly setting to the default (if the field's type implements it)
        add(field = field4, default)
    )]
    struct Bar {
        field1: String,
        field2: i64,
    }

    enum FooEnum {
        One,
        Two,
        Three,
        Four,
    }

    /// Neither on the enum if we explicitly provide defaults.
    /// Allowing us to provide a different default value for each additional variant
    #[derive(Mapper)]
    #[mapper(
        from,
        ty = FooEnum,
        add(field = Three, default(value = BarEnum::One)),
        add(field = Four, default(value = BarEnum::Two))
    )]
    enum BarEnum {
        One,
        Two,
    }
}

fn main() {}
