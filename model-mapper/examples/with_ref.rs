use std::time::Duration;

use model_mapper::Mapper;

pub struct Source {
    pub duration: Duration,
}

#[derive(Mapper)]
#[mapper(from, ty = Source)]
pub struct Destination {
    #[mapper(with = Duration::as_secs)]
    pub duration: u64,
}

fn main() {
    let source = Source {
        duration: Duration::from_secs(10),
    };
    let dest = Destination::from(source);
    assert_eq!(dest.duration, 10);
}
