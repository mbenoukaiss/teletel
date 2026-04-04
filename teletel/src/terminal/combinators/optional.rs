use crate::terminal::{ReadableTerminal, WriteableTerminal};
use crate::Error;

/// A terminal that may or may not be connected.
///
/// All operations are silently ignored when the inner terminal is absent.
/// Construct with [`Optional::new`] which takes a `Result` — if the
/// connection failed, the terminal becomes a no-op instead of propagating
/// the error.
///
/// ```ignore
/// // Never fails — if the emulator isn't running, writes are just dropped
/// let emu = Optional::new(TcpTerminal::emulator());
/// let mut term = Tee::new(serial, emu);
/// ```
pub struct Optional<T> {
    inner: Option<T>,
}

impl<T> Optional<T> {
    pub fn new(result: Result<T, impl Into<Error>>) -> Self {
        Self {
            inner: result.ok(),
        }
    }

    pub fn is_connected(&self) -> bool {
        self.inner.is_some()
    }
}

impl<T: WriteableTerminal> WriteableTerminal for Optional<T> {
    fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        if let Some(inner) = &mut self.inner {
            if inner.write(buf).is_err() {
                self.inner = None;
            }
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Error> {
        if let Some(inner) = &mut self.inner {
            if inner.flush().is_err() {
                self.inner = None;
            }
        }
        Ok(())
    }
}

impl<T: ReadableTerminal> ReadableTerminal for Optional<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        if let Some(inner) = &mut self.inner {
            match inner.read(buf) {
                Ok(n) => return Ok(n),
                Err(_) => self.inner = None,
            }
        }
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::RawBuffer;

    #[test]
    fn connected_writes() {
        let mut opt = Optional::new(Ok::<_, Error>(RawBuffer::new()));
        assert!(opt.is_connected());
        opt.write(b"hello").unwrap();
        assert_eq!(opt.inner.as_ref().unwrap().data(), b"hello");
    }

    #[test]
    fn disconnected_is_noop() {
        let mut opt = Optional::new(Err::<RawBuffer, _>(Error::ConnectionFailure));
        assert!(!opt.is_connected());
        opt.write(b"hello").unwrap();
        opt.flush().unwrap();
    }
}
