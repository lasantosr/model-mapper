use model_mapper::Mapper;

struct Target {
    x: i32,
}

#[derive(Mapper)]
#[mapper(into, ty = "Target")]
struct Source {
    #[mapper(from_with = "my_func")]
    x: i32,
}

fn main() {}
