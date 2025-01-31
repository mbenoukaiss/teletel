use crate::error::Error;
use crate::receiver::TeletelReceiver;
use crate::BaudRate;
use serial2::{CharSize, FlowControl, Parity, SerialPort, Settings, StopBits};

pub struct SerialReceiver {
    port: SerialPort,
    write_buffer: Vec<u8>,
}

impl SerialReceiver {
    pub fn new<S: AsRef<str>>(path: S, baud_rate: BaudRate) -> Result<Self, Error> {
        let mut port = SerialPort::open(path.as_ref(), |mut settings: Settings| {
            settings.set_raw();
            settings.set_baud_rate(baud_rate as u32)?;
            settings.set_char_size(CharSize::Bits7);
            settings.set_parity(Parity::Even);
            settings.set_stop_bits(StopBits::One);
            settings.set_flow_control(FlowControl::None);

            Ok(settings)
        })?;

        Ok(SerialReceiver {
            port,
            write_buffer: Vec::new(),
        })
    }
}

impl TeletelReceiver for SerialReceiver {
    #[inline(always)]
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        Ok(self.port.read(buffer)?)
    }

    #[inline(always)]
    fn send(&mut self, bytes: &[u8]) {
        self.write_buffer.extend_from_slice(bytes);
    }

    #[inline(always)]
    fn flush(&mut self) -> Result<(), Error> {
        self.port.write_all(&self.write_buffer)?;
        self.write_buffer.clear();

        Ok(())
    }
}
