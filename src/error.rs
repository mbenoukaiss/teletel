use std::fmt::Debug;
use std::io::Error as IoError;

pub enum Error {
    Io(IoError),
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(error) => write!(f, "IoError: {}", error),
        }
    }
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Error::Io(error)
    }
}
