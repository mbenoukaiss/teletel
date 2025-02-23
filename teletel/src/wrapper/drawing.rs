use crate::Error;
use crate::functions::{Direction, MoveCursor, Repeat, SemiGraphic};
use crate::terminal::{ToTerminal, WriteableTerminal};
use teletel_protocol::codes::{SI, SO};

pub struct HLine(pub u8, pub u8);

impl HLine {
    pub const FULL: u8 = 0b111;
    pub const TOP: u8 = 0b100;
    pub const MID: u8 = 0b010;
    pub const BOT: u8 = 0b001;
}

impl ToTerminal for HLine {
    fn to_terminal(&self, term: &mut dyn WriteableTerminal) -> Result<(), Error> {
        assert!(self.0 <= 40);

        let mut character = 0x00;

        if self.1 & HLine::TOP != 0 {
            character |= sg!(11/00/00);
        }

        if self.1 & HLine::MID != 0 {
            character |= sg!(00/11/00);
        }

        if self.1 & HLine::BOT != 0 {
            character |= sg!(00/00/11);
        }

        SemiGraphic(Repeat(character, self.0)).to_terminal(term)
    }
}

pub struct VLine(pub u8, pub u8);

impl VLine {
    pub const FULL: u8 = 0b11;
    pub const LEFT: u8 = 0b10;
    pub const RIGHT: u8 = 0b01;
}

impl ToTerminal for VLine {
    fn to_terminal(&self, term: &mut dyn WriteableTerminal) -> Result<(), Error> {
        assert!(self.0 <= 22);

        let mut character = 0x00;

        if self.1 & VLine::LEFT != 0 {
            character |= sg!(10/10/10);
        }

        if self.1 & VLine::RIGHT != 0 {
            character |= sg!(01/01/01);
        }

        SO.to_terminal(term)?;

        for _ in 0..self.0 {
            character.to_terminal(term)?;
            MoveCursor(Direction::Down, 1).to_terminal(term)?;
            MoveCursor(Direction::Left, 1).to_terminal(term)?;
        }

        SI.to_terminal(term)?;

        Ok(())
    }
}

//TODO: fix when nearing the right and bottom border of the screen
pub struct RectangleOutline(pub u8, pub u8, pub u8);

impl RectangleOutline {
    pub const FULL: u8 = 0b11;
    pub const OUT: u8 = 0b10;
    pub const IN: u8 = 0b01;
}

impl ToTerminal for RectangleOutline {
    fn to_terminal(&self, term: &mut dyn WriteableTerminal) -> Result<(), Error> {
        assert!(self.0 >= 2 && self.0 <= 40);
        assert!(self.1 >= 2 && self.1 <= 24);

        let character_set = RectangleOutlineCharacterSet::new(self.2);

        SO.to_terminal(term)?;
        character_set.top_left_corner.to_terminal(term)?;
        Repeat(character_set.top_line, self.0 - 2).to_terminal(term)?;
        character_set.top_right_corner.to_terminal(term)?;

        for _ in 0..(self.1 - 2) {
            MoveCursor(Direction::Down, 1).to_terminal(term)?;
            MoveCursor(Direction::Left, 1).to_terminal(term)?;
            character_set.right_line.to_terminal(term)?;
        }

        MoveCursor(Direction::Down, 1).to_terminal(term)?;
        MoveCursor(Direction::Left, 1).to_terminal(term)?;
        character_set.bottom_right_corner.to_terminal(term)?;

        MoveCursor(Direction::Left, self.0 - 2 + 1).to_terminal(term)?;
        Repeat(character_set.bottom_line, self.0 - 2).to_terminal(term)?;
        MoveCursor(Direction::Left, self.0 - 2 + 1).to_terminal(term)?;
        character_set.bottom_left_corner.to_terminal(term)?;

        MoveCursor(Direction::Up, self.1 - 1).to_terminal(term)?;

        for _ in 0..self.1 - 2 {
            MoveCursor(Direction::Down, 1).to_terminal(term)?;
            MoveCursor(Direction::Left, 1).to_terminal(term)?;
            character_set.left_line.to_terminal(term)?;
        }

        SI.to_terminal(term)?;
        
        Ok(())
    }
}

struct RectangleOutlineCharacterSet {
    top_left_corner: u8,
    top_right_corner: u8,
    bottom_left_corner: u8,
    bottom_right_corner: u8,
    top_line: u8,
    bottom_line: u8,
    left_line: u8,
    right_line: u8,
}

impl RectangleOutlineCharacterSet {
    pub fn new(settings: u8) -> RectangleOutlineCharacterSet {
        match settings {
            RectangleOutline::FULL => RectangleOutlineCharacterSet {
                top_left_corner: sg!(11/11/11),
                top_right_corner: sg!(11/11/11),
                bottom_left_corner: sg!(11/11/11),
                bottom_right_corner: sg!(11/11/11),
                top_line: sg!(11/11/11),
                bottom_line: sg!(11/11/11),
                left_line: sg!(11/11/11),
                right_line: sg!(11/11/11),
            },
            RectangleOutline::OUT => RectangleOutlineCharacterSet {
                top_left_corner: sg!(11/10/10),
                top_right_corner: sg!(11/01/01),
                bottom_left_corner: sg!(10/10/11),
                bottom_right_corner: sg!(01/01/11),
                top_line: sg!(11/00/00),
                bottom_line: sg!(00/00/11),
                left_line: sg!(10/10/10),
                right_line: sg!(01/01/01),
            },
            RectangleOutline::IN => RectangleOutlineCharacterSet {
                top_left_corner: sg!(00/00/01),
                top_right_corner: sg!(00/00/10),
                bottom_left_corner: sg!(01/00/00),
                bottom_right_corner: sg!(10/00/00),
                top_line: sg!(00/00/11),
                bottom_line: sg!(11/00/00),
                left_line: sg!(01/01/01),
                right_line: sg!(10/10/10),
            },
            invalid => panic!("Invalid rectangle settings: {}", invalid)
        }
    }
}

pub struct FilledRectangle(pub u8, pub u8);

impl ToTerminal for FilledRectangle {
    fn to_terminal(&self, term: &mut dyn WriteableTerminal) -> Result<(), Error> {
        assert!(self.0 <= 40);
        assert!(self.1 <= 23);

        SO.to_terminal(term)?;
        Repeat(sg!(11/11/11), self.0).to_terminal(term)?;

        for _ in 0..self.1 - 2 {
            MoveCursor(Direction::Down, 1).to_terminal(term)?;
            MoveCursor(Direction::Left, self.0).to_terminal(term)?;
            Repeat(sg!(11/11/11), self.0).to_terminal(term)?;
        }

        SI.to_terminal(term)?;

        Ok(())
    }
}
