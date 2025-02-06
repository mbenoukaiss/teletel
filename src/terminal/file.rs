use std::borrow::Borrow;
use std::fs::{File, OpenOptions};
use std::io::{Result as IoResult, Write};
use std::path::{Path};
use crate::terminal::{Context, Contextualized, ToTerminal, WriteableTerminal};

pub struct FileReceiver {
    ctx: Context,
    file: File,
}

impl FileReceiver {
    pub fn new<P: AsRef<Path>>(path: P) -> IoResult<Self> {
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .append(false)
            .open(path)?;

        Ok(Self {
            ctx: Context,
            file,
        })
    }

    #[inline(always)]
    pub fn send<T: ToTerminal, R: Borrow<T>>(&mut self, data: R) -> IoResult<usize> {
        data.borrow().to_terminal(self)
    }
}

impl Contextualized for FileReceiver {
    fn ctx(&self) -> &Context {
        &self.ctx
    }
}

impl Write for FileReceiver {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.file.write(buf)
    }

    #[inline(always)]
    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

impl WriteableTerminal for FileReceiver {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, fs};
    use std::io::Read;

    #[test]
    fn test_file() {
        let path = format!("{}/test_file.vdt", env::temp_dir().to_str().unwrap());
        let mut file_receiver = FileReceiver::new(&path).unwrap();

        file_receiver.write(&[0x01]).unwrap();
        file_receiver.write(&[0x02, 0x03]).unwrap();
        file_receiver.write(&[0x04, 0x05, 0x06]).unwrap();
        file_receiver.flush().unwrap();

        let mut file = File::open(&path).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();

        assert_eq!(buffer, [0x01, 0x02, 0x03, 0x04, 0x05, 0x06]);

        fs::remove_file(&path).unwrap();
    }
}
