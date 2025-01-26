use crate::backend::Backend;

impl Backend for Vec<u8> {
    fn send(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }
}
