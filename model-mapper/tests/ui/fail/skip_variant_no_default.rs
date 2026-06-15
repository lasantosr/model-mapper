use model_mapper::Mapper;

enum Target {
    A,
}

#[derive(Mapper)]
#[mapper(into, ty = "Target")]
enum Source {
    #[mapper(skip)]
    A,
}

fn main() {}
