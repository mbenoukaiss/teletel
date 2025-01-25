use crate::backend::Backend;
use serialport::{DataBits, Parity, SerialPort};
use std::time::Duration;

pub enum BaudRate {
    B300 = 300,
    B1200 = 1200,
    B4800 = 4800,
    B9600 = 9600,
}

pub struct SerialBackend {
    port: Box<dyn SerialPort>,
}

impl SerialBackend {
    pub fn new<S: AsRef<str>>(path: S, baud_rate: BaudRate) -> Self {
        SerialBackend {
            port: serialport::new(path.as_ref(), baud_rate as u32)
                .timeout(Duration::from_millis(10))
                .parity(Parity::Even)
                .data_bits(DataBits::Seven)
                .open()
                .expect("Failed to open port"),
        }
    }
}

impl Backend for SerialBackend {
    fn send(&mut self, bytes: &[u8]) {
        self.port.write(&bytes).expect("Write failed");
    }
}
