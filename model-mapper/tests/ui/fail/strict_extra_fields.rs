use model_mapper::Mapper;

struct Target {
    a: i32,
}

#[derive(Mapper)]
#[mapper(into, ty = "Target")]
struct Source {
    a: i32,
    b: i32,
}

fn main() {}
