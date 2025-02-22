use std::fmt::{Debug, Formatter, Result as FmtResult, Write};
use std::mem;
use crate::specifications::codes::*;

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
pub enum DisplayComponent {
    VGP2,
    VGP5,
}

#[derive(Eq, PartialEq, Debug)]
pub enum PageMode {
    Page,
    Scroll,
}

trait ToCharacter {
    fn to_character(&self) -> char;
}

trait Parsable {
    fn new(ctx: &Context, byte: u8) -> Self;
    fn supports(ctx: &Context, byte: u8) -> bool;
    fn consume(&mut self, ctx: &Context, byte: u8) -> Self;
    fn is_complete(&self) -> bool;
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
#[repr(u8)]
pub enum CharacterSet {
    G0,
    G1,
}

impl Parsable for CharacterSet {
    fn new(_ctx: &Context, byte: u8) -> Self {
        match byte {
            SI => CharacterSet::G0,
            SO => CharacterSet::G1,
            invalid => panic!("Invalid character set {:#04X}", invalid),
        }
    }

    fn supports(_ctx: &Context, byte: u8) -> bool {
        byte == SI || byte == SO
    }

    fn consume(&mut self, _ctx: &Context, byte: u8) -> Self {
        panic!("Character set {:?} does not support more bytes ({:#04X})", self, byte);
    }

    fn is_complete(&self) -> bool {
        true
    }
}

//fully implemented
#[derive(Eq, PartialEq, Debug)]
struct SimpleCharacter(u8);

impl Parsable for SimpleCharacter {
    fn new(ctx: &Context, byte: u8) -> Self {
        if !Self::supports(ctx, byte) {
            panic!("Invalid simple character {:#04X}", byte);
        }

        SimpleCharacter(byte)
    }

    fn supports(ctx: &Context, byte: u8) -> bool {
        ctx.attributes.character_set == CharacterSet::G0 && (0x20..=0x7F).contains(&byte)
    }

    fn consume(&mut self, _ctx: &Context, byte: u8) -> Self {
        panic!("Simple character {:?} does not support more bytes ({:#04X})", self, byte);
    }

    fn is_complete(&self) -> bool {
        true
    }
}

impl ToCharacter for SimpleCharacter {
    fn to_character(&self) -> char {
        self.0 as char
    }
}

//fully implemented
#[derive(Eq, PartialEq)]
struct SemiGraphicCharacter(u8);

impl Parsable for SemiGraphicCharacter {
    fn new(ctx: &Context, byte: u8) -> Self {
        if !Self::supports(ctx, byte) {
            panic!("Invalid semi-graphic character {:#04X}", byte);
        }

        SemiGraphicCharacter(byte)
    }

    fn supports(ctx: &Context, byte: u8) -> bool {
        ctx.attributes.character_set == CharacterSet::G1 && ((0x20..=0x3F).contains(&byte) || (0x5F..=0x7F).contains(&byte))
    }

    fn consume(&mut self, _ctx: &Context, byte: u8) -> Self {
        panic!("Semi-graphic character {:?} does not support more bytes ({:#04X})", self, byte);
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
    fn new(ctx: &Context, byte: u8) -> Self {
        if !Self::supports(ctx, byte) {
            panic!("Invalid special character {:#04X}", byte);
        }

        if ctx.attributes.character_set == CharacterSet::G1 {
            panic!("Special characters are not supported in G1");
        }

        SpecialCharacter::Incomplete
    }

    fn supports(_ctx: &Context, byte: u8) -> bool {
        byte == SS2
    }

    fn consume(&mut self, ctx: &Context, byte: u8) -> Self {
        match self {
            SpecialCharacter::Incomplete => match byte {
                GRAVE => SpecialCharacter::Grave(None),
                ACUTE => SpecialCharacter::Acute(None),
                CIRCUMFLEX => SpecialCharacter::Circumflex(None),
                DIAERESIS => SpecialCharacter::Diaeresis(None),
                CEDILLA => SpecialCharacter::Cedilla(None),
                LOWER_OE => SpecialCharacter::LowerOE,
                UPPER_OE => SpecialCharacter::UpperOE,
                ESZETT if ctx.display_component == DisplayComponent::VGP5 => SpecialCharacter::Eszett,
                POUND => SpecialCharacter::Pound,
                DOLLAR => SpecialCharacter::Dollar,
                NUMBER_SIGN => SpecialCharacter::NumberSign,
                ARROW_LEFT => SpecialCharacter::ArrowLeft,
                ARROW_UP => SpecialCharacter::ArrowUp,
                ARROW_RIGHT => SpecialCharacter::ArrowRight,
                ARROW_DOWN => SpecialCharacter::ArrowDown,
                PARAGRAPH if ctx.display_component == DisplayComponent::VGP5 => SpecialCharacter::Paragraph,
                DEGREE => SpecialCharacter::Degree,
                PLUS_OR_MINUS => SpecialCharacter::PlusOrMinus,
                OBELUS => SpecialCharacter::Obelus,
                ONE_QUARTER => SpecialCharacter::OneQuarter,
                ONE_HALF => SpecialCharacter::OneHalf,
                THREE_QUARTERS => SpecialCharacter::ThreeQuarters,
                _ => panic!("Invalid special character {:#04X}", byte),
            },
            SpecialCharacter::Grave(None) => match byte as char {
                'a' | 'e' | 'u' => SpecialCharacter::Grave(Some(byte)),
                _ => panic!("Invalid character for grave accent {:#04X}", byte),
            },
            SpecialCharacter::Acute(None) => match byte as char {
                'e' => SpecialCharacter::Acute(Some(byte)),
                _ => panic!("Invalid character for acute accent {:#04X}", byte),
            },
            SpecialCharacter::Circumflex(None) => match byte as char {
                'a' | 'e' | 'i' | 'o' | 'u' => SpecialCharacter::Circumflex(Some(byte)),
                _ => panic!("Invalid character for circumflex accent {:#04X}", byte),
            },
            SpecialCharacter::Diaeresis(None) => match byte as char {
                'e' | 'i' => SpecialCharacter::Diaeresis(Some(byte)),
                'a' | 'o' | 'u' if ctx.display_component == DisplayComponent::VGP5 => SpecialCharacter::Diaeresis(Some(byte)),
                _ => panic!("Invalid character for diaeresis accent {:#04X}", byte),
            },
            SpecialCharacter::Cedilla(None) => match byte as char {
                'c' => SpecialCharacter::Cedilla(Some(byte)),
                _ => panic!("Invalid character for cedilla {:#04X}", byte),
            },
            _ => panic!("Escaped sequence {:?} does not support more bytes ({:#04X})", self, byte),
        }
    }

    fn is_complete(&self) -> bool {
        !matches!(
            self,
            SpecialCharacter::Incomplete |
            SpecialCharacter::Grave(None) |
            SpecialCharacter::Acute(None) |
            SpecialCharacter::Circumflex(None)|
            SpecialCharacter::Diaeresis(None)|
            SpecialCharacter::Cedilla(None)
        )
    }
}

impl ToCharacter for SpecialCharacter {
    fn to_character(&self) -> char {
        match self {
            SpecialCharacter::Grave(Some(byte)) => match byte {
                b'a' => 'à',
                b'e' => 'è',
                b'u' => 'ù',
                _ => panic!("Invalid character for grave accent {:#04X}", byte),
            },
            SpecialCharacter::Acute(Some(byte)) => match byte {
                b'e' => 'é',
                _ => panic!("Invalid character for acute accent {:#04X}", byte),
            },
            SpecialCharacter::Circumflex(Some(byte)) => match byte {
                b'a' => 'â',
                b'e' => 'ê',
                b'i' => 'î',
                b'o' => 'ô',
                b'u' => 'û',
                _ => panic!("Invalid character for circumflex accent {:#04X}", byte),
            },
            SpecialCharacter::Diaeresis(Some(byte)) => match byte {
                b'a' => 'ä',
                b'e' => 'ë',
                b'i' => 'ï',
                b'o' => 'ö',
                b'u' => 'ü',
                _ => panic!("Invalid character for diaeresis accent {:#04X}", byte),
            },
            SpecialCharacter::Cedilla(Some(byte)) => match byte {
                b'c' => 'ç',
                _ => panic!("Invalid character for cedilla {:#04X}", byte),
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
            _ => panic!("Special character {:?} is not complete", self),
        }
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
    Toggle(bool),
    ToggleScreen(bool),
    Sleep(bool),
}

impl Parsable for Protocol {
    fn new(_ctx: &Context, byte: u8) -> Self {
        match byte {
            PRO1 => Protocol::Pro1,
            PRO2 => Protocol::Pro2,
            PRO3 => Protocol::Pro3,
            _ => panic!("Invalid protocol sequence starting with {:#04X}", byte)
        }
    }

    fn supports(_ctx: &Context, byte: u8) -> bool {
        byte == PRO1 || byte == PRO2 || byte == PRO3
    }

    fn consume(&mut self, _ctx: &Context, byte: u8) -> Self {
        match self {
            Protocol::Pro1 => match byte {
                RESET => Protocol::Reset,
                REQ_SPEED => Protocol::RequestSpeed,
                _ => panic!("Unsupported or invalid PRO1 sequence starting with {:#04X}", byte)
            },
            Protocol::Pro2 => match byte {
                PROG => Protocol::SetSpeed(None),
                _ => panic!("Unsupported or invalid PRO2 sequence starting with {:#04X}", byte)
            },
            Protocol::Pro3 => match byte {
                START => Protocol::Toggle(true),
                STOP => Protocol::Toggle(false),
                _ => panic!("Unsupported or invalid PRO3 sequence starting with {:#04X}", byte)
            },
            Protocol::SetSpeed(None) => Protocol::SetSpeed(Some(byte)),
            Protocol::Toggle(value) => match byte {
                SCREEN => Protocol::ToggleScreen(*value),
                _ => panic!("Unsupported or invalid protocol start/stop sequence starting with {:#04X}", byte)
            },
            Protocol::ToggleScreen(value) => match byte {
                0x41 => Protocol::Sleep(*value),
                _ => panic!("Unsupported or invalid protocol toggle screen sequence starting with {:#04X}", byte)
            }
            _ => panic!("Protocol sequence {:?} does not support additional bytes ({:#04X})", self, byte),
        }
    }

    fn is_complete(&self) -> bool {
        !matches!(self, Protocol::Pro1 | Protocol::Pro2 | Protocol::Pro3 | Protocol::SetSpeed(None) | Protocol::Toggle(_) | Protocol::ToggleScreen(_))
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
    fn new(ctx: &Context, byte: u8) -> Self {
        if !Self::supports(ctx, byte) {
            panic!("Unsupported or invalid CSI sequence starting with {:#04X}", byte);
        }

        if ctx.cursor_y == 0 {
            panic!("CSI codes are not supported in row 0"); //p95
        }

        Csi::Incomplete
    }

    fn supports(_ctx: &Context, byte: u8) -> bool {
        byte == CSI
    }

    fn consume(&mut self, ctx: &Context, byte: u8) -> Self {
        match self {
            Csi::Incomplete => match byte {
                CAN => Csi::InsertSpacesFromCursorToEol,
                0x30..=0x39 => Csi::Quantified(byte - 0x30),
                0x4A => Csi::ClearFromCursorToEos,
                0x4B => Csi::ClearFromCursorToEol,
                _ => panic!("Unsupported or invalid CSI sequence starting with {:#04X}", byte),
            }
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
                _ => panic!("Unsupported or invalid byte {:#04X} for quantified CSI sequence", byte),
            }
            Csi::IncompleteSetCursor(x, Some(y)) if (0x30..=0x39).contains(&byte) => Csi::IncompleteSetCursor(
                Some(x.unwrap_or(0) * 10 + (byte - 0x30)),
                Some(*y)
            ),
            Csi::IncompleteSetCursor(Some(x), Some(y)) if byte == 0x48 => if (1..=ctx.screen_width).contains(x) && (1..=ctx.screen_height).contains(y) {
                Csi::SetCursor(*x, *y)
            } else {
                panic!("Invalid cursor position ({}, {}) for screen size ({}, {})", x, y, ctx.screen_width, ctx.screen_height)
            },

            //TODO: implement other CSI sequences
            //TODO: implement end-of-page 95 recommendations but not here
            _ => panic!("Unsupported or invalid byte {:#04X} for sequence {:?}", byte, self),
        }
    }

    fn is_complete(&self) -> bool {
        !matches!(self, Csi::Incomplete | Csi::Quantified(_) | Csi::IncompleteSetCursor(_, _))
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
    fn new(ctx: &Context, byte: u8) -> Self {
        if !Self::supports(ctx, byte) {
            panic!("Unsupported or invalid escaped sequence starting with {:#04X}", byte);
        }

        EscapedSequence::Incomplete
    }

    fn supports(_ctx: &Context, byte: u8) -> bool {
        byte == ESC
    }

    fn consume(&mut self, ctx: &Context, byte: u8) -> Self {
        match self {
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
                PRO1 | PRO2 | PRO3 => EscapedSequence::Protocol(Protocol::new(ctx, byte)),
                CSI => EscapedSequence::Csi(Csi::new(ctx, byte)),
                0x61 => EscapedSequence::GetCursorPosition,
                0x25 => EscapedSequence::Ignore(None),
                0x2F => EscapedSequence::IncompleteStopIgnore,
                0x23 => EscapedSequence::IncompleteScreenMasking(1),
                _ => panic!("Invalid escaped sequence starting with {:#04X}", byte),
            },
            EscapedSequence::IncompleteStopIgnore if byte == 0x3F  => EscapedSequence::Ignore(Some(false)),
            EscapedSequence::Ignore(None) => EscapedSequence::Ignore(Some(byte != 0x40)),
            EscapedSequence::Csi(csi) => EscapedSequence::Csi(csi.consume(ctx, byte)),
            EscapedSequence::Protocol(pro) => EscapedSequence::Protocol(pro.consume(ctx, byte)),
            EscapedSequence::IncompleteScreenMasking(1) if byte == 0x20 => EscapedSequence::IncompleteScreenMasking(2),
            EscapedSequence::IncompleteScreenMasking(2) => match byte {
                MASK => EscapedSequence::ScreenMasking(true),
                UNMASK => EscapedSequence::ScreenMasking(false),
                _ => panic!("Invalid screen masking byte ({:#04X}), expected 0x58 or 0x5F", byte),
            },
            _ => panic!("Escaped sequence {:?} does not support additional bytes ({:#04X})", self, byte),
        }
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
}

impl Parsable for Sequence {
    fn new(_ctx: &Context, _byte: u8) -> Self {
        Sequence::Incomplete
    }

    fn supports(_ctx: &Context, _byte: u8) -> bool {
        true
    }

    fn consume(&mut self, ctx: &Context, byte: u8) -> Self {
        match self {
            Sequence::Incomplete => {
                if EscapedSequence::supports(ctx, byte) {
                    Sequence::Escaped(EscapedSequence::new(ctx, byte))
                } else if CharacterSet::supports(ctx, byte) {
                    Sequence::SetCharacterSet(CharacterSet::new(ctx, byte))
                } else if SpecialCharacter::supports(ctx, byte) {
                    Sequence::SpecialCharacter(SpecialCharacter::new(ctx, byte))
                } else if SemiGraphicCharacter::supports(ctx, byte) {
                    Sequence::SemiGraphicCharacter(SemiGraphicCharacter::new(ctx, byte))
                } else if SimpleCharacter::supports(ctx, byte) {
                    Sequence::SimpleCharacter(SimpleCharacter::new(ctx, byte))
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
                } else if byte == BEEP {
                    Sequence::Beep
                } else if byte == CURSOR_ON {
                    Sequence::VisibleCursor(true)
                } else if byte == CURSOR_OFF {
                    Sequence::VisibleCursor(false)
                } else {
                    panic!("Unsupported or invalid sequence starting with {:#04X}", byte)
                }
            }
            Sequence::Escaped(escaped_sequence) => Sequence::Escaped(escaped_sequence.consume(ctx, byte)),
            Sequence::SetCharacterSet(character_set) => Sequence::SetCharacterSet(character_set.consume(ctx, byte)),
            Sequence::SpecialCharacter(special_character) => Sequence::SpecialCharacter(special_character.consume(ctx, byte)),
            Sequence::SemiGraphicCharacter(semi_graphic_character) => Sequence::SemiGraphicCharacter(semi_graphic_character.consume(ctx, byte)),
            Sequence::SimpleCharacter(simple_character) => Sequence::SimpleCharacter(simple_character.consume(ctx, byte)),
            Sequence::SubSection(None, None) if (0x40..=0x7F).contains(&byte) => Sequence::SubSection(Some(byte - 0x40), None),
            Sequence::SubSection(Some(x), None) if (0x40..=0x7F).contains(&byte) => Sequence::SubSection(Some(*x), Some(byte - 0x40)),
            Sequence::Repeat(None) => if (0x40..=0x7F).contains(&byte) {
                Sequence::Repeat(Some(byte - 0x40))
            } else {
                panic!("Repeat sequence expects a number between 0x40 and 0x7F, got {:#04X}", byte)
            },
            _ => panic!("Sequence {:?} does not support additional bytes ({:#04X})", self, byte),
        }
    }

    fn is_complete(&self) -> bool {
        match self {
            Sequence::Incomplete => false,
            Sequence::Escaped(escaped_sequence) => escaped_sequence.is_complete(),
            Sequence::SetCharacterSet(character_set) => character_set.is_complete(),
            Sequence::SpecialCharacter(special_character) => special_character.is_complete(),
            Sequence::SemiGraphicCharacter(semi_graphic_character) => semi_graphic_character.is_complete(),
            Sequence::SimpleCharacter(simple_character) => simple_character.is_complete(),
            Sequence::SubSection(Some(_), Some(_)) => true,
            Sequence::SubSection(_, _) => false,
            Sequence::Repeat(None) => false,
            _ => true,
        }
    }
}

#[derive(Default, Debug)]
pub struct PendingAttributes {
    apply_on_delimiter: bool,
    background: Option<u8>,
    underline: Option<bool>,  //+ caractère disjoint en mode semi-graphique
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
    pub underline: bool,  //+ caractère disjoint en mode semi-graphique
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
        write!(f, "Attr({:?}, Background({}), Foreground({})", self.character_set, self.background, self.foreground)?;

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
}

impl Debug for Cell {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let name = if self.is_delimiter { "Delim" } else { "Cell" };

        if self.attributes.character_set == CharacterSet::G1 {
            write!(f, "{}({:#04X}, {:?})", name, self.content as u8, self.attributes)
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
            data: vec![Cell::default(); width * height],
        }
    }

    pub fn cell(&self, x: u8, y: u8) -> &Cell {
        assert!(x >= 1 && x <= self.width as u8, "Invalid x coordinate {}", x);
        assert!(y >= 1 && y <= self.height as u8, "Invalid y coordinate {}", y);

        &self.data[(y as usize - 1) * self.width + x as usize - 1]
    }

    pub fn previous_cell(&self, x: u8, y: u8) -> Option<&Cell> {
        assert!(x >= 1 && x <= self.width as u8, "Invalid x coordinate {}", x);
        assert!(y >= 1 && y <= self.height as u8, "Invalid y coordinate {}", y);

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

        self.data.get((y as usize - 1) * self.width + x as usize - 1)
    }

    fn cell_mut(&mut self, x: u8, y: u8) -> &mut Cell {
        assert!(x >= 1 && x <= self.width as u8, "Invalid x coordinate {}", x);
        assert!(y >= 1 && y <= self.height as u8, "Invalid y coordinate {}", y);

        &mut self.data[(y as usize - 1) * self.width + x as usize - 1]
    }

    fn reset(&mut self) {
        self.data.iter_mut().for_each(Cell::reset);
    }

    fn scroll(&mut self, count: i8) {
        if count == 0 {
            return;
        }

        let cells_to_scroll = count.unsigned_abs() as usize * self.width;

        if count > 0 {
            self.data.splice((self.data.len() - cells_to_scroll).., vec![]);
            self.data.splice(0..0, vec![Cell::default(); cells_to_scroll]);
        } else {
            self.data.splice(0..cells_to_scroll, vec![]);
            self.data.splice(self.data.len().., vec![Cell::default(); cells_to_scroll]);
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

    fn insert_after(&mut self, count: u8, x: u8, y: u8) {
        let start = (y as usize - 1) * self.width + x as usize - 1;
        self.data.splice(start..start, vec![Cell::default(); count as usize]);
        self.data.truncate(self.width * self.height);
    }

    fn delete_after(&mut self, count: u8, x: u8, y: u8) {
        let start = (y as usize - 1) * self.width + x as usize - 1;
        let end = start + count as usize;
        self.data.splice(start..end, vec![]);
        self.data.resize(self.width * self.height, Cell::default());
    }

    fn insert_rows_after(&mut self, count: u8, y: u8) {
        //don't -1 to y because we're inserting after the given row
        let start = (y as usize) * self.width;
        self.data.splice(start..start, vec![Cell::default(); count as usize * self.width]);
        self.data.truncate(self.width * self.height);
    }

    fn delete_rows_after(&mut self, count: u8, y: u8) {
        //don't -1 to y because we're deleting after the given row
        let start = (y as usize) * self.width;
        let end = start + count as usize * self.width;
        self.data.splice(start..end, vec![]);
        self.data.resize(self.width * self.height, Cell::default());
    }

    fn recalculate_row_attributes(&mut self, x: u8, y: u8) {
        let start = (y as usize - 1) * self.width + x as usize - 1;
        let end = start + self.width - x as usize ;

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
        self.apply_range((x1, y1), (x2, y2), Cell::reset);
    }

    fn apply_range(&mut self, (x1, y1): (u8, u8), (x2, y2): (u8, u8), f: impl Fn(&mut Cell)) {
        self.range_mut((x1, y1), (x2, y2)).for_each(f);
    }

    fn range_mut(&mut self, (x1, y1): (u8, u8), (x2, y2): (u8, u8)) -> impl Iterator<Item = &mut Cell> {
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
}

impl Context {
    pub(crate) fn new(display_component: DisplayComponent) -> Self {
        Self {
            display_component,
            screen_width: 40,
            screen_height: 24,

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
        }
    }

    fn consume(&mut self, sequence: Sequence) {
        if self.ignore_sequences && !matches!(sequence, Sequence::Escaped(EscapedSequence::Ignore(Some(false)))) {
            return;
        }

        match sequence {
            Sequence::Incomplete => {}
            Sequence::Escaped(esc) => match esc {
                //todo: seulement en mode alphabétique sinon on applique direct?
                EscapedSequence::Background(color) => self.pending_attributes.set_background(color),
                EscapedSequence::Foreground(color) => self.attributes.foreground = color,
                EscapedSequence::Csi(csi) => match csi {
                    //these four sequences do not support wrapping and the cursor will just
                    //stop moving when it reaches the border of the screen (p94 and 95)
                    //todo: check if we need to copy zone attributes as well ? probably need to copy on all operations ?
                    Csi::MoveUp(offset) => self.move_cursor_y(-(offset as i8), false),
                    Csi::MoveDown(offset) => self.move_cursor_y(offset as i8, false),
                    Csi::MoveRight(offset) => self.move_cursor_x(offset as i8, false),
                    Csi::MoveLeft(offset) => self.move_cursor_x(-(offset as i8), false),

                    Csi::SetCursor(x, y) => {
                        self.set_cursor(x, y, false);
                    },
                    Csi::InsertRowsFromCursor(count)  => self.grid.insert_rows_after(count, self.cursor_y),
                    Csi::EraseRowsFromCursor(count) => self.grid.delete_rows_after(count, self.cursor_y),
                    //todo: check if attributes and pending attributes are reset?
                    Csi::ClearRow => self.grid.clear_row(self.cursor_y),
                    Csi::InsertSpacesFromCursorToEol => panic!("CAN not yet implemented"),
                    Csi::ClearFromCursorToEos => self.grid.clear_after(self.cursor_x, self.cursor_y),
                    Csi::ClearFromSosToCursor => self.grid.clear_before(self.cursor_x, self.cursor_y),
                    Csi::ClearScreenKeepCursorPos => self.grid.reset(),
                    Csi::ClearFromCursorToEol => self.grid.clear_in_row_after(self.cursor_x, self.cursor_y),
                    Csi::ClearFromSolToCursor => self.grid.clear_in_row_before(self.cursor_x, self.cursor_y),
                    Csi::ClearAfterCursor(count) => self.grid.delete_after(count, self.cursor_x, self.cursor_y),
                    Csi::InsertFromCursor(count) => self.grid.insert_after(count, self.cursor_x, self.cursor_y),
                    Csi::StartInsert => self.insert = true,
                    Csi::EndInsert => self.insert = true,
                    _ => panic!("Received incomplete CSI sequence {:?}", csi),
                }
                EscapedSequence::NormalSize | EscapedSequence::DoubleSize
                | EscapedSequence::DoubleHeight | EscapedSequence::DoubleWidth
                | EscapedSequence::Invert(_)
                if self.attributes.character_set == CharacterSet::G1 => {
                    panic!("Changing invert and character size while in G1 is not supported");
                },
                EscapedSequence::Blink(blink) => self.attributes.blinking = blink,
                EscapedSequence::Invert(invert) => self.attributes.invert = invert,
                EscapedSequence::NormalSize => {
                    self.attributes.double_height = false;
                    self.attributes.double_width = false;
                },
                EscapedSequence::DoubleHeight | EscapedSequence::DoubleSize if self.cursor_y <= 1 => {
                    panic!("Tried to set double height or double size while in row 0 or 1"); //p93
                },
                EscapedSequence::DoubleHeight => self.attributes.double_height = true,
                EscapedSequence::DoubleWidth => self.attributes.double_width = true,
                EscapedSequence::DoubleSize => {
                    self.attributes.double_height = true;
                    self.attributes.double_width = true;
                },
                //todo: seulement en mode alphabétique sinon on applique direct?
                EscapedSequence::Underline(underline) => self.pending_attributes.set_underline(underline),
                //todo: seulement en mode alphabétique sinon on applique direct?
                EscapedSequence::Mask(mask) => self.pending_attributes.set_mask(mask),
                EscapedSequence::Ignore(Some(ignore)) => self.ignore_sequences = ignore,
                EscapedSequence::Protocol(_) => {} //todo: handle
                EscapedSequence::GetCursorPosition => {} //todo: handle
                EscapedSequence::ScreenMasking(mask) => self.screen_mask = mask,
                _ => panic!("Received incomplete escaped sequence {:?}", esc),
            }
            Sequence::SetCharacterSet(set) => {
                self.attributes.character_set = set;
                self.attributes.underline = false; //p94 todo: on peut supprimer?
                self.pending_attributes.underline = None; //p94

                //documented p91
                if self.attributes.character_set == CharacterSet::G1 {
                    self.attributes.invert = false;
                    self.attributes.double_height = false;
                    self.attributes.double_width = false;
                }
            },
            Sequence::SpecialCharacter(character) => self.print(character.to_character()),
            Sequence::SimpleCharacter(character) => self.print(character.to_character()),
            Sequence::SemiGraphicCharacter(character) => self.print(character.0 as char),
            Sequence::MoveCursor(direction) => match direction {
                Direction::Up => self.move_cursor_y(-1, true),
                Direction::Down => self.move_cursor_y(1, true),
                Direction::Right => self.move_cursor_x(1, true),
                Direction::Left => self.move_cursor_x(-1, true),
            },
            //todo: check if we should reset attributes or not
            Sequence::CarriageReturn => self.cursor_x = 1,
            Sequence::RecordSeparator => self.set_cursor(1, 1, false),
            Sequence::ClearScreen => {
                self.set_cursor(1, 1, false);
                self.reset_screen();
            }
            Sequence::SubSection(x, y) => {
                //the only way to access row 0 documented p97
                self.set_cursor(x.unwrap(), y.unwrap(), true);
                self.reset_attributes();
            },
            Sequence::Repeat(value) => {
                if let Some(previous_char) = self.grid.previous_cell(self.cursor_x, self.cursor_y).map(|cell| cell.content) {
                    if previous_char == '\0' {
                        panic!("Tried to repeat a character but no character was present");
                    }

                    for _ in 0..value.unwrap() {
                        self.print(previous_char);
                    }
                } else {
                    panic!("Tried to repeat a character at the beginning of the screen");
                }
            },
            Sequence::VisibleCursor(value) => self.visible_cursor = value,
            Sequence::Beep => {} //todo: handle beep idk how
        }
    }

    fn set_cursor(&mut self, x: u8, y: u8, allow_row_zero: bool) {
        let minimum_y = if allow_row_zero {
            0
        } else {
            1
        };

        if x > self.screen_width || x < 1 || y > self.screen_height || y < minimum_y {
            panic!("Tried to move cursor outside of screen ({}, {})", x, y);
        }

        self.cursor_x = x;
        self.cursor_y = y;

        //copy new position's zone attributes
        if let Some(cell) = self.grid.cell_opt(x - 1, y) {
            self.attributes.copy_zone_attributes(&cell.attributes);
        } else {
            self.attributes.reset_zone_attributes();
        }
    }

    /// Moves the cursor horizontally by the given amount of characters.
    /// If the cursor reaches the end of the screen, it will wrap to the
    /// next line if `wrap` is set to `true` else it will stay at the border.
    /// This function does not allow moving in row 0 (not yet implemented, p97)
    fn move_cursor_x(&mut self, x: i8, wrap: bool) {
        if self.cursor_y == 0 {
            todo!("Moving cursor in row 0 is not yet supported");
        }

        if !wrap {
            self.cursor_x = (self.cursor_x as i8 + x).clamp(1, self.screen_width as i8) as u8;
            return;
        }

        let x_translation_total = self.cursor_x as i16 + x as i16 - 1;
        let y_offset = x_translation_total.div_euclid(self.screen_width as i16);
        let new_x = x_translation_total.rem_euclid(self.screen_width as i16) + 1;
        let new_y = self.cursor_y as i16 + y_offset;

        self.cursor_x = new_x as u8;
        if self.page_mode == PageMode::Scroll {
            self.grid.scroll(y_offset as i8);
            self.cursor_y = new_y.clamp(1, self.screen_height as i16) as u8;
        } else {
            self.cursor_y = new_y.rem_euclid(self.screen_height as i16) as u8;
        }

        if y_offset != 0 {
            self.attributes.reset_zone_attributes();
        }
    }

    /// Moves the cursor vertically by the given amount of characters.
    /// If the cursor reaches the end of the screen, it will wrap to the
    /// other side by staying in the same column if `wrap` is set to `true`
    /// else it will stay at the border.
    /// This function does not allow accessing row 0 however it should
    /// allow getting out of row 0 (not yet implemented, p97)
    fn move_cursor_y(&mut self, y: i8, wrap: bool) {
        if self.cursor_y == 0 {
            todo!("Moving cursor in row 0 is not yet supported");
        }

        if !wrap {
            self.cursor_y = (self.cursor_y as i8 + y).clamp(1, self.screen_height as i8) as u8;
            return;
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
            self.cursor_y = new_y.rem_euclid(self.screen_width as i16) as u8;
        }

        self.attributes.reset_zone_attributes();
    }

    fn print(&mut self, character: char) {
        let is_delimiter = self.attributes.apply_pending(character, &mut self.pending_attributes);

        let cell = self.grid.cell_mut(self.cursor_x, self.cursor_y);
        cell.set_content(character);
        cell.set_delimiter(is_delimiter);
        cell.set_attributes(self.attributes);

        self.grid.recalculate_row_attributes(self.cursor_x, self.cursor_y);

        self.next(1);
    }

    /// Moves the cursor to the right by the given amount of characters.
    /// If the cursor reaches the end of the screen, it will wrap to the next line.
    fn next(&mut self, amount: u8) {
        //todo: check how double width and height behave when multiline
        //todo: check zone attributes with double width and height
        let amount = if self.attributes.double_width {
            amount * 2
        } else {
            amount
        } as i8;

        self.move_cursor_x(amount, true);
    }

    fn reset_screen(&mut self) {
        self.grid.reset();
        self.attributes.reset();
        self.pending_attributes.reset();
    }

    fn reset_attributes(&mut self) {
        self.attributes.reset();
        self.pending_attributes.reset();
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

    pub fn consume_all(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.consume(*byte);
        }
    }

    pub fn consume(&mut self, byte: u8) {
        self.sequence = self.sequence.consume(&self.ctx, byte);

        if self.sequence.is_complete() {
            let complete_sequence = mem::replace(&mut self.sequence, Sequence::Incomplete);
            self.ctx.consume(complete_sequence);
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

        assert_eq!(CharacterSet::new(&ctx, 0x0F), CharacterSet::G0);
        assert_eq!(CharacterSet::new(&ctx, 0x0E), CharacterSet::G1);

        assert!(CharacterSet::G0.is_complete());
        assert!(CharacterSet::G1.is_complete());

        assert_panics!(CharacterSet::new(&ctx, SI).consume(&ctx, 0x00));
        assert_panics!(CharacterSet::new(&ctx, SI).consume(&ctx, 0x0E));
        assert_panics!(CharacterSet::new(&ctx, SI).consume(&ctx, 0x00));

        assert_panics!(CharacterSet::new(&ctx, SO).consume(&ctx, 0x00));
        assert_panics!(CharacterSet::new(&ctx, SO).consume(&ctx, 0x0F));
        assert_panics!(CharacterSet::new(&ctx, SO).consume(&ctx, 0x00));
    }

    #[test]
    fn test_simple_character() {
        let mut ctx = Context::new(DisplayComponent::VGP2);
        ctx.attributes.character_set = CharacterSet::G0;

        assert_eq!(SimpleCharacter::new(&ctx, 0x20), SimpleCharacter(0x20));
        assert_eq!(SimpleCharacter::new(&ctx, 0x3F), SimpleCharacter(0x3F));
        assert_eq!(SimpleCharacter::new(&ctx, 0x4A), SimpleCharacter(0x4A));
        assert_eq!(SimpleCharacter::new(&ctx, 0x60), SimpleCharacter(0x60));
        assert_eq!(SimpleCharacter::new(&ctx, 0x7F), SimpleCharacter(0x7F));

        assert!(!SimpleCharacter::supports(&ctx, 0x00));
        assert!(!SimpleCharacter::supports(&ctx, 0x1F));
        assert!(SimpleCharacter::supports(&ctx, 0x20));
        assert!(SimpleCharacter::supports(&ctx, 0x7F));

        assert!(SimpleCharacter::new(&ctx, 0x20).is_complete());
        assert!(SimpleCharacter::new(&ctx, 0x7F).is_complete());
    }

    #[test]
    fn test_simple_character_wrong_set() {
        let mut ctx = Context::new(DisplayComponent::VGP2);
        ctx.attributes.character_set = CharacterSet::G1;

        assert!(!SimpleCharacter::supports(&ctx, 0x00));
        assert!(!SimpleCharacter::supports(&ctx, 0x1F));
        assert!(!SimpleCharacter::supports(&ctx, 0x20));
        assert!(!SimpleCharacter::supports(&ctx, 0x3F));

        assert_panics!(SimpleCharacter::new(&ctx, 0x20));
        assert_panics!(SimpleCharacter::new(&ctx, 0x3F));
        assert_panics!(SimpleCharacter::new(&ctx, 0x4A));
        assert_panics!(SimpleCharacter::new(&ctx, 0x60));
    }

    #[test]
    fn test_semigraphic_character() {
        let mut ctx = Context::new(DisplayComponent::VGP2);
        ctx.attributes.character_set = CharacterSet::G1;

        assert_eq!(SemiGraphicCharacter::new(&ctx, 0x20), SemiGraphicCharacter(0x20));
        assert_eq!(SemiGraphicCharacter::new(&ctx, 0x3F), SemiGraphicCharacter(0x3F));
        assert_eq!(SemiGraphicCharacter::new(&ctx, 0x60), SemiGraphicCharacter(0x60));
        assert_eq!(SemiGraphicCharacter::new(&ctx, 0x7F), SemiGraphicCharacter(0x7F));

        assert!(!SemiGraphicCharacter::supports(&ctx, 0x00));
        assert!(!SemiGraphicCharacter::supports(&ctx, 0x1F));
        assert!(!SemiGraphicCharacter::supports(&ctx, 0x40));
        assert!(SemiGraphicCharacter::supports(&ctx, 0x5F));
        assert!(SemiGraphicCharacter::supports(&ctx, 0x60));
        assert!(SemiGraphicCharacter::supports(&ctx, 0x7F));

        assert!(SemiGraphicCharacter::new(&ctx, 0x20).is_complete());
        assert!(SemiGraphicCharacter::new(&ctx, 0x7F).is_complete());
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

        assert_panics!(SemiGraphicCharacter::new(&ctx, 0x20));
        assert_panics!(SemiGraphicCharacter::new(&ctx, 0x3F));
        assert_panics!(SemiGraphicCharacter::new(&ctx, 0x45));
        assert_panics!(SemiGraphicCharacter::new(&ctx, 0x5F));
        assert_panics!(SemiGraphicCharacter::new(&ctx, 0x60));
        assert_panics!(SemiGraphicCharacter::new(&ctx, 0x7F));
    }

    #[test]
    fn test_special_character() {
        let ctx = Context::new(DisplayComponent::VGP2);

        assert_eq!(SpecialCharacter::new(&ctx, 0x19), SpecialCharacter::Incomplete);

        assert!(SpecialCharacter::supports(&ctx, 0x19));
        assert!(!SpecialCharacter::supports(&ctx, 0x1B));
        assert!(!SpecialCharacter::supports(&ctx, 0x00));
        assert!(!SpecialCharacter::supports(&ctx, 0x1F));
        assert!(!SpecialCharacter::supports(&ctx, 0x7F));

        assert_eq!(SpecialCharacter::new(&ctx, 0x19), SpecialCharacter::Incomplete);

        assert_panics!(SpecialCharacter::new(&ctx, 0x00));
        assert_panics!(SpecialCharacter::new(&ctx, 0x1B));
        assert_panics!(SpecialCharacter::new(&ctx, 0x1F));

        assert_eq!(SpecialCharacter::new(&ctx, 0x19).consume(&ctx, GRAVE), SpecialCharacter::Grave(None));
        assert_eq!(SpecialCharacter::new(&ctx, 0x19).consume(&ctx, ACUTE), SpecialCharacter::Acute(None));
        assert_eq!(SpecialCharacter::new(&ctx, 0x19).consume(&ctx, CIRCUMFLEX), SpecialCharacter::Circumflex(None));
        assert_eq!(SpecialCharacter::new(&ctx, 0x19).consume(&ctx, DIAERESIS), SpecialCharacter::Diaeresis(None));
        assert_eq!(SpecialCharacter::new(&ctx, 0x19).consume(&ctx, CEDILLA), SpecialCharacter::Cedilla(None));
        assert_eq!(SpecialCharacter::new(&ctx, 0x19).consume(&ctx, LOWER_OE), SpecialCharacter::LowerOE);
        assert_eq!(SpecialCharacter::new(&ctx, 0x19).consume(&ctx, UPPER_OE), SpecialCharacter::UpperOE);
        assert_eq!(SpecialCharacter::new(&ctx, 0x19).consume(&ctx, POUND), SpecialCharacter::Pound);
        assert_eq!(SpecialCharacter::new(&ctx, 0x19).consume(&ctx, DOLLAR), SpecialCharacter::Dollar);
        assert_eq!(SpecialCharacter::new(&ctx, 0x19).consume(&ctx, NUMBER_SIGN), SpecialCharacter::NumberSign);
        assert_eq!(SpecialCharacter::new(&ctx, 0x19).consume(&ctx, ARROW_LEFT), SpecialCharacter::ArrowLeft);
        assert_eq!(SpecialCharacter::new(&ctx, 0x19).consume(&ctx, ARROW_UP), SpecialCharacter::ArrowUp);
        assert_eq!(SpecialCharacter::new(&ctx, 0x19).consume(&ctx, ARROW_RIGHT), SpecialCharacter::ArrowRight);
        assert_eq!(SpecialCharacter::new(&ctx, 0x19).consume(&ctx, ARROW_DOWN), SpecialCharacter::ArrowDown);
        assert_eq!(SpecialCharacter::new(&ctx, 0x19).consume(&ctx, DEGREE), SpecialCharacter::Degree);
        assert_eq!(SpecialCharacter::new(&ctx, 0x19).consume(&ctx, PLUS_OR_MINUS), SpecialCharacter::PlusOrMinus);
        assert_eq!(SpecialCharacter::new(&ctx, 0x19).consume(&ctx, OBELUS), SpecialCharacter::Obelus);
        assert_eq!(SpecialCharacter::new(&ctx, 0x19).consume(&ctx, ONE_QUARTER), SpecialCharacter::OneQuarter);
        assert_eq!(SpecialCharacter::new(&ctx, 0x19).consume(&ctx, ONE_HALF), SpecialCharacter::OneHalf);
        assert_eq!(SpecialCharacter::new(&ctx, 0x19).consume(&ctx, THREE_QUARTERS), SpecialCharacter::ThreeQuarters);

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
        assert_eq!(SpecialCharacter::Grave(None).consume(&ctx, 'a' as u8), SpecialCharacter::Grave(Some('a' as u8)));
        assert_eq!(SpecialCharacter::Acute(None).consume(&ctx, 'e' as u8), SpecialCharacter::Acute(Some('e' as u8)));
        assert_eq!(SpecialCharacter::Circumflex(None).consume(&ctx, 'o' as u8), SpecialCharacter::Circumflex(Some('o' as u8)));
        assert_eq!(SpecialCharacter::Diaeresis(None).consume(&ctx, 'i' as u8), SpecialCharacter::Diaeresis(Some('i' as u8)));
        assert_eq!(SpecialCharacter::Cedilla(None).consume(&ctx, 'c' as u8), SpecialCharacter::Cedilla(Some('c' as u8)));

        assert_panics!(SpecialCharacter::Acute(None).consume(&ctx, 'a' as u8));
        assert_panics!(SpecialCharacter::Acute(None).consume(&ctx, 'o' as u8));
        assert_panics!(SpecialCharacter::Grave(None).consume(&ctx, 'c' as u8));
        assert_panics!(SpecialCharacter::Cedilla(None).consume(&ctx, 'i' as u8));
        assert_panics!(SpecialCharacter::Diaeresis(None).consume(&ctx, 'a' as u8));
        assert_panics!(SpecialCharacter::Diaeresis(None).consume(&ctx, 'o' as u8));
        assert_panics!(SpecialCharacter::Diaeresis(None).consume(&ctx, 'u' as u8));
        assert_panics!(SpecialCharacter::new(&ctx, 0x19).consume(&ctx, ESZETT));
        assert_panics!(SpecialCharacter::new(&ctx, 0x19).consume(&ctx, PARAGRAPH));
    }

    #[test]
    fn test_special_character_vgp5_specific() {
        let ctx = Context::new(DisplayComponent::VGP5);
        assert_eq!(SpecialCharacter::Diaeresis(None).consume(&ctx, 'a' as u8), SpecialCharacter::Diaeresis(Some('a' as u8)));
        assert_eq!(SpecialCharacter::Diaeresis(None).consume(&ctx, 'o' as u8), SpecialCharacter::Diaeresis(Some('o' as u8)));
        assert_eq!(SpecialCharacter::Diaeresis(None).consume(&ctx, 'u' as u8), SpecialCharacter::Diaeresis(Some('u' as u8)));
        assert_eq!(SpecialCharacter::new(&ctx, 0x19).consume(&ctx, ESZETT), SpecialCharacter::Eszett);
        assert_eq!(SpecialCharacter::new(&ctx, 0x19).consume(&ctx, PARAGRAPH), SpecialCharacter::Paragraph);
    }

    #[test]
    fn test_csi() {
        let ctx = Context::new(DisplayComponent::VGP2);

        assert_eq!(Csi::new(&ctx, 0x5B), Csi::Incomplete);

        assert!(Csi::supports(&ctx, 0x5B));
        assert!(!Csi::supports(&ctx, 0x00));
        assert!(!Csi::supports(&ctx, 0x1F));

        assert!(!Csi::new(&ctx, 0x5B).is_complete());

        assert_panics!(Csi::new(&ctx, 0x00));
        assert_panics!(Csi::new(&ctx, 0x1F));

        assert_panics!(Csi::new(&ctx, 0x5B).consume(&ctx, 0x00));
        assert_panics!(Csi::new(&ctx, 0x5B).consume(&ctx, 0x11));
        assert_panics!(Csi::new(&ctx, 0x5B).consume(&ctx, 0x7F));
    }

    #[test]
    fn test_csi_move_cursor() {
        let ctx = Context::new(DisplayComponent::VGP2);

        let move_left_29 = Csi::new(&ctx, 0x5B)
            .consume(&ctx, 0x32)
            .consume(&ctx, 0x39)
            .consume(&ctx, 0x44);

        assert_eq!(move_left_29, Csi::MoveLeft(29));
        assert!(move_left_29.is_complete());

        let move_right_11 = Csi::new(&ctx, 0x5B)
            .consume(&ctx, 0x31)
            .consume(&ctx, 0x31)
            .consume(&ctx, 0x43);

        assert_eq!(move_right_11, Csi::MoveRight(11));
        assert!(move_right_11.is_complete());

        let move_up_7 = Csi::new(&ctx, 0x5B)
            .consume(&ctx, 0x37)
            .consume(&ctx, 0x41);

        assert_eq!(move_up_7, Csi::MoveUp(7));
        assert!(move_up_7.is_complete());

        let move_down_3 = Csi::new(&ctx, 0x5B)
            .consume(&ctx, 0x30)
            .consume(&ctx, 0x33)
            .consume(&ctx, 0x42);

        assert_eq!(move_down_3, Csi::MoveDown(3));
        assert!(move_down_3.is_complete());

        let move_right_1 = Csi::new(&ctx, 0x5B)
            .consume(&ctx, 0x31)
            .consume(&ctx, 0x43);

        assert_eq!(move_right_1, Csi::MoveRight(1));
        assert!(move_right_1.is_complete());

        let move_left_0 = Csi::new(&ctx, 0x5B)
            .consume(&ctx, 0x30)
            .consume(&ctx, 0x44);

        assert_eq!(move_left_0, Csi::MoveLeft(0));
        assert!(move_left_0.is_complete());

        assert_panics!(Csi::new(&ctx, 0x5B).consume(&ctx, 0x44));
        assert_panics!(Csi::new(&ctx, 0x5B).consume(&ctx, 0x41));
    }

    #[test]
    fn test_csi_set_cursor() {
        let ctx = Context::new(DisplayComponent::VGP2);

        let set_cursor_0_1 = Csi::new(&ctx, 0x5B)
            .consume(&ctx, 0x31)
            .consume(&ctx, 0x3B)
            .consume(&ctx, 0x31)
            .consume(&ctx, 0x48);

        assert_eq!(set_cursor_0_1, Csi::SetCursor(1, 1));
        assert!(set_cursor_0_1.is_complete());

        let set_cursor_1_2 = Csi::new(&ctx, 0x5B)
            .consume(&ctx, 0x32)
            .consume(&ctx, 0x3B)
            .consume(&ctx, 0x31)
            .consume(&ctx, 0x48);

        assert_eq!(set_cursor_1_2, Csi::SetCursor(1, 2));
        assert!(set_cursor_1_2.is_complete());

        let set_cursor_34_13 = Csi::new(&ctx, 0x5B)
            .consume(&ctx, 0x31)
            .consume(&ctx, 0x33)
            .consume(&ctx, 0x3B)
            .consume(&ctx, 0x33)
            .consume(&ctx, 0x34)
            .consume(&ctx, 0x48);

        assert_eq!(set_cursor_34_13, Csi::SetCursor(34, 13));
        assert!(set_cursor_34_13.is_complete());
    }

    #[test]
    fn test_invalid_set_cursor() {
        let ctx = Context::new(DisplayComponent::VGP2);

        assert_panics!(Csi::new(&ctx, 0x5B).consume(&ctx, 0x31).consume(&ctx, 0x3B).consume(&ctx, 0x48));
        assert_panics!(Csi::new(&ctx, 0x5B).consume(&ctx, 0x3F));
        assert_panics!(Csi::new(&ctx, 0x5B).consume(&ctx, 0x31).consume(&ctx, 0x3B).consume(&ctx, 0x3B));
        assert_panics!(Csi::new(&ctx, 0x5B).consume(&ctx, 0x31).consume(&ctx, 0x3B).consume(&ctx, 0x30).consume(&ctx, 0x3B));
        assert_panics!(Csi::new(&ctx, 0x5B).consume(&ctx, 0x31).consume(&ctx, 0x3B).consume(&ctx, 0x30).consume(&ctx, 0x48).consume(&ctx, 0x48));
    }
}
