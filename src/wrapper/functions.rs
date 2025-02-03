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
declare!(Background<T: ToMinitel>(pub Color, pub T), |self| [ESC, BACKGROUND + self.0 as u8, self.1, ESC, BACKGROUND + Color::Black as u8]);
declare!(Foreground<T: ToMinitel>(pub Color, pub T), |self| [ESC, FOREGROUND + self.0 as u8, self.1, ESC, FOREGROUND + Color::White as u8]);
declare!(Inverted<T: ToMinitel>(pub T), |self| [ESC, START_INVERT, self.0, ESC, STOP_INVERT]);
declare!(Big<T: ToMinitel>(pub T), |self| [ESC, DOUBLE_SIZE, self.0, ESC, NORMAL_SIZE]);
declare!(SemiGraphic<T: ToMinitel>(pub T), |self| [SO, self.0, SI]);

#[repr(u8)]
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
#[cfg(feature = "colors")]
pub enum Color {
    Black = BLACK,
    Blue = BLUE,
    Red = RED,
    Magenta = MAGENTA,
    Green = GREEN,
    Cyan = CYAN,
    Yellow = YELLOW,
    White = WHITE,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
#[cfg(not(feature = "colors"))]
pub enum Color {
    Black = BLACK,
    Gray40 = BLUE,
    Gray50 = RED,
    Gray60 = MAGENTA,
    Gray70 = GREEN,
    Gray80 = CYAN,
    Gray90 = YELLOW,
    White = WHITE,
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

pub struct MoveCursor(pub Direction, pub u8);

impl ToMinitel for MoveCursor {
    fn to_minitel(&self, mt: &mut Minitel) {
        if self.1 == 1 {
            match self.0 {
                Direction::Up => CURSOR_UP,
                Direction::Down => CURSOR_DOWN,
                Direction::Right => CURSOR_RIGHT,
                Direction::Left => CURSOR_LEFT,
            }.to_minitel(mt);
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

#[cfg(test)]
mod tests {
    use std::{env, fs};
    use super::*;
    use teletel_derive::sg;

    #[test]
    fn test_clear() {
        let mut data = Vec::new();
        Clear.to_minitel(&mut Minitel::from(&mut data));
        assert_eq!(data, [0x0C]);
    }

    #[test]
    fn test_clear_row() {
        let mut data = Vec::new();
        ClearRow.to_minitel(&mut Minitel::from(&mut data));
        assert_eq!(data, [0x1B, 0x5B, 0x32, 0x4B]);
    }

    #[test]
    fn test_beep() {
        let mut data = Vec::new();
        Beep.to_minitel(&mut Minitel::from(&mut data));
        assert_eq!(data, [0x07]);
    }

    #[test]
    fn test_blink() {
        let mut data = Vec::new();
        Blink('A').to_minitel(&mut Minitel::from(&mut data));
        assert_eq!(data, [0x1B, 0x48, 'A' as u8, 0x1B, 0x49]);

        let mut data = Vec::new();
        Blink("bonjour").to_minitel(&mut Minitel::from(&mut data));
        assert_eq!(data, [0x1B, 0x48, 'b' as u8, 'o' as u8, 'n' as u8, 'j' as u8, 'o' as u8, 'u' as u8, 'r' as u8, 0x1B, 0x49]);
    }

    #[test]
    fn test_background() {
        let mut data = Vec::new();
        Background(Color::Gray60, 'A').to_minitel(&mut Minitel::from(&mut data));
        assert_eq!(data, [0x1B, 0x55, 'A' as u8, 0x1B, 0x50]);

        let mut data = Vec::new();
        Background(Color::Gray90, "bonjour").to_minitel(&mut Minitel::from(&mut data));
        assert_eq!(data, [0x1B, 0x53, 'b' as u8, 'o' as u8, 'n' as u8, 'j' as u8, 'o' as u8, 'u' as u8, 'r' as u8, 0x1B, 0x50]);
    }

    #[test]
    fn test_foreground() {
        let mut data = Vec::new();
        Foreground(Color::Gray70, 'A').to_minitel(&mut Minitel::from(&mut data));
        assert_eq!(data, [0x1B, 0x42, 'A' as u8, 0x1B, 0x47]);

        let mut data = Vec::new();
        Foreground(Color::Gray40, "bonjour").to_minitel(&mut Minitel::from(&mut data));
        assert_eq!(data, [0x1B, 0x44, 'b' as u8, 'o' as u8, 'n' as u8, 'j' as u8, 'o' as u8, 'u' as u8, 'r' as u8, 0x1B, 0x47]);
    }

    #[test]
    fn test_inverted() {
        let mut data = Vec::new();
        Inverted('A').to_minitel(&mut Minitel::from(&mut data));
        assert_eq!(data, [0x1B, 0x5D, 'A' as u8, 0x1B, 0x5C]);

        let mut data = Vec::new();
        Inverted("bonjour").to_minitel(&mut Minitel::from(&mut data));
        assert_eq!(data, [0x1B, 0x5D, 'b' as u8, 'o' as u8, 'n' as u8, 'j' as u8, 'o' as u8, 'u' as u8, 'r' as u8, 0x1B, 0x5C]);
    }

    #[test]
    fn test_big() {
        let mut data = Vec::new();
        Big('A').to_minitel(&mut Minitel::from(&mut data));
        assert_eq!(data, [0x1B, 0x4F, 'A' as u8, 0x1B, 0x4C]);

        let mut data = Vec::new();
        Big("bonjour").to_minitel(&mut Minitel::from(&mut data));
        assert_eq!(data, [0x1B, 0x4F, 'b' as u8, 'o' as u8, 'n' as u8, 'j' as u8, 'o' as u8, 'u' as u8, 'r' as u8, 0x1B, 0x4C]);
    }

    #[test]
    fn test_semigraphic() {
        let mut data = Vec::new();
        SemiGraphic(list![
            sg!(00/00/00),
            sg!(00/01/11),
            sg!(01/11/01),
            sg!(11/11/11),
        ]).to_minitel(&mut Minitel::from(&mut data));

        assert_eq!(data, [0x0E, 0x20, 0x78, 0x6E, 0x5F, 0x0F]);
    }

    #[test]
    fn test_repeat() {
        let mut data = Vec::new();
        Repeat('A', 3).to_minitel(&mut Minitel::from(&mut data));

        assert_eq!(data, ['A' as u8, 0x12, 0x42]);

        let mut data = Vec::new();
        Repeat(sg!(01/11/01), 3).to_minitel(&mut Minitel::from(&mut data));

        assert_eq!(data, [0x6E, 0x12, 0x42]);
    }

    #[test]
    fn test_repeat_fails() {
        assert_panics!(|| {
            let mut data = Vec::new();
            Repeat('A', 0).to_minitel(&mut Minitel::from(&mut data));
        });

        assert_panics!(|| {
            let mut data = Vec::new();
            Repeat('A', 65).to_minitel(&mut Minitel::from(&mut data));
        });
    }

    #[test]
    fn test_set_cursor() {
        let mut data = Vec::new();
        SetCursor(0, 0).to_minitel(&mut Minitel::from(&mut data));

        assert_eq!(data, [0x1B, 0x5B, 0x30, 0x30, 0x3B, 0x30, 0x30, 0x48]);

        let mut data = Vec::new();
        SetCursor(0, 1).to_minitel(&mut Minitel::from(&mut data));

        assert_eq!(data, [0x1B, 0x5B, 0x30, 0x31, 0x3B, 0x30, 0x30, 0x48]);

        let mut data = Vec::new();
        SetCursor(10, 20).to_minitel(&mut Minitel::from(&mut data));

        assert_eq!(data, [0x1B, 0x5B, 0x32, 0x30, 0x3B, 0x31, 0x30, 0x48]);
    }

    #[test]
    fn test_set_cursor_fails() {
        assert_panics!(|| {
            let mut data = Vec::new();
            SetCursor(41, 10).to_minitel(&mut Minitel::from(&mut data));
        });

        assert_panics!(|| {
            let mut data = Vec::new();
            SetCursor(10, 25).to_minitel(&mut Minitel::from(&mut data));
        });
    }

    #[test]
    fn test_move_cursor() {
        let mut data = Vec::new();
        MoveCursor(Direction::Up, 1).to_minitel(&mut Minitel::from(&mut data));
        MoveCursor(Direction::Down, 1).to_minitel(&mut Minitel::from(&mut data));
        MoveCursor(Direction::Right, 1).to_minitel(&mut Minitel::from(&mut data));
        MoveCursor(Direction::Left, 1).to_minitel(&mut Minitel::from(&mut data));

        assert_eq!(data, [0x0B, 0x0A, 0x09, 0x08]);

        let mut data = Vec::new();
        MoveCursor(Direction::Up, 2).to_minitel(&mut Minitel::from(&mut data));
        MoveCursor(Direction::Down, 3).to_minitel(&mut Minitel::from(&mut data));
        MoveCursor(Direction::Right, 4).to_minitel(&mut Minitel::from(&mut data));
        MoveCursor(Direction::Left, 5).to_minitel(&mut Minitel::from(&mut data));

        assert_eq!(data, [
            0x1B, 0x5B, 0x30, 0x32, 0x41,
            0x1B, 0x5B, 0x30, 0x33, 0x42,
            0x1B, 0x5B, 0x30, 0x34, 0x43,
            0x1B, 0x5B, 0x30, 0x35, 0x44,
        ]);

        let mut data = Vec::new();
        MoveCursor(Direction::Up, 15).to_minitel(&mut Minitel::from(&mut data));
        MoveCursor(Direction::Down, 21).to_minitel(&mut Minitel::from(&mut data));
        MoveCursor(Direction::Right, 33).to_minitel(&mut Minitel::from(&mut data));
        MoveCursor(Direction::Left, 40).to_minitel(&mut Minitel::from(&mut data));

        assert_eq!(data, [
            0x1B, 0x5B, 0x31, 0x35, 0x41,
            0x1B, 0x5B, 0x32, 0x31, 0x42,
            0x1B, 0x5B, 0x33, 0x33, 0x43,
            0x1B, 0x5B, 0x34, 0x30, 0x44,
        ]);
    }

    #[test]
    fn test_videotex() {
        let mut data = Vec::new();
        Videotex::new("bonjour").to_minitel(&mut Minitel::from(&mut data));
        assert_eq!(data, "bonjour".as_bytes());

        let mut data = Vec::new();
        Videotex::new(vec![0x01, 0x02, 0x03]).to_minitel(&mut Minitel::from(&mut data));
        assert_eq!(data, vec![0x01, 0x02, 0x03]);

        let path = format!("{}/test_videotex.vdt", env::temp_dir().to_str().unwrap());
        fs::write(&path, "bonjour le fichier").unwrap();

        let mut data = Vec::new();
        Videotex::from_path(&path).unwrap().to_minitel(&mut Minitel::from(&mut data));

        assert_eq!(data, "bonjour le fichier".as_bytes());

        fs::remove_file(&path).unwrap();
    }
}
