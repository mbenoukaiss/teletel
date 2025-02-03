use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path};
use crate::error::Error;
use crate::receiver::TeletelReceiver;

pub struct FileReceiver {
    file: File,
    buffer: Vec<u8>,
}

impl FileReceiver {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .append(false)
            .open(path)?;

        Ok(Self {
            file,
            buffer: Vec::new(),
        })
    }
}

impl TeletelReceiver for FileReceiver {
    #[inline(always)]
    fn read(&mut self, _buffer: &mut [u8]) -> Result<usize, Error> {
        panic!("FileReceiver does not support reading");
    }

    #[inline(always)]
    fn send(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    #[inline(always)]
    fn flush(&mut self) -> Result<(), Error> {
        self.file.write_all(&self.buffer)?;
        self.buffer.clear();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, fs};
    use std::io::Read;

    #[test]
    fn test_file() {
        let path = format!("{}/test_file.vdt", env::temp_dir().to_str().unwrap());
        let mut file_receiver = FileReceiver::new(&path).unwrap();

        file_receiver.send(&[0x01]);
        file_receiver.send(&[0x02, 0x03]);
        file_receiver.send(&[0x04, 0x05, 0x06]);

        file_receiver.flush().unwrap();

        let mut file = fs::File::open(&path).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();

        assert_eq!(buffer, [0x01, 0x02, 0x03, 0x04, 0x05, 0x06]);

        fs::remove_file(&path).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_file_read() {
        let path = format!("{}/test_file_read.vdt", env::temp_dir().to_str().unwrap());
        let mut file_receiver = FileReceiver::new(path).unwrap();

        let mut buffers_buffer = vec![0; 10];
        file_receiver.read(&mut buffers_buffer).unwrap();
    }
}