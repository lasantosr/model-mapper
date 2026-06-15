use model_mapper::Mapper;

struct Target {
    x: i32,
}

#[derive(Mapper)]
#[mapper(try_from(err = "String"), ty = "Target")]
struct Source {
    #[mapper(err = "my_err", err_with = "my_err_fn")]
    x: i32,
}

fn main() {}
