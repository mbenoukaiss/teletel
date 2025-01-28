#![allow(dead_code)]

use crate::protocol::codes::*;
use crate::protocol::ToTeletel;
use crate::receiver::TeletelReceiver;

macro_rules! declare {
    ($name:ident $(<$($lt:tt$(:$clt:tt$(+$dlt:tt)*)?),+>)? $(($($vis:vis $ty:ty),*))?, [$($code:expr),+ $(,)?]) => {
        declare!($name $(<$($lt$(:$clt$(+$dlt)*)?),+>)? $(($($vis $ty),*))?, |self| [$($code),+]);
    };
    ($name:ident $(<$($lt:tt$(:$clt:tt$(+$dlt:tt)*)?),+>)? $(($($vis:vis $ty:ty),*))?, |$self:ident| [$($code:expr),+ $(,)?]) => {
        #[derive(Eq, PartialEq, Copy, Clone, Debug)]
        pub struct $name $(<$($lt),+>)? $(($($vis $ty),*))?;

        impl $(<$($lt$(:$clt$(+$dlt)*)?),+>)? ToTeletel for $name $(<$($lt),+>)? {
            #[inline(always)]
            fn to_teletel(&$self, receiver: &mut dyn TeletelReceiver) {
                $($code.to_teletel(receiver);)+
            }
        }
    };
}

declare!(Clear, [FF]);
declare!(ClearRow, [CSI_2_K]);
declare!(Beep, [BEEP]);
declare!(Blink<T: ToTeletel>(pub T), |self| [ESC, BLINK, self.0, ESC, STILL]);
declare!(Background<T: ToTeletel>(pub Color, pub T), |self| [ESC, 0x50, self.0 as u8, self.1, ESC, 0x50, Color::Black as u8]);
declare!(Foreground<T: ToTeletel>(pub Color, pub T), |self| [ESC, 0x40 + self.0 as u8, self.1, ESC, 0x40 + Color::White as u8]);
declare!(SemiGraphic<T: ToTeletel>(pub T), |self| [SO, self.0, SI]);
declare!(Inverted<T: ToTeletel>(pub T), |self| [ESC, START_INVERT, self.0, ESC, STOP_INVERT]);
declare!(Big<T: ToTeletel>(pub T), |self| [ESC, DOUBLE_SIZE, self.0, ESC, NORMAL_SIZE]);

#[repr(u8)]
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Color {
    Black = 0x0,
    Blue = 0x4,
    Red = 0x1,
    Magenta = 0x5,
    Green = 0x2,
    Cyan = 0x6,
    Yellow = 0x3,
    White = 0x7,
}

pub struct Repeat<T: ToTeletel>(pub T, pub u8);

impl ToTeletel for Repeat<char> {
    fn to_teletel(&self, receiver: &mut dyn TeletelReceiver) {
        repeat(self.0 as u8, self.1).to_teletel(receiver);
    }
}

impl ToTeletel for Repeat<u8> {
    fn to_teletel(&self, receiver: &mut dyn TeletelReceiver) {
        repeat(self.0, self.1).to_teletel(receiver);
    }
}


pub struct SetCursor(pub u8, pub u8);

impl ToTeletel for SetCursor {
    fn to_teletel(&self, receiver: &mut dyn TeletelReceiver) {
        assert!(self.0 <= 40);
        assert!(self.1 <= 24);

        //documented on page 95
        [ESC, 0x5B].to_teletel(receiver);
        to_decimal(self.1).to_teletel(receiver);
        0x3B.to_teletel(receiver);
        to_decimal(self.0).to_teletel(receiver);
        0x48.to_teletel(receiver);
    }
}

pub struct Videotex {
    pub data: Vec<u8>,
}

impl Videotex {
    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn from_path(path: &str) -> std::io::Result<Self> {
        Ok(Self {
            data: std::fs::read(path)?,
        })
    }
}

impl ToTeletel for Videotex {
    fn to_teletel(&self, receiver: &mut dyn TeletelReceiver) {
        self.data.to_teletel(receiver);
    }
}

#[repr(u8)]
pub enum Direction {
    Up,
    Down,
    Right,
    Left,
}

pub struct MoveCursor(pub Direction, pub u8);

impl ToTeletel for MoveCursor {
    fn to_teletel(&self, receiver: &mut dyn TeletelReceiver) {
        if self.1 == 1 {
            match self.0 {
                Direction::Up => CURSOR_UP,
                Direction::Down => CURSOR_DOWN,
                Direction::Right => CURSOR_RIGHT,
                Direction::Left => CURSOR_LEFT,
            }
            .to_teletel(receiver);
        } else {
            let direction = match self.0 {
                Direction::Up => 0x41,
                Direction::Down => 0x42,
                Direction::Right => 0x43,
                Direction::Left => 0x44,
            };

            [ESC, 0x5B].to_teletel(receiver);
            to_decimal(self.1).to_teletel(receiver);
            direction.to_teletel(receiver);
        }
    }
}
