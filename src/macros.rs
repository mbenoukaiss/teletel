pub use teletel_derive::sg;

#[macro_export]
macro_rules! send {
    ($term:expr, [$($code:expr),+ $(,)?]) => {{
        $($crate::terminal::ToTerminal::to_terminal(&$code, $term)?;)+
        $crate::terminal::WriteableTerminal::flush($term)
    }};
    ($term:expr, $code:expr) => {
        send!($term, [$code])
    };
}

#[macro_export]
macro_rules! list {
    ($($code:expr),+ $(,)?) => {{
        |term: &mut dyn $crate::terminal::WriteableTerminal| -> std::result::Result<(), $crate::Error> {
            $($crate::terminal::ToTerminal::to_terminal(&$code, term)?;)+

            std::result::Result::Ok(())
        }
    }};
}

#[cfg(test)]
#[macro_export]
macro_rules! assert_panics {
    (|| $code:expr) => {
        assert_panics!($code);
    };
    ($code:expr) => {
        assert!(std::panic::catch_unwind(|| $code).is_err());
    };
    ($code:block) => {
        assert!(std::panic::catch_unwind(|| $code).is_err());
    };
}

#[cfg(test)]
#[macro_export]
macro_rules! assert_err {
    ($code:expr, $expected_err:literal$(,)?) => {
        assert_eq!(&format!("{}", $code.unwrap_err()), $expected_err);
    };
}


#[cfg(test)]
#[macro_export]
macro_rules! assert_times_out {
    ($d:expr, $code:expr) => {
        let (done_tx, done_rx) = std::sync::mpsc::channel();
        let handle = std::thread::spawn(move || {
            $code();

            done_tx.send(()).expect("Unable to send completion signal");
        });

        match done_rx.recv_timeout($d) {
            Ok(_) => {
                handle.join().expect("Thread panicked");
                panic!("Thread did not time out");
            },
            Err(_) => (),
        }
    };
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
    pub fn test_works_with_spaces() {
        assert_eq!(sg!(00      / 00 /   00     ), 0x20);
        assert_eq!(sg!(11 /     11 / 11), 0x5F);
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
