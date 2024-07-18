#![allow(dead_code)]

use model_mapper::Mapper;

mod ignore_extra_into {
    use super::*;

    /// The [Default] trait is required to be implemented when deriving into other struct
    #[derive(Default)]
    struct Foo {
        field1: String,
        field2: i64,
        field3: Option<i32>,
        field4: Option<String>,
    }

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

    /// The [Default] is not required when deriving from a struct
    struct Foo {
        field1: String,
        field2: i64,
        field3: Option<i32>,
        field4: Option<String>,
    }

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

    /// But it's required to be present on the enum when deriving from it
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

    /// The [Default] is not required if the additional fields are explicitly set.
    /// For example if [Foo] resides in other crate and we can't modify it
    struct Foo {
        field1: String,
        field2: i64,
        field3: Option<i32>,
        field4: Option<String>,
    }

    #[derive(Mapper)]
    #[mapper(
        into, from,
        ty = Foo,
        add(field = field3, default),
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
