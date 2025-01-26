use std::io::Write;
use crate::receiver::TeletelReceiver;
use serialport::{DataBits, Parity, SerialPort};
use std::time::Duration;

pub enum BaudRate {
    B300 = 300,
    B1200 = 1200,
    B4800 = 4800,
    B9600 = 9600,
}

pub struct SerialReceiver {
    port: Box<dyn SerialPort>,
}

impl SerialReceiver {
    pub fn new<S: AsRef<str>>(path: S, baud_rate: BaudRate) -> Self {
        SerialReceiver {
            port: serialport::new(path.as_ref(), baud_rate as u32)
                .timeout(Duration::from_millis(10))
                .parity(Parity::Even)
                .data_bits(DataBits::Seven)
                .open()
                .expect("Failed to open port"),
        }
    }
}

impl TeletelReceiver for SerialReceiver {
    fn send(&mut self, bytes: &[u8]) {
        self.port.write(&bytes).expect("Write failed");
    }
}
