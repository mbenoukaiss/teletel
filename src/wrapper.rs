#![allow(dead_code)]

use crate::receiver::TeletelReceiver;
use crate::protocol::codes::*;
use crate::protocol::ToTeletel;

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

declare!(Clear, [CLEAR]);
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

pub struct Move(pub u8, pub u8);

impl ToTeletel for Move {
    fn to_teletel(&self, receiver: &mut dyn TeletelReceiver) {
        assert!(self.0 <= 40);
        assert!(self.1 <= 24);

        [SET_CURSOR, 0x40 + self.1, 0x40 + self.0].to_teletel(receiver);
    }
}
