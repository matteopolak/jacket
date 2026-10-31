//! A fast, non-cryptographic hasher for the compiler's own maps (the multiply-rotate hash rustc
//! uses internally). Keys are ids and identifiers from source the user controls, so `SipHash`'s
//! flooding resistance buys nothing, and `SipHash` showed up in profiles of every name lookup.
use std::hash::{BuildHasherDefault, Hasher};

#[expect(
    clippy::disallowed_types,
    reason = "this alias is the sanctioned replacement"
)]
pub type HashMap<K, V> = std::collections::HashMap<K, V, BuildHasherDefault<FxHasher>>;

#[expect(
    clippy::disallowed_types,
    reason = "this alias is the sanctioned replacement"
)]
pub type HashSet<K> = std::collections::HashSet<K, BuildHasherDefault<FxHasher>>;

const SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

#[derive(Default, Clone, Copy)]
pub struct FxHasher {
    hash: u64,
}

impl FxHasher {
    #[inline]
    fn add(&mut self, word: u64) {
        self.hash = (self.hash.rotate_left(5) ^ word).wrapping_mul(SEED);
    }
}

impl Hasher for FxHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        let (words, rest) = bytes.as_chunks::<8>();
        for word in words {
            self.add(u64::from_le_bytes(*word));
        }
        if !rest.is_empty() {
            let mut word = [0u8; 8];
            word[..rest.len()].copy_from_slice(rest);
            // The tail's length goes in the top byte so `b"ab"` and `b"ab\0"` differ.
            self.add(u64::from_le_bytes(word) ^ ((rest.len() as u64) << 56));
        }
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.add(i as u64);
    }

    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.add(i as u64);
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.add(i as u64);
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.add(i);
    }

    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.add(i as u64);
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.hash
    }
}

#[cfg(test)]
mod tests {
    use super::FxHasher;
    use std::hash::{Hash, Hasher};

    fn of(text: &str) -> u64 {
        let mut h = FxHasher::default();
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
