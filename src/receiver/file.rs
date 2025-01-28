use std::path::PathBuf;
use crate::receiver::TeletelReceiver;

pub struct FileReceiver {
    path: PathBuf,
    buffer: Vec<u8>,
}

impl FileReceiver {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            path: path.into(),
            buffer: Vec::new(),
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        std::fs::write(&self.path, &self.buffer)
    }
}

impl TeletelReceiver for FileReceiver {
    fn send(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }
}
