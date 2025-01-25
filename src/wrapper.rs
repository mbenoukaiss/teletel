#![allow(dead_code)]

use crate::backend::Backend;
use crate::protocol::codes::*;
use crate::protocol::ToBackend;

macro_rules! declare {
    ($name:ident $(<$($lt:tt$(:$clt:tt$(+$dlt:tt)*)?),+>)? $(($($vis:vis $ty:ty),*))?, [$($code:expr),+ $(,)?]) => {
        declare!($name $(<$($lt$(:$clt$(+$dlt)*)?),+>)? $(($($vis $ty),*))?, |self| [$($code),+]);
    };
    ($name:ident $(<$($lt:tt$(:$clt:tt$(+$dlt:tt)*)?),+>)? $(($($vis:vis $ty:ty),*))?, |$self:ident| [$($code:expr),+ $(,)?]) => {
        #[derive(Eq, PartialEq, Copy, Clone, Debug)]
        pub struct $name $(<$($lt),+>)? $(($($vis $ty),*))?;

        impl $(<$($lt$(:$clt$(+$dlt)*)?),+>)? ToBackend for $name $(<$($lt),+>)? {
            #[inline(always)]
            fn to_backend(&$self, backend: &mut dyn Backend) {
                $($code.to_backend(backend);)+
            }
        }
    };
}

declare!(Clear, [CLEAR]);
declare!(Beep, [BEEP]);
declare!(Blink<T: ToBackend>(pub T), |self| [ESC, BLINK, self.0, ESC, STILL]);
declare!(Background<T: ToBackend>(pub Color, pub T), |self| [ESC, 0x50, self.0 as u8, self.1, ESC, 0x50, Color::Black as u8]);
declare!(Foreground<T: ToBackend>(pub Color, pub T), |self| [ESC, 0x40 + self.0 as u8, self.1, ESC, 0x40 + Color::White as u8]);

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

pub struct Repeat<T: ToBackend>(pub T, pub u8);

impl ToBackend for Repeat<char> {
    fn to_backend(&self, backend: &mut dyn Backend) {
        repeat(self.0 as u8, self.1).to_backend(backend);
    }
}

impl ToBackend for Repeat<u8> {
    fn to_backend(&self, backend: &mut dyn Backend) {
        repeat(self.0, self.1).to_backend(backend);
    }
}

pub struct Move(pub u8, pub u8);

impl ToBackend for Move {
    fn to_backend(&self, backend: &mut dyn Backend) {
        assert!(self.0 <= 40);
        assert!(self.1 <= 24);

        [SET_CURSOR, 0x40 + self.1, 0x40 + self.0].to_backend(backend);
    }
}
