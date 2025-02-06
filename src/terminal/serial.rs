use std::borrow::Borrow;
use serial2::{CharSize, FlowControl, Parity, SerialPort, Settings, StopBits};
use std::io::{Result as IoResult, Read, Write};
use crate::error::Error;
use crate::terminal::{BaudRate, Context, Contextualized, ReadableTerminal, ToTerminal, WriteableTerminal};

pub struct SerialTerminal {
    ctx: Context,
    port: SerialPort,
}

impl SerialTerminal {
    pub fn new<S: AsRef<str>>(path: S, baud_rate: BaudRate) -> Result<Self, Error> {
        let port = SerialPort::open(path.as_ref(), |mut settings: Settings| {
            settings.set_raw();
            settings.set_baud_rate(baud_rate as u32)?;
            settings.set_char_size(CharSize::Bits7);
            settings.set_parity(Parity::Even);
            settings.set_stop_bits(StopBits::One);
            settings.set_flow_control(FlowControl::None);

            Ok(settings)
        })?;

        Ok(SerialTerminal {
            ctx: Context,
            port,
        })
    }

    #[inline(always)]
    pub fn send<T: ToTerminal, R: Borrow<T>>(&mut self, data: R) -> IoResult<usize> {
        data.borrow().to_terminal(self)
    }
}

impl Read for SerialTerminal {
    #[inline(always)]
    fn read(&mut self, buffer: &mut [u8]) -> IoResult<usize> {
        self.port.read(buffer)
    }
}

impl Write for SerialTerminal {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.port.write(buf)
    }

    #[inline(always)]
    fn flush(&mut self) -> IoResult<()> {
        self.port.flush()
    }
}

impl Contextualized for SerialTerminal {
    fn ctx(&self) -> &Context {
        &self.ctx
    }
}

impl ReadableTerminal for SerialTerminal {}

impl WriteableTerminal for SerialTerminal {}
