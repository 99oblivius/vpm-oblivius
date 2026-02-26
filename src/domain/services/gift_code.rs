use rand::Rng;

const ALPHABET: &[u8] = b"23456789ABCDEFGHJKMNPQRSTVWXYZ";
const GROUP_SIZE: usize = 5;
const GROUP_COUNT: usize = 4;

fn calculate_checksum(bytes: &[u8]) -> usize {
    bytes.iter().enumerate().fold(0, |acc, (i, &byte)| {
        let pos = ALPHABET.iter().position(|&c| c == byte).unwrap_or(0);
        let value = if i % 2 == 0 { pos * 2 } else { pos };
        acc + value
    })
}

pub fn check_gift(key: &str) -> bool {
    let clean: String = key
        .chars()
        .filter(|c| *c != '-')
        .flat_map(|c| c.to_uppercase())
        .collect();

    if clean.len() != GROUP_SIZE * GROUP_COUNT {
        return false;
    }

    let bytes: Vec<u8> = clean.bytes().collect();
    if !bytes.iter().all(|b| ALPHABET.contains(b)) {
        return false;
    }

    let data = &bytes[..bytes.len() - 1];
    let expected = calculate_checksum(data) % ALPHABET.len();
    let actual = ALPHABET.iter().position(|&c| c == bytes[bytes.len() - 1]);

    actual == Some(expected)
}

#[derive(Clone)]
pub struct GiftCode {
    pub key: String,
}

impl GiftCode {
    pub fn generate() -> Self {
        let mut rng = rand::rng();
        let n = ALPHABET.len();

        let total_chars = GROUP_SIZE * GROUP_COUNT;
        let mut chars: Vec<u8> = (0..total_chars - 1)
            .map(|_| ALPHABET[rng.random_range(0..n)])
            .collect();

        let checksum = calculate_checksum(&chars);
        chars.push(ALPHABET[checksum % n]);

        let key = chars
            .chunks(GROUP_SIZE)
            .map(|chunk| String::from_utf8_lossy(chunk).to_string())
            .collect::<Vec<_>>()
            .join("-");

        Self { key }
    }
}
