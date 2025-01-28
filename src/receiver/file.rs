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
    fn send(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    fn flush(&mut self) -> Result<(), Error> {
        self.file.write_all(&self.buffer)?;
        self.buffer.clear();

        Ok(())
    }
}
