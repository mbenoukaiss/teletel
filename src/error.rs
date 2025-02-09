use std::error::{Error as StdError};
use std::fmt::{Debug, Display};
use std::io::Error as IoError;

#[derive(Debug)]
pub enum Error {
    ConnectionFailure,
    UnexpectedSequence(Vec<u8>),
    Io(IoError),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ConnectionFailure => write!(f, "Failed to connect to Minitel"),
            Error::UnexpectedSequence(seq) => write!(f, "Unexpected sequence {:X?}", seq),
            Error::Io(error) => write!(f, "IoError: {}", error),
        }
    }
}

impl StdError for Error {}

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Error::Io(error)
    }
}
