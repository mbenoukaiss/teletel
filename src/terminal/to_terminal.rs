use std::io::Result as IoResult;
use crate::protocol::codes::{ACUTE, CEDILLA, CIRCUMFLEX, DIAERESIS, GRAVE, SS2};
use crate::terminal::WriteableTerminal;

pub trait ToTerminal {
    fn to_terminal(&self, term: &mut dyn WriteableTerminal) -> IoResult<usize>;
}

impl ToTerminal for u8 {
    #[inline(always)]
    fn to_terminal(&self, term: &mut dyn WriteableTerminal) -> IoResult<usize> {
        term.write(&[*self])
    }
}

impl ToTerminal for char {
    fn to_terminal(&self, term: &mut dyn WriteableTerminal) -> IoResult<usize> {
        match *self {
            'à' => term.write(&[SS2, GRAVE, b'a']),
            'ä' => term.write(&[SS2, DIAERESIS, b'a']),
            'â' => term.write(&[SS2, CIRCUMFLEX, b'a']),
            'é' => term.write(&[SS2, ACUTE, b'e']),
            'è' => term.write(&[SS2, GRAVE, b'e']),
            'ê' => term.write(&[SS2, CIRCUMFLEX, b'e']),
            'ë' => term.write(&[SS2, DIAERESIS, b'e']),
            'î' => term.write(&[SS2, CIRCUMFLEX, b'i']),
            'ï' => term.write(&[SS2, DIAERESIS, b'i']),
            'ö' => term.write(&[SS2, DIAERESIS, b'o']),
            'ô' => term.write(&[SS2, CIRCUMFLEX, b'o']),
            'ù' => term.write(&[SS2, GRAVE, b'u']),
            'ü' => term.write(&[SS2, DIAERESIS, b'u']),
            'û' => term.write(&[SS2, CIRCUMFLEX, b'u']),
            'ç' => term.write(&[SS2, CEDILLA, b'c']),
            c => term.write(unidecode::unidecode_char(c).as_bytes()),
        }
    }
}

impl ToTerminal for &str {
    #[inline(always)]
    fn to_terminal(&self, term: &mut dyn WriteableTerminal) -> IoResult<usize> {
        let mut written_bytes = 0;
        for char in self.chars() {
            written_bytes += char.to_terminal(term)?;
        }

        Ok(written_bytes)
    }
}

impl ToTerminal for String {
    #[inline(always)]
    fn to_terminal(&self, term: &mut dyn WriteableTerminal) -> IoResult<usize> {
        self.as_str().to_terminal(term)
    }
}

impl ToTerminal for Vec<u8> {
    #[inline(always)]
    fn to_terminal(&self, term: &mut dyn WriteableTerminal) -> IoResult<usize> {
        term.write(self)
    }
}

impl<const SIZE: usize> ToTerminal for [u8; SIZE] {
    #[inline(always)]
    fn to_terminal(&self, term: &mut dyn WriteableTerminal) -> IoResult<usize> {
        term.write(self)
    }
}

impl<F: Fn(&mut dyn WriteableTerminal) -> IoResult<usize>> ToTerminal for F {
    #[inline(always)]
    fn to_terminal(&self, term: &mut dyn WriteableTerminal) -> IoResult<usize> {
        self(term)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp;
    use std::io::{Result as IoResult, Read};
    use std::time::Duration;
    use crate::terminal::{Buffer, Context, Contextualized, ReadableTerminal};

    #[test]
    fn test_to_terminal() {
        let mut buf = Buffer::new();
        0x01.to_terminal(&mut buf).unwrap();
        'A'.to_terminal(&mut buf).unwrap();
        [0x02, 0x03].to_terminal(&mut buf).unwrap();
        vec![0x02, 0x03].to_terminal(&mut buf).unwrap();
        (&[0x02, 0x03]).to_terminal(&mut buf).unwrap();
        "bonjour".to_terminal(&mut buf).unwrap();

        assert_eq!(buf.data(), [
            0x01,
            b'A',
            0x02, 0x03,
            0x02, 0x03,
            0x02, 0x03,
            b'b', b'o', b'n',b'j', b'o', b'u', b'r',
        ]);
    }

    #[test]
    fn test_accents() {
        let mut buf = Buffer::new();
        'à'.to_terminal(&mut buf).unwrap();

        assert_eq!(buf.data(), &[0x19, 0x41, b'a']);

        let mut buf = Buffer::new();
        'é'.to_terminal(&mut buf).unwrap();

        assert_eq!(buf.data(), &[0x19, 0x42, b'e']);

        let mut buf = Buffer::new();
        'ç'.to_terminal(&mut buf).unwrap();

        assert_eq!(buf.data(), &[0x19, 0x4B, b'c']);

        let mut buf = Buffer::new();
        'ç'.to_terminal(&mut buf).unwrap();

        assert_eq!(buf.data(), &[0x19, 0x4B, b'c']);

        let mut buf = Buffer::new();
        "Bonjour ceci est une chaine sans accents".to_terminal(&mut buf).unwrap();

        assert_eq!(buf.data(), "Bonjour ceci est une chaine sans accents".as_bytes());

        let mut buf = Buffer::new();
        "àäâéèêëîïöôùüûç".to_terminal(&mut buf).unwrap();

        assert_eq!(buf.data(), &[
            0x19, 0x41, b'a',
            0x19, 0x48, b'a',
            0x19, 0x43, b'a',
            0x19, 0x42, b'e',
            0x19, 0x41, b'e',
            0x19, 0x43, b'e',
            0x19, 0x48, b'e',
            0x19, 0x43, b'i',
            0x19, 0x48, b'i',
            0x19, 0x48, b'o',
            0x19, 0x43, b'o',
            0x19, 0x41, b'u',
            0x19, 0x48, b'u',
            0x19, 0x43, b'u',
            0x19, 0x4B, b'c',
        ]);

        let mut buf = Buffer::new();
        "ÀÄÂÉÈÊËÎÏÖÔÙÜÛÇ".to_terminal(&mut buf).unwrap();

        assert_eq!(buf.data(), "AAAEEEEIIOOUUUC".as_bytes());
    }

    #[test]
    fn test_closure() {
        let mut buf = Buffer::new();
        (|term: &mut dyn WriteableTerminal| {
            let mut written_bytes = 0;
            written_bytes += 'A'.to_terminal(term)?;
            written_bytes += 'B'.to_terminal(term)?;
            written_bytes += 'C'.to_terminal(term)?;

            Ok(written_bytes)
        }).to_terminal(&mut buf).unwrap();

        assert_eq!(buf.data(), [b'A', b'B', b'C']);
    }

    struct MockReceiver {
        ctx: Context,
        buffer: Vec<u8>,
    }

    impl From<Vec<u8>> for MockReceiver {
        fn from(value: Vec<u8>) -> Self {
            Self {
                ctx: Context,
                buffer: value,
            }
        }
    }

    impl Contextualized for MockReceiver {
        fn ctx(&self) -> &Context {
            &self.ctx
        }
    }

    impl Read for MockReceiver {
        fn read(&mut self, buffer: &mut [u8]) -> IoResult<usize> {
            let bytes_to_read = cmp::min(buffer.len(), self.buffer.len());
            let read_bytes = self.buffer.drain(..bytes_to_read).collect::<Vec<u8>>();

            buffer[..bytes_to_read].copy_from_slice(&read_bytes);

            Ok(bytes_to_read)
        }
    }

    impl ReadableTerminal for MockReceiver {}

    #[test]
    fn test_read_to_vec() {
        let mut buffer = MockReceiver::from(vec![]);

        let data = buffer.read_to_vec().unwrap();
        assert_eq!(data, []);

        let mut buffer = MockReceiver::from(vec![0x01]);

        let data = buffer.read_to_vec().unwrap();
        assert_eq!(data, [0x01]);

        let mut buffer = MockReceiver::from(vec![0x01, 0x02, 0x03, 0x04, 0x05]);

        let data = buffer.read_to_vec().unwrap();
        assert_eq!(data, [0x01, 0x02, 0x03, 0x04, 0x05]);
    }

    #[test]
    fn test_read_until_enter() {
        assert_times_out!(Duration::from_millis(10), || {
            let mut buffer = MockReceiver::from(vec![]);

            buffer.read_until_enter().unwrap();
        });

        assert_times_out!(Duration::from_millis(10), || {
            let mut buffer = MockReceiver::from(vec![0x01, 0x02, 0x03, 0x04, 0x05]);

            buffer.read_until_enter().unwrap();
        });

        let mut buffer = MockReceiver::from(vec![0x01, 0x02, 0x03, b'\r', 0x04, 0x05]);

        let data = buffer.read_until_enter().unwrap();
        assert_eq!(data, [0x01, 0x02, 0x03]);
    }
}

