use std::error::{Error as StdError};
use std::fmt::{Debug, Display};
use std::io::Error as IoError;
use teletel_protocol::parser::Error as ParseError;

#[derive(Debug)]
pub enum Error {
    ConnectionFailure,
    UnexpectedSequence(Vec<u8>),
    Io(IoError),
    ReadExactEof,
    Parse(ParseError)
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ConnectionFailure => write!(f, "Failed to connect to Minitel"),
            Error::UnexpectedSequence(seq) => write!(f, "Unexpected sequence {:X?}", seq),
            Error::Io(error) => write!(f, "IoError: {}", error),
            Error::ReadExactEof => write!(f, "ReadExactEof"),
            Error::Parse(error) => write!(f, "ParseError: {}", error)
        }
    }
}

impl StdError for Error {}

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Error::Io(error)
    }
}

impl From<ParseError> for Error {
    fn from(error: ParseError) -> Self {
        Error::Parse(error)
    }
}
