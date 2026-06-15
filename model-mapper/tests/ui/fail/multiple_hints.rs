use model_mapper::Mapper;

struct Target {
    x: Vec<Option<i32>>,
}

#[derive(Mapper)]
#[mapper(from, ty = "Target")]
struct Source {
    #[mapper(opt, iter)]
    x: Vec<Option<i32>>,
}

fn main() {}
