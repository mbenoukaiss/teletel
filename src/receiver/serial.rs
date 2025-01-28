use std::io::Write;
use crate::receiver::TeletelReceiver;
use serialport::{DataBits, Parity, SerialPort};
use std::time::Duration;
use crate::error::Error;

pub enum BaudRate {
    B300 = 300,
    B1200 = 1200,
    B4800 = 4800,
    B9600 = 9600,
}

pub struct SerialReceiver {
    port: Box<dyn SerialPort>,
    buffer: Vec<u8>,
}

impl SerialReceiver {
    pub fn new<S: AsRef<str>>(path: S, baud_rate: BaudRate) -> Result<Self, Error> {
        Ok(SerialReceiver {
            port: serialport::new(path.as_ref(), baud_rate as u32)
                .timeout(Duration::from_secs(1)) //TODO: correct value?
                .parity(Parity::Even)
                .data_bits(DataBits::Seven)
                .open()?,
            buffer: Vec::new(),
        })
    }
}

impl TeletelReceiver for SerialReceiver {
    fn send(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    fn flush(&mut self) -> Result<(), Error> {
        self.port.write_all(&self.buffer)?;
        self.buffer.clear();

        Ok(())
    }
}
