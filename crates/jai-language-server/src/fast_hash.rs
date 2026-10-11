//! A quick, non-cryptographic hasher for cache keys over whole document texts. The standard
//! `DefaultHasher` (SipHash) was a tenth of the time of a hover when many files are open,
//! since every request rehashes the open documents to find its cached compile.

use std::hash::Hasher;

const K: u64 = 0x9E37_79B9_7F4A_7C15;

#[derive(Default)]
pub(crate) struct FastHasher(u64);

impl FastHasher {
    fn add(&mut self, word: u64) {
        self.0 = (self.0.rotate_left(5) ^ word).wrapping_mul(K);
    }
}

impl Hasher for FastHasher {
    fn write(&mut self, bytes: &[u8]) {
        let (words, rest) = bytes.as_chunks::<8>();
        for word in words {
            self.add(u64::from_le_bytes(*word));
        }
        let mut tail = [0u8; 8];
        tail[..rest.len()].copy_from_slice(rest);
        self.add(u64::from_le_bytes(tail) ^ ((bytes.len() as u64) << 56));
    }

    fn write_u64(&mut self, n: u64) {
        self.add(n);
    }

    fn finish(&self) -> u64 {
        // Spread the last word's bits over the whole result.
        let h = self.0;
        (h ^ (h >> 29)).wrapping_mul(K) ^ (h >> 32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hash::Hash;

    fn of(text: &str) -> u64 {
        let mut h = FastHasher::default();
        text.hash(&mut h);
        h.finish()
    }

    #[test]
    fn different_texts_hash_differently() {
        assert_ne!(of("a"), of("b"));
        assert_ne!(of("abcdefgh"), of("abcdefgi"));
        assert_ne!(of(""), of("\0"));
        assert_ne!(of("ab"), of("ab\0"));
        assert_eq!(of("same text"), of("same text"));
    }
}
