use std::io::{Result as IoResult, Write};
use crate::terminal::{Context, Contextualized, ToTerminal, WriteableTerminal};

pub struct Buffer {
    ctx: Context,
    inner: Vec<u8>,
}

impl Buffer {
    pub fn data(&self) -> &[u8] {
        &self.inner
    }
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer::default()
    }

    #[inline(always)]
    pub fn send(&mut self, data: impl ToTerminal) -> IoResult<usize> {
        data.to_terminal(self)
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Buffer {
            ctx: Context,
            inner: Vec::new(),
        }
    }
}

impl Contextualized for Buffer {
    fn ctx(&self) -> &Context {
        &self.ctx
    }
}

impl Write for Buffer {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.inner.write(buf)
    }

    #[inline(always)]
    fn flush(&mut self) -> IoResult<()> {
        self.inner.flush()
    }
}

impl WriteableTerminal for Buffer {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer() {
        let mut buffer = Buffer::new();

        buffer.write(&[0x00]).unwrap();
        buffer.write(&[0x02, 0x03]).unwrap();
        buffer.write(&[0x04, 0x05, 0x06]).unwrap();

        assert_eq!(buffer.data(), [0x00, 0x02, 0x03, 0x04, 0x05, 0x06]);
    }

    #[test]
    fn test_buffer_mut_reference() {
        let mut buffer = Buffer::new();
        let buffer_ref = &mut buffer;

        buffer_ref.write(&[0x00]).unwrap();
        buffer_ref.write(&[0x02, 0x03]).unwrap();
        buffer_ref.write(&[0x04, 0x05, 0x06]).unwrap();

        assert_eq!(buffer.data(), [0x00, 0x02, 0x03, 0x04, 0x05, 0x06]);
    }
}
