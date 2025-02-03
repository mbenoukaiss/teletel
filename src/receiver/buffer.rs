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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer() {
        let mut buffer = Buffer::new();

        buffer.send(&[0x00]);
        buffer.send(&[0x02, 0x03]);
        buffer.send(&[0x04, 0x05, 0x06]);

        assert_eq!(buffer, [0x00, 0x02, 0x03, 0x04, 0x05, 0x06]);
    }

    #[test]
    fn test_buffer_mut_reference() {
        let mut buffer = Buffer::new();
        let buffer_ref = &mut buffer;

        buffer_ref.send(&[0x00]);
        buffer_ref.send(&[0x02, 0x03]);
        buffer_ref.send(&[0x04, 0x05, 0x06]);

        assert_eq!(buffer, [0x00, 0x02, 0x03, 0x04, 0x05, 0x06]);
    }

    #[test]
    #[should_panic]
    fn test_buffer_read() {
        let mut buffer = Buffer::new();
        let mut buffers_buffer = vec![0; 10];
        buffer.read(&mut buffers_buffer).unwrap();
    }
}
