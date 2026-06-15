use model_mapper::Mapper;

struct Target;

#[derive(Mapper)]
#[mapper(derive(from, ty = "Target"))]
struct Source {
    #[mapper(rename = "y", when(ty = "Target", rename = "y"))]
    x: i32,
}

fn main() {}
