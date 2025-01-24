#![allow(dead_code)]

use crate::backend::Backend;

macro_rules! declare {
    ($name:ident, [$($code:expr),+ $(,)?]) => {
        declare!($name(), $esc, |self| [$($code),+]);
    };
    ($name:ident, |$self:ident| [$($code:expr),+ $(,)?]) => {
        declare!($name(), $esc, |$self| [$($code),+]);
    };
    ($name:ident ($($vis:vis $ty:ty),*), [$($code:expr),+ $(,)?]) => {
        declare!($name(), $esc, |self| [$($code),+]);
    };
    ($name:ident $(<$($lt:tt$(:$clt:tt$(+$dlt:tt)*)?),+>)? ($($vis:vis $ty:ty),*), |$self:ident| [$($code:expr),+ $(,)?]) => {
        #[derive(Eq, PartialEq, Copy, Clone, Debug)]
        pub struct $name $(<$($lt),+>)? ($($vis $ty),*);

        impl $(<$($lt$(:$clt$(+$dlt)*)?),+>)? ToBackend for $name $(<$($lt),+>)? {
            #[inline(always)]
            fn to_backend(&$self, backend: &mut dyn Backend) {
                $($code.to_backend(backend);)+
            }
        }
    };
}

declare!(Blink<T: ToBackend>(pub T), |self| [ESC, BLINK, self.0, ESC, STILL]);
declare!(Background(pub Color), |self| [self.0 as u8 + 0x50]);
declare!(Foreground(pub Color), |self| [self.0 as u8 + 0x40]);

#[repr(u8)]
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Color {
    Black = 0x0,
    Red = 0x1,
    Green = 0x2,
    Yellow = 0x3,
    Blue = 0x4,
    Magenta = 0x5,
    Cyan = 0x6,
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
