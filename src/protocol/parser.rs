use std::mem;
use crate::protocol::codes::*;

/// - If G2 character set is requested but the following code does not exist in G2 a
///  lower horizontal line will be displayed instead except if it's contained in C0 then
///  the C0 character is displayed and SS2 is ignored (??? end of p90).
/// - While G1 is active, any attempt to switch to G2 is ignored
/// - Attempting to use an accent on a character that does not support it will result
///  in displaying the character without the accent. VGP5 supports àâäéèêëîïöôùûüç and
///  VGP2 supports àâéèêëîïôùûç. No uppercase characters are supported.
/// - ß and § are only supported on VGP5
/// - Double size and double height are ignored in line 0 and 1
/// - Size and inverted are not ignored when in semi-graphic mode and are cleared when
///  switching to semi-graphic mode
/// - An attribute stops being applied and reset to their default value when starting
///  or exiting a section or a subsection
/// - Double height, width or size characters are displayed from the bottom left corner

#[derive(Eq, PartialEq)]
pub enum DisplayComponent {
    VGP2,
    VGP5,
}

pub enum PageMode {
    Page,
    Scroll,
}

pub struct Context {
    pub display_component: DisplayComponent,

    pub character_set: CharacterSet,
    pub esc: bool,
    pub page_mode: PageMode,
    pub visible_cursor: bool,

    pub cursor_x: u8,
    pub cursor_y: u8,

    pub background: u8,
    pub foreground: u8,
    pub blinking: bool,
    pub double_height: bool,
    pub double_width: bool,
    pub double_size: bool,
    pub inverted: bool,
    pub underline: bool, //+ caractère disjoint en mode semi-graphique
    pub masking: bool,
}

impl Context {
    pub fn new(display_component: DisplayComponent) -> Self {
        Self {
            display_component,

            character_set: CharacterSet::G0,
            esc: false,
            page_mode: PageMode::Page,
            visible_cursor: false,

            cursor_x: 0,
            cursor_y: 1,

            background: BLACK,
            foreground: WHITE,
            blinking: false,
            double_height: false,
            double_width: false,
            double_size: false,
            inverted: false,
            underline: false,
            masking: false,
        }
    }
}

pub trait Parsable {
    fn new(ctx: &Context, byte: u8) -> Self;
    fn supports(ctx: &Context, byte: u8) -> bool;
    fn consume(&mut self, ctx: &Context, byte: u8) -> Self;
    fn is_complete(&self) -> bool;
}

#[derive(Eq, PartialEq, Debug)]
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
pub struct SimpleCharacter(pub u8);

impl Parsable for SimpleCharacter {
    fn new(ctx: &Context, byte: u8) -> Self {
        if !Self::supports(ctx, byte) {
            panic!("Invalid simple character {:#04X}", byte);
        }

        SimpleCharacter(byte)
    }

    fn supports(ctx: &Context, byte: u8) -> bool {
        ctx.character_set == CharacterSet::G0 && byte >= 0x20 && byte <= 0x7F
    }

    fn consume(&mut self, _ctx: &Context, byte: u8) -> Self {
        panic!("Simple character {:?} does not support more bytes ({:#04X})", self, byte);
    }

    fn is_complete(&self) -> bool {
        true
    }
}

//fully implemented
#[derive(Eq, PartialEq, Debug)]
pub struct SemiGraphicCharacter(pub u8);

impl Parsable for SemiGraphicCharacter {
    fn new(ctx: &Context, byte: u8) -> Self {
        if !Self::supports(ctx, byte) {
            panic!("Invalid semi-graphic character {:#04X}", byte);
        }

        SemiGraphicCharacter(byte)
    }

    fn supports(ctx: &Context, byte: u8) -> bool {
        ctx.character_set == CharacterSet::G1 && (byte >= 0x20 && byte <= 0x3F || byte >= 0x60 && byte <= 0x7F)
    }

    fn consume(&mut self, _ctx: &Context, byte: u8) -> Self {
        panic!("Semi-graphic character {:?} does not support more bytes ({:#04X})", self, byte);
    }

    fn is_complete(&self) -> bool {
        true
    }
}

//fully implemented
#[derive(Eq, PartialEq, Debug)]
pub enum SpecialCharacter {
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
                ESZETT => SpecialCharacter::Eszett,
                POUND => SpecialCharacter::Pound,
                DOLLAR => SpecialCharacter::Dollar,
                NUMBER_SIGN => SpecialCharacter::NumberSign,
                ARROW_LEFT => SpecialCharacter::ArrowLeft,
                ARROW_UP => SpecialCharacter::ArrowUp,
                ARROW_RIGHT => SpecialCharacter::ArrowRight,
                ARROW_DOWN => SpecialCharacter::ArrowDown,
                PARAGRAPH => SpecialCharacter::Paragraph,
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
        match self {
            SpecialCharacter::Grave(None) => false,
            SpecialCharacter::Acute(None) => false,
            SpecialCharacter::Circumflex(None) => false,
            SpecialCharacter::Diaeresis(None) => false,
            SpecialCharacter::Cedilla(None) => false,
            _ => true,
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum Csi {
    Incomplete,
    Quantified(u8),
    MoveUp(u8),
    MoveDown(u8),
    MoveRight(u8),
    MoveLeft(u8),
    IncompleteSetCursor(u8, u8),
    SetCursor(u8, u8),
}

impl Parsable for Csi {
    fn new(ctx: &Context, byte: u8) -> Self {
        if !Self::supports(ctx, byte) {
            panic!("Unsupported or invalid CSI sequence starting with {:#04X}", byte);
        }

        Csi::Incomplete
    }

    fn supports(_ctx: &Context, byte: u8) -> bool {
        byte == CSI
    }

    fn consume(&mut self, _ctx: &Context, byte: u8) -> Self {
        match self {
            Csi::Incomplete => {
                if byte >= 0x30 && byte <= 0x39 {
                    Csi::Quantified(byte - 0x30)
                } else {
                    panic!("Unsupported or invalid CSI sequence starting with {:#04X}", byte);
                }
            }
            Csi::Quantified(value) => match byte {
                0x30..=0x39 => Csi::Quantified(*value * 10 + (byte - 0x30)),
                0x3B => Csi::IncompleteSetCursor(0, *value),
                0x41 => Csi::MoveUp(*value),
                0x42 => Csi::MoveDown(*value),
                0x43 => Csi::MoveRight(*value),
                0x44 => Csi::MoveLeft(*value),
                _ => panic!("Unsupported or invalid byte {:#04X} for quantified CSI sequence", byte),
            }
            Csi::IncompleteSetCursor(x, y) => match byte {
                0x30..=0x39 => Csi::IncompleteSetCursor(*x * 10 + (byte - 0x30), *y),
                0x48 => Csi::SetCursor(*x, *y),
                _ => panic!("Invalid byte {:#04X} for incomplete set cursor sequence", byte),
            }

            //TODO: implement other CSI sequences
            _ => panic!("Unsupported or invalid byte {:#04X} for sequence {:?}", byte, self),
        }
    }

    fn is_complete(&self) -> bool {
        match self {
            Csi::Incomplete => false,
            Csi::Quantified(_) => false,
            Csi::IncompleteSetCursor(_, _) => false,
            _ => true,
        }
    }
}

//fully implemented
#[derive(Eq, PartialEq, Debug)]
pub enum EscapedSequence {
    Incomplete,
    Csi(Csi),
    Background(u8),
    Foreground(u8),
    Blink,
    Still,
    StartInvert,
    StopInvert,
    NormalSize,
    DoubleHeight,
    DoubleWidth,
    DoubleSize,
    StartUnderline,
    StopUnderline,
    Mask,
    Unmask,
}

impl Parsable for EscapedSequence {
    fn new(ctx: &Context, byte: u8) -> Self {
        if !Self::supports(ctx, byte) {
            panic!("Unsupported or invalid escaped sequence starting with {:#04X}", byte);
        }

        if ctx.character_set == CharacterSet::G1 {
            panic!("Escaped sequences are not supported in G1");
        }

        EscapedSequence::Incomplete
    }

    fn supports(_ctx: &Context, byte: u8) -> bool {
        byte == ESC
    }

    fn consume(&mut self, ctx: &Context, byte: u8) -> Self {
        match self {
            EscapedSequence::Incomplete => match byte {
                0x40..=0x47 => EscapedSequence::Foreground(byte),
                0x50..=0x57 => EscapedSequence::Background(byte),
                BLINK => EscapedSequence::Blink,
                STILL => EscapedSequence::Still,
                START_INVERT => EscapedSequence::StartInvert,
                STOP_INVERT => EscapedSequence::StopInvert,
                NORMAL_SIZE => EscapedSequence::NormalSize,
                DOUBLE_HEIGHT => EscapedSequence::DoubleHeight,
                DOUBLE_WIDTH => EscapedSequence::DoubleWidth,
                DOUBLE_SIZE => EscapedSequence::DoubleSize,
                START_UNDERLINE => EscapedSequence::StartUnderline,
                STOP_UNDERLINE => EscapedSequence::StopUnderline,
                MASK => EscapedSequence::Mask,
                UNMASK => EscapedSequence::Unmask,
                CSI => EscapedSequence::Csi(Csi::new(ctx, byte)),
                _ => panic!("Invalid escaped sequence starting with {:#04X}", byte),
            },
            EscapedSequence::Csi(csi) => EscapedSequence::Csi(csi.consume(ctx, byte)),
            _ => panic!("Escaped sequence {:?} does not support additional bytes ({:#04X})", self, byte),
        }
    }

    fn is_complete(&self) -> bool {
        match self {
            EscapedSequence::Incomplete => false,
            EscapedSequence::Csi(csi) => csi.is_complete(),
            _ => true,
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum Sequence {
    Incomplete,
    EscapedSequence(EscapedSequence),
    SetCharacterSet(CharacterSet),
    SpecialCharacter(SpecialCharacter),
    SemiGraphicCharacter(SemiGraphicCharacter),
    SimpleCharacter(SimpleCharacter),
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
                    Sequence::EscapedSequence(EscapedSequence::new(ctx, byte))
                } else if CharacterSet::supports(ctx, byte) {
                    Sequence::SetCharacterSet(CharacterSet::new(ctx, byte))
                } else if SpecialCharacter::supports(ctx, byte) {
                    Sequence::SpecialCharacter(SpecialCharacter::new(ctx, byte))
                } else if SemiGraphicCharacter::supports(ctx, byte) {
                    Sequence::SemiGraphicCharacter(SemiGraphicCharacter::new(ctx, byte))
                } else if SimpleCharacter::supports(ctx, byte) {
                    Sequence::SimpleCharacter(SimpleCharacter::new(ctx, byte))
                } else {
                    panic!("Unsupported or invalid sequence starting with {:#04X}", byte)
                }
            }
            Sequence::EscapedSequence(escaped_sequence) => Sequence::EscapedSequence(escaped_sequence.consume(ctx, byte)),
            Sequence::SetCharacterSet(character_set) => Sequence::SetCharacterSet(character_set.consume(ctx, byte)),
            Sequence::SpecialCharacter(special_character) => Sequence::SpecialCharacter(special_character.consume(ctx, byte)),
            Sequence::SemiGraphicCharacter(semi_graphic_character) => Sequence::SemiGraphicCharacter(semi_graphic_character.consume(ctx, byte)),
            Sequence::SimpleCharacter(simple_character) => Sequence::SimpleCharacter(simple_character.consume(ctx, byte)),
        }
    }

    fn is_complete(&self) -> bool {
        match self {
            Sequence::Incomplete => false,
            Sequence::EscapedSequence(escaped_sequence) => escaped_sequence.is_complete(),
            Sequence::SetCharacterSet(character_set) => character_set.is_complete(),
            Sequence::SpecialCharacter(special_character) => special_character.is_complete(),
            Sequence::SemiGraphicCharacter(semi_graphic_character) => semi_graphic_character.is_complete(),
            Sequence::SimpleCharacter(simple_character) => simple_character.is_complete(),
        }
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

    pub fn consume(&mut self, byte: u8) -> Option<Sequence> {
        self.sequence = self.sequence.consume(&self.ctx, byte);

        if self.sequence.is_complete() {
            Some(mem::replace(&mut self.sequence, Sequence::Incomplete))
        } else {
            None
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
        ctx.character_set = CharacterSet::G0;

        assert_eq!(SimpleCharacter::new(&ctx, 0x20), SimpleCharacter(0x20));
        assert_eq!(SimpleCharacter::new(&ctx, 0x3F), SimpleCharacter(0x3F));
        assert_eq!(SimpleCharacter::new(&ctx, 0x4A), SimpleCharacter(0x4A));
        assert_eq!(SimpleCharacter::new(&ctx, 0x60), SimpleCharacter(0x60));
        assert_eq!(SimpleCharacter::new(&ctx, 0x7F), SimpleCharacter(0x7F));

        assert!(!SimpleCharacter::supports(&ctx, 0x00));
        assert!(!SimpleCharacter::supports(&ctx, 0x1F));
        assert!(SimpleCharacter::supports(&ctx, 0x20));
        assert!(SimpleCharacter::supports(&ctx, 0x7F));

        let mut ctx = Context::new(DisplayComponent::VGP2);
        ctx.character_set = CharacterSet::G1;

        assert!(!SimpleCharacter::supports(&ctx, 0x00));
        assert!(!SimpleCharacter::supports(&ctx, 0x1F));
        assert!(!SimpleCharacter::supports(&ctx, 0x20));
        assert!(!SimpleCharacter::supports(&ctx, 0x3F));

        assert_panics!(SimpleCharacter::new(&ctx, 0x20));
        assert_panics!(SimpleCharacter::new(&ctx, 0x3F));
        assert_panics!(SimpleCharacter::new(&ctx, 0x4A));
        assert_panics!(SimpleCharacter::new(&ctx, 0x60));
    }
}
