use model_mapper::Mapper;

struct Target {
    x: i32,
}

#[derive(Mapper)]
#[mapper(try_from, ty = "Target")]
struct Source {
    #[mapper(err = "|e| MyError")]
    x: i32,
}

fn main() {}
