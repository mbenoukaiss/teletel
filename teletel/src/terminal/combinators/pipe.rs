use crate::terminal::{ReadableTerminal, WriteableTerminal};
use crate::Error;

/// Reads from `source` and writes every byte to `sink` in a loop.
///
/// Returns when the source returns an error (e.g. connection closed).
/// Flushes the sink after each read batch.
pub fn pipe(
    source: &mut dyn ReadableTerminal,
    sink: &mut dyn WriteableTerminal,
) -> Result<(), Error> {
    let mut buf = [0u8; 256];
    loop {
        let n = source.read(&mut buf)?;
        if n > 0 {
            sink.write(&buf[..n])?;
            sink.flush()?;
        }
    }
}

/// Two-way relay: reads from each side and writes to the other.
///
/// Both terminals should use non-blocking reads (returning `Ok(0)` on
/// timeout) so that both directions are polled in the same loop.
/// Returns when either side returns an error.
pub fn bidirectional_pipe(
    a: &mut (impl ReadableTerminal + WriteableTerminal),
    b: &mut (impl ReadableTerminal + WriteableTerminal),
) -> Result<(), Error> {
    let mut buf = [0u8; 256];
    loop {
        let n = a.read(&mut buf)?;
        if n > 0 {
            b.write(&buf[..n])?;
            b.flush()?;
        }

        let n = b.read(&mut buf)?;
        if n > 0 {
            a.write(&buf[..n])?;
            a.flush()?;
        }
    }
}
