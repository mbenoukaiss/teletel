use crate::protocol::codes::{ACUTE, CEDILLA, CIRCUMFLEX, DIAERESIS, GRAVE, SS2};
use crate::receiver::TeletelReceiver;

pub trait ToTeletel {
    fn to_teletel(&self, receiver: &mut dyn TeletelReceiver);
}

impl ToTeletel for u8 {
    fn to_teletel(&self, receiver: &mut dyn TeletelReceiver) {
        receiver.send(&[*self]);
    }
}

impl ToTeletel for char {
    fn to_teletel(&self, receiver: &mut dyn TeletelReceiver) {
        match *self {
            'à' => receiver.send(&[SS2, GRAVE, 'a' as u8]),
            'ä' => receiver.send(&[SS2, DIAERESIS, 'a' as u8]),
            'â' => receiver.send(&[SS2, CIRCUMFLEX, 'a' as u8]),
            'é' => receiver.send(&[SS2, ACUTE, 'e' as u8]),
            'è' => receiver.send(&[SS2, GRAVE, 'e' as u8]),
            'ê' => receiver.send(&[SS2, CIRCUMFLEX, 'e' as u8]),
            'ë' => receiver.send(&[SS2, DIAERESIS, 'e' as u8]),
            'î' => receiver.send(&[SS2, CIRCUMFLEX, 'i' as u8]),
            'ï' => receiver.send(&[SS2, DIAERESIS, 'i' as u8]),
            'ö' => receiver.send(&[SS2, DIAERESIS, 'o' as u8]),
            'ô' => receiver.send(&[SS2, CIRCUMFLEX, 'o' as u8]),
            'ù' => receiver.send(&[SS2, GRAVE, 'u' as u8]),
            'ü' => receiver.send(&[SS2, DIAERESIS, 'u' as u8]),
            'û' => receiver.send(&[SS2, CIRCUMFLEX, 'u' as u8]),
            'ç' => receiver.send(&[SS2, CEDILLA, 'c' as u8]),
            c => receiver.send(unidecode::unidecode_char(c).as_bytes()),
        }
    }
}

impl ToTeletel for &str {
    fn to_teletel(&self, receiver: &mut dyn TeletelReceiver) {
        for char in self.chars() {
            char.to_teletel(receiver);
        }
    }
}

impl ToTeletel for String {
    fn to_teletel(&self, receiver: &mut dyn TeletelReceiver) {
        for char in self.chars() {
            char.to_teletel(receiver);
        }
    }
}

impl ToTeletel for Vec<u8> {
    fn to_teletel(&self, receiver: &mut dyn TeletelReceiver) {
        receiver.send(self.as_slice());
    }
}

impl<const SIZE: usize> ToTeletel for [u8; SIZE] {
    fn to_teletel(&self, receiver: &mut dyn TeletelReceiver) {
        receiver.send(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_receiver() {
        let mut receiver = Vec::new();
        0x01.to_teletel(&mut receiver);
        'A'.to_teletel(&mut receiver);
        [0x02, 0x03].to_teletel(&mut receiver);
        (&[0x02, 0x03]).to_teletel(&mut receiver);
        "bonjour".to_teletel(&mut receiver);

        assert_eq!(
            receiver,
            vec![
                0x01, 'A' as u8, 0x02, 0x03, 0x02, 0x03, 'b' as u8, 'o' as u8, 'n' as u8,
                'j' as u8, 'o' as u8, 'u' as u8, 'r' as u8,
            ],
        );
    }

    #[test]
    fn test_accents() {
        let mut receiver = Vec::new();
        'à'.to_teletel(&mut receiver);

        assert_eq!(receiver, vec![0x19, 0x41, 'a' as u8]);

        let mut receiver = Vec::new();
        'é'.to_teletel(&mut receiver);

        assert_eq!(receiver, vec![0x19, 0x42, 'e' as u8]);

        let mut receiver = Vec::new();
        'ç'.to_teletel(&mut receiver);

        assert_eq!(receiver, vec![0x19, 0x4B, 'c' as u8]);

        let mut receiver = Vec::new();
        'ç'.to_teletel(&mut receiver);

        assert_eq!(receiver, vec![0x19, 0x4B, 'c' as u8]);

        let mut receiver = Vec::new();
        "Bonjour ceci est une chaine sans accents".to_teletel(&mut receiver);

        assert_eq!(receiver.as_slice(), "Bonjour ceci est une chaine sans accents".as_bytes());

        let mut receiver = Vec::new();
        "àäâéèêëîïöôùüûç".to_teletel(&mut receiver);

        assert_eq!(receiver, vec![
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

        let mut receiver = Vec::new();
        "ÀÄÂÉÈÊËÎÏÖÔÙÜÛÇ".to_teletel(&mut receiver);

        assert_eq!(receiver.as_slice(), "AAAEEEEIIOOUUUC".as_bytes());
    }
}
