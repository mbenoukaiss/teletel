#![allow(dead_code)]

use std::io::Result as IoResult;
use crate::specifications::codes::*;
use crate::terminal::{ToTerminal, WriteableTerminal};
use crate::declare;

declare!(Clear, [FF]);
declare!(ClearRow, [CSI_2_K]);
declare!(Beep, [BEEP]);
declare!(Underline<T: ToTerminal>(pub T), |self| [ESC, START_UNDERLINE, self.0, ESC, STOP_UNDERLINE]);
declare!(Blink<T: ToTerminal>(pub T), |self| [ESC, BLINK, self.0, ESC, STILL]);
declare!(Background<T: ToTerminal>(pub Color, pub T), |self| [ESC, BACKGROUND + self.0 as u8, self.1, ESC, BACKGROUND + Color::Black as u8]);
declare!(Foreground<T: ToTerminal>(pub Color, pub T), |self| [ESC, FOREGROUND + self.0 as u8, self.1, ESC, FOREGROUND + Color::White as u8]);
declare!(Inverted<T: ToTerminal>(pub T), |self| [ESC, START_INVERT, self.0, ESC, STOP_INVERT]);
declare!(Big<T: ToTerminal>(pub T), |self| [ESC, DOUBLE_SIZE, self.0, ESC, NORMAL_SIZE]);
declare!(Mask<T: ToTerminal>(pub T), |self| [ESC, MASK, self.0, ESC, UNMASK]);
declare!(SemiGraphic<T: ToTerminal>(pub T), |self| [SO, self.0, SI]);

//not an ideal setup: the teletel standard provides no way to just reset the double height
//or just reset the double width, so we have to reset both, however we would want to keep
//double width when double height ends for example, so ideally this would be refactored
//to take context into account when context/parsing is implemented
declare!(Tall<T: ToTerminal>(pub T), |self| [ESC, DOUBLE_HEIGHT, self.0, ESC, NORMAL_SIZE]);
declare!(Wide<T: ToTerminal>(pub T), |self| [ESC, DOUBLE_WIDTH, self.0, ESC, NORMAL_SIZE]);

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

pub struct Repeat<T: ToTerminal>(pub T, pub u8);

impl ToTerminal for Repeat<char> {
    #[inline(always)]
    fn to_terminal(&self, term: &mut dyn WriteableTerminal) -> IoResult<usize> {
        repeat(self.0 as u8, self.1).to_terminal(term)
    }
}

impl ToTerminal for Repeat<u8> {
    #[inline(always)]
    fn to_terminal(&self, term: &mut dyn WriteableTerminal) -> IoResult<usize> {
        repeat(self.0, self.1).to_terminal(term)
    }
}

pub struct SetCursor(pub u8, pub u8);

impl ToTerminal for SetCursor {
    fn to_terminal(&self, term: &mut dyn WriteableTerminal) -> IoResult<usize> {
        assert!(self.0 <= 40);
        assert!(self.1 <= 24);

        //documented on page 95
        let mut written_bytes = 0;
        written_bytes += [ESC, 0x5B].to_terminal(term)?;
        written_bytes += to_decimal(self.1).to_terminal(term)?;
        written_bytes += 0x3B.to_terminal(term)?;
        written_bytes += to_decimal(self.0).to_terminal(term)?;
        written_bytes += 0x48.to_terminal(term)?;

        Ok(written_bytes)
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

impl ToTerminal for MoveCursor {
    fn to_terminal(&self, term: &mut dyn WriteableTerminal) -> IoResult<usize> {
        if self.1 == 1 {
            match self.0 {
                Direction::Up => CURSOR_UP,
                Direction::Down => CURSOR_DOWN,
                Direction::Right => CURSOR_RIGHT,
                Direction::Left => CURSOR_LEFT,
            }.to_terminal(term)
        } else {
            let direction = match self.0 {
                Direction::Up => 0x41,
                Direction::Down => 0x42,
                Direction::Right => 0x43,
                Direction::Left => 0x44,
            };

            let mut written_bytes = 0;
            written_bytes += [ESC, 0x5B].to_terminal(term)?;
            written_bytes += to_decimal(self.1).to_terminal(term)?;
            written_bytes += direction.to_terminal(term)?;

            Ok(written_bytes)
        }
    }
}

pub struct Videotex<B: AsRef<[u8]> + ToTerminal> {
    pub data: B,
}

impl<B: AsRef<[u8]> + ToTerminal> Videotex<B> {
    pub fn new(data: B) -> Self {
        Self { data }
    }
}

impl Videotex<Vec<u8>> {
    pub fn from_path(path: &str) -> IoResult<Self> {
        Ok(Self {
            data: std::fs::read(path)?,
        })
    }
}

impl<B: AsRef<[u8]> + ToTerminal> ToTerminal for Videotex<B> {
    #[inline(always)]
    fn to_terminal(&self, term: &mut dyn WriteableTerminal) -> IoResult<usize> {
        self.data.to_terminal(term)
    }
}

pub enum ScreenMasking {
    On,
    Off,
}

impl ToTerminal for ScreenMasking {
    fn to_terminal(&self, term: &mut dyn WriteableTerminal) -> IoResult<usize> {
        match self {
            ScreenMasking::On => [ESC, 0x23, 0x20, MASK].to_terminal(term),
            ScreenMasking::Off => [ESC, 0x23, 0x20, UNMASK].to_terminal(term),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{env, fs};
    use super::*;
    use teletel_derive::sg;
    use crate::terminal::Buffer;

    #[test]
    fn test_clear() {
        let mut data = Buffer::new();
        Clear.to_terminal(&mut data).unwrap();
        assert_eq!(data.data(), [0x0C]);
    }

    #[test]
    fn test_clear_row() {
        let mut data = Buffer::new();
        ClearRow.to_terminal(&mut data).unwrap();
        assert_eq!(data.data(), [0x1B, 0x5B, 0x32, 0x4B]);
    }

    #[test]
    fn test_beep() {
        let mut data = Buffer::new();
        Beep.to_terminal(&mut data).unwrap();
        assert_eq!(data.data(), [0x07]);
    }

    #[test]
    fn test_blink() {
        let mut data = Buffer::new();
        Blink('A').to_terminal(&mut data).unwrap();
        assert_eq!(data.data(), [0x1B, 0x48, 'A' as u8, 0x1B, 0x49]);

        let mut data = Buffer::new();
        Blink("bonjour").to_terminal(&mut data).unwrap();
        assert_eq!(data.data(), [0x1B, 0x48, 'b' as u8, 'o' as u8, 'n' as u8, 'j' as u8, 'o' as u8, 'u' as u8, 'r' as u8, 0x1B, 0x49]);
    }

    #[test]
    fn test_background() {
        let mut data = Buffer::new();
        Background(Color::Gray60, 'A').to_terminal(&mut data).unwrap();
        assert_eq!(data.data(), [0x1B, 0x55, 'A' as u8, 0x1B, 0x50]);

        let mut data = Buffer::new();
        Background(Color::Gray90, "bonjour").to_terminal(&mut data).unwrap();
        assert_eq!(data.data(), [0x1B, 0x53, 'b' as u8, 'o' as u8, 'n' as u8, 'j' as u8, 'o' as u8, 'u' as u8, 'r' as u8, 0x1B, 0x50]);
    }

    #[test]
    fn test_foreground() {
        let mut data = Buffer::new();
        Foreground(Color::Gray70, 'A').to_terminal(&mut data).unwrap();
        assert_eq!(data.data(), [0x1B, 0x42, 'A' as u8, 0x1B, 0x47]);

        let mut data = Buffer::new();
        Foreground(Color::Gray40, "bonjour").to_terminal(&mut data).unwrap();
        assert_eq!(data.data(), [0x1B, 0x44, 'b' as u8, 'o' as u8, 'n' as u8, 'j' as u8, 'o' as u8, 'u' as u8, 'r' as u8, 0x1B, 0x47]);
    }

    #[test]
    fn test_inverted() {
        let mut data = Buffer::new();
        Inverted('A').to_terminal(&mut data).unwrap();
        assert_eq!(data.data(), [0x1B, 0x5D, 'A' as u8, 0x1B, 0x5C]);

        let mut data = Buffer::new();
        Inverted("bonjour").to_terminal(&mut data).unwrap();
        assert_eq!(data.data(), [0x1B, 0x5D, 'b' as u8, 'o' as u8, 'n' as u8, 'j' as u8, 'o' as u8, 'u' as u8, 'r' as u8, 0x1B, 0x5C]);
    }

    #[test]
    fn test_big() {
        let mut data = Buffer::new();
        Big('A').to_terminal(&mut data).unwrap();
        assert_eq!(data.data(), [0x1B, 0x4F, 'A' as u8, 0x1B, 0x4C]);

        let mut data = Buffer::new();
        Big("bonjour").to_terminal(&mut data).unwrap();
        assert_eq!(data.data(), [0x1B, 0x4F, 'b' as u8, 'o' as u8, 'n' as u8, 'j' as u8, 'o' as u8, 'u' as u8, 'r' as u8, 0x1B, 0x4C]);
    }

    #[test]
    fn test_mask() {
        let mut data = Buffer::new();
        Mask('A').to_terminal(&mut data).unwrap();
        assert_eq!(data.data(), [0x1B, 0x58, 'A' as u8, 0x1B, 0x5F]);

        let mut data = Buffer::new();
        Mask("bonjour").to_terminal(&mut data).unwrap();
        assert_eq!(data.data(), [0x1B, 0x58, 'b' as u8, 'o' as u8, 'n' as u8, 'j' as u8, 'o' as u8, 'u' as u8, 'r' as u8, 0x1B, 0x5F]);
    }

    #[test]
    fn test_semigraphic() {
        let mut data = Buffer::new();
        SemiGraphic(list![
            sg!(00/00/00),
            sg!(00/01/11),
            sg!(01/11/01),
            sg!(11/11/11),
        ]).to_terminal(&mut data).unwrap();

        assert_eq!(data.data(), [0x0E, 0x20, 0x78, 0x6E, 0x5F, 0x0F]);
    }

    #[test]
    fn test_repeat() {
        let mut data = Buffer::new();
        Repeat('A', 3).to_terminal(&mut data).unwrap();

        assert_eq!(data.data(), ['A' as u8, 0x12, 0x42]);

        let mut data = Buffer::new();
        Repeat(sg!(01/11/01), 3).to_terminal(&mut data).unwrap();

        assert_eq!(data.data(), [0x6E, 0x12, 0x42]);
    }

    #[test]
    fn test_repeat_fails() {
        assert_panics!(|| {
            let mut data = Buffer::new();
            Repeat('A', 0).to_terminal(&mut data).unwrap();
        });

        assert_panics!(|| {
            let mut data = Buffer::new();
            Repeat('A', 65).to_terminal(&mut data).unwrap();
        });
    }

    #[test]
    fn test_set_cursor() {
        let mut data = Buffer::new();
        SetCursor(0, 0).to_terminal(&mut data).unwrap();

        assert_eq!(data.data(), [0x1B, 0x5B, 0x30, 0x30, 0x3B, 0x30, 0x30, 0x48]);

        let mut data = Buffer::new();
        SetCursor(0, 1).to_terminal(&mut data).unwrap();

        assert_eq!(data.data(), [0x1B, 0x5B, 0x30, 0x31, 0x3B, 0x30, 0x30, 0x48]);

        let mut data = Buffer::new();
        SetCursor(10, 20).to_terminal(&mut data).unwrap();

        assert_eq!(data.data(), [0x1B, 0x5B, 0x32, 0x30, 0x3B, 0x31, 0x30, 0x48]);
    }

    #[test]
    fn test_set_cursor_fails() {
        assert_panics!(|| {
            let mut data = Buffer::new();
            SetCursor(41, 10).to_terminal(&mut data).unwrap();
        });

        assert_panics!(|| {
            let mut data = Buffer::new();
            SetCursor(10, 25).to_terminal(&mut data).unwrap();
        });
    }

    #[test]
    fn test_move_cursor() {
        let mut data = Buffer::new();
        MoveCursor(Direction::Up, 1).to_terminal(&mut data).unwrap();
        MoveCursor(Direction::Down, 1).to_terminal(&mut data).unwrap();
        MoveCursor(Direction::Right, 1).to_terminal(&mut data).unwrap();
        MoveCursor(Direction::Left, 1).to_terminal(&mut data).unwrap();

        assert_eq!(data.data(), [0x0B, 0x0A, 0x09, 0x08]);

        let mut data = Buffer::new();
        MoveCursor(Direction::Up, 2).to_terminal(&mut data).unwrap();
        MoveCursor(Direction::Down, 3).to_terminal(&mut data).unwrap();
        MoveCursor(Direction::Right, 4).to_terminal(&mut data).unwrap();
        MoveCursor(Direction::Left, 5).to_terminal(&mut data).unwrap();

        assert_eq!(data.data(), [
            0x1B, 0x5B, 0x30, 0x32, 0x41,
            0x1B, 0x5B, 0x30, 0x33, 0x42,
            0x1B, 0x5B, 0x30, 0x34, 0x43,
            0x1B, 0x5B, 0x30, 0x35, 0x44,
        ]);

        let mut data = Buffer::new();
        MoveCursor(Direction::Up, 15).to_terminal(&mut data).unwrap();
        MoveCursor(Direction::Down, 21).to_terminal(&mut data).unwrap();
        MoveCursor(Direction::Right, 33).to_terminal(&mut data).unwrap();
        MoveCursor(Direction::Left, 40).to_terminal(&mut data).unwrap();

        assert_eq!(data.data(), [
            0x1B, 0x5B, 0x31, 0x35, 0x41,
            0x1B, 0x5B, 0x32, 0x31, 0x42,
            0x1B, 0x5B, 0x33, 0x33, 0x43,
            0x1B, 0x5B, 0x34, 0x30, 0x44,
        ]);
    }

    #[test]
    fn test_videotex() {
        let mut buf = Buffer::new();
        Videotex::new("bonjour").to_terminal(&mut buf).unwrap();
        assert_eq!(buf.data(), "bonjour".as_bytes());

        let mut buf = Buffer::new();
        Videotex::new(vec![0x01, 0x02, 0x03]).to_terminal(&mut buf).unwrap();
        assert_eq!(buf.data(), vec![0x01, 0x02, 0x03]);

        let path = format!("{}/test_videotex.vdt", env::temp_dir().to_str().unwrap());
        fs::write(&path, "bonjour le fichier").unwrap();

        let mut buf = Buffer::new();
        Videotex::from_path(&path).unwrap().to_terminal(&mut buf).unwrap();

        assert_eq!(buf.data(), "bonjour le fichier".as_bytes());

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_screen_masking() {
        let mut data = Buffer::new();
        ScreenMasking::On.to_terminal(&mut data).unwrap();
        assert_eq!(data.data(), [0x1B, 0x23, 0x20, 0x58]);

        let mut data = Buffer::new();
        ScreenMasking::Off.to_terminal(&mut data).unwrap();
        assert_eq!(data.data(), [0x1B, 0x23, 0x20, 0x5F]);
    }
}
