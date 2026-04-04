use crate::codes::*;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult, Write};
use std::mem;

macro_rules! err {
    ($($arg:tt)*) => {
        return Err(Error::new(format!($($arg)*)))
    }
}

/// - If G2 character set is requested but the following code does not exist in G2 a
///   lower horizontal line will be displayed instead except if it's contained in C0 then
///   the C0 character is displayed and SS2 is ignored (??? end of p90).
/// - While G1 is active, any attempt to switch to G2 is ignored
/// - Attempting to use an accent on a character that does not support it will result
///   in displaying the character without the accent. VGP5 supports àâäéèêëîïöôùûüç and
///   VGP2 supports àâéèêëîïôùûç. No uppercase characters are supported.
/// - ß and § are only supported on VGP5
/// - Double size and double height are ignored in line 0 and 1
/// - Size and inverted are not ignored when in semi-graphic mode and are cleared when
///   switching to semi-graphic mode
/// - An attribute stops being applied and reset to their default value when starting
///   or exiting a section or a subsection
/// - Double height, width or size characters are displayed from the bottom left corner

#[derive(Eq, PartialEq, Debug)]
pub struct Error {
    msg: String,
}

impl Error {
    fn new<S: Into<String>>(msg: S) -> Error {
        Error { msg: msg.into() }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.msg)
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum DisplayComponent {
    VGP2,
    VGP5,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum PageMode {
    Page,
    Scroll,
}

trait ToCharacter {
    fn to_character(&self) -> Result<char, Error>;
}

trait Parsable: Sized {
    fn new(ctx: &Context, byte: u8) -> Result<Self, Error>;
    fn supports(ctx: &Context, byte: u8) -> bool;
    fn consume(&mut self, ctx: &Context, byte: u8) -> Result<Self, Error>;
    fn is_complete(&self) -> bool;
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
#[repr(u8)]
pub enum CharacterSet {
    G0,
    G1,
}

impl Parsable for CharacterSet {
    fn new(_ctx: &Context, byte: u8) -> Result<Self, Error> {
        match byte {
            SI => Ok(CharacterSet::G0),
            SO => Ok(CharacterSet::G1),
            invalid => err!("Invalid character set {:#04X}", invalid),
        }
    }

    fn supports(_ctx: &Context, byte: u8) -> bool {
        byte == SI || byte == SO
    }

    fn consume(&mut self, _ctx: &Context, byte: u8) -> Result<Self, Error> {
        err!(
            "Character set {:?} does not support more bytes ({:#04X})",
            self,
            byte
        );
    }

    fn is_complete(&self) -> bool {
        true
    }
}

//fully implemented
#[derive(Eq, PartialEq, Debug)]
struct SimpleCharacter(u8);

impl Parsable for SimpleCharacter {
    fn new(ctx: &Context, byte: u8) -> Result<Self, Error> {
        if !Self::supports(ctx, byte) {
            err!("Invalid simple character {:#04X}", byte);
        }

        Ok(SimpleCharacter(byte))
    }

    fn supports(ctx: &Context, byte: u8) -> bool {
        ctx.attributes.character_set == CharacterSet::G0 && (0x20..=0x7F).contains(&byte)
    }

    fn consume(&mut self, _ctx: &Context, byte: u8) -> Result<Self, Error> {
        err!(
            "Simple character {:?} does not support more bytes ({:#04X})",
            self,
            byte
        );
    }

    fn is_complete(&self) -> bool {
        true
    }
}

impl ToCharacter for SimpleCharacter {
    fn to_character(&self) -> Result<char, Error> {
        Ok(self.0 as char)
    }
}

//fully implemented
#[derive(Eq, PartialEq)]
struct SemiGraphicCharacter(u8);

impl Parsable for SemiGraphicCharacter {
    fn new(ctx: &Context, mut byte: u8) -> Result<Self, Error> {
        if !Self::supports(ctx, byte) {
            err!("Invalid semi-graphic character {:#04X}", byte);
        }

        if byte == 0x7F {
            byte = 0x5F; //p98
        } else if (0x40..=0x5E).contains(&byte) {
            //characters from 0x40 to 0x5E are not documented and are left empty in
            //the table p101, but they correspond to characters from 0x60 to 0x7E
            byte += 0x20;
        }

        Ok(SemiGraphicCharacter(byte))
    }

    fn supports(ctx: &Context, byte: u8) -> bool {
        ctx.attributes.character_set == CharacterSet::G1 && (0x20..=0x7F).contains(&byte)
    }

    fn consume(&mut self, _ctx: &Context, byte: u8) -> Result<Self, Error> {
        err!(
            "Semi-graphic character {:?} does not support more bytes ({:#04X})",
            self,
            byte
        );
    }

    fn is_complete(&self) -> bool {
        true
    }
}

impl Debug for SemiGraphicCharacter {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "SemiGraphicCharacter({:#04X})", self.0)
    }
}

//fully implemented
#[derive(Eq, PartialEq, Debug)]
enum SpecialCharacter {
    Incomplete,
    Grave(Option<u8>),
    Acute(Option<u8>),
    Circumflex(Option<u8>),
    Diaeresis(Option<u8>),
    Cedilla(Option<u8>),
    LowerOE,
    UpperOE,
    Eszett,
    Pound,
    Dollar,
    NumberSign,
    ArrowLeft,
    ArrowUp,
    ArrowRight,
    ArrowDown,
    Paragraph,
    Degree,
    PlusOrMinus,
    Obelus,
    OneQuarter,
    OneHalf,
    ThreeQuarters,
}

impl Parsable for SpecialCharacter {
    fn new(ctx: &Context, byte: u8) -> Result<Self, Error> {
        if !Self::supports(ctx, byte) {
            err!("Invalid special character {:#04X}", byte);
        }

        if ctx.attributes.character_set == CharacterSet::G1 {
            err!("Special characters are not supported in G1");
        }

        Ok(SpecialCharacter::Incomplete)
    }

    fn supports(_ctx: &Context, byte: u8) -> bool {
        byte == SS2
    }

    fn consume(&mut self, ctx: &Context, byte: u8) -> Result<Self, Error> {
        let result = match self {
            SpecialCharacter::Incomplete => match byte {
                GRAVE => SpecialCharacter::Grave(None),
                ACUTE => SpecialCharacter::Acute(None),
                CIRCUMFLEX => SpecialCharacter::Circumflex(None),
                DIAERESIS => SpecialCharacter::Diaeresis(None),
                CEDILLA => SpecialCharacter::Cedilla(None),
                LOWER_OE => SpecialCharacter::LowerOE,
                UPPER_OE => SpecialCharacter::UpperOE,
                ESZETT if ctx.display_component == DisplayComponent::VGP5 => {
                    SpecialCharacter::Eszett
                }
                POUND => SpecialCharacter::Pound,
                DOLLAR => SpecialCharacter::Dollar,
                NUMBER_SIGN => SpecialCharacter::NumberSign,
                ARROW_LEFT => SpecialCharacter::ArrowLeft,
                ARROW_UP => SpecialCharacter::ArrowUp,
                ARROW_RIGHT => SpecialCharacter::ArrowRight,
                ARROW_DOWN => SpecialCharacter::ArrowDown,
                PARAGRAPH if ctx.display_component == DisplayComponent::VGP5 => {
                    SpecialCharacter::Paragraph
                }
                DEGREE => SpecialCharacter::Degree,
                PLUS_OR_MINUS => SpecialCharacter::PlusOrMinus,
                OBELUS => SpecialCharacter::Obelus,
                ONE_QUARTER => SpecialCharacter::OneQuarter,
                ONE_HALF => SpecialCharacter::OneHalf,
                THREE_QUARTERS => SpecialCharacter::ThreeQuarters,
                _ => err!("Invalid special character {:#04X}", byte),
            },
            SpecialCharacter::Grave(None) => match byte {
                b'a' | b'e' | b'u' => SpecialCharacter::Grave(Some(byte)),
                _ => err!("Invalid character for grave accent {:#04X}", byte),
            },
            SpecialCharacter::Acute(None) => match byte {
                b'e' => SpecialCharacter::Acute(Some(byte)),
                _ => err!("Invalid character for acute accent {:#04X}", byte),
            },
            SpecialCharacter::Circumflex(None) => match byte {
                b'a' | b'e' | b'i' | b'o' | b'u' => SpecialCharacter::Circumflex(Some(byte)),
                _ => err!("Invalid character for circumflex accent {:#04X}", byte),
            },
            SpecialCharacter::Diaeresis(None) => match byte {
                b'e' | b'i' => SpecialCharacter::Diaeresis(Some(byte)),
                b'a' | b'o' | b'u' if ctx.display_component == DisplayComponent::VGP5 => {
                    SpecialCharacter::Diaeresis(Some(byte))
                }
                _ => err!("Invalid character for diaeresis {:#04X}", byte),
            },
            SpecialCharacter::Cedilla(None) => match byte {
                b'c' => SpecialCharacter::Cedilla(Some(byte)),
                _ => err!("Invalid character for cedilla {:#04X}", byte),
            },
            _ => err!(
                "Escaped sequence {:?} does not support more bytes ({:#04X})",
                self,
                byte
            ),
        };

        Ok(result)
    }

    fn is_complete(&self) -> bool {
        !matches!(
            self,
            SpecialCharacter::Incomplete
                | SpecialCharacter::Grave(None)
                | SpecialCharacter::Acute(None)
                | SpecialCharacter::Circumflex(None)
                | SpecialCharacter::Diaeresis(None)
                | SpecialCharacter::Cedilla(None)
        )
    }
}

impl ToCharacter for SpecialCharacter {
    fn to_character(&self) -> Result<char, Error> {
        let char = match self {
            SpecialCharacter::Grave(Some(byte)) => match byte {
                b'a' => 'à',
                b'e' => 'è',
                b'u' => 'ù',
                _ => err!("Invalid character for grave accent {:#04X}", byte),
            },
            SpecialCharacter::Acute(Some(byte)) => match byte {
                b'e' => 'é',
                _ => err!("Invalid character for acute accent {:#04X}", byte),
            },
            SpecialCharacter::Circumflex(Some(byte)) => match byte {
                b'a' => 'â',
                b'e' => 'ê',
                b'i' => 'î',
                b'o' => 'ô',
                b'u' => 'û',
                _ => err!("Invalid character for circumflex accent {:#04X}", byte),
            },
            SpecialCharacter::Diaeresis(Some(byte)) => match byte {
                b'a' => 'ä',
                b'e' => 'ë',
                b'i' => 'ï',
                b'o' => 'ö',
                b'u' => 'ü',
                _ => err!("Invalid character for diaeresis {:#04X}", byte),
            },
            SpecialCharacter::Cedilla(Some(byte)) => match byte {
                b'c' => 'ç',
                _ => err!("Invalid character for cedilla {:#04X}", byte),
            },
            SpecialCharacter::LowerOE => 'œ',
            SpecialCharacter::UpperOE => 'Œ',
            SpecialCharacter::Eszett => 'ß',
            SpecialCharacter::Pound => '£',
            SpecialCharacter::Dollar => '$',
            SpecialCharacter::NumberSign => '#',
            SpecialCharacter::ArrowLeft => '←',
            SpecialCharacter::ArrowUp => '↑',
            SpecialCharacter::ArrowRight => '→',
            SpecialCharacter::ArrowDown => '↓',
            SpecialCharacter::Paragraph => '§',
            SpecialCharacter::Degree => '°',
            SpecialCharacter::PlusOrMinus => '±',
            SpecialCharacter::Obelus => '÷',
            SpecialCharacter::OneQuarter => '¼',
            SpecialCharacter::OneHalf => '½',
            SpecialCharacter::ThreeQuarters => '¾',
            _ => err!("Special character {:?} is not complete", self),
        };

        Ok(char)
    }
}

#[derive(Eq, PartialEq, Debug)]
enum Protocol {
    Pro1,
    Pro2,
    Pro3,
    Reset,
    RequestSpeed,
    SetSpeed(Option<u8>),
    Toggle2(bool),
    Toggle3(bool),
    Scroll(bool),
    ToggleScreen(bool),
    Sleep(bool),
}

impl Parsable for Protocol {
    fn new(_ctx: &Context, byte: u8) -> Result<Self, Error> {
        let result = match byte {
            PRO1 => Protocol::Pro1,
            PRO2 => Protocol::Pro2,
            PRO3 => Protocol::Pro3,
            _ => err!("Invalid protocol sequence starting with {:#04X}", byte),
        };

        Ok(result)
    }

    fn supports(_ctx: &Context, byte: u8) -> bool {
        byte == PRO1 || byte == PRO2 || byte == PRO3
    }

    fn consume(&mut self, _ctx: &Context, byte: u8) -> Result<Self, Error> {
        let result = match self {
            Protocol::Pro1 => match byte {
                RESET => Protocol::Reset,
                REQ_SPEED => Protocol::RequestSpeed,
                _ => err!(
                    "Unsupported or invalid PRO1 sequence starting with {:#04X}",
                    byte
                ),
            },
            Protocol::Pro2 => match byte {
                PROG => Protocol::SetSpeed(None),
                START => Protocol::Toggle2(true),
                STOP => Protocol::Toggle2(false),
                _ => err!(
                    "Unsupported or invalid PRO2 sequence starting with {:#04X}",
                    byte
                ),
            },
            Protocol::Pro3 => match byte {
                START => Protocol::Toggle3(true),
                STOP => Protocol::Toggle3(false),
                _ => err!(
                    "Unsupported or invalid PRO3 sequence starting with {:#04X}",
                    byte
                ),
            },
            Protocol::SetSpeed(None) => Protocol::SetSpeed(Some(byte)),
            Protocol::Toggle2(value) => match byte {
                SCROLL => Protocol::Scroll(*value),
                _ => err!(
                    "Unsupported or invalid PRO2 start/stop sequence starting with {:#04X}",
                    byte
                ),
            },
            Protocol::Toggle3(value) => match byte {
                SCREEN => Protocol::ToggleScreen(*value),
                _ => err!(
                    "Unsupported or invalid PRO3 start/stop sequence starting with {:#04X}",
                    byte
                ),
            },
            Protocol::ToggleScreen(value) => match byte {
                0x41 => Protocol::Sleep(*value),
                _ => err!(
                    "Unsupported or invalid protocol toggle screen sequence starting with {:#04X}",
                    byte
                ),
            },
            _ => err!(
                "Protocol sequence {:?} does not support additional bytes ({:#04X})",
                self,
                byte
            ),
        };

        Ok(result)
    }

    fn is_complete(&self) -> bool {
        !matches!(
            self,
            Protocol::Pro1
                | Protocol::Pro2
                | Protocol::Pro3
                | Protocol::SetSpeed(None)
                | Protocol::Toggle2(_)
                | Protocol::Toggle3(_)
                | Protocol::ToggleScreen(_)
        )
    }
}

#[derive(Eq, PartialEq, Debug)]
enum Csi {
    Incomplete,
    Quantified(u8),
    MoveUp(u8),
    MoveDown(u8),
    MoveRight(u8),
    MoveLeft(u8),
    IncompleteSetCursor(Option<u8>, Option<u8>),
    SetCursor(u8, u8),
    InsertSpacesFromCursorToEol,
    ClearFromCursorToEos,
    ClearFromSosToCursor,
    ClearScreenKeepCursorPos,
    ClearFromCursorToEol,
    ClearFromSolToCursor,
    ClearRow,
    ClearAfterCursor(u8),
    InsertFromCursor(u8), //rtic only
    StartInsert,
    EndInsert,
    EraseRowsFromCursor(u8),
    InsertRowsFromCursor(u8),
}

impl Parsable for Csi {
    fn new(ctx: &Context, byte: u8) -> Result<Self, Error> {
        if !Self::supports(ctx, byte) {
            err!(
                "Unsupported or invalid CSI sequence starting with {:#04X}",
                byte
            );
        }

        if ctx.cursor_y == 0 {
            err!("CSI codes are not supported in row 0"); //p95
        }

        Ok(Csi::Incomplete)
    }

    fn supports(_ctx: &Context, byte: u8) -> bool {
        byte == CSI
    }

    fn consume(&mut self, ctx: &Context, byte: u8) -> Result<Self, Error> {
        let result = match self {
            Csi::Incomplete => match byte {
                CAN => Csi::InsertSpacesFromCursorToEol,
                0x30..=0x39 => Csi::Quantified(byte - 0x30),
                0x4A => Csi::ClearFromCursorToEos,
                0x4B => Csi::ClearFromCursorToEol,
                _ => err!(
                    "Unsupported or invalid CSI sequence starting with {:#04X}",
                    byte
                ),
            },
            Csi::Quantified(value) => match byte {
                0x30..=0x39 => Csi::Quantified(*value * 10 + (byte - 0x30)),
                0x3B => Csi::IncompleteSetCursor(None, Some(*value)),
                0x41 => Csi::MoveUp(*value),
                0x42 => Csi::MoveDown(*value),
                0x43 => Csi::MoveRight(*value),
                0x44 => Csi::MoveLeft(*value),
                0x4A if *value == 0x00 => Csi::ClearFromCursorToEos,
                0x4A if *value == 0x01 => Csi::ClearFromSosToCursor,
                0x4A if *value == 0x02 => Csi::ClearScreenKeepCursorPos,
                0x4B if *value == 0x00 => Csi::ClearFromCursorToEol,
                0x4B if *value == 0x01 => Csi::ClearFromSolToCursor,
                0x4B if *value == 0x02 => Csi::ClearRow,
                0x50 => Csi::ClearAfterCursor(*value),
                0x40 => Csi::InsertFromCursor(*value),
                0x68 if *value == 0x04 => Csi::StartInsert,
                0x6C if *value == 0x04 => Csi::EndInsert,
                0x4D => Csi::EraseRowsFromCursor(*value),
                0x4C => Csi::InsertRowsFromCursor(*value),
                _ => err!(
                    "Unsupported or invalid byte {:#04X} for quantified CSI sequence",
                    byte
                ),
            },
            Csi::IncompleteSetCursor(x, Some(y)) if (0x30..=0x39).contains(&byte) => {
                Csi::IncompleteSetCursor(Some(x.unwrap_or(0) * 10 + (byte - 0x30)), Some(*y))
            }
            Csi::IncompleteSetCursor(Some(x), Some(y)) if byte == 0x48 => {
                if (1..=ctx.screen_width).contains(x) && (1..=ctx.screen_height).contains(y) {
                    Csi::SetCursor(*x, *y)
                } else {
                    err!(
                        "Invalid cursor position ({}, {}) for screen size ({}, {})",
                        x,
                        y,
                        ctx.screen_width,
                        ctx.screen_height
                    )
                }
            }

            //TODO: implement other CSI sequences
            //TODO: implement end-of-page 95 recommendations but not here
            _ => err!(
                "Unsupported or invalid byte {:#04X} for sequence {:?}",
                byte,
                self
            ),
        };

        Ok(result)
    }

    fn is_complete(&self) -> bool {
        !matches!(
            self,
            Csi::Incomplete | Csi::Quantified(_) | Csi::IncompleteSetCursor(_, _)
        )
    }
}

//fully implemented
#[derive(Eq, PartialEq, Debug)]
enum EscapedSequence {
    Incomplete,
    Protocol(Protocol),
    Csi(Csi),
    Background(u8),
    Foreground(u8),
    Blink(bool),
    Invert(bool),
    NormalSize,
    DoubleHeight,
    DoubleWidth,
    DoubleSize,
    Underline(bool),
    Mask(bool),
    GetCursorPosition,
    IncompleteStopIgnore,
    Ignore(Option<bool>),
    IncompleteScreenMasking(u8),
    ScreenMasking(bool),
}

impl Parsable for EscapedSequence {
    fn new(ctx: &Context, byte: u8) -> Result<Self, Error> {
        if !Self::supports(ctx, byte) {
            err!(
                "Unsupported or invalid escaped sequence starting with {:#04X}",
                byte
            );
        }

        Ok(EscapedSequence::Incomplete)
    }

    fn supports(_ctx: &Context, byte: u8) -> bool {
        byte == ESC
    }

    fn consume(&mut self, ctx: &Context, byte: u8) -> Result<Self, Error> {
        let result = match self {
            EscapedSequence::Incomplete => match byte {
                0x40..=0x47 => EscapedSequence::Foreground(byte ^ FOREGROUND),
                0x50..=0x57 => EscapedSequence::Background(byte ^ BACKGROUND),
                BLINK => EscapedSequence::Blink(true),
                STILL => EscapedSequence::Blink(false),
                START_INVERT => EscapedSequence::Invert(true),
                STOP_INVERT => EscapedSequence::Invert(false),
                NORMAL_SIZE => EscapedSequence::NormalSize,
                DOUBLE_HEIGHT => EscapedSequence::DoubleHeight,
                DOUBLE_WIDTH => EscapedSequence::DoubleWidth,
                DOUBLE_SIZE => EscapedSequence::DoubleSize,
                START_UNDERLINE => EscapedSequence::Underline(true),
                STOP_UNDERLINE => EscapedSequence::Underline(false),
                MASK => EscapedSequence::Mask(true),
                UNMASK => EscapedSequence::Mask(false),
                PRO1 | PRO2 | PRO3 => EscapedSequence::Protocol(Protocol::new(ctx, byte)?),
                CSI => EscapedSequence::Csi(Csi::new(ctx, byte)?),
                0x61 => EscapedSequence::GetCursorPosition,
                0x25 => EscapedSequence::Ignore(None),
                0x2F => EscapedSequence::IncompleteStopIgnore,
                0x23 => EscapedSequence::IncompleteScreenMasking(1),
                _ => err!("Invalid escaped sequence starting with {:#04X}", byte),
            },
            EscapedSequence::IncompleteStopIgnore if byte == 0x3F => {
                EscapedSequence::Ignore(Some(false))
            }
            EscapedSequence::Ignore(None) => EscapedSequence::Ignore(Some(byte != 0x40)),
            EscapedSequence::Csi(csi) => EscapedSequence::Csi(csi.consume(ctx, byte)?),
            EscapedSequence::Protocol(pro) => EscapedSequence::Protocol(pro.consume(ctx, byte)?),
            EscapedSequence::IncompleteScreenMasking(1) if byte == 0x20 => {
                EscapedSequence::IncompleteScreenMasking(2)
            }
            EscapedSequence::IncompleteScreenMasking(2) => match byte {
                MASK => EscapedSequence::ScreenMasking(true),
                UNMASK => EscapedSequence::ScreenMasking(false),
                _ => err!(
                    "Invalid screen masking byte ({:#04X}), expected 0x58 or 0x5F",
                    byte
                ),
            },
            _ => err!(
                "Escaped sequence {:?} does not support additional bytes ({:#04X})",
                self,
                byte
            ),
        };

        Ok(result)
    }

    fn is_complete(&self) -> bool {
        match self {
            EscapedSequence::Incomplete => false,
            EscapedSequence::Csi(csi) => csi.is_complete(),
            EscapedSequence::Protocol(pro) => pro.is_complete(),
            EscapedSequence::IncompleteStopIgnore => false,
            EscapedSequence::Ignore(None) => false,
            EscapedSequence::IncompleteScreenMasking(_) => false,
            _ => true,
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
enum Direction {
    Up,
    Down,
    Right,
    Left,
}

#[derive(Eq, PartialEq, Debug)]
enum Sequence {
    Incomplete,
    Escaped(EscapedSequence),
    SetCharacterSet(CharacterSet),
    SpecialCharacter(SpecialCharacter),
    SemiGraphicCharacter(SemiGraphicCharacter),
    SimpleCharacter(SimpleCharacter),
    MoveCursor(Direction),
    CarriageReturn,
    RecordSeparator,
    ClearScreen,
    SubSection(Option<u8>, Option<u8>),
    Repeat(Option<u8>),
    Beep,
    VisibleCursor(bool),
    ErrorCharacter,
}

impl Parsable for Sequence {
    fn new(_ctx: &Context, _byte: u8) -> Result<Self, Error> {
        Ok(Sequence::Incomplete)
    }

    fn supports(_ctx: &Context, _byte: u8) -> bool {
        true
    }

    fn consume(&mut self, ctx: &Context, byte: u8) -> Result<Self, Error> {
        let result = match self {
            Sequence::Incomplete => {
                if EscapedSequence::supports(ctx, byte) {
                    Sequence::Escaped(EscapedSequence::new(ctx, byte)?)
                } else if CharacterSet::supports(ctx, byte) {
                    Sequence::SetCharacterSet(CharacterSet::new(ctx, byte)?)
                } else if SpecialCharacter::supports(ctx, byte) {
                    Sequence::SpecialCharacter(SpecialCharacter::new(ctx, byte)?)
                } else if SemiGraphicCharacter::supports(ctx, byte) {
                    Sequence::SemiGraphicCharacter(SemiGraphicCharacter::new(ctx, byte)?)
                } else if SimpleCharacter::supports(ctx, byte) {
                    Sequence::SimpleCharacter(SimpleCharacter::new(ctx, byte)?)
                } else if (BS..=VT).contains(&byte) {
                    match byte {
                        BS => Sequence::MoveCursor(Direction::Left),
                        HT => Sequence::MoveCursor(Direction::Right),
                        LF => Sequence::MoveCursor(Direction::Down),
                        VT => Sequence::MoveCursor(Direction::Up),
                        _ => unreachable!(),
                    }
                } else if byte == CR {
                    Sequence::CarriageReturn
                } else if byte == RS {
                    Sequence::RecordSeparator
                } else if byte == FF {
                    Sequence::ClearScreen
                } else if byte == US {
                    Sequence::SubSection(None, None)
                } else if byte == REP {
                    Sequence::Repeat(None)
                } else if byte == 0x00 {
                    // NUL is a padding character, no action (p99)
                    Sequence::Incomplete
                } else if byte == 0x1A {
                    // SUB displays the error symbol (p99)
                    Sequence::ErrorCharacter
                } else if byte == BEEP {
                    Sequence::Beep
                } else if byte == CURSOR_ON {
                    Sequence::VisibleCursor(true)
                } else if byte == CURSOR_OFF {
                    Sequence::VisibleCursor(false)
                } else {
                    err!(
                        "Unsupported or invalid sequence starting with {:#04X}",
                        byte
                    )
                }
            }
            Sequence::Escaped(escaped_sequence) => {
                Sequence::Escaped(escaped_sequence.consume(ctx, byte)?)
            }
            Sequence::SetCharacterSet(character_set) => {
                Sequence::SetCharacterSet(character_set.consume(ctx, byte)?)
            }
            Sequence::SpecialCharacter(special_character) => {
                Sequence::SpecialCharacter(special_character.consume(ctx, byte)?)
            }
            Sequence::SemiGraphicCharacter(semi_graphic_character) => {
                Sequence::SemiGraphicCharacter(semi_graphic_character.consume(ctx, byte)?)
            }
            Sequence::SimpleCharacter(simple_character) => {
                Sequence::SimpleCharacter(simple_character.consume(ctx, byte)?)
            }
            Sequence::SubSection(None, None) if (0x40..=0x7F).contains(&byte) => {
                Sequence::SubSection(Some(byte - 0x40), None)
            }
            Sequence::SubSection(Some(x), None) if (0x40..=0x7F).contains(&byte) => {
                Sequence::SubSection(Some(*x), Some(byte - 0x40))
            }
            Sequence::Repeat(None) => {
                if (0x40..=0x7F).contains(&byte) {
                    Sequence::Repeat(Some(byte - 0x40))
                } else {
                    err!(
                        "Repeat sequence expects a number between 0x40 and 0x7F, got {:#04X}",
                        byte
                    )
                }
            }
            _ => err!(
                "Sequence {:?} does not support additional bytes ({:#04X})",
                self,
                byte
            ),
        };

        Ok(result)
    }

    fn is_complete(&self) -> bool {
        match self {
            Sequence::Incomplete => false,
            Sequence::Escaped(escaped_sequence) => escaped_sequence.is_complete(),
            Sequence::SetCharacterSet(character_set) => character_set.is_complete(),
            Sequence::SpecialCharacter(special_character) => special_character.is_complete(),
            Sequence::SemiGraphicCharacter(semi_graphic_character) => {
                semi_graphic_character.is_complete()
            }
            Sequence::SimpleCharacter(simple_character) => simple_character.is_complete(),
            Sequence::SubSection(Some(_), Some(_)) => true,
            Sequence::SubSection(_, _) => false,
            Sequence::Repeat(None) => false,
            _ => true,
        }
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct PendingAttributes {
    apply_on_delimiter: bool,
    background: Option<u8>,
    underline: Option<bool>, //+ caractère disjoint en mode semi-graphique
    mask: Option<bool>,
}

impl PendingAttributes {
    fn set_background(&mut self, background: u8) {
        self.background = Some(background);
        self.apply_on_delimiter = true;
    }

    fn set_underline(&mut self, underline: bool) {
        self.underline = Some(underline);
        self.apply_on_delimiter = true;
    }

    fn set_mask(&mut self, mask: bool) {
        self.mask = Some(mask);
        self.apply_on_delimiter = true;
    }

    fn should_apply(&self) -> bool {
        self.apply_on_delimiter
    }

    fn mark_applied(&mut self) {
        self.apply_on_delimiter = false;
    }

    fn reset(&mut self) {
        *self = PendingAttributes::default();
    }
}

#[derive(Copy, Clone)]
pub struct Attributes {
    pub character_set: CharacterSet,
    pub background: u8,
    pub foreground: u8,
    pub underline: bool, //+ caractère disjoint en mode semi-graphique
    pub blinking: bool,
    pub double_height: bool,
    pub double_width: bool,
    pub invert: bool,
    pub mask: bool,
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            character_set: CharacterSet::G0,
            background: BLACK,
            foreground: WHITE,
            underline: false,
            blinking: false,
            double_height: false,
            double_width: false,
            invert: false,
            mask: false,
        }
    }
}

impl Attributes {
    /// Returns true if the provided character is considered as a delimiter
    fn apply_pending(&mut self, character: char, pending: &mut PendingAttributes) -> bool {
        if self.character_set == CharacterSet::G1 {
            self.background = pending.background.unwrap_or(self.background);
            self.underline = pending.underline.unwrap_or(self.underline);
        }

        let mut anything_applied = false;

        if character == ' ' && pending.should_apply() {
            if let Some(background) = pending.background {
                self.background = background;
                anything_applied = true;
            }

            if let Some(underline) = pending.underline {
                self.underline = underline;
                anything_applied = true;
            }

            if let Some(mask) = pending.mask {
                self.mask = mask;
                anything_applied = true;
            }
        }

        if anything_applied {
            pending.mark_applied();
        }

        anything_applied
    }
    //todo: check when in semi-graphic mode, zone attributs are not the same
    fn copy_zone_attributes(&mut self, other: &Attributes) {
        self.background = other.background;
        self.mask = other.mask;
        self.underline = self.character_set == other.character_set && other.underline;
    }

    fn reset_zone_attributes(&mut self) {
        if self.character_set == CharacterSet::G0 {
            self.background = BLACK;
            self.underline = false;
        }

        self.mask = false;
    }

    fn reset(&mut self) {
        *self = Attributes::default();
    }
}

impl Debug for Attributes {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "Attr({:?}, Background({}), Foreground({})",
            self.character_set, self.background, self.foreground
        )?;

        if self.underline {
            f.write_str(", underline")?;
        }

        if self.blinking {
            f.write_str(", blinking")?;
        }

        if self.double_height {
            f.write_str(", double height")?;
        }

        if self.double_width {
            f.write_str(", double width")?;
        }

        if self.invert {
            f.write_str(", invert")?;
        }

        if self.mask {
            f.write_str(", mask")?;
        }

        f.write_char(')')
    }
}

#[derive(Default, Copy, Clone)]
pub struct Cell {
    pub content: char,
    pub is_delimiter: bool,
    pub attributes: Attributes,
}

impl Cell {
    fn set_content(&mut self, content: char) {
        self.content = content;
    }

    fn set_delimiter(&mut self, is_delimiter: bool) {
        self.is_delimiter = is_delimiter;
    }

    fn set_attributes(&mut self, attributes: Attributes) {
        self.attributes = attributes;
    }

    fn reset(&mut self) {
        self.set_content('\0');
        self.set_delimiter(false);
        self.set_attributes(Attributes::default());
    }

    fn delimiter(background: u8) -> Self {
        let mut cell = Cell::default();
        cell.set_content(' ');
        cell.set_delimiter(true);
        cell.set_attributes(Attributes {
            character_set: CharacterSet::G1,
            background,
            foreground: BLACK,
            ..Attributes::default()
        });

        cell
    }

    fn space(attributes: Attributes, is_delimiter: bool) -> Self {
        let mut cell = Cell::default();
        cell.set_content(' ');
        cell.set_delimiter(is_delimiter);
        cell.set_attributes(attributes);

        cell
    }
}

impl Debug for Cell {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let name = if self.is_delimiter { "Delim" } else { "Cell" };

        if self.attributes.character_set == CharacterSet::G1 {
            write!(
                f,
                "{}({:#04X}, {:?})",
                name, self.content as u8, self.attributes
            )
        } else {
            write!(f, "{}('{}', {:?})", name, self.content, self.attributes)
        }
    }
}

#[derive(Debug)]
pub struct Grid {
    width: usize,
    height: usize,
    data: Vec<Cell>,
}

impl Grid {
    pub fn new(width: u8, height: u8) -> Self {
        let width = width as usize;
        let height = height as usize;

        Self {
            width,
            height,
            data: vec![Cell::delimiter(BLACK); width * height],
        }
    }

    pub fn cell(&self, x: u8, y: u8) -> &Cell {
        assert!(
            x >= 1 && x <= self.width as u8,
            "Invalid x coordinate {}",
            x
        );
        assert!(
            y >= 1 && y <= self.height as u8,
            "Invalid y coordinate {}",
            y
        );

        &self.data[(y as usize - 1) * self.width + x as usize - 1]
    }

    pub fn previous_cell(&self, x: u8, y: u8) -> Option<&Cell> {
        assert!(
            x >= 1 && x <= self.width as u8,
            "Invalid x coordinate {}",
            x
        );
        assert!(
            y >= 1 && y <= self.height as u8,
            "Invalid y coordinate {}",
            y
        );

        let offset = (y as usize - 1) * self.width + x as usize - 1;
        if offset == 0 {
            None
        } else {
            Some(&self.data[offset - 1])
        }
    }

    pub fn cell_opt(&self, x: u8, y: u8) -> Option<&Cell> {
        if x == 0 || y == 0 || x > self.width as u8 || y > self.height as u8 {
            return None;
        }

        self.data
            .get((y as usize - 1) * self.width + x as usize - 1)
    }

    fn cell_mut(&mut self, x: u8, y: u8) -> &mut Cell {
        assert!(
            x >= 1 && x <= self.width as u8,
            "Invalid x coordinate {}",
            x
        );
        assert!(
            y >= 1 && y <= self.height as u8,
            "Invalid y coordinate {}",
            y
        );

        &mut self.data[(y as usize - 1) * self.width + x as usize - 1]
    }

    fn reset(&mut self) {
        self.data
            .iter_mut()
            .for_each(|cell| *cell = Cell::delimiter(BLACK));
    }

    fn scroll(&mut self, count: i8) {
        if count == 0 {
            return;
        }

        let cells_to_scroll = count.unsigned_abs() as usize * self.width;

        if count > 0 {
            self.data.splice(0..cells_to_scroll, vec![]);
            self.data.splice(
                self.data.len()..,
                vec![Cell::delimiter(BLACK); cells_to_scroll],
            );
        } else {
            self.data
                .splice((self.data.len() - cells_to_scroll).., vec![]);
            self.data
                .splice(0..0, vec![Cell::delimiter(BLACK); cells_to_scroll]);
        }
    }

    fn clear_before(&mut self, x: u8, y: u8) {
        self.clear_range((1, 1), (x, y));
    }

    fn clear_after(&mut self, x: u8, y: u8) {
        self.clear_range((x, y), (self.width as u8, self.height as u8));
    }

    fn clear_row(&mut self, y: u8) {
        self.clear_range((1, y), (self.width as u8, y));
    }

    fn clear_in_row_before(&mut self, x: u8, y: u8) {
        self.clear_range((1, y), (x, y));
    }

    fn clear_in_row_after(&mut self, x: u8, y: u8) {
        self.clear_range((x, y), (self.width as u8, y));
    }

    fn insert_after(&mut self, count: u8, x: u8, y: u8, attributes: Attributes) {
        let row_start = (y as usize - 1) * self.width;
        let row_end = row_start + self.width;
        let max_insert = self.width as u8 - (x - 1);
        let count = count.min(max_insert);

        if count == 0 {
            return;
        }

        let mut row = self.data[row_start..row_end].to_vec();
        let insert_at = x as usize - 1;
        row.splice(
            insert_at..insert_at,
            vec![Cell::space(attributes, false); count as usize],
        );
        row.truncate(self.width);

        self.data.splice(row_start..row_end, row);
    }

    fn delete_after(&mut self, count: u8, x: u8, y: u8, attributes: Attributes) {
        let row_start = (y as usize - 1) * self.width;
        let row_end = row_start + self.width;
        let max_delete = self.width as u8 - (x - 1);
        let count = count.min(max_delete);

        if count == 0 {
            return;
        }

        let mut row = self.data[row_start..row_end].to_vec();
        let delete_from = x as usize - 1;
        row.drain(delete_from..delete_from + count as usize);
        row.extend(std::iter::repeat_n(
            Cell::space(attributes, false),
            count as usize,
        ));
        row.truncate(self.width);

        self.data.splice(row_start..row_end, row);
    }

    fn insert_rows_after(&mut self, count: u8, y: u8) {
        //don't -1 to y because we're inserting after the given row
        let start = (y as usize) * self.width;
        self.data.splice(
            start..start,
            vec![Cell::delimiter(BLACK); count as usize * self.width],
        );
        self.data.truncate(self.width * self.height);
    }

    fn delete_rows_after(&mut self, count: u8, y: u8) {
        //don't -1 to y because we're deleting after the given row
        let start = (y as usize) * self.width;
        let end = start + count as usize * self.width;
        self.data.splice(start..end, vec![]);
        self.data
            .resize(self.width * self.height, Cell::delimiter(BLACK));
    }

    fn recalculate_row_attributes(&mut self, x: u8, y: u8) {
        let start = (y as usize - 1) * self.width + x as usize - 1;
        let end = start + self.width - x as usize;

        let mut attributes = if let Some(previous) = self.cell_opt(x - 1, y) {
            previous.attributes
        } else {
            let cell = self.cell_mut(x, y);
            cell.attributes.reset_zone_attributes();

            cell.attributes
        };

        for cell in &mut self.data[start..=end] {
            if cell.content == '\0' {
                cell.reset();
                attributes.reset_zone_attributes();
            } else if cell.is_delimiter || cell.attributes.character_set == CharacterSet::G1 {
                attributes.copy_zone_attributes(&cell.attributes);
            } else {
                cell.attributes.copy_zone_attributes(&attributes);
            }
        }
    }

    /// Clears all cells between the inclusive given coordinates
    fn clear_range(&mut self, (x1, y1): (u8, u8), (x2, y2): (u8, u8)) {
        self.apply_range((x1, y1), (x2, y2), |cell| *cell = Cell::delimiter(BLACK));
    }

    fn apply_range(&mut self, (x1, y1): (u8, u8), (x2, y2): (u8, u8), f: impl Fn(&mut Cell)) {
        self.range_mut((x1, y1), (x2, y2)).for_each(f);
    }

    fn range_mut(
        &mut self,
        (x1, y1): (u8, u8),
        (x2, y2): (u8, u8),
    ) -> impl Iterator<Item = &mut Cell> {
        let start = (y1 - 1) as usize * self.width + (x1 - 1) as usize;
        let end = (y2 - 1) as usize * self.width + (x2 - 1) as usize;

        self.data[start..=end].iter_mut()
    }
}

#[derive(Debug)]
pub struct Context {
    pub display_component: DisplayComponent,
    pub screen_width: u8,
    pub screen_height: u8,

    pub attributes: Attributes,
    pub page_mode: PageMode,
    pub cursor_x: u8,
    pub cursor_y: u8,
    pub visible_cursor: bool,
    pub ignore_sequences: bool,
    pub screen_mask: bool,
    pub insert: bool,

    pub grid: Grid,
    pub pending_attributes: PendingAttributes,
    saved_state_for_row_zero: Option<SavedState>,

    response: Vec<u8>,
    beep: bool,
}

#[derive(Copy, Clone, Debug)]
struct SavedState {
    cursor_x: u8,
    cursor_y: u8,
    attributes: Attributes,
    pending_attributes: PendingAttributes,
}

impl Context {
    pub fn new(display_component: DisplayComponent) -> Self {
        Self {
            display_component,
            screen_width: 40,
            screen_height: 24,

            //defaults are documented page 87 and 88
            attributes: Attributes::default(),
            page_mode: PageMode::Page,
            cursor_x: 1,
            cursor_y: 1,
            visible_cursor: false,
            ignore_sequences: false,
            screen_mask: true,
            insert: false,

            grid: Grid::new(40, 24),
            pending_attributes: PendingAttributes::default(),
            saved_state_for_row_zero: None,

            response: Vec::new(),
            beep: false,
        }
    }

    fn consume(&mut self, sequence: &Sequence) -> Result<(), Error> {
        if self.ignore_sequences
            && !matches!(
                sequence,
                Sequence::Escaped(EscapedSequence::Ignore(Some(false)))
            )
        {
            return Ok(());
        }

        match sequence {
            Sequence::Incomplete => {}
            Sequence::Escaped(esc) => match esc {
                EscapedSequence::Background(color) => {
                    self.pending_attributes.set_background(*color)
                }
                EscapedSequence::Foreground(color) => self.attributes.foreground = *color,
                EscapedSequence::Csi(csi) => match csi {
                    //these four sequences do not support wrapping and the cursor will just
                    //stop moving when it reaches the border of the screen (p94 and 95)
                    //todo: check if we need to copy zone attributes as well ? probably need to copy on all operations ?
                    Csi::MoveUp(offset) => self.move_cursor_y(-(*offset as i8), false)?,
                    Csi::MoveDown(offset) => self.move_cursor_y(*offset as i8, false)?,
                    Csi::MoveRight(offset) => self.move_cursor_x(*offset as i8, false)?,
                    Csi::MoveLeft(offset) => self.move_cursor_x(-(*offset as i8), false)?,

                    Csi::SetCursor(x, y) => {
                        self.set_cursor(*x, *y, false)?;
                    }
                    Csi::InsertRowsFromCursor(count) => {
                        self.grid.insert_rows_after(*count, self.cursor_y)
                    }
                    Csi::EraseRowsFromCursor(count) => {
                        self.grid.delete_rows_after(*count, self.cursor_y)
                    }
                    //todo: check if attributes and pending attributes are reset?
                    Csi::ClearRow => self.grid.clear_row(self.cursor_y),
                    Csi::InsertSpacesFromCursorToEol => self.fill_with_spaces_to_eol(),
                    Csi::ClearFromCursorToEos => {
                        self.grid.clear_after(self.cursor_x, self.cursor_y)
                    }
                    Csi::ClearFromSosToCursor => {
                        self.grid.clear_before(self.cursor_x, self.cursor_y)
                    }
                    Csi::ClearScreenKeepCursorPos => self.grid.reset(),
                    Csi::ClearFromCursorToEol => {
                        self.grid.clear_in_row_after(self.cursor_x, self.cursor_y)
                    }
                    Csi::ClearFromSolToCursor => {
                        self.grid.clear_in_row_before(self.cursor_x, self.cursor_y)
                    }
                    Csi::ClearAfterCursor(count) => {
                        self.grid.delete_after(
                            *count,
                            self.cursor_x,
                            self.cursor_y,
                            self.attributes,
                        );
                        self.grid.recalculate_row_attributes(1, self.cursor_y);
                    }
                    Csi::InsertFromCursor(count) => {
                        self.grid.insert_after(
                            *count,
                            self.cursor_x,
                            self.cursor_y,
                            self.attributes,
                        );
                        self.grid.recalculate_row_attributes(1, self.cursor_y);
                    }
                    Csi::StartInsert => self.insert = true,
                    Csi::EndInsert => self.insert = false,
                    _ => err!("Received incomplete CSI sequence {:?}", csi),
                },
                EscapedSequence::NormalSize
                | EscapedSequence::DoubleSize
                | EscapedSequence::DoubleHeight
                | EscapedSequence::DoubleWidth
                | EscapedSequence::Invert(_)
                    if self.attributes.character_set == CharacterSet::G1 =>
                {
                    err!("Changing invert and character size while in G1 is not supported");
                }
                EscapedSequence::Blink(blink) => self.attributes.blinking = *blink,
                EscapedSequence::Invert(invert) => self.attributes.invert = *invert,
                EscapedSequence::NormalSize => {
                    self.attributes.double_height = false;
                    self.attributes.double_width = false;
                }
                EscapedSequence::DoubleHeight | EscapedSequence::DoubleSize
                    if self.cursor_y <= 1 =>
                {
                    err!("Tried to set double height or double size while in row 0 or 1");
                }
                EscapedSequence::DoubleHeight => self.attributes.double_height = true,
                EscapedSequence::DoubleWidth => self.attributes.double_width = true,
                EscapedSequence::DoubleSize => {
                    self.attributes.double_height = true;
                    self.attributes.double_width = true;
                }
                EscapedSequence::Underline(underline) => {
                    self.pending_attributes.set_underline(*underline)
                }
                EscapedSequence::Mask(mask) => self.pending_attributes.set_mask(*mask),
                EscapedSequence::Ignore(Some(ignore)) => self.ignore_sequences = *ignore,
                EscapedSequence::Protocol(pro) => match pro {
                    //todo: impl missing sequences
                    Protocol::Reset => *self = Self::new(self.display_component),
                    Protocol::RequestSpeed => {}
                    Protocol::SetSpeed(_) => {}
                    //todo: not actually parse this but the response
                    Protocol::Scroll(scroll) => {
                        if *scroll {
                            self.page_mode = PageMode::Scroll;
                        } else {
                            self.page_mode = PageMode::Page;
                        }
                    }
                    Protocol::Sleep(_) => {}
                    _ => err!("Received incomplete protocol sequence {:?}", pro),
                },
                EscapedSequence::GetCursorPosition => {
                    self.response.push(US);
                    self.response.push(0x40 + self.cursor_y);
                    self.response.push(0x40 + self.cursor_x);
                }
                EscapedSequence::ScreenMasking(mask) => self.screen_mask = *mask,
                _ => err!("Received incomplete escaped sequence {:?}", esc),
            },
            Sequence::SetCharacterSet(set) => {
                self.attributes.character_set = *set;
                self.attributes.underline = false; //p94 todo: on peut supprimer?
                self.pending_attributes.underline = None; //p94

                //documented p91
                if self.attributes.character_set == CharacterSet::G1 {
                    self.attributes.invert = false;
                    self.attributes.double_height = false;
                    self.attributes.double_width = false;
                }
            }
            Sequence::SpecialCharacter(character) => self.print(character.to_character()?)?,
            Sequence::SimpleCharacter(character) => self.print(character.to_character()?)?,
            Sequence::SemiGraphicCharacter(character) => self.print(character.0 as char)?,
            Sequence::MoveCursor(direction) => match direction {
                Direction::Up => self.move_cursor_y(-1, true)?,
                Direction::Down => self.move_cursor_y(1, true)?,
                Direction::Right => self.move_cursor_x(1, true)?,
                Direction::Left => self.move_cursor_x(-1, true)?,
            },
            //todo: check if we should reset attributes or not
            Sequence::CarriageReturn => self.cursor_x = 1,
            Sequence::RecordSeparator => {
                self.set_cursor(1, 1, false)?;
                self.reset_attributes();
            }
            Sequence::ClearScreen => {
                self.set_cursor(1, 1, false)?;
                self.reset_screen();
            }
            Sequence::SubSection(row, column) => {
                //the only way to access row 0 documented p97
                let row = row.unwrap();
                let column = column.unwrap();

                if row == 0 && self.cursor_y != 0 {
                    self.saved_state_for_row_zero = Some(SavedState {
                        cursor_x: self.cursor_x,
                        cursor_y: self.cursor_y,
                        attributes: self.attributes,
                        pending_attributes: self.pending_attributes,
                    });
                }

                self.set_cursor(column, row, true)?;
                self.reset_attributes();
            }
            Sequence::Repeat(value) => {
                if let Some(previous_char) = self
                    .grid
                    .previous_cell(self.cursor_x, self.cursor_y)
                    .map(|cell| cell.content)
                {
                    if previous_char == '\0' {
                        err!("Tried to repeat a character but no character was present");
                    }

                    for _ in 0..value.unwrap() {
                        self.print(previous_char)?;
                    }
                } else {
                    err!("Tried to repeat a character at the beginning of the screen");
                }
            }
            Sequence::VisibleCursor(value) => self.visible_cursor = *value,
            Sequence::ErrorCharacter => self.print('\u{7F}')?,
            Sequence::Beep => self.beep = true,
        };

        Ok(())
    }

    fn set_cursor(&mut self, x: u8, y: u8, allow_row_zero: bool) -> Result<(), Error> {
        let minimum_y = if allow_row_zero { 0 } else { 1 };

        if x > self.screen_width || x < 1 || y > self.screen_height || y < minimum_y {
            err!("Tried to move cursor outside of screen ({}, {})", x, y);
        }

        self.cursor_x = x;
        self.cursor_y = y;

        //copy new position's zone attributes
        if let Some(cell) = self.grid.cell_opt(x - 1, y) {
            self.attributes.copy_zone_attributes(&cell.attributes);
        } else {
            self.attributes.reset_zone_attributes();
        }

        Ok(())
    }

    /// Moves the cursor horizontally by the given amount of characters.
    /// If the cursor reaches the end of the screen, it will wrap to the
    /// next line if `wrap` is set to `true` else it will stay at the border.
    /// This function does not allow moving in row 0 (not yet implemented, p97)
    fn move_cursor_x(&mut self, x: i8, wrap: bool) -> Result<(), Error> {
        if self.cursor_y == 0 {
            self.cursor_x = (self.cursor_x as i8 + x).clamp(1, self.screen_width as i8) as u8;
            return Ok(());
        }

        if !wrap {
            self.cursor_x = (self.cursor_x as i8 + x).clamp(1, self.screen_width as i8) as u8;
            return Ok(());
        }

        let x_translation_total = self.cursor_x as i16 + x as i16 - 1;
        let y_offset = x_translation_total.div_euclid(self.screen_width as i16);
        let new_x = x_translation_total.rem_euclid(self.screen_width as i16) + 1;
        let new_y = self.cursor_y as i16 + y_offset;

        self.cursor_x = new_x as u8;
        if self.page_mode == PageMode::Scroll {
            let to_scroll = if new_y < 1 {
                new_y - 1
            } else if new_y > self.screen_height as i16 {
                new_y - self.screen_height as i16
            } else {
                0
            } as i8;

            self.grid.scroll(to_scroll);
            self.cursor_y = new_y.clamp(1, self.screen_height as i16) as u8;
        } else {
            self.cursor_y = (new_y - 1).rem_euclid(self.screen_height as i16) as u8 + 1;
        }

        if y_offset != 0 {
            self.attributes.reset_zone_attributes();
        }

        Ok(())
    }

    /// Moves the cursor vertically by the given amount of characters.
    /// If the cursor reaches the end of the screen, it will wrap to the
    /// other side by staying in the same column if `wrap` is set to `true`
    /// else it will stay at the border.
    /// This function does not allow accessing row 0 however it should
    /// allow getting out of row 0 (not yet implemented, p97)
    fn move_cursor_y(&mut self, y: i8, wrap: bool) -> Result<(), Error> {
        if self.cursor_y == 0 {
            if y > 0 {
                self.exit_row_zero()?;
            }

            return Ok(());
        }

        if !wrap {
            self.cursor_y = (self.cursor_y as i8 + y).clamp(1, self.screen_height as i8) as u8;
            return Ok(());
        }

        let new_y = self.cursor_y as i16 + y as i16;

        if self.page_mode == PageMode::Scroll {
            let scroll_amount = if new_y < 1 {
                -new_y + 1
            } else if new_y > self.screen_height as i16 {
                new_y - self.screen_height as i16
            } else {
                0
            };

            self.grid.scroll(scroll_amount as i8);
            self.cursor_y = new_y.clamp(1, self.screen_height as i16) as u8;
        } else {
            self.cursor_y = new_y.rem_euclid(self.screen_height as i16) as u8;
        }

        self.attributes.reset_zone_attributes();

        Ok(())
    }

    fn fill_with_spaces_to_eol(&mut self) {
        let y = self.cursor_y;
        let x = self.cursor_x;
        let attributes = self.attributes;

        for column in x..=self.screen_width {
            let cell = self.grid.cell_mut(column, y);
            *cell = Cell::space(attributes, false);
        }
    }

    fn print(&mut self, character: char) -> Result<(), Error> {
        if self.insert {
            let width = if self.attributes.double_width { 2 } else { 1 };
            self.grid
                .insert_after(width, self.cursor_x, self.cursor_y, self.attributes);
        }

        let is_delimiter = self
            .attributes
            .apply_pending(character, &mut self.pending_attributes);

        // if self.attributes.double_width && self.cursor_x == self.screen_width {
        //     self.move_cursor_x(1, true);
        // }

        let cell = self.grid.cell_mut(self.cursor_x, self.cursor_y);
        cell.set_content(character);
        cell.set_delimiter(is_delimiter);
        cell.set_attributes(self.attributes);

        self.grid
            .recalculate_row_attributes(self.cursor_x, self.cursor_y);

        self.next(1)?;

        Ok(())
    }

    /// Moves the cursor to the right by the given amount of characters.
    /// If the cursor reaches the end of the screen, it will wrap to the next line.
    fn next(&mut self, amount: u8) -> Result<(), Error> {
        //todo: check how double width and height behave when multiline
        //todo: check zone attributes with double width and height
        let amount = if self.attributes.double_width {
            amount * 2
        } else {
            amount
        } as i8;

        self.move_cursor_x(amount, true)
    }

    fn reset_screen(&mut self) {
        self.grid.reset();
        self.attributes.reset();
        self.pending_attributes.reset();
        self.saved_state_for_row_zero = None;
    }

    fn reset_attributes(&mut self) {
        self.attributes.reset();
        self.pending_attributes.reset();
    }

    fn exit_row_zero(&mut self) -> Result<(), Error> {
        if let Some(saved_state) = self.saved_state_for_row_zero.take() {
            self.cursor_x = saved_state.cursor_x;
            self.cursor_y = saved_state.cursor_y;
            self.attributes = saved_state.attributes;
            self.pending_attributes = saved_state.pending_attributes;
        } else {
            self.set_cursor(1, 1, false)?;
            self.reset_attributes();
        }

        Ok(())
    }
}

pub struct Parser {
    ctx: Context,
    sequence: Sequence,
}

impl Parser {
    pub fn new(display_component: DisplayComponent) -> Self {
        Self {
            ctx: Context::new(display_component),
            sequence: Sequence::Incomplete,
        }
    }

    pub fn ctx(&self) -> &Context {
        &self.ctx
    }

    /// Returns and clears any pending protocol response bytes.
    pub fn take_response(&mut self) -> Vec<u8> {
        mem::take(&mut self.ctx.response)
    }

    /// Returns true if there are pending protocol response bytes.
    pub fn has_response(&self) -> bool {
        !self.ctx.response.is_empty()
    }

    /// Returns true if a beep was triggered since the last call,
    /// and clears the flag.
    pub fn take_beep(&mut self) -> bool {
        mem::replace(&mut self.ctx.beep, false)
    }

    pub fn consume(&mut self, byte: u8) -> Result<(), Error> {
        let mut result = || {
            self.sequence = self.sequence.consume(&self.ctx, byte)?;

            if self.sequence.is_complete() {
                let complete_sequence = mem::replace(&mut self.sequence, Sequence::Incomplete);
                self.ctx.consume(&complete_sequence)?;
            }

            Ok(())
        };

        if cfg!(feature = "strict") {
            result().or_else(|err| {
                if self.sequence != Sequence::Incomplete && byte < 0x20 && byte != 0x00 {
                    self.sequence = Sequence::Incomplete;
                    self.consume(byte)
                } else {
                    Err(err)
                }
            })
        } else {
            if let Err(err) = result() {
                if self.sequence != Sequence::Incomplete && byte < 0x20 && byte != 0x00 {
                    self.sequence = Sequence::Incomplete;
                    return self.consume(byte);
                }

                //todo: use logging library
                eprintln!("{}", err);
            }

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_set() {
        let ctx = Context::new(DisplayComponent::VGP2);

        assert!(!CharacterSet::supports(&ctx, 0x00));
        assert!(!CharacterSet::supports(&ctx, 0x7F));
        assert!(!CharacterSet::supports(&ctx, 0xFF));
        assert!(!CharacterSet::supports(&ctx, 0x1F));
        assert!(!CharacterSet::supports(&ctx, 0x1E));
        assert!(!CharacterSet::supports(&ctx, 0x0D));
        assert!(CharacterSet::supports(&ctx, 0x0F));
        assert!(CharacterSet::supports(&ctx, 0x0E));

        assert_eq!(CharacterSet::new(&ctx, 0x0F), Ok(CharacterSet::G0));
        assert_eq!(CharacterSet::new(&ctx, 0x0E), Ok(CharacterSet::G1));

        assert!(CharacterSet::G0.is_complete());
        assert!(CharacterSet::G1.is_complete());

        assert_eq!(
            CharacterSet::new(&ctx, SI).unwrap().consume(&ctx, 0x00),
            Err(Error::new(
                "Character set G0 does not support more bytes (0x00)"
            )),
        );

        assert_eq!(
            CharacterSet::new(&ctx, SI).unwrap().consume(&ctx, 0x0E),
            Err(Error::new(
                "Character set G0 does not support more bytes (0x0E)"
            )),
        );

        assert_eq!(
            CharacterSet::new(&ctx, SO).unwrap().consume(&ctx, 0x00),
            Err(Error::new(
                "Character set G1 does not support more bytes (0x00)"
            )),
        );

        assert_eq!(
            CharacterSet::new(&ctx, SO).unwrap().consume(&ctx, 0x0F),
            Err(Error::new(
                "Character set G1 does not support more bytes (0x0F)"
            )),
        );
    }

    #[test]
    fn test_simple_character() {
        let mut ctx = Context::new(DisplayComponent::VGP2);
        ctx.attributes.character_set = CharacterSet::G0;

        assert_eq!(SimpleCharacter::new(&ctx, 0x20), Ok(SimpleCharacter(0x20)));
        assert_eq!(SimpleCharacter::new(&ctx, 0x3F), Ok(SimpleCharacter(0x3F)));
        assert_eq!(SimpleCharacter::new(&ctx, 0x4A), Ok(SimpleCharacter(0x4A)));
        assert_eq!(SimpleCharacter::new(&ctx, 0x60), Ok(SimpleCharacter(0x60)));
        assert_eq!(SimpleCharacter::new(&ctx, 0x7F), Ok(SimpleCharacter(0x7F)));

        assert!(!SimpleCharacter::supports(&ctx, 0x00));
        assert!(!SimpleCharacter::supports(&ctx, 0x1F));
        assert!(!SimpleCharacter::supports(&ctx, 0x8A));
        assert!(SimpleCharacter::supports(&ctx, 0x20));
        assert!(SimpleCharacter::supports(&ctx, 0x7F));

        assert!(SimpleCharacter::new(&ctx, 0x20).unwrap().is_complete());
        assert!(SimpleCharacter::new(&ctx, 0x7F).unwrap().is_complete());
    }

    #[test]
    fn test_simple_character_wrong_set() {
        let mut ctx = Context::new(DisplayComponent::VGP2);
        ctx.attributes.character_set = CharacterSet::G1;

        assert!(!SimpleCharacter::supports(&ctx, 0x00));
        assert!(!SimpleCharacter::supports(&ctx, 0x1F));
        assert!(!SimpleCharacter::supports(&ctx, 0x20));
        assert!(!SimpleCharacter::supports(&ctx, 0x3F));

        assert_err!(
            SimpleCharacter::new(&ctx, 0x20),
            "Invalid simple character 0x20"
        );
        assert_err!(
            SimpleCharacter::new(&ctx, 0x7F),
            "Invalid simple character 0x7F"
        );
        assert_err!(
            SimpleCharacter::new(&ctx, 0x4A),
            "Invalid simple character 0x4A"
        );
        assert_err!(
            SimpleCharacter::new(&ctx, 0x60),
            "Invalid simple character 0x60"
        );
    }

    #[test]
    fn test_semigraphic_character() {
        let mut ctx = Context::new(DisplayComponent::VGP2);
        ctx.attributes.character_set = CharacterSet::G1;

        assert_eq!(
            SemiGraphicCharacter::new(&ctx, 0x20),
            Ok(SemiGraphicCharacter(0x20))
        );
        assert_eq!(
            SemiGraphicCharacter::new(&ctx, 0x3F),
            Ok(SemiGraphicCharacter(0x3F))
        );
        assert_eq!(
            SemiGraphicCharacter::new(&ctx, 0x60),
            Ok(SemiGraphicCharacter(0x60))
        );
        assert_eq!(
            SemiGraphicCharacter::new(&ctx, 0x5F),
            Ok(SemiGraphicCharacter(0x5F))
        );
        assert_eq!(
            SemiGraphicCharacter::new(&ctx, 0x40),
            Ok(SemiGraphicCharacter(0x60))
        );
        assert_eq!(
            SemiGraphicCharacter::new(&ctx, 0x55),
            Ok(SemiGraphicCharacter(0x75))
        );
        assert_eq!(
            SemiGraphicCharacter::new(&ctx, 0x7F),
            Ok(SemiGraphicCharacter(0x5F))
        );

        assert!(!SemiGraphicCharacter::supports(&ctx, 0x00));
        assert!(!SemiGraphicCharacter::supports(&ctx, 0x1F));
        assert!(!SemiGraphicCharacter::supports(&ctx, 0x8A));
        assert!(SemiGraphicCharacter::supports(&ctx, 0x5F));
        assert!(SemiGraphicCharacter::supports(&ctx, 0x60));
        assert!(SemiGraphicCharacter::supports(&ctx, 0x7F));

        assert!(SemiGraphicCharacter::new(&ctx, 0x20).unwrap().is_complete());
        assert!(SemiGraphicCharacter::new(&ctx, 0x7F).unwrap().is_complete());
    }

    #[test]
    fn test_semigraphic_character_wrong_set() {
        let mut ctx = Context::new(DisplayComponent::VGP2);
        ctx.attributes.character_set = CharacterSet::G0;

        assert!(!SemiGraphicCharacter::supports(&ctx, 0x00));
        assert!(!SemiGraphicCharacter::supports(&ctx, 0x1F));
        assert!(!SemiGraphicCharacter::supports(&ctx, 0x45));
        assert!(!SemiGraphicCharacter::supports(&ctx, 0x5F));
        assert!(!SemiGraphicCharacter::supports(&ctx, 0x20));
        assert!(!SemiGraphicCharacter::supports(&ctx, 0x3F));

        assert_err!(
            SemiGraphicCharacter::new(&ctx, 0x20),
            "Invalid semi-graphic character 0x20"
        );
        assert_err!(
            SemiGraphicCharacter::new(&ctx, 0x3F),
            "Invalid semi-graphic character 0x3F"
        );
        assert_err!(
            SemiGraphicCharacter::new(&ctx, 0x45),
            "Invalid semi-graphic character 0x45"
        );
        assert_err!(
            SemiGraphicCharacter::new(&ctx, 0x5F),
            "Invalid semi-graphic character 0x5F"
        );
        assert_err!(
            SemiGraphicCharacter::new(&ctx, 0x60),
            "Invalid semi-graphic character 0x60"
        );
        assert_err!(
            SemiGraphicCharacter::new(&ctx, 0x7F),
            "Invalid semi-graphic character 0x7F"
        );
    }

    #[test]
    fn test_special_character() {
        let ctx = Context::new(DisplayComponent::VGP2);

        assert_eq!(
            SpecialCharacter::new(&ctx, 0x19),
            Ok(SpecialCharacter::Incomplete)
        );

        assert!(SpecialCharacter::supports(&ctx, 0x19));
        assert!(!SpecialCharacter::supports(&ctx, 0x1B));
        assert!(!SpecialCharacter::supports(&ctx, 0x00));
        assert!(!SpecialCharacter::supports(&ctx, 0x1F));
        assert!(!SpecialCharacter::supports(&ctx, 0x7F));

        assert_eq!(
            SpecialCharacter::new(&ctx, 0x19),
            Ok(SpecialCharacter::Incomplete)
        );

        assert_err!(
            SpecialCharacter::new(&ctx, 0x00),
            "Invalid special character 0x00"
        );
        assert_err!(
            SpecialCharacter::new(&ctx, 0x1B),
            "Invalid special character 0x1B"
        );
        assert_err!(
            SpecialCharacter::new(&ctx, 0x1F),
            "Invalid special character 0x1F"
        );

        assert_eq!(
            SpecialCharacter::new(&ctx, 0x19)
                .unwrap()
                .consume(&ctx, GRAVE),
            Ok(SpecialCharacter::Grave(None))
        );
        assert_eq!(
            SpecialCharacter::new(&ctx, 0x19)
                .unwrap()
                .consume(&ctx, ACUTE),
            Ok(SpecialCharacter::Acute(None))
        );
        assert_eq!(
            SpecialCharacter::new(&ctx, 0x19)
                .unwrap()
                .consume(&ctx, CIRCUMFLEX),
            Ok(SpecialCharacter::Circumflex(None))
        );
        assert_eq!(
            SpecialCharacter::new(&ctx, 0x19)
                .unwrap()
                .consume(&ctx, DIAERESIS),
            Ok(SpecialCharacter::Diaeresis(None))
        );
        assert_eq!(
            SpecialCharacter::new(&ctx, 0x19)
                .unwrap()
                .consume(&ctx, CEDILLA),
            Ok(SpecialCharacter::Cedilla(None))
        );
        assert_eq!(
            SpecialCharacter::new(&ctx, 0x19)
                .unwrap()
                .consume(&ctx, LOWER_OE),
            Ok(SpecialCharacter::LowerOE)
        );
        assert_eq!(
            SpecialCharacter::new(&ctx, 0x19)
                .unwrap()
                .consume(&ctx, UPPER_OE),
            Ok(SpecialCharacter::UpperOE)
        );
        assert_eq!(
            SpecialCharacter::new(&ctx, 0x19)
                .unwrap()
                .consume(&ctx, POUND),
            Ok(SpecialCharacter::Pound)
        );
        assert_eq!(
            SpecialCharacter::new(&ctx, 0x19)
                .unwrap()
                .consume(&ctx, DOLLAR),
            Ok(SpecialCharacter::Dollar)
        );
        assert_eq!(
            SpecialCharacter::new(&ctx, 0x19)
                .unwrap()
                .consume(&ctx, NUMBER_SIGN),
            Ok(SpecialCharacter::NumberSign)
        );
        assert_eq!(
            SpecialCharacter::new(&ctx, 0x19)
                .unwrap()
                .consume(&ctx, ARROW_LEFT),
            Ok(SpecialCharacter::ArrowLeft)
        );
        assert_eq!(
            SpecialCharacter::new(&ctx, 0x19)
                .unwrap()
                .consume(&ctx, ARROW_UP),
            Ok(SpecialCharacter::ArrowUp)
        );
        assert_eq!(
            SpecialCharacter::new(&ctx, 0x19)
                .unwrap()
                .consume(&ctx, ARROW_RIGHT),
            Ok(SpecialCharacter::ArrowRight)
        );
        assert_eq!(
            SpecialCharacter::new(&ctx, 0x19)
                .unwrap()
                .consume(&ctx, ARROW_DOWN),
            Ok(SpecialCharacter::ArrowDown)
        );
        assert_eq!(
            SpecialCharacter::new(&ctx, 0x19)
                .unwrap()
                .consume(&ctx, DEGREE),
            Ok(SpecialCharacter::Degree)
        );
        assert_eq!(
            SpecialCharacter::new(&ctx, 0x19)
                .unwrap()
                .consume(&ctx, PLUS_OR_MINUS),
            Ok(SpecialCharacter::PlusOrMinus)
        );
        assert_eq!(
            SpecialCharacter::new(&ctx, 0x19)
                .unwrap()
                .consume(&ctx, OBELUS),
            Ok(SpecialCharacter::Obelus)
        );
        assert_eq!(
            SpecialCharacter::new(&ctx, 0x19)
                .unwrap()
                .consume(&ctx, ONE_QUARTER),
            Ok(SpecialCharacter::OneQuarter)
        );
        assert_eq!(
            SpecialCharacter::new(&ctx, 0x19)
                .unwrap()
                .consume(&ctx, ONE_HALF),
            Ok(SpecialCharacter::OneHalf)
        );
        assert_eq!(
            SpecialCharacter::new(&ctx, 0x19)
                .unwrap()
                .consume(&ctx, THREE_QUARTERS),
            Ok(SpecialCharacter::ThreeQuarters)
        );

        assert!(!SpecialCharacter::Grave(None).is_complete());
        assert!(!SpecialCharacter::Acute(None).is_complete());
        assert!(!SpecialCharacter::Circumflex(None).is_complete());
        assert!(!SpecialCharacter::Diaeresis(None).is_complete());
        assert!(!SpecialCharacter::Cedilla(None).is_complete());
        assert!(SpecialCharacter::LowerOE.is_complete());
        assert!(SpecialCharacter::ThreeQuarters.is_complete());
    }

    #[test]
    fn test_special_character_vgp2_specific() {
        let ctx = Context::new(DisplayComponent::VGP2);
        assert_eq!(
            SpecialCharacter::Grave(None).consume(&ctx, b'a'),
            Ok(SpecialCharacter::Grave(Some(b'a')))
        );
        assert_eq!(
            SpecialCharacter::Acute(None).consume(&ctx, b'e'),
            Ok(SpecialCharacter::Acute(Some(b'e')))
        );
        assert_eq!(
            SpecialCharacter::Circumflex(None).consume(&ctx, b'o'),
            Ok(SpecialCharacter::Circumflex(Some(b'o')))
        );
        assert_eq!(
            SpecialCharacter::Diaeresis(None).consume(&ctx, b'i'),
            Ok(SpecialCharacter::Diaeresis(Some(b'i')))
        );
        assert_eq!(
            SpecialCharacter::Cedilla(None).consume(&ctx, b'c'),
            Ok(SpecialCharacter::Cedilla(Some(b'c')))
        );

        assert_err!(
            SpecialCharacter::Acute(None).consume(&ctx, b'a'),
            "Invalid character for acute accent 0x61",
        );

        assert_err!(
            SpecialCharacter::Acute(None).consume(&ctx, b'o'),
            "Invalid character for acute accent 0x6F",
        );

        assert_err!(
            SpecialCharacter::Grave(None).consume(&ctx, b'c'),
            "Invalid character for grave accent 0x63",
        );

        assert_err!(
            SpecialCharacter::Cedilla(None).consume(&ctx, b'i'),
            "Invalid character for cedilla 0x69",
        );

        assert_err!(
            SpecialCharacter::Diaeresis(None).consume(&ctx, b'a'),
            "Invalid character for diaeresis 0x61",
        );

        assert_err!(
            SpecialCharacter::Diaeresis(None).consume(&ctx, b'o'),
            "Invalid character for diaeresis 0x6F",
        );

        assert_err!(
            SpecialCharacter::Diaeresis(None).consume(&ctx, b'u'),
            "Invalid character for diaeresis 0x75",
        );

        //test characters not supported in VGP2 todo: better error message

        assert_err!(
            SpecialCharacter::new(&ctx, 0x19)
                .unwrap()
                .consume(&ctx, ESZETT),
            "Invalid special character 0x7B",
        );

        assert_err!(
            SpecialCharacter::new(&ctx, 0x19)
                .unwrap()
                .consume(&ctx, PARAGRAPH),
            "Invalid special character 0x27",
        );
    }

    #[test]
    fn test_special_character_vgp5_specific() {
        let ctx = Context::new(DisplayComponent::VGP5);
        assert_eq!(
            SpecialCharacter::Diaeresis(None).consume(&ctx, b'a'),
            Ok(SpecialCharacter::Diaeresis(Some(b'a')))
        );
        assert_eq!(
            SpecialCharacter::Diaeresis(None).consume(&ctx, b'o'),
            Ok(SpecialCharacter::Diaeresis(Some(b'o')))
        );
        assert_eq!(
            SpecialCharacter::Diaeresis(None).consume(&ctx, b'u'),
            Ok(SpecialCharacter::Diaeresis(Some(b'u')))
        );
        assert_eq!(
            SpecialCharacter::new(&ctx, 0x19)
                .unwrap()
                .consume(&ctx, ESZETT),
            Ok(SpecialCharacter::Eszett)
        );
        assert_eq!(
            SpecialCharacter::new(&ctx, 0x19)
                .unwrap()
                .consume(&ctx, PARAGRAPH),
            Ok(SpecialCharacter::Paragraph)
        );
    }

    #[test]
    fn test_csi() {
        let ctx = Context::new(DisplayComponent::VGP2);

        assert_eq!(Csi::new(&ctx, 0x5B), Ok(Csi::Incomplete));

        assert!(Csi::supports(&ctx, 0x5B));
        assert!(!Csi::supports(&ctx, 0x00));
        assert!(!Csi::supports(&ctx, 0x1F));

        assert!(!Csi::new(&ctx, 0x5B).unwrap().is_complete());

        assert_err!(
            Csi::new(&ctx, 0x00),
            "Unsupported or invalid CSI sequence starting with 0x00",
        );

        assert_err!(
            Csi::new(&ctx, 0x1F),
            "Unsupported or invalid CSI sequence starting with 0x1F",
        );

        assert_err!(
            Csi::new(&ctx, 0x5B).unwrap().consume(&ctx, 0x00),
            "Unsupported or invalid CSI sequence starting with 0x00",
        );

        assert_err!(
            Csi::new(&ctx, 0x5B).unwrap().consume(&ctx, 0x11),
            "Unsupported or invalid CSI sequence starting with 0x11",
        );
    }

    #[test]
    fn test_csi_move_cursor() {
        let ctx = Context::new(DisplayComponent::VGP2);

        let move_left_29 = Csi::new(&ctx, 0x5B)
            .unwrap()
            .consume(&ctx, 0x32)
            .unwrap()
            .consume(&ctx, 0x39)
            .unwrap()
            .consume(&ctx, 0x44)
            .unwrap();

        assert_eq!(move_left_29, Csi::MoveLeft(29));
        assert!(move_left_29.is_complete());

        let move_right_11 = Csi::new(&ctx, 0x5B)
            .unwrap()
            .consume(&ctx, 0x31)
            .unwrap()
            .consume(&ctx, 0x31)
            .unwrap()
            .consume(&ctx, 0x43)
            .unwrap();

        assert_eq!(move_right_11, Csi::MoveRight(11));
        assert!(move_right_11.is_complete());

        let move_up_7 = Csi::new(&ctx, 0x5B)
            .unwrap()
            .consume(&ctx, 0x37)
            .unwrap()
            .consume(&ctx, 0x41)
            .unwrap();

        assert_eq!(move_up_7, Csi::MoveUp(7));
        assert!(move_up_7.is_complete());

        let move_down_3 = Csi::new(&ctx, 0x5B)
            .unwrap()
            .consume(&ctx, 0x30)
            .unwrap()
            .consume(&ctx, 0x33)
            .unwrap()
            .consume(&ctx, 0x42)
            .unwrap();

        assert_eq!(move_down_3, Csi::MoveDown(3));
        assert!(move_down_3.is_complete());

        let move_right_1 = Csi::new(&ctx, 0x5B)
            .unwrap()
            .consume(&ctx, 0x31)
            .unwrap()
            .consume(&ctx, 0x43)
            .unwrap();

        assert_eq!(move_right_1, Csi::MoveRight(1));
        assert!(move_right_1.is_complete());

        let move_left_0 = Csi::new(&ctx, 0x5B)
            .unwrap()
            .consume(&ctx, 0x30)
            .unwrap()
            .consume(&ctx, 0x44)
            .unwrap();

        assert_eq!(move_left_0, Csi::MoveLeft(0));
        assert!(move_left_0.is_complete());

        assert_err!(
            Csi::new(&ctx, 0x5B).unwrap().consume(&ctx, 0x44),
            "Unsupported or invalid CSI sequence starting with 0x44",
        );

        assert_err!(
            Csi::new(&ctx, 0x5B).unwrap().consume(&ctx, 0x41),
            "Unsupported or invalid CSI sequence starting with 0x41",
        );
    }

    #[test]
    fn test_csi_set_cursor() {
        let ctx = Context::new(DisplayComponent::VGP2);

        let set_cursor_0_1 = Csi::new(&ctx, 0x5B)
            .unwrap()
            .consume(&ctx, 0x31)
            .unwrap()
            .consume(&ctx, 0x3B)
            .unwrap()
            .consume(&ctx, 0x31)
            .unwrap()
            .consume(&ctx, 0x48)
            .unwrap();

        assert_eq!(set_cursor_0_1, Csi::SetCursor(1, 1));
        assert!(set_cursor_0_1.is_complete());

        let set_cursor_1_2 = Csi::new(&ctx, 0x5B)
            .unwrap()
            .consume(&ctx, 0x32)
            .unwrap()
            .consume(&ctx, 0x3B)
            .unwrap()
            .consume(&ctx, 0x31)
            .unwrap()
            .consume(&ctx, 0x48)
            .unwrap();

        assert_eq!(set_cursor_1_2, Csi::SetCursor(1, 2));
        assert!(set_cursor_1_2.is_complete());

        let set_cursor_34_13 = Csi::new(&ctx, 0x5B)
            .unwrap()
            .consume(&ctx, 0x31)
            .unwrap()
            .consume(&ctx, 0x33)
            .unwrap()
            .consume(&ctx, 0x3B)
            .unwrap()
            .consume(&ctx, 0x33)
            .unwrap()
            .consume(&ctx, 0x34)
            .unwrap()
            .consume(&ctx, 0x48)
            .unwrap();

        assert_eq!(set_cursor_34_13, Csi::SetCursor(34, 13));
        assert!(set_cursor_34_13.is_complete());
    }

    #[test]
    fn test_invalid_set_cursor() {
        let ctx = Context::new(DisplayComponent::VGP2);

        assert_err!(
            Csi::new(&ctx, 0x5B)
                .unwrap()
                .consume(&ctx, 0x31)
                .unwrap()
                .consume(&ctx, 0x3B)
                .unwrap()
                .consume(&ctx, 0x48),
            "Unsupported or invalid byte 0x48 for sequence IncompleteSetCursor(None, Some(1))",
        );

        assert_err!(
            Csi::new(&ctx, 0x5B).unwrap().consume(&ctx, 0x3F),
            "Unsupported or invalid CSI sequence starting with 0x3F",
        );

        assert_err!(
            Csi::new(&ctx, 0x5B)
                .unwrap()
                .consume(&ctx, 0x31)
                .unwrap()
                .consume(&ctx, 0x3B)
                .unwrap()
                .consume(&ctx, 0x3B),
            "Unsupported or invalid byte 0x3B for sequence IncompleteSetCursor(None, Some(1))",
        );

        assert_err!(
            Csi::new(&ctx, 0x5B)
                .unwrap()
                .consume(&ctx, 0x31)
                .unwrap()
                .consume(&ctx, 0x3B)
                .unwrap()
                .consume(&ctx, 0x30)
                .unwrap()
                .consume(&ctx, 0x3B),
            "Unsupported or invalid byte 0x3B for sequence IncompleteSetCursor(Some(0), Some(1))",
        );

        assert_err!(
            Csi::new(&ctx, 0x5B)
                .unwrap()
                .consume(&ctx, 0x31)
                .unwrap()
                .consume(&ctx, 0x3B)
                .unwrap()
                .consume(&ctx, 0x31)
                .unwrap()
                .consume(&ctx, 0x48)
                .unwrap()
                .consume(&ctx, 0x48),
            "Unsupported or invalid byte 0x48 for sequence SetCursor(1, 1)",
        );
    }

    #[test]
    fn test_can_fills_line_without_moving_cursor() {
        let mut parser = Parser::new(DisplayComponent::VGP2);
        parser.ctx.cursor_x = 3;
        parser.ctx.attributes.background = BLUE;
        parser.ctx.attributes.foreground = GREEN;

        parser.consume(ESC).unwrap();
        parser.consume(CSI).unwrap();
        parser.consume(CAN).unwrap();

        assert_eq!(parser.ctx.cursor_x, 3);
        assert!(parser.ctx.grid.cell(2, 1).is_delimiter);

        for column in 3..=parser.ctx.screen_width {
            let cell = parser.ctx.grid.cell(column, 1);
            assert_eq!(cell.content, ' ');
            assert!(!cell.is_delimiter);
            assert_eq!(cell.attributes.background, parser.ctx.attributes.background);
            assert_eq!(cell.attributes.foreground, parser.ctx.attributes.foreground);
        }
    }

    #[test]
    fn test_clear_screen_uses_delimiters_and_resets_attributes() {
        let mut parser = Parser::new(DisplayComponent::VGP2);
        parser.ctx.attributes.background = BLUE;
        parser.ctx.attributes.foreground = RED;
        parser.ctx.visible_cursor = true;

        parser.consume(FF).unwrap();

        let cell = parser.ctx.grid.cell(1, 1);
        assert!(cell.is_delimiter);
        assert_eq!(cell.attributes.character_set, CharacterSet::G1);
        assert_eq!(cell.attributes.background, BLACK);
        assert_eq!(cell.attributes.foreground, BLACK);

        assert_eq!(parser.ctx.attributes.character_set, CharacterSet::G0);
        assert_eq!(parser.ctx.attributes.background, BLACK);
        assert_eq!(parser.ctx.attributes.foreground, WHITE);
        assert!(!parser.ctx.attributes.blinking);
        assert!(!parser.ctx.attributes.double_height);
        assert!(!parser.ctx.attributes.double_width);
        assert!(!parser.ctx.attributes.invert);
        assert!(!parser.ctx.attributes.mask);
        assert!(!parser.ctx.attributes.underline);
    }

    #[test]
    fn test_insert_mode_shifts_row_only() {
        let mut ctx = Context::new(DisplayComponent::VGP2);

        ctx.print('A').unwrap();
        ctx.print('B').unwrap();
        ctx.print('C').unwrap();

        ctx.set_cursor(1, 2, false).unwrap();
        ctx.print('Y').unwrap();

        ctx.set_cursor(2, 1, false).unwrap();
        ctx.insert = true;
        ctx.print('Z').unwrap();

        assert_eq!(ctx.grid.cell(1, 1).content, 'A');
        assert_eq!(ctx.grid.cell(2, 1).content, 'Z');
        assert_eq!(ctx.grid.cell(3, 1).content, 'B');
        assert_eq!(ctx.grid.cell(4, 1).content, 'C');
        assert_eq!(ctx.grid.cell(1, 2).content, 'Y');
    }

    #[test]
    fn test_delete_characters_keeps_attributes() {
        let mut ctx = Context::new(DisplayComponent::VGP2);
        ctx.attributes.foreground = RED;

        ctx.print('A').unwrap();
        ctx.print('B').unwrap();
        ctx.print('C').unwrap();
        ctx.print('D').unwrap();

        ctx.set_cursor(2, 1, false).unwrap();
        ctx.consume(&Sequence::Escaped(EscapedSequence::Csi(
            Csi::ClearAfterCursor(2),
        )))
        .unwrap();

        assert_eq!(ctx.grid.cell(1, 1).content, 'A');
        assert_eq!(ctx.grid.cell(2, 1).content, 'D');
        assert_eq!(ctx.grid.cell(ctx.screen_width - 1, 1).content, ' ');
        assert_eq!(ctx.grid.cell(ctx.screen_width, 1).content, ' ');
        assert_eq!(
            ctx.grid.cell(ctx.screen_width - 1, 1).attributes.foreground,
            RED
        );
        assert_eq!(
            ctx.grid.cell(ctx.screen_width, 1).attributes.foreground,
            RED
        );
    }

    #[test]
    fn test_move_cursor_y_wraps_in_page_mode() {
        let mut ctx = Context::new(DisplayComponent::VGP2);
        ctx.cursor_y = ctx.screen_height;

        ctx.move_cursor_y(1, true).unwrap();
        assert_eq!(ctx.cursor_y, 1);
    }

    #[test]
    fn test_record_separator_resets_attributes() {
        let mut ctx = Context::new(DisplayComponent::VGP2);
        ctx.attributes.blinking = true;
        ctx.attributes.foreground = RED;
        ctx.attributes.character_set = CharacterSet::G1;

        ctx.consume(&Sequence::RecordSeparator).unwrap();

        assert_eq!(ctx.cursor_x, 1);
        assert_eq!(ctx.cursor_y, 1);
        assert_eq!(ctx.attributes.character_set, CharacterSet::G0);
        assert_eq!(ctx.attributes.foreground, WHITE);
        assert!(!ctx.attributes.blinking);
    }

    #[test]
    fn test_ss2_in_g1_returns_error() {
        let mut ctx = Context::new(DisplayComponent::VGP2);
        ctx.attributes.character_set = CharacterSet::G1;

        assert_err!(
            SpecialCharacter::new(&ctx, SS2),
            "Special characters are not supported in G1"
        );
    }

    #[test]
    fn test_ss2_with_c0_resynchronizes() {
        let mut parser = Parser::new(DisplayComponent::VGP2);
        parser.consume(SS2).unwrap();
        parser.consume(RS).unwrap();

        assert_eq!(parser.ctx.cursor_x, 1);
        assert_eq!(parser.ctx.cursor_y, 1);
        assert_eq!(parser.ctx.attributes.character_set, CharacterSet::G0);
    }

    #[test]
    fn test_row_zero_entry_and_exit_restore_context() {
        let mut parser = Parser::new(DisplayComponent::VGP2);
        parser.ctx.cursor_x = 12;
        parser.ctx.cursor_y = 7;
        parser.ctx.attributes.foreground = RED;
        parser.ctx.attributes.background = BLUE;
        parser.ctx.pending_attributes.set_mask(true);

        parser.consume(US).unwrap();
        parser.consume(0x40).unwrap();
        parser.consume(0x45).unwrap();

        assert_eq!(parser.ctx.cursor_x, 5);
        assert_eq!(parser.ctx.cursor_y, 0);
        assert_eq!(parser.ctx.attributes.foreground, WHITE);
        assert_eq!(parser.ctx.attributes.background, BLACK);

        parser.consume(HT).unwrap();
        assert_eq!(parser.ctx.cursor_x, 6);
        assert_eq!(parser.ctx.cursor_y, 0);

        parser.consume(LF).unwrap();
        assert_eq!(parser.ctx.cursor_x, 12);
        assert_eq!(parser.ctx.cursor_y, 7);
        assert_eq!(parser.ctx.attributes.foreground, RED);
        assert_eq!(parser.ctx.attributes.background, BLUE);
        assert!(parser.ctx.pending_attributes.mask.unwrap());
    }

    #[test]
    fn test_g1_size_and_invert_sequences_return_errors() {
        let mut ctx = Context::new(DisplayComponent::VGP2);
        ctx.attributes.character_set = CharacterSet::G1;

        assert_err!(
            ctx.consume(&Sequence::Escaped(EscapedSequence::DoubleWidth)),
            "Changing invert and character size while in G1 is not supported"
        );
        assert_err!(
            ctx.consume(&Sequence::Escaped(EscapedSequence::DoubleHeight)),
            "Changing invert and character size while in G1 is not supported"
        );
        assert_err!(
            ctx.consume(&Sequence::Escaped(EscapedSequence::DoubleSize)),
            "Changing invert and character size while in G1 is not supported"
        );
        assert_err!(
            ctx.consume(&Sequence::Escaped(EscapedSequence::Invert(true))),
            "Changing invert and character size while in G1 is not supported"
        );
    }

    #[test]
    fn test_repeat_without_previous_character_returns_errors() {
        let mut ctx = Context::new(DisplayComponent::VGP2);

        assert_err!(
            ctx.consume(&Sequence::Repeat(Some(3))),
            "Tried to repeat a character at the beginning of the screen"
        );
    }

    #[test]
    fn test_double_size_in_row_one_returns_errors() {
        let mut ctx = Context::new(DisplayComponent::VGP2);
        ctx.cursor_y = 1;

        assert_err!(
            ctx.consume(&Sequence::Escaped(EscapedSequence::DoubleHeight)),
            "Tried to set double height or double size while in row 0 or 1"
        );
        assert_err!(
            ctx.consume(&Sequence::Escaped(EscapedSequence::DoubleSize)),
            "Tried to set double height or double size while in row 0 or 1"
        );
    }

    #[cfg(not(feature = "strict"))]
    #[test]
    fn test_parser_non_strict_logs_and_continues_after_semantic_error() {
        let mut parser = Parser::new(DisplayComponent::VGP2);
        parser.ctx.attributes.character_set = CharacterSet::G1;

        assert_eq!(parser.consume(ESC), Ok(()));
        assert_eq!(parser.consume(DOUBLE_WIDTH), Ok(()));
        assert_eq!(parser.consume(SI), Ok(()));
        assert_eq!(parser.consume(b'A'), Ok(()));

        assert_eq!(parser.ctx.grid.cell(1, 1).content, 'A');
        assert_eq!(parser.ctx.cursor_x, 2);
        assert_eq!(parser.ctx.attributes.character_set, CharacterSet::G0);
    }

    #[cfg(feature = "strict")]
    #[test]
    fn test_parser_strict_returns_semantic_error() {
        let mut parser = Parser::new(DisplayComponent::VGP2);
        parser.ctx.attributes.character_set = CharacterSet::G1;

        assert_eq!(parser.consume(ESC), Ok(()));
        assert_err!(
            parser.consume(DOUBLE_WIDTH),
            "Changing invert and character size while in G1 is not supported"
        );
    }

    #[test]
    fn test_get_cursor_position_emits_response() {
        let mut parser = Parser::new(DisplayComponent::VGP2);

        // Move cursor to a known position
        assert_eq!(parser.consume(ESC), Ok(()));
        assert_eq!(parser.consume(CSI), Ok(()));
        assert_eq!(parser.consume(0x30), Ok(())); // 0
        assert_eq!(parser.consume(0x35), Ok(())); // 5
        assert_eq!(parser.consume(0x3B), Ok(())); // ;
        assert_eq!(parser.consume(0x31), Ok(())); // 1
        assert_eq!(parser.consume(0x30), Ok(())); // 0
        assert_eq!(parser.consume(0x48), Ok(())); // H

        assert_eq!(parser.ctx().cursor_y, 5);
        assert_eq!(parser.ctx().cursor_x, 10);

        // Request cursor position (ESC 0x61)
        assert_eq!(parser.consume(ESC), Ok(()));
        assert_eq!(parser.consume(0x61), Ok(()));

        // Response should be US <row> <col>
        assert!(parser.has_response());
        let response = parser.take_response();
        assert_eq!(response, vec![US, 0x40 + 5, 0x40 + 10]);
        assert!(!parser.has_response());
    }

    #[test]
    fn test_beep_sets_flag() {
        let mut parser = Parser::new(DisplayComponent::VGP2);

        assert!(!parser.take_beep());

        assert_eq!(parser.consume(BEEP), Ok(()));
        assert!(parser.take_beep());

        // Flag should be cleared after take
        assert!(!parser.take_beep());
    }

    #[test]
    fn test_nul_is_ignored() {
        let mut parser = Parser::new(DisplayComponent::VGP2);
        assert_eq!(parser.consume(0x00), Ok(()));
        assert_eq!(parser.ctx().cursor_x, 1);
        assert_eq!(parser.ctx().cursor_y, 1);
    }

    #[test]
    fn test_sub_displays_error_character() {
        let mut parser = Parser::new(DisplayComponent::VGP2);
        assert_eq!(parser.consume(0x1A), Ok(()));

        // SUB should display the error symbol (0x7F) and advance cursor
        assert_eq!(parser.ctx().grid.cell(1, 1).content, '\u{7F}');
        assert_eq!(parser.ctx().cursor_x, 2);
    }
}
