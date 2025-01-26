mod buffer;

#[cfg(feature = "serial")]
mod serial;

#[cfg(feature = "serial")]
pub use serial::{BaudRate, SerialReceiver};

pub trait TeletelReceiver {
    fn send(&mut self, bytes: &[u8]);
}
