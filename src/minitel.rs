use crate::receiver::{Buffer, FileReceiver, TeletelReceiver};
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
    cursor: Position,
}

impl Minitel<'_> {
    pub fn buffer() -> Self {
        Self {
            receiver: Box::new(Vec::new()),
            cursor: Position::new(0, 1),
        }
    }

    pub fn file(path: &str) -> Result<Self, Error> {
        Ok(Self {
            receiver: Box::new(FileReceiver::new(path)?),
            cursor: Position::new(0, 1),
        })
    }

    #[cfg(feature = "serial")]
    pub fn serial<S: AsRef<str>>(path: S, baud_rate: BaudRate) -> Result<Self, Error> {
        Ok(Self {
            receiver: Box::new(SerialReceiver::new(path, baud_rate)?),
            cursor: Position::new(0, 1),
        })
    }

    pub fn cursor_position(&self) -> &Position {
        &self.cursor
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

impl<'a> From<&'a mut Buffer> for Minitel<'a> {
    fn from(value: &'a mut Buffer) -> Self {
        Self {
            receiver: Box::new(value),
            cursor: Position::new(0, 1),
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
