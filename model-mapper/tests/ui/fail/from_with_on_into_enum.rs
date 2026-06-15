use model_mapper::Mapper;

enum Target {
    A(i32),
}

#[derive(Mapper)]
#[mapper(into, ty = "Target")]
enum Source {
    A(#[mapper(from_with = "my_func")] i32),
}

fn main() {}
