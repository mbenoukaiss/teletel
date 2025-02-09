use crate::specifications::codes::{ESC, PRO1, PRO2, PRO3, REQ_SPEED, RESET, RESP_SPEED, SEP, START, STOP};
use crate::terminal::{BaudRate, ReadableTerminal, WriteableTerminal};
use crate::Error;

macro_rules! count_args {
    ($item:pat) => { 1 };
    ($item:pat, $($rest:pat),*) => {
        1 + count_args!($($rest),*)
    }
}

macro_rules! expect_sequence {
    ($self:ident, [$($item:pat),+ $(,)?]) => {
        expect_sequence!($self, [$($item),+] => Ok(()))
    };
    ($self:ident, [$($item:pat),+ $(,)?] => $then:expr) => {{
        let mut buffer = vec![0; count_args!($($item),+)];
        $self.read_exact(&mut buffer)?;

        match buffer.as_slice() {
            [$($item),+] => $then,
            _ => Err(Error::UnexpectedSequence(buffer))
        }
    }};
}

pub trait SpeedAwareTerminal {
    fn match_connector_speed(&mut self) -> Result<(), Error>;
    fn set_connector_speed(&mut self, baud_rate: BaudRate) -> Result<(), Error>;
}

pub trait ProtocolExtension: ReadableTerminal + WriteableTerminal {
    fn reset(&mut self) -> Result<(), Error> {
        //p145
        self.discard()?;
        self.write_all(&[ESC, PRO1, RESET])?;
        self.flush()?;

        expect_sequence!(self, [SEP, 0x5E])
    }

    #[cfg(feature = "minitel2")]
    fn sleep(&mut self) -> Result<(), Error> {
        self.discard()?;
        self.write_all(&[ESC, PRO3, START, 0x58, 0x41])?;
        self.flush()?;

        expect_sequence!(self, [SEP, 0x72])
    }

    #[cfg(feature = "minitel2")]
    fn wake(&mut self) -> Result<(), Error> {
        self.discard()?;
        self.write_all(&[ESC, PRO3, STOP, 0x58, 0x41])?;
        self.flush()?;

        expect_sequence!(self, [SEP, 0x72])
    }

    fn get_connector_speed(&mut self) -> Result<BaudRate, Error> {
        self.discard()?;
        self.write_all(&[ESC, PRO1, REQ_SPEED])?;
        self.flush()?;

        expect_sequence!(self, [ESC, PRO2, RESP_SPEED, speed] => {
            BaudRate::try_from(*speed)
        })
    }
}

impl<T: ReadableTerminal + WriteableTerminal> ProtocolExtension for T {}

// à gérer en extension de protocole
// pub enum PageMode {
//     Page,
//     Scroll,
// }
//
// impl ToTerminal for PageMode {
//     fn to_terminal(&self, term: &mut dyn WriteableTerminal) -> IoResult<usize> {
//         match self {
//             PageMode::Page => term.write(&[ESC, PRO2, STOP, SCROLL]),
//             PageMode::Scroll => term.write(&[ESC, PRO2, START, SCROLL]),
//         }
//     }
// }

//uniquement en standard teleinformatique, pas encore assez bien compris et
//géré par la librairie pour être implémenté, porterait juste à confusion
//
// pub enum Echo {
//     Enable,
//     Disable,
// }
//
// impl ToTerminal for Echo {
//     fn to_terminal(&self, term: &mut crate::Minitel) {
//         match self {
//             Echo::Enable => term.send(&[ESC, CSI, 0x31, 0x32, 0x6C]),
//             Echo::Disable => term.send(&[ESC, CSI, 0x31, 0x32, 0x68]),
//         }
//     }
// }
//
// pub enum Standard {
//     Teleinformatique,
//     Teletel,
// }
//
// impl ToMinitel for Standard {
//     fn to_terminal(&self, term: &mut crate::Minitel) {
//         match self {
//             Standard::Teleinformatique => term.send(&[ESC, 0x3A, 0x31, 0x7D]),
//             Standard::Teletel => term.send(&[ESC, CSI, 0x3F, 0x7B]),
//         }
//     }
// }
//
// pub enum Mode {
//     Mixte,
//     Videotex,
// }
//
// impl ToMinitel for Mode {
//     fn to_terminal(&self, term: &mut crate::Minitel) {
//         match self {
//             Mode::Mixte => term.send(&[ESC, 0x3A, 0x32, 0x7D]),
//             Mode::Videotex => term.send(&[ESC, 0x3A, 0x32, 0x7E]),
//         }
//     }
// }
//
// pub enum Columns {
//     Forty,
//     Eighty,
// }
//
// impl ToMinitel for Columns {
//     fn to_terminal(&self, term: &mut crate::Minitel) {
//         match self {
//             Columns::Forty => term.send(&[ESC, CSI, 0x3C, 0x33, 0x68]),
//             Columns::Eighty => term.send(&[ESC, CSI, 0x3F, 0x33, 0x6C]),
//         }
//     }
// }
