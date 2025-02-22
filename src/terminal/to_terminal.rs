use std::io::Result as IoResult;
use crate::specifications::codes::*;
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
            'œ' => term.write(&[SS2, LOWER_OE]),
            'Œ' => term.write(&[SS2, UPPER_OE]),
            'ß' | 'ẞ' => term.write(&[SS2, ESZETT]),
            '£' => term.write(&[SS2, POUND]),
            '$' => term.write(&[SS2, DOLLAR]),
            '#' => term.write(&[SS2, NUMBER_SIGN]),
            '←' => term.write(&[SS2, ARROW_LEFT]),
            '↑' => term.write(&[SS2, ARROW_UP]),
            '→' => term.write(&[SS2, ARROW_RIGHT]),
            '↓' => term.write(&[SS2, ARROW_DOWN]),
            '§' => term.write(&[SS2, PARAGRAPH]),
            '°' => term.write(&[SS2, DEGREE]),
            '±' => term.write(&[SS2, PLUS_OR_MINUS]),
            '÷' => term.write(&[SS2, OBELUS]),
            '¼' => term.write(&[SS2, ONE_QUARTER]),
            '½' => term.write(&[SS2, ONE_HALF]),
            '¾' => term.write(&[SS2, THREE_QUARTERS]),
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
    use crate::parser::DisplayComponent;
    use crate::terminal::{RawBuffer, Context, Contextualized, ReadableTerminal};

    #[test]
    fn test_to_terminal() {
        let mut buf = RawBuffer::new();
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
        let mut buf = RawBuffer::new();
        'à'.to_terminal(&mut buf).unwrap();

        assert_eq!(buf.data(), &[0x19, 0x41, b'a']);

        let mut buf = RawBuffer::new();
        'é'.to_terminal(&mut buf).unwrap();

        assert_eq!(buf.data(), &[0x19, 0x42, b'e']);

        let mut buf = RawBuffer::new();
        'ç'.to_terminal(&mut buf).unwrap();

        assert_eq!(buf.data(), &[0x19, 0x4B, b'c']);

        let mut buf = RawBuffer::new();
        'ç'.to_terminal(&mut buf).unwrap();

        assert_eq!(buf.data(), &[0x19, 0x4B, b'c']);

        let mut buf = RawBuffer::new();
        "Bonjour ceci est une chaine sans accents".to_terminal(&mut buf).unwrap();

        assert_eq!(buf.data(), "Bonjour ceci est une chaine sans accents".as_bytes());

        let mut buf = RawBuffer::new();
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

        let mut buf = RawBuffer::new();
        "ÀÄÂÉÈÊËÎÏÖÔÙÜÛÇ".to_terminal(&mut buf).unwrap();

        assert_eq!(buf.data(), "AAAEEEEIIOOUUUC".as_bytes());

        let mut buf = RawBuffer::new();
        "œŒßẞ".to_terminal(&mut buf).unwrap();

        assert_eq!(buf.data(), &[
            0x19, 0x7A,
            0x19, 0x6A,
            0x19, 0x7B,
            0x19, 0x7B,
        ]);
    }

    #[test]
    fn test_special_characters() {
        let mut buf = RawBuffer::new();
        '£'.to_terminal(&mut buf).unwrap();
        assert_eq!(buf.data(), &[0x19, 0x23]);

        let mut buf = RawBuffer::new();
        '$'.to_terminal(&mut buf).unwrap();
        assert_eq!(buf.data(), &[0x19, 0x24]);

        let mut buf = RawBuffer::new();
        '#'.to_terminal(&mut buf).unwrap();
        assert_eq!(buf.data(), &[0x19, 0x26]);

        let mut buf = RawBuffer::new();
        '←'.to_terminal(&mut buf).unwrap();
        assert_eq!(buf.data(), &[0x19, 0x2C]);

        let mut buf = RawBuffer::new();
        '↑'.to_terminal(&mut buf).unwrap();
        assert_eq!(buf.data(), &[0x19, 0x2D]);

        let mut buf = RawBuffer::new();
        '→'.to_terminal(&mut buf).unwrap();
        assert_eq!(buf.data(), &[0x19, 0x2E]);

        let mut buf = RawBuffer::new();
        '↓'.to_terminal(&mut buf).unwrap();
        assert_eq!(buf.data(), &[0x19, 0x2F]);

        let mut buf = RawBuffer::new();
        '§'.to_terminal(&mut buf).unwrap();
        assert_eq!(buf.data(), &[0x19, 0x27]);

        let mut buf = RawBuffer::new();
        '°'.to_terminal(&mut buf).unwrap();
        assert_eq!(buf.data(), &[0x19, 0x30]);

        let mut buf = RawBuffer::new();
        '±'.to_terminal(&mut buf).unwrap();
        assert_eq!(buf.data(), &[0x19, 0x31]);

        let mut buf = RawBuffer::new();
        '÷'.to_terminal(&mut buf).unwrap();
        assert_eq!(buf.data(), &[0x19, 0x38]);

        let mut buf = RawBuffer::new();
        '¼'.to_terminal(&mut buf).unwrap();
        assert_eq!(buf.data(), &[0x19, 0x3C]);

        let mut buf = RawBuffer::new();
        '½'.to_terminal(&mut buf).unwrap();
        assert_eq!(buf.data(), &[0x19, 0x3D]);

        let mut buf = RawBuffer::new();
        '¾'.to_terminal(&mut buf).unwrap();
        assert_eq!(buf.data(), &[0x19, 0x3E]);
    }

    #[test]
    fn test_closure() {
        let mut buf = RawBuffer::new();
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
                ctx: Context::new(DisplayComponent::VGP5),
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

