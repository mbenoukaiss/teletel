use crate::terminal::{Context, Contextualized, ReadableTerminal, WriteableTerminal};
use crate::Error;

/// Writes to a terminal while logging bytes to a side-channel.
///
/// Unlike [`super::Tee`], the logger's errors are silently ignored so it
/// never disrupts the primary terminal.
pub struct Tap<T, L> {
    pub inner: T,
    pub logger: L,
}

impl<T, L> Tap<T, L> {
    pub fn new(inner: T, logger: L) -> Self {
        Self { inner, logger }
    }
}

impl<T: WriteableTerminal, L: WriteableTerminal> WriteableTerminal for Tap<T, L> {
    fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        let _ = self.logger.write(buf);
        self.inner.write(buf)
    }

    fn flush(&mut self) -> Result<(), Error> {
        let _ = self.logger.flush();
        self.inner.flush()
    }
}

impl<T: ReadableTerminal, L> ReadableTerminal for Tap<T, L> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.inner.read(buf)
    }
}

impl<T: Contextualized, L> Contextualized for Tap<T, L> {
    fn ctx(&self) -> &Context {
        self.inner.ctx()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::RawBuffer;

    #[test]
    fn writes_to_both() {
        let mut tap = Tap::new(RawBuffer::new(), RawBuffer::new());
        tap.write(b"hello").unwrap();
        tap.flush().unwrap();

        assert_eq!(tap.inner.data(), b"hello");
        assert_eq!(tap.logger.data(), b"hello");
    }
}
