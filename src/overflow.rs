use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    io,
};

#[derive(Clone, Copy, Debug)]
pub struct OverflowError {
    size: usize,
}

impl OverflowError {
    pub fn new(size: usize) -> Self {
        Self { size }
    }
}

impl Display for OverflowError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}-length int will overflow u64", self.size,)
    }
}

impl Error for OverflowError {}

impl From<OverflowError> for io::Error {
    fn from(err: OverflowError) -> Self {
        Self::new(io::ErrorKind::InvalidInput, err)
    }
}
