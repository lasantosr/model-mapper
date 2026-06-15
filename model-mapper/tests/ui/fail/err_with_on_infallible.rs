use model_mapper::Mapper;

struct Target {
    x: i32,
}

#[derive(Mapper)]
#[mapper(from, ty = "Target")]
struct Source {
    #[mapper(err_with = "my_err_fn")]
    x: i32,
}

fn main() {}
