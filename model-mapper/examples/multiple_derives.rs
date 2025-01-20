#![allow(dead_code)]

use model_mapper::Mapper;

struct StructA {
    id: i64,
    name: String,
    tag: String,
}

// We can include multiple derives for different structs
#[derive(Mapper)]
#[mapper(derive(from, ty = StructA))]
#[mapper(derive(into, ty = StructC))]
struct StructB {
    id: i64,
    name: String,
    #[mapper(when(ty = StructC, skip))] // skip only for StructC
    tag: String,
    #[mapper(skip(default))] // skip for both derives
    dummy: bool,
}

#[derive(Debug, PartialEq, Eq)]
struct StructC {
    id: i64,
    name: String,
}

fn main() {
    let a = StructA {
        id: 1,
        name: "name".into(),
        tag: "tag".into(),
    };

    let b = StructB::from(a);
    let c: StructC = b.into();

    assert_eq!(
        StructC {
            id: 1,
            name: "name".into(),
        },
        c
    );
}
