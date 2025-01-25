pub const SO: u8 = 0x0E; //p90
pub const SI: u8 = 0x0F; //p90

/// Activates the G2 character set which contains accented and
/// other special characters. It is only temporarily activated
/// for the next character that will be sent.
/// This character set can not be activated while the G1 character
/// set is active
///
/// Documented on pages 88 to 90.
/// Table with the whole character set on pages 103 or 104
/// depending on the display system.
pub const SS2: u8 = 0x19; //p88

pub const ESC: u8 = 0x1B; //p90
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
pub const GRAVE: u8 = 0x41; //p103
pub const ACUTE: u8 = 0x42; //p103
pub const CIRCUMFLEX: u8 = 0x43; //p103
pub const DIAERESIS: u8 = 0x48; //p103
pub const CEDILLA: u8 = 0x4B; //p103
pub const LOWER_OE: u8 = 0x7A; //p103
pub const UPPER_OE: u8 = 0x6A; //p103
pub const ESZETT: u8 = 0x7B; //p103
pub const POUND: u8 = 0x23; //p103
pub const DOLLAR: u8 = 0x24; //p103
pub const NUMBER_SIGN: u8 = 0x26; //p103
pub const ARROW_LEFT: u8 = 0x2C; //p103
pub const ARROW_UP: u8 = 0x2D; //p103
pub const ARROW_RIGHT: u8 = 0x2E; //p103
pub const ARROW_DOWN: u8 = 0x2F; //p103
pub const PARAGRAPH: u8 = 0x27; //p103
pub const DEGREE: u8 = 0x30; //p103
pub const PLUS_OR_MINUS: u8 = 0x31; //p103
pub const OBELUS: u8 = 0x38; //p103
pub const ONE_QUARTER: u8 = 0x3C; //p103
pub const ONE_HALF: u8 = 0x3D; //p103
pub const THREE_QUARTERS: u8 = 0x3E; //p103

pub const BEEP: u8 = 0x07; //p98
pub const CLEAR: u8 = 0x0C;
pub const SCROLL_DOWN: u8 = 0x0A; //p34
pub const SCROLL_UP: u8 = 0x0B; //p34
pub const SET_CURSOR: u8 = 0x1F; //p??

pub const fn repeat(character: u8, count: u8) -> [u8; 3] {
    assert!(count > 0);
    assert!(count <= 64);

    [character, 0x12, 0x40 + count - 1] //p98
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
}
