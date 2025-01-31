mod buffer;
mod file;
#[cfg(feature = "serial")]
mod serial;

use crate::error::Error;

pub use buffer::Buffer;
pub use file::FileReceiver;
#[cfg(feature = "serial")]
pub use serial::SerialReceiver;

pub trait TeletelReceiver {
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Error>;

    fn send(&mut self, bytes: &[u8]);
    fn flush(&mut self) -> Result<(), Error> {
        Ok(())
    }
}
