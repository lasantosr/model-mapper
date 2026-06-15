use model_mapper::Mapper;

enum Target {
    A,
}

#[derive(Mapper)]
#[mapper(into(accumulate), ty = "Target")]
enum Source {
    A,
}

fn main() {}
