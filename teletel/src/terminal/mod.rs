mod buffer;
mod file;
#[cfg(feature = "serial")]
mod serial;
mod to_terminal;

use std::io::ErrorKind;
use crate::Error;
use teletel_protocol::codes::*;

pub use to_terminal::ToTerminal;
pub use buffer::{Buffer, RawBuffer};
pub use file::FileReceiver;
#[cfg(feature = "serial")]
pub use serial::SerialTerminal;
pub use teletel_protocol::parser::Context;

pub trait Contextualized {
    fn ctx(&self) -> &Context;
}

pub trait ReadableTerminal {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error>;

    fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<(), Error> {
        while !buf.is_empty() {
            match self.read(buf) {
                Ok(0) => break,
                Ok(n) => buf = &mut buf[n..],
                Err(Error::Io(ref e)) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }

        if buf.is_empty() {
            Ok(())
        } else {
            Err(Error::ReadExactEof)
        }
    }

    fn discard(&mut self) ->  Result<(), Error> {
        //todo: stop using this function, put the current buffer
        // aside and just read the protocol sequences we need
        self.read_to_vec()?;

        Ok(())
    }

    fn read_to_vec(&mut self) -> Result<Vec<u8>, Error> {
        let mut data = Vec::new();

        let mut buffer = vec![0; 10];
        while let Ok(bytes_read) = self.read(&mut buffer) {
            if bytes_read == 0 {
                return Ok(data);
            }

            data.extend_from_slice(&buffer[..bytes_read]);
        }

        Ok(data)
    }

    fn read_until_enter(&mut self) -> Result<Vec<u8>, Error> {
        let mut buffer = vec![0; 10];
        let mut data = Vec::new();

        loop {
            let bytes_read = self.read(&mut buffer)?;
            if bytes_read == 0 {
                continue;
            }

            if let Some(pos) = buffer.iter().position(|&b| b == b'\r') {
                data.extend_from_slice(&buffer[..pos]);
                break;
            }

            data.extend_from_slice(&buffer[..bytes_read]);
        }

        Ok(data)
    }
}

pub trait WriteableTerminal {
    fn write(&mut self, buf: &[u8]) -> Result<(), Error>;
    fn flush(&mut self) -> Result<(), Error>;
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
#[repr(u16)]
pub enum BaudRate {
    B300 = 300,
    B1200 = 1200,
    B4800 = 4800,
    #[cfg(feature = "minitel2")]
    B9600 = 9600,
}

impl TryFrom<u8> for BaudRate {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            B300 => Ok(BaudRate::B300),
            B1200 => Ok(BaudRate::B1200),
            B4800 => Ok(BaudRate::B4800),
            #[cfg(feature = "minitel2")]
            B9600 => Ok(BaudRate::B9600),
            _ => Err(Error::UnexpectedSequence(vec![value])),
        }
    }
}

impl ToTerminal for BaudRate {
    #[inline(always)]
    fn to_terminal(&self, term: &mut dyn WriteableTerminal) -> Result<(), Error> {
        term.write(&[ESC, PRO2, PROG, match self {
            BaudRate::B300 => B300,
            BaudRate::B1200 => B1200,
            BaudRate::B4800 => B4800,
            #[cfg(feature = "minitel2")]
            BaudRate::B9600 => B9600,
        }])
    }
}
