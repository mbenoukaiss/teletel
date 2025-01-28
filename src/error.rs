use std::fmt::Debug;
use std::io::Error as IoError;

#[cfg(feature = "serial")]
use serialport::Error as SerialError;

pub enum Error {
    Io(IoError),
    #[cfg(feature = "serial")]
    Serial(SerialError),
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(error) => write!(f, "IoError: {}", error),
            #[cfg(feature = "serial")]
            Error::Serial(error) => write!(f, "SerialError: {}", error),
        }
    }
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Error::Io(error)
    }
}

#[cfg(feature = "serial")]
impl From<SerialError> for Error {
    fn from(error: SerialError) -> Self {
        Error::Serial(error)
    }
}
