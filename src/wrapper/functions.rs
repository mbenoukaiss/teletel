#![allow(dead_code)]

use crate::protocol::codes::*;
use crate::protocol::ToMinitel;
use crate::Minitel;

macro_rules! declare {
    ($name:ident $(<$($lt:tt$(:$clt:tt$(+$dlt:tt)*)?),+>)? $(($($vis:vis $ty:ty),*))?, [$($code:expr),+ $(,)?]) => {
        declare!($name $(<$($lt$(:$clt$(+$dlt)*)?),+>)? $(($($vis $ty),*))?, |self| [$($code),+]);
    };
    ($name:ident $(<$($lt:tt$(:$clt:tt$(+$dlt:tt)*)?),+>)? $(($($vis:vis $ty:ty),*))?, |$self:ident| [$($code:expr),+ $(,)?]) => {
        #[derive(Eq, PartialEq, Copy, Clone, Debug)]
        pub struct $name $(<$($lt$(:$clt$(+$dlt)*)?),+>)? $(($($vis $ty),*))?;

        impl $(<$($lt$(:$clt$(+$dlt)*)?),+>)? ToMinitel for $name $(<$($lt),+>)? {
            #[inline(always)]
            fn to_minitel(&$self, mt: &mut Minitel) {
                $($code.to_minitel(mt);)+
            }
        }
    };
}

declare!(Clear, [FF]);
declare!(ClearRow, [CSI_2_K]);
declare!(Beep, [BEEP]);
declare!(Blink<T: ToMinitel>(pub T), |self| [ESC, BLINK, self.0, ESC, STILL]);
declare!(Background<T: ToMinitel>(pub Color, pub T), |self| [ESC, 0x50, self.0 as u8, self.1, ESC, 0x50, Color::Black as u8]);
declare!(Foreground<T: ToMinitel>(pub Color, pub T), |self| [ESC, 0x40 + self.0 as u8, self.1, ESC, 0x40 + Color::White as u8]);
declare!(Inverted<T: ToMinitel>(pub T), |self| [ESC, START_INVERT, self.0, ESC, STOP_INVERT]);
declare!(Big<T: ToMinitel>(pub T), |self| [ESC, DOUBLE_SIZE, self.0, ESC, NORMAL_SIZE]);
declare!(SemiGraphic<T: ToMinitel>(pub T), |self| [SO, self.0, SI]);

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

pub struct Repeat<T: ToMinitel>(pub T, pub u8);

impl ToMinitel for Repeat<char> {
    fn to_minitel(&self, mt: &mut Minitel) {
        repeat(self.0 as u8, self.1).to_minitel(mt);
    }
}

impl ToMinitel for Repeat<u8> {
    fn to_minitel(&self, mt: &mut Minitel) {
        repeat(self.0, self.1).to_minitel(mt);
    }
}

pub struct SetCursor(pub u8, pub u8);

impl ToMinitel for SetCursor {
    fn to_minitel(&self, mt: &mut Minitel) {
        assert!(self.0 <= 40);
        assert!(self.1 <= 24);

        //documented on page 95
        [ESC, 0x5B].to_minitel(mt);
        to_decimal(self.1).to_minitel(mt);
        0x3B.to_minitel(mt);
        to_decimal(self.0).to_minitel(mt);
        0x48.to_minitel(mt);
    }
}

pub struct Videotex<B: AsRef<[u8]> + ToMinitel> {
    pub data: B,
}

impl<B: AsRef<[u8]> + ToMinitel> Videotex<B> {
    pub fn new(data: B) -> Self {
        Self { data }
    }
}

impl Videotex<Vec<u8>> {
    pub fn from_path(path: &str) -> std::io::Result<Self> {
        Ok(Self {
            data: std::fs::read(path)?,
        })
    }
}

impl<B: AsRef<[u8]> + ToMinitel> ToMinitel for Videotex<B> {
    #[inline(always)]
    fn to_minitel(&self, mt: &mut Minitel) {
        self.data.to_minitel(mt);
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

impl ToMinitel for MoveCursor {
    fn to_minitel(&self, mt: &mut Minitel) {
        if self.1 == 1 {
            match self.0 {
                Direction::Up => CURSOR_UP,
                Direction::Down => CURSOR_DOWN,
                Direction::Right => CURSOR_RIGHT,
                Direction::Left => CURSOR_LEFT,
            }
            .to_minitel(mt);
        } else {
            let direction = match self.0 {
                Direction::Up => 0x41,
                Direction::Down => 0x42,
                Direction::Right => 0x43,
                Direction::Left => 0x44,
            };

            [ESC, 0x5B].to_minitel(mt);
            to_decimal(self.1).to_minitel(mt);
            direction.to_minitel(mt);
        }
    }
}

impl<F: Fn(&mut Minitel)> ToMinitel for F {
    #[inline(always)]
    fn to_minitel(&self, mt: &mut Minitel) {
        self(mt);
    }
}
