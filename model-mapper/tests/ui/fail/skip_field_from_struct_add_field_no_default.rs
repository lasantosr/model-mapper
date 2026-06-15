use model_mapper::Mapper;

struct Target {
    x: i32,
}

#[derive(Mapper)]
#[mapper(from, ty = "Target", add(field = "x"))]
struct Source {
    x: i32,
}

fn main() {}
