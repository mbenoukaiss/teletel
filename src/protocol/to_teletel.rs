use crate::protocol::codes::{ACUTE, CEDILLA, CIRCUMFLEX, DIAERESIS, GRAVE, SS2};
use crate::Minitel;

pub trait ToMinitel {
    fn to_minitel(&self, mt: &mut Minitel);
}

impl ToMinitel for u8 {
    #[inline(always)]
    fn to_minitel(&self, mt: &mut Minitel) {
        mt.send(&[*self]);
    }
}

impl ToMinitel for char {
    fn to_minitel(&self, mt: &mut Minitel) {
        match *self {
            'à' => mt.send(&[SS2, GRAVE, b'a']),
            'ä' => mt.send(&[SS2, DIAERESIS, b'a']),
            'â' => mt.send(&[SS2, CIRCUMFLEX, b'a']),
            'é' => mt.send(&[SS2, ACUTE, b'e']),
            'è' => mt.send(&[SS2, GRAVE, b'e']),
            'ê' => mt.send(&[SS2, CIRCUMFLEX, b'e']),
            'ë' => mt.send(&[SS2, DIAERESIS, b'e']),
            'î' => mt.send(&[SS2, CIRCUMFLEX, b'i']),
            'ï' => mt.send(&[SS2, DIAERESIS, b'i']),
            'ö' => mt.send(&[SS2, DIAERESIS, b'o']),
            'ô' => mt.send(&[SS2, CIRCUMFLEX, b'o']),
            'ù' => mt.send(&[SS2, GRAVE, b'u']),
            'ü' => mt.send(&[SS2, DIAERESIS, b'u']),
            'û' => mt.send(&[SS2, CIRCUMFLEX, b'u']),
            'ç' => mt.send(&[SS2, CEDILLA, b'c']),
            c => mt.send(unidecode::unidecode_char(c).as_bytes()),
        }
    }
}

impl ToMinitel for &str {
    #[inline(always)]
    fn to_minitel(&self, mt: &mut Minitel) {
        for char in self.chars() {
            char.to_minitel(mt);
        }
    }
}

impl ToMinitel for String {
    #[inline(always)]
    fn to_minitel(&self, mt: &mut Minitel) {
        self.as_str().to_minitel(mt);
    }
}

impl ToMinitel for Vec<u8> {
    #[inline(always)]
    fn to_minitel(&self, mt: &mut Minitel) {
        mt.send(self);
    }
}

impl<const SIZE: usize> ToMinitel for [u8; SIZE] {
    #[inline(always)]
    fn to_minitel(&self, mt: &mut Minitel) {
        mt.send(self);
    }
}

impl<F: Fn(&mut Minitel)> ToMinitel for F {
    #[inline(always)]
    fn to_minitel(&self, mt: &mut Minitel) {
        self(mt);
    }
}

#[cfg(test)]
mod tests {
    use crate::receiver::Buffer;
    use super::*;

    #[test]
    fn test_to_minitel() {
        let mut data = Buffer::new();
        {
            let mut mt = Minitel::from(&mut data);

            0x01.to_minitel(&mut mt);
            'A'.to_minitel(&mut mt);
            [0x02, 0x03].to_minitel(&mut mt);
            vec![0x02, 0x03].to_minitel(&mut mt);
            (&[0x02, 0x03]).to_minitel(&mut mt);
            "bonjour".to_minitel(&mut mt);
        }

        assert_eq!(data, [
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
        let mut data = Buffer::new();
        {
            let mut mt = Minitel::from(&mut data);
            'à'.to_minitel(&mut mt);
        }

        assert_eq!(data, &[0x19, 0x41, b'a']);

        let mut data = Buffer::new();
        {
            let mut mt = Minitel::from(&mut data);
            'é'.to_minitel(&mut mt);
        }

        assert_eq!(data, &[0x19, 0x42, b'e']);

        let mut data = Buffer::new();
        {
            let mut mt = Minitel::from(&mut data);
            'ç'.to_minitel(&mut mt);
        }

        assert_eq!(data, &[0x19, 0x4B, b'c']);

        let mut data = Buffer::new();
        {
            let mut mt = Minitel::from(&mut data);
            'ç'.to_minitel(&mut mt);
        }

        assert_eq!(data, &[0x19, 0x4B, b'c']);

        let mut data = Buffer::new();
        {
            let mut mt = Minitel::from(&mut data);
            "Bonjour ceci est une chaine sans accents".to_minitel(&mut mt);
        }

        assert_eq!(data, "Bonjour ceci est une chaine sans accents".as_bytes());

        let mut data = Buffer::new();
        {
            let mut mt = Minitel::from(&mut data);
            "àäâéèêëîïöôùüûç".to_minitel(&mut mt);
        }

        assert_eq!(data, &[
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

        let mut data = Buffer::new();
        {
            let mut mt = Minitel::from(&mut data);
            "ÀÄÂÉÈÊËÎÏÖÔÙÜÛÇ".to_minitel(&mut mt);
        }

        assert_eq!(data, "AAAEEEEIIOOUUUC".as_bytes());
    }

    #[test]
    fn test_closure() {
        let mut data = Buffer::new();
        {
            let mut mt = Minitel::from(&mut data);
            (|mt: &mut Minitel| {
                'A'.to_minitel(mt);
                'B'.to_minitel(mt);
                'C'.to_minitel(mt);
            }).to_minitel(&mut mt);
        }

        assert_eq!(data, [b'A', b'B', b'C']);
    }
}
