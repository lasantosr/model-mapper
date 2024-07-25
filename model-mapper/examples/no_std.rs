#![no_std]
#![allow(dead_code)]

use model_mapper::Mapper;

struct Foo {
    field1: i32,
    field2: bool,
}

#[derive(Mapper)]
#[mapper(from, into, ty = Foo)]
struct Bar {
    field1: i32,
    field2: bool,
}

fn main() {}
