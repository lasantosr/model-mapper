use model_mapper::Mapper;

struct Target;

#[derive(Mapper)]
#[mapper(derive(from, ty = "Target"))]
struct Source {
    #[mapper(when(ty = "OtherTarget", rename = "x"))]
    x: i32,
}

fn main() {}
