#[macro_export]
macro_rules! send {
    ($backend:ident, [$($code:expr),+ $(,)?]) => {
        $($crate::protocol::ToBackend::to_backend(&$code, &mut $backend);)+
    };
}
