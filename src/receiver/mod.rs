mod buffer;
mod file;

pub use file::FileReceiver;

#[cfg(feature = "serial")]
mod serial;

use crate::error::Error;
#[cfg(feature = "serial")]
pub use serial::{BaudRate, SerialReceiver};

pub trait TeletelReceiver {
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Error>;

    fn read_to_vec(&mut self) -> Result<Vec<u8>, Error> {
        let mut buffer = vec![0; 1024];
        let bytes_read = self.read(&mut buffer)?;

        let mut data = Vec::with_capacity(bytes_read);
        data.extend_from_slice(&buffer[..bytes_read]);

        Ok(data)
    }

    fn read_until_enter(&mut self) -> Result<Vec<u8>, Error> {
        let mut buffer = vec![0; 1024];
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

    fn send(&mut self, bytes: &[u8]);
    fn flush(&mut self) -> Result<(), Error> {
        Ok(())
    }
}
