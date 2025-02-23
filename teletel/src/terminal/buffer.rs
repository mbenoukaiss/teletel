use std::io::Write;
use teletel_protocol::parser::{DisplayComponent, Parser};
use crate::Error;
use crate::terminal::{Context, Contextualized, ToTerminal, WriteableTerminal};

pub struct Buffer {
    inner: Vec<u8>,
    parser: Parser,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer::default()
    }

    pub fn data(&self) -> &[u8] {
        &self.inner
    }

    #[inline(always)]
    pub fn send(&mut self, data: impl ToTerminal) -> Result<(), Error> {
        data.to_terminal(self)
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Buffer {
            inner: Vec::new(),
            parser: Parser::new(DisplayComponent::VGP5),
        }
    }
}

impl Contextualized for Buffer {
    fn ctx(&self) -> &Context {
        self.parser.ctx()
    }
}

impl WriteableTerminal for Buffer {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        for byte in buf {
            self.parser.consume(*byte)?;
            self.inner.push(*byte);
        }

        Ok(())
    }

    #[inline(always)]
    fn flush(&mut self) -> Result<(), Error> {
        self.inner.flush().map_err(Into::into)
    }
}

//temporary buffer without parser mainly for tests
//while there is not a better design
#[derive(Default)]
pub struct RawBuffer {
    inner: Vec<u8>,
}

impl RawBuffer {
    pub fn new() -> RawBuffer {
        RawBuffer::default()
    }

    pub fn data(&self) -> &[u8] {
        &self.inner
    }

    #[inline(always)]
    pub fn send(&mut self, data: impl ToTerminal) -> Result<(), Error> {
        data.to_terminal(self)
    }
}

impl WriteableTerminal for RawBuffer {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        self.inner.extend(buf);
        Ok(())
    }

    #[inline(always)]
    fn flush(&mut self) -> Result<(), Error> {
        self.inner.flush().map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer() {
        let mut buffer = RawBuffer::new();

        buffer.write(&[0x00]).unwrap();
        buffer.write(&[0x02, 0x03]).unwrap();
        buffer.write(&[0x04, 0x05, 0x06]).unwrap();

        assert_eq!(buffer.data(), [0x00, 0x02, 0x03, 0x04, 0x05, 0x06]);
    }

    #[test]
    fn test_buffer_mut_reference() {
        let mut buffer = RawBuffer::new();
        let buffer_ref = &mut buffer;

        buffer_ref.write(&[0x00]).unwrap();
        buffer_ref.write(&[0x02, 0x03]).unwrap();
        buffer_ref.write(&[0x04, 0x05, 0x06]).unwrap();

        assert_eq!(buffer.data(), [0x00, 0x02, 0x03, 0x04, 0x05, 0x06]);
    }
}
