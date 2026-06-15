use model_mapper::Mapper;

#[derive(Mapper)]
#[mapper(from, ty = "Target")]
struct Source;

struct Target;

fn main() {}
