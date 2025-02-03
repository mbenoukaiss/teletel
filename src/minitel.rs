use crate::receiver::{FileReceiver, TeletelReceiver};
use crate::Error;

#[cfg(feature = "serial")]
use crate::receiver::SerialReceiver;

pub enum BaudRate {
    B300 = 300,
    B1200 = 1200,
    B4800 = 4800,
    B9600 = 9600,
}

pub struct Minitel<'a> {
    receiver: Box<dyn TeletelReceiver + 'a>,
}

impl Minitel<'_> {
    pub fn buffer() -> Self {
        Self {
            receiver: Box::new(Vec::new()),
        }
    }

    pub fn file(path: &str) -> Result<Self, Error> {
        Ok(Self {
            receiver: Box::new(FileReceiver::new(path)?),
        })
    }

    #[cfg(feature = "serial")]
    pub fn serial<S: AsRef<str>>(path: S, baud_rate: BaudRate) -> Result<Self, Error> {
        Ok(Self {
            receiver: Box::new(SerialReceiver::new(path, baud_rate)?),
        })
    }

    #[inline(always)]
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        self.receiver.read(buffer)
    }

    pub fn read_to_vec(&mut self) -> Result<Vec<u8>, Error> {
        let mut buffer = vec![0; 10];
        let bytes_read = self.read(&mut buffer)?;

        let mut data = Vec::with_capacity(bytes_read);
        data.extend_from_slice(&buffer[..bytes_read]);

        Ok(data)
    }

    pub fn read_until_enter(&mut self) -> Result<Vec<u8>, Error> {
        let mut buffer = vec![0; 10];
        let mut data = Vec::new();

        loop {
            let bytes_read = match self.read(&mut buffer) {
                Ok(bytes_read) => bytes_read,
                Err(Error::Io(ref e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => 0,
                Err(e) => return Err(e),
            };

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

    #[inline(always)]
    pub fn send(&mut self, bytes: &[u8]) {
        self.receiver.send(bytes);
    }

    #[inline(always)]
    pub fn flush(&mut self) -> Result<(), Error> {
        self.receiver.flush()
    }

    #[inline(always)]
    pub fn send_all(&mut self, bytes: &[u8]) -> Result<(), Error> {
        self.send(bytes);
        self.flush()?;

        Ok(())
    }
}

impl<'a, T: TeletelReceiver + 'a> From<T> for Minitel<'a> {
    fn from(value: T) -> Self {
        Self {
            receiver: Box::new(value),
        }
    }
}

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub struct Position {
    pub x: u8,
    pub y: u8,
}

impl Position {
    pub fn new(x: u8, y: u8) -> Self {
        Self { x, y }
    }
}

#[cfg(test)]
mod tests {
    use std::cmp;
    use std::time::Duration;
    use super::*;

    struct MockReceiver {
        buffer: Vec<u8>,
    }

    impl From<Vec<u8>> for MockReceiver {
        fn from(value: Vec<u8>) -> Self {
            Self {
                buffer: value,
            }
        }
    }

    impl TeletelReceiver for MockReceiver {
        fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
            let bytes_to_read = cmp::min(buffer.len(), self.buffer.len());
            let read_bytes = self.buffer.drain(..bytes_to_read).collect::<Vec<u8>>();

            buffer[..bytes_to_read].copy_from_slice(&read_bytes);

            Ok(bytes_to_read)
        }

        fn send(&mut self, _bytes: &[u8]) {
            panic!("MockReceiver does not support sending");
        }
    }

    #[test]
    fn test_read_to_vec() {
        let buffer = MockReceiver::from(vec![]);
        let mut mt = Minitel::from(buffer);

        let data = mt.read_to_vec().unwrap();
        assert_eq!(data, []);

        let buffer = MockReceiver::from(vec![0x01]);
        let mut mt = Minitel::from(buffer);

        let data = mt.read_to_vec().unwrap();
        assert_eq!(data, [0x01]);

        let buffer = MockReceiver::from(vec![0x01, 0x02, 0x03, 0x04, 0x05]);
        let mut mt = Minitel::from(buffer);

        let data = mt.read_to_vec().unwrap();
        assert_eq!(data, [0x01, 0x02, 0x03, 0x04, 0x05]);
    }

    #[test]
    fn test_read_until_enter() {
        assert_times_out!(Duration::from_millis(10), || {
            let buffer = MockReceiver::from(vec![]);
            let mut mt = Minitel::from(buffer);

            mt.read_until_enter().unwrap();
        });

        assert_times_out!(Duration::from_millis(10), || {
            let buffer = MockReceiver::from(vec![0x01, 0x02, 0x03, 0x04, 0x05]);
            let mut mt = Minitel::from(buffer);

            mt.read_until_enter().unwrap();
        });

        let buffer = MockReceiver::from(vec![0x01, 0x02, 0x03, b'\r', 0x04, 0x05]);
        let mut mt = Minitel::from(buffer);

        let data = mt.read_until_enter().unwrap();
        assert_eq!(data, [0x01, 0x02, 0x03]);
    }
}
