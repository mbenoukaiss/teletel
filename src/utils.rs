
macro_rules! send {
    ($backend:ident, [$($code:expr),+ $(,)?]) => {
        let mut __buffer = $crate::codes::Buffer::new();
        $($crate::codes::ToBuffer::to_buffer(&$code, &mut __buffer);)+

        $crate::Backend::send(&mut $backend, __buffer);
    };
}
