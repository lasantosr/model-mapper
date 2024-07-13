use std::{borrow::Cow, error::Error, fmt};

/// [Error] while mapping between types
#[derive(Debug)]
pub struct MapperError(Cow<'static, str>);
impl MapperError {
    pub(crate) fn new(message: impl Into<Cow<'static, str>>) -> Self {
        Self(message.into())
    }

    pub(crate) fn from(error: impl fmt::Display) -> Self {
        Self::new(error.to_string())
    }
}

impl fmt::Display for MapperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for MapperError {}
