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
            'à' => mt.send(&[SS2, GRAVE, 'a' as u8]),
            'ä' => mt.send(&[SS2, DIAERESIS, 'a' as u8]),
            'â' => mt.send(&[SS2, CIRCUMFLEX, 'a' as u8]),
            'é' => mt.send(&[SS2, ACUTE, 'e' as u8]),
            'è' => mt.send(&[SS2, GRAVE, 'e' as u8]),
            'ê' => mt.send(&[SS2, CIRCUMFLEX, 'e' as u8]),
            'ë' => mt.send(&[SS2, DIAERESIS, 'e' as u8]),
            'î' => mt.send(&[SS2, CIRCUMFLEX, 'i' as u8]),
            'ï' => mt.send(&[SS2, DIAERESIS, 'i' as u8]),
            'ö' => mt.send(&[SS2, DIAERESIS, 'o' as u8]),
            'ô' => mt.send(&[SS2, CIRCUMFLEX, 'o' as u8]),
            'ù' => mt.send(&[SS2, GRAVE, 'u' as u8]),
            'ü' => mt.send(&[SS2, DIAERESIS, 'u' as u8]),
            'û' => mt.send(&[SS2, CIRCUMFLEX, 'u' as u8]),
            'ç' => mt.send(&[SS2, CEDILLA, 'c' as u8]),
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
            'A' as u8,
            0x02, 0x03,
            0x02, 0x03,
            0x02, 0x03,
            'b' as u8, 'o' as u8, 'n' as u8,'j' as u8, 'o' as u8, 'u' as u8, 'r' as u8,
        ]);
    }

    #[test]
    fn test_accents() {
        let mut data = Buffer::new();
        {
            let mut mt = Minitel::from(&mut data);
            'à'.to_minitel(&mut mt);
        }

        assert_eq!(data, &[0x19, 0x41, 'a' as u8]);

        let mut data = Buffer::new();
        {
            let mut mt = Minitel::from(&mut data);
            'é'.to_minitel(&mut mt);
        }

        assert_eq!(data, &[0x19, 0x42, 'e' as u8]);

        let mut data = Buffer::new();
        {
            let mut mt = Minitel::from(&mut data);
            'ç'.to_minitel(&mut mt);
        }

        assert_eq!(data, &[0x19, 0x4B, 'c' as u8]);

        let mut data = Buffer::new();
        {
            let mut mt = Minitel::from(&mut data);
            'ç'.to_minitel(&mut mt);
        }

        assert_eq!(data, &[0x19, 0x4B, 'c' as u8]);

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
            0x19, 0x41, 'a' as u8,
            0x19, 0x48, 'a' as u8,
            0x19, 0x43, 'a' as u8,
            0x19, 0x42, 'e' as u8,
            0x19, 0x41, 'e' as u8,
            0x19, 0x43, 'e' as u8,
            0x19, 0x48, 'e' as u8,
            0x19, 0x43, 'i' as u8,
            0x19, 0x48, 'i' as u8,
            0x19, 0x48, 'o' as u8,
            0x19, 0x43, 'o' as u8,
            0x19, 0x41, 'u' as u8,
            0x19, 0x48, 'u' as u8,
            0x19, 0x43, 'u' as u8,
            0x19, 0x4B, 'c' as u8,
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

        assert_eq!(data, ['A' as u8, 'B' as u8, 'C' as u8]);
    }
}
