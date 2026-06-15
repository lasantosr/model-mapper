use model_mapper::Mapper;

struct Target {
    x: i32,
}

#[derive(Mapper)]
#[mapper(into(accumulate), ty = "Target")]
struct Source {
    x: i32,
}

fn main() {}
