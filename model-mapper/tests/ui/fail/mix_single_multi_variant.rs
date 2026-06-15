use model_mapper::Mapper;

struct TargetEnum;

#[derive(Mapper)]
#[mapper(derive(from, ty = "TargetEnum"))]
enum SourceEnum {
    #[mapper(rename = "Val", when(ty = "TargetEnum", rename = "Val"))]
    A,
}

fn main() {}
