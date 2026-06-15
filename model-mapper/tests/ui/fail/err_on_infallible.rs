use model_mapper::Mapper;

struct Target {
    x: i32,
}

#[derive(Mapper)]
#[mapper(from, ty = "Target")]
struct Source {
    #[mapper(err = "my_err")]
    x: i32,
}

fn main() {}
