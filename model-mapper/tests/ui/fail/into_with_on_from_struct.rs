use model_mapper::Mapper;

struct Target {
    x: i32,
}

#[derive(Mapper)]
#[mapper(from, ty = "Target")]
struct Source {
    #[mapper(into_with = "my_func")]
    x: i32,
}

fn main() {}
