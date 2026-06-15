use model_mapper::Mapper;

struct Target;

#[derive(Mapper)]
#[mapper(derive(from, ty = "Target"), derive(from, ty = "Target"))]
struct Source {
    x: i32,
}

fn main() {}
