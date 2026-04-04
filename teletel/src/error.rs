use std::io::Error as IoError;
use teletel_protocol::parser::Error as ParseError;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("failed to connect to Minitel")]
    ConnectionFailure,
    #[error("character {0} is not supported by the Minitel")]
    InvalidCharacter(char),
    #[error("unexpected sequence {0:X?}")]
    UnexpectedSequence(Vec<u8>),
    #[error("io error: {0}")]
    Io(#[from] IoError),
    #[error("read exact eof")]
    ReadExactEof,
    #[error("parse error: {0}")]
    Parse(#[from] ParseError),
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
}
