use crate::Error;
use crate::receiver::TeletelReceiver;

impl TeletelReceiver for Vec<u8> {
    fn read(&mut self, _buffer: &mut [u8]) -> Result<usize, Error> {
        panic!("Vec<u8> does not support reading");
    }

    fn send(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }
}
