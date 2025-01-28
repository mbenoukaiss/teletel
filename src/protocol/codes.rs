/// Activates the G1 character set which contains semi-graphic
/// characters. Once activated the G1 character set remains active
/// until deactivated by `SI` code.
///
/// Semi-graphic codes can be easily created using the `sg!` macro.
pub const SO: u8 = 0x0E;

/// Deactivates the G1 character set and switch back to G0.
pub const SI: u8 = 0x0F;

/// The C1 character set which contains formatting codes like
/// coloring, underlining, size, blinking etc. Each code must
/// be preceded by the ESC character (0x1B) to enable C1.
///
/// Documented on pages 91 and 92.
pub mod c1 {
    /// Activates the C1 character set. It is only temporarily
    /// activated for the next code that is sent and must thus
    /// be sent before each C1 character.
    pub const ESC: u8 = 0x1B;
    pub const BLACK_CHARACTER: u8 = 0x40;
    pub const RED_CHARACTER: u8 = 0x41;
    pub const GREEN_CHARACTER: u8 = 0x42;
    pub const YELLOW_CHARACTER: u8 = 0x43;
    pub const BLUE_CHARACTER: u8 = 0x44;
    pub const MAGENTA_CHARACTER: u8 = 0x45;
    pub const CYAN_CHARACTER: u8 = 0x46;
    pub const WHITE_CHARACTER: u8 = 0x47;
    pub const BLACK_BACKGROUND: u8 = 0x50;
    pub const RED_BACKGROUND: u8 = 0x51;
    pub const GREEN_BACKGROUND: u8 = 0x52;
    pub const YELLOW_BACKGROUND: u8 = 0x53;
    pub const BLUE_BACKGROUND: u8 = 0x54;
    pub const MAGENTA_BACKGROUND: u8 = 0x55;
    pub const CYAN_BACKGROUND: u8 = 0x56;
    pub const WHITE_BACKGROUND: u8 = 0x57;
    pub const BLINK: u8 = 0x48;
    pub const STILL: u8 = 0x49;
    pub const START_INVERT: u8 = 0x5D;
    pub const STOP_INVERT: u8 = 0x5C;
    pub const NORMAL_SIZE: u8 = 0x4C;
    pub const DOUBLE_HEIGHT: u8 = 0x4D;
    pub const DOUBLE_WIDTH: u8 = 0x4E;
    pub const DOUBLE_SIZE: u8 = 0x4F;
    pub const START_UNDERLINE: u8 = 0x5A;
    pub const STOP_UNDERLINE: u8 = 0x59;
    pub const MASK: u8 = 0x58;
    pub const UNMASK: u8 = 0x5F;
}

/// The G2 character set which contains accented and other special
/// characters. This character set can not be activated while the G1
/// character set is active
///
/// Documented on pages 88 to 90.
/// Table with the whole character set on pages 103 or 104
/// depending on the display system.
pub mod ss2 {
    /// Activates the G2 character set. It is only temporarily
    /// activated for the next character that is sent and must thus
    /// be sent before each G2 character.
    pub const SS2: u8 = 0x19;

    pub const GRAVE: u8 = 0x41;
    pub const ACUTE: u8 = 0x42;
    pub const CIRCUMFLEX: u8 = 0x43;
    pub const DIAERESIS: u8 = 0x48;
    pub const CEDILLA: u8 = 0x4B;
    pub const LOWER_OE: u8 = 0x7A;
    pub const UPPER_OE: u8 = 0x6A;
    pub const ESZETT: u8 = 0x7B;
    pub const POUND: u8 = 0x23;
    pub const DOLLAR: u8 = 0x24;
    pub const NUMBER_SIGN: u8 = 0x26;
    pub const ARROW_LEFT: u8 = 0x2C;
    pub const ARROW_UP: u8 = 0x2D;
    pub const ARROW_RIGHT: u8 = 0x2E;
    pub const ARROW_DOWN: u8 = 0x2F;
    pub const PARAGRAPH: u8 = 0x27;
    pub const DEGREE: u8 = 0x30;
    pub const PLUS_OR_MINUS: u8 = 0x31;
    pub const OBELUS: u8 = 0x38;
    pub const ONE_QUARTER: u8 = 0x3C;
    pub const ONE_HALF: u8 = 0x3D;
    pub const THREE_QUARTERS: u8 = 0x3E;
}

/// Layout codes for moving the cursor around the screen.
///
/// Documented on page 94 to 97.
pub mod layout {
    use super::*;

    pub const CURSOR_LEFT: u8 = 0x08;
    pub const CURSOR_RIGHT: u8 = 0x09;
    pub const CURSOR_DOWN: u8 = 0x0A;
    pub const CURSOR_UP: u8 = 0x0B;
    pub const CR: u8 = 0x0D;
    pub const RS: u8 = 0x1E;
    pub const FF: u8 = 0x0C;
    pub const US: u8 = 0x1F;

    /// Fill the current line from the cursor position to the end
    /// of the line with spaces
    pub const CAN: u8 = 0x18;

    /// Erase characters from the cursor position to the end of the screen
    pub const CSI_J: [u8; 3] = [ESC, 0x5B, 0x4A];

    /// Erase characters from the beginning of the screen to the cursor position
    pub const CSI_1_J: [u8; 4] = [ESC, 0x5B, 0x31, 0x4A];

    /// Erase the whole screen, does reset the cursor position
    pub const CSI_2_J: [u8; 4] = [ESC, 0x5B, 0x32, 0x4A];

    /// Erase characters from the cursor position to the end of the row
    pub const CSI_K: [u8; 3] = [ESC, 0x5B, 0x4B];

    /// Erase characters from the beginning of the row to the cursor position
    pub const CSI_1_K: [u8; 4] = [ESC, 0x5B, 0x31, 0x4B];

    /// Erase all characters in the current row
    pub const CSI_2_K: [u8; 4] = [ESC, 0x5B, 0x32, 0x4B];
}

pub use c1::*;
pub use ss2::*;
pub use layout::*;

pub const BEEP: u8 = 0x07; //p98
pub const SCROLL_DOWN: u8 = 0x0A; //p34
pub const SCROLL_UP: u8 = 0x0B; //p34

/// Repeats a character a given number of times. The count
/// must be between 1 and 64 or the function will panic.
/// TODO: what about 80 characters wide screens in dual mode
///
/// Documented on page 98.
pub const fn repeat(character: u8, count: u8) -> [u8; 3] {
    assert!(count > 0);
    assert!(count <= 64);

    [character, 0x12, 0x40 + count - 1]
}

pub const fn to_decimal(value: u8) -> [u8; 2] {
    [0x30 + value / 10, 0x30 + value % 10]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repeat() {
        assert_eq!(repeat('A' as u8, 1), ['A' as u8, 0x12, 0x40]);
        assert_eq!(repeat('B' as u8, 2), ['B' as u8, 0x12, 0x41]);
        assert_eq!(repeat('C' as u8, 64), ['C' as u8, 0x12, 0x7F]);
    }

    #[test]
    fn test_repeat_fails() {
        assert!(std::panic::catch_unwind(|| repeat('A' as u8, 0)).is_err());
        assert!(std::panic::catch_unwind(|| repeat('A' as u8, 65)).is_err());
    }

    #[test]
    fn test_to_decimal() {
        assert_eq!(to_decimal(0), [0x30, 0x30]);
        assert_eq!(to_decimal(1), [0x30, 0x31]);
        assert_eq!(to_decimal(9), [0x30, 0x39]);
        assert_eq!(to_decimal(10), [0x31, 0x30]);
        assert_eq!(to_decimal(15), [0x31, 0x35]);
        assert_eq!(to_decimal(77), [0x37, 0x37]);
        assert_eq!(to_decimal(99), [0x39, 0x39]);
    }
}
