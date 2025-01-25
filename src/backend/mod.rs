#[cfg(feature = "serial")]
mod serial;

#[cfg(feature = "serial")]
pub use serial::{BaudRate, SerialBackend};

pub trait Backend {
    fn send(&mut self, bytes: &[u8]);
}
