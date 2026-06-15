#![allow(unused, dead_code, clippy::restriction, reason = "example")]

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
    let struct_a = StructA {
        id: 1,
        name: "name".into(),
        tag: "tag".into(),
    };

    let struct_b = StructB::from(struct_a);
    let struct_c: StructC = struct_b.into();

    assert_eq!(
        StructC {
            id: 1,
            name: "name".into(),
        },
        struct_c,
        "Mapped struct does not match expected value"
    );
}
