pub use teletel_derive::sg;

#[macro_export]
macro_rules! send {
    ($mt:expr, [$($code:expr),+ $(,)?]) => {{
        $($crate::protocol::ToMinitel::to_minitel(&$code, $mt);)+
        $crate::Minitel::flush($mt)
    }};
}

#[macro_export]
macro_rules! list {
    ($($code:expr),+ $(,)?) => {{
        |mt: &mut $crate::Minitel| {
            $($crate::protocol::ToMinitel::to_minitel(&$code, mt);)+
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_empty_and_full() {
        assert_eq!(sg!(00/00/00), 0x20);
        assert_eq!(sg!(11/11/11), 0x5F);
    }

    #[test]
    pub fn test_semi_graphic_below_0x40() {
        assert_eq!(sg!(10/00/00), 0x21);
        assert_eq!(sg!(11/10/00), 0x27);
        assert_eq!(sg!(01/01/10), 0x3A);
        assert_eq!(sg!(01/11/10), 0x3E);
    }

    #[test]
    pub fn test_semi_graphic_above_0x40() {
        assert_eq!(sg!(00/00/01), 0x60);
        assert_eq!(sg!(10/00/01), 0x61);
        assert_eq!(sg!(11/10/01), 0x67);
        assert_eq!(sg!(01/01/11), 0x7A);
        assert_eq!(sg!(01/11/11), 0x7E);
    }
}
