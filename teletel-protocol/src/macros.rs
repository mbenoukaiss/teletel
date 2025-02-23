#[cfg(test)]
#[macro_export]
macro_rules! assert_err {
    ($code:expr, $expected_err:literal$(,)?) => {
        assert_eq!(&format!("{}", $code.unwrap_err()), $expected_err);
    };
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
