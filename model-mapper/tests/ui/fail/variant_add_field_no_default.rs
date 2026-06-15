use model_mapper::Mapper;

enum Target {
    A { x: i32 },
}

#[derive(Mapper)]
#[mapper(into, ty = "Target")]
enum Source {
    #[mapper(add(field = "x"))]
    A,
}

fn main() {}
