use std::error::{Error as StdError};
use std::fmt::{Debug, Display};
use std::io::Error as IoError;
use teletel_protocol::parser::Error as ParseError;

pub enum Error {
    ConnectionFailure,
    InvalidCharacter(char),
    UnexpectedSequence(Vec<u8>),
    Io(IoError),
    ReadExactEof,
    Parse(ParseError)
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ConnectionFailure => write!(f, "Failed to connect to Minitel"),
            Error::InvalidCharacter(ch) => write!(f, "Character {} is not supported by the Minitel", ch),
            Error::UnexpectedSequence(seq) => write!(f, "Unexpected sequence {:X?}", seq),
            Error::Io(error) => write!(f, "IoError: {}", error),
            Error::ReadExactEof => write!(f, "ReadExactEof"),
            Error::Parse(error) => write!(f, "ParseError: {}", error)
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
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
