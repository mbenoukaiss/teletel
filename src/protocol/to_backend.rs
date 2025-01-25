use crate::backend::Backend;

pub trait ToBackend {
    fn to_backend(&self, backend: &mut dyn Backend);
}

impl ToBackend for u8 {
    fn to_backend(&self, backend: &mut dyn Backend) {
        backend.send(&[*self]);
    }
}

impl ToBackend for char {
    fn to_backend(&self, backend: &mut dyn Backend) {
        backend.send(&[*self as u8]);
    }
}

impl ToBackend for &str {
    fn to_backend(&self, backend: &mut dyn Backend) {
        backend.send(self.as_bytes());
    }
}

impl ToBackend for String {
    fn to_backend(&self, backend: &mut dyn Backend) {
        backend.send(self.as_bytes());
    }
}

impl<const SIZE: usize> ToBackend for [u8; SIZE] {
    fn to_backend(&self, backend: &mut dyn Backend) {
        backend.send(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockBackend {
        bytes: Vec<u8>,
    }

    impl Backend for MockBackend {
        fn send(&mut self, bytes: &[u8]) {
            self.bytes.extend_from_slice(bytes);
        }
    }

    #[test]
    fn test_to_backend() {
        let mut backend = MockBackend::default();
        0x01.to_backend(&mut backend);
        'A'.to_backend(&mut backend);
        [0x02, 0x03].to_backend(&mut backend);
        "bonjour".to_backend(&mut backend);

        assert_eq!(
            backend.bytes,
            vec![
                0x01,
                'A' as u8,
                0x02, 0x03,
                'b' as u8, 'o' as u8, 'n' as u8, 'j' as u8, 'o' as u8, 'u' as u8, 'r' as u8,
            ],
        );
    }
}
