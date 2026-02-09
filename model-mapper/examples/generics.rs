#![allow(dead_code)]

use model_mapper::Mapper;

mod same_generics {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Foo<T> {
        val: T,
    }

    #[derive(Debug, PartialEq, Mapper)]
    // Using the same name 'T' signals that these fields represent the same data.
    // The macro decouples them internally and adds 'Into' bounds so you can map between compatible types,
    // like Foo<i32> to Bar<i64>.
    #[mapper(into, from, ty = "Foo<T>")]
    struct Bar<T> {
        val: T,
    }
}

mod decoupled_generics {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct MultiFoo<T, Z> {
        field1: T,
        field2: i32,
        field3: Z,
    }

    #[derive(Debug, PartialEq, Mapper)]
    // We define the source generics (A, Z) to be used in field-level mapping.
    // If we used 'T' here, the macro would assume both T's represent the same data, which they don't.
    #[mapper(into, from, ty = "MultiFoo<A, Z>")]
    struct MultiBar<T, Z> {
        // We can explicitly map a source generic (A) into a concrete target type (i64)
        #[mapper(other_ty = A)]
        field1: i64,
        // Or map a concrete source type (i32) into a target generic (T)
        #[mapper(other_ty = i32)]
        field2: T,
        // Since 'Z' is shared by both, the macro still applies the "same data" logic,
        // allowing field3 to convert between different representations of Z.
        field3: Z,
    }
}

mod simple_enums {
    use super::*;

    #[derive(Debug, PartialEq)]
    enum EnumFoo<T> {
        Variant(T),
    }

    #[derive(Debug, PartialEq, Mapper)]
    #[mapper(from, ty = "EnumFoo<T>")]
    enum EnumBar<T> {
        Variant(T),
    }
}

mod complex_enums {
    use super::*;

    #[derive(Debug, PartialEq)]
    enum DataFoo<T, U> {
        Pair(T, U),
        Named { x: T, y: U },
        Unit,
    }

    #[derive(Debug, PartialEq, Mapper)]
    #[mapper(from, ty = "DataFoo<T, U>")]
    enum DataBar<A, B> {
        Pair(#[mapper(other_ty = T)] A, #[mapper(other_ty = U)] B),
        Named {
            #[mapper(other_ty = T)]
            x: A,
            #[mapper(other_ty = U)]
            y: B,
        },
        Unit,
    }
}

fn main() {}
