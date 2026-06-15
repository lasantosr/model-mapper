use model_mapper::Mapper;

#[derive(Mapper)]
#[mapper(ty = "Target", derive(from, ty = "Target"))]
struct Source {
    x: i32,
}

struct Target {
    x: i32,
}

fn main() {}
