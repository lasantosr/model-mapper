use model_mapper::Mapper;

struct Target {
    x: i32,
}

#[derive(Mapper)]
#[mapper(into(custom), ty = "Target", add(field = "x"))]
struct Source {}

fn main() {}
