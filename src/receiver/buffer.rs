use crate::Error;
use crate::receiver::TeletelReceiver;

pub type Buffer = Vec<u8>;

impl TeletelReceiver for Buffer {
    #[inline(always)]
    fn read(&mut self, _buffer: &mut [u8]) -> Result<usize, Error> {
        panic!("Vec<u8> does not support reading");
    }

    #[inline(always)]
    fn send(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }
}

impl TeletelReceiver for &mut Buffer {
    #[inline(always)]
    fn read(&mut self, _buffer: &mut [u8]) -> Result<usize, Error> {
        panic!("Vec<u8> does not support reading");
    }

    #[inline(always)]
    fn send(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }
}
