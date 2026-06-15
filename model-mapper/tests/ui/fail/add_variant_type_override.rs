use model_mapper::Mapper;

enum Target {
    A,
}

#[derive(Mapper)]
#[mapper(from, ty = "Target", add(field = "A", ty = "i32"))]
enum Source {}

fn main() {}
