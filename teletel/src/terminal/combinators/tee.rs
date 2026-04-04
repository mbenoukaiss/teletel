use crate::terminal::{Context, Contextualized, ReadableTerminal, WriteableTerminal};
use crate::Error;

/// Writes to two terminals simultaneously.
///
/// Reads and context are delegated to the primary terminal.
/// Both terminals always receive every write, even if one fails.
pub struct Tee<P, S> {
    pub primary: P,
    pub secondary: S,
}

impl<P, S> Tee<P, S> {
    pub fn new(primary: P, secondary: S) -> Self {
        Self { primary, secondary }
    }
}

impl<P: WriteableTerminal, S: WriteableTerminal> WriteableTerminal for Tee<P, S> {
    fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        let r1 = self.primary.write(buf);
        let r2 = self.secondary.write(buf);
        r1.and(r2)
    }

    fn flush(&mut self) -> Result<(), Error> {
        let r1 = self.primary.flush();
        let r2 = self.secondary.flush();
        r1.and(r2)
    }
}

impl<P: ReadableTerminal, S> ReadableTerminal for Tee<P, S> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.primary.read(buf)
    }
}

impl<P: Contextualized, S> Contextualized for Tee<P, S> {
    fn ctx(&self) -> &Context {
        self.primary.ctx()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::RawBuffer;

    #[test]
    fn writes_to_both() {
        let mut tee = Tee::new(RawBuffer::new(), RawBuffer::new());
        tee.write(b"hello").unwrap();
        tee.flush().unwrap();

        assert_eq!(tee.primary.data(), b"hello");
        assert_eq!(tee.secondary.data(), b"hello");
    }
}
