use std::fs::{File, OpenOptions};
use std::io::{Result as IoResult, Write};
use std::path::{Path};
use crate::parser::{DisplayComponent, Parser};
use crate::terminal::{Context, Contextualized, ToTerminal, WriteableTerminal};

pub struct FileReceiver {
    file: File,
    parser: Parser,
}

impl FileReceiver {
    pub fn new<P: AsRef<Path>>(path: P) -> IoResult<Self> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .append(false)
            .open(path)?;

        Ok(Self {
            file,
            parser: Parser::new(DisplayComponent::VGP5),
        })
    }

    #[inline(always)]
    pub fn send(&mut self, data: impl ToTerminal) -> IoResult<usize> {
        data.to_terminal(self)
    }
}

impl Contextualized for FileReceiver {
    fn ctx(&self) -> &Context {
        self.parser.ctx()
    }
}

impl Write for FileReceiver {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.parser.consume_all(buf);
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

        file_receiver.write(&[b'a']).unwrap();
        file_receiver.write(&[b'b', b'c']).unwrap();
        file_receiver.write(&[b'd', b'e', b'f']).unwrap();
        file_receiver.flush().unwrap();

        let mut file = File::open(&path).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();

        assert_eq!(buffer, [b'a', b'b', b'c', b'd', b'e', b'f']);

        fs::remove_file(&path).unwrap();
    }
}
