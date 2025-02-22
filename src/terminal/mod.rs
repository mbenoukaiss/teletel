mod buffer;
mod file;
#[cfg(feature = "serial")]
mod serial;
mod to_terminal;

use std::io::{Read, Result as IoResult, Write};
use crate::Error;
use crate::specifications::codes::{B1200, B300, B4800, ESC, PRO2, PROG};
#[cfg(feature = "minitel2")]
use crate::specifications::codes::B9600;

pub use to_terminal::ToTerminal;
pub use buffer::{Buffer, RawBuffer};
pub use file::FileReceiver;
#[cfg(feature = "serial")]
pub use serial::SerialTerminal;
pub use crate::parser::Context;

pub trait Contextualized {
    fn ctx(&self) -> &Context;
}

pub trait ReadableTerminal: Read {
    fn discard(&mut self) -> IoResult<()> {
        //todo: stop using this function, put the current buffer
        // aside and just read the protocol sequences we need
        self.read_to_vec()?;

        Ok(())
    }

    fn read_to_vec(&mut self) -> IoResult<Vec<u8>> {
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

    fn read_until_enter(&mut self) -> IoResult<Vec<u8>> {
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

pub trait WriteableTerminal: Write {}

#[derive(Eq, PartialEq, Copy, Clone)]
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
    fn to_terminal(&self, term: &mut dyn WriteableTerminal) -> IoResult<usize> {
        term.write(&[ESC, PRO2, PROG, match self {
            BaudRate::B300 => B300,
            BaudRate::B1200 => B1200,
            BaudRate::B4800 => B4800,
            #[cfg(feature = "minitel2")]
            BaudRate::B9600 => B9600,
        }])
    }
}
