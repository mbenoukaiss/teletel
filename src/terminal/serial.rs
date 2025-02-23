use crate::error::Error;
use crate::protocol::{ProtocolExtension, SpeedAwareTerminal};
use crate::specifications::codes::{FF, PRO2, PROG};
use crate::terminal::{BaudRate, Contextualized, ReadableTerminal, ToTerminal, WriteableTerminal};
use serial2::{CharSize, FlowControl, Parity, SerialPort, Settings, StopBits};
use std::io::ErrorKind;
use std::time::Duration;
use crate::parser::{Context, DisplayComponent, Parser};

pub struct SerialTerminal {
    path: String,
    baud_rate: BaudRate,
    port: SerialPort,
    parser: Parser,
}

impl SerialTerminal {
    pub fn new<S: AsRef<str>>(path: S, baud_rate: Option<BaudRate>) -> Result<Self, Error> {
        let default_baud_rate = if let Some(baud_rate) = baud_rate {
            baud_rate
        } else {
            BaudRate::B1200
        };

        let mut term = SerialTerminal {
            path: path.as_ref().to_owned(),
            baud_rate: default_baud_rate,
            port: SerialTerminal::connect(path, default_baud_rate)?,
            parser: Parser::new(DisplayComponent::VGP5),
        };

        if baud_rate.is_none() {
            term.match_connector_speed()?;
        }

        Ok(term)
    }

    fn connect<S: AsRef<str>>(path: S, baud_rate: BaudRate) -> Result<SerialPort, Error> {
        let mut port = SerialPort::open(path.as_ref(), |mut settings: Settings| {
            settings.set_raw();
            settings.set_baud_rate(baud_rate as u32)?;
            settings.set_char_size(CharSize::Bits7);
            settings.set_parity(Parity::Even);
            settings.set_stop_bits(StopBits::One);
            settings.set_flow_control(FlowControl::None);

            Ok(settings)
        })?;

        //arbitrary values selected by testing, may not work on all setups
        //todo: allow manually setting it ?
        port.set_read_timeout(Duration::from_millis(match baud_rate {
            BaudRate::B300 => 500,
            BaudRate::B1200 => 180,
            BaudRate::B4800 => 48,
            #[cfg(feature = "minitel2")]
            BaudRate::B9600 => 24,
        }))?;

        Ok(port)
    }

    #[inline(always)]
    fn send(&mut self, data: impl ToTerminal) -> Result<(), Error> {
        data.to_terminal(self)
    }
}

impl ReadableTerminal for SerialTerminal {
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        match self.port.read(buffer) {
            Ok(bytes_read) => Ok(bytes_read),
            Err(ref e) if e.kind() == ErrorKind::TimedOut => Ok(0),
            Err(e) => Err(Error::Io(e)),
        }
    }
}

impl WriteableTerminal for SerialTerminal {
    fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        for i in 0..buf.len() {
            self.parser.consume(buf[i])?;
            self.port.write(&buf[i..i+1])?;
        }

        Ok(())
    }

    #[inline(always)]
    fn flush(&mut self) -> Result<(), Error> {
        self.port.flush().map_err(Into::into)
    }
}

impl Contextualized for SerialTerminal {
    fn ctx(&self) -> &Context {
        self.parser.ctx()
    }
}

impl SpeedAwareTerminal for SerialTerminal {
    fn match_connector_speed(&mut self) -> Result<(), Error> {
        let speeds = [
            BaudRate::B1200,
            BaudRate::B4800,
            BaudRate::B300,
            #[cfg(feature = "minitel2")]
            BaudRate::B9600,
        ];

        let mut i = 0;
        loop {
            let baud_rate = speeds[i % speeds.len()];
            i += 1;

            if self.port.get_configuration()?.get_baud_rate()? != baud_rate as u32 {
                self.port = SerialTerminal::connect(&self.path, baud_rate)?;
            }

            if matches!(self.get_connector_speed(), Ok(claim) if claim == baud_rate) {
                #[cfg(feature = "minitel2")]
                self.set_connector_speed(BaudRate::B9600)?;

                #[cfg(not(feature = "minitel2"))]
                self.set_connector_speed(BaudRate::B4800)?;

                match self.get_connector_speed() {
                    Ok(speed) if speed == self.baud_rate => return Ok(()),
                    _ => continue,
                }
            }
        }
    }

    fn set_connector_speed(&mut self, baud_rate: BaudRate) -> Result<(), Error> {
        self.read_to_vec()?;
        self.write(&[PRO2, PROG])?;
        baud_rate.to_terminal(self)?;
        self.flush()?;

        //arbitrary value but required, maybe less works
        std::thread::sleep(Duration::from_secs(1));
        self.port = SerialTerminal::connect(&self.path, baud_rate)?;
        self.baud_rate = baud_rate;

        self.reset()?;
        self.send(FF)?;
        self.flush()?;

        Ok(())
    }
}
