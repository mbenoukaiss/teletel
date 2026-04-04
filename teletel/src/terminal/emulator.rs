use crate::terminal::{Context, Contextualized, ReadableTerminal, ToTerminal, WriteableTerminal};
use crate::Error;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;
use teletel_protocol::parser::{DisplayComponent, Parser};

const EMULATOR_PORT: u16 = 3615;

/// A terminal that connects to the minitel emulator over TCP.
pub struct EmulatorTerminal {
    stream: TcpStream,
    parser: Parser,
}

impl EmulatorTerminal {
    /// Connect to the emulator on `127.0.0.1:3615`.
    pub fn connect() -> Result<Self, Error> {
        let stream = TcpStream::connect(("127.0.0.1", EMULATOR_PORT)).map_err(|e| {
            Error::Io(std::io::Error::new(
                e.kind(),
                format!("could not connect to emulator on port {EMULATOR_PORT}: {e} (is the emulator running?)"),
            ))
        })?;

        Ok(Self {
            stream,
            parser: Parser::new(DisplayComponent::VGP5),
        })
    }

    #[inline(always)]
    pub fn send(&mut self, data: impl ToTerminal) -> Result<(), Error> {
        data.to_terminal(self)
    }
}

impl ReadableTerminal for EmulatorTerminal {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        match self.stream.read(buf) {
            Ok(bytes_read) => Ok(bytes_read),
            Err(ref e) if e.kind() == ErrorKind::TimedOut => Ok(0),
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => Ok(0),
            Err(e) => Err(Error::Io(e)),
        }
    }
}

impl WriteableTerminal for EmulatorTerminal {
    fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        for i in 0..buf.len() {
            self.parser.consume(buf[i])?;
            self.stream.write_all(&buf[i..i + 1])?;
        }

        Ok(())
    }

    #[inline(always)]
    fn flush(&mut self) -> Result<(), Error> {
        self.stream.flush().map_err(Into::into)
    }
}

impl Contextualized for EmulatorTerminal {
    fn ctx(&self) -> &Context {
        self.parser.ctx()
    }
}
