use model_mapper::Mapper;

enum Target {
    A(i32),
}

#[derive(Mapper)]
#[mapper(from, ty = "Target")]
enum Source {
    A(#[mapper(into_with = "my_func")] i32),
}

fn main() {}
