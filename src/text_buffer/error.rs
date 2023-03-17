use crate::position::Point;
use std::io;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("{0}")]
    OutOfBounds(Point),
    #[error("Buffer file not set")]
    FileNotSet,
}
