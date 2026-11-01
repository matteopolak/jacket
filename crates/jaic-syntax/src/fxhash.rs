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

/// The one to seven bytes `rest` as a little-endian word, zero-padded. Overlapping loads from
/// both ends do what a copy into a zeroed buffer would, without the call to `memcpy` that a
/// copy of unknown length makes (names are mostly shorter than eight bytes past their last
/// whole word).
#[inline]
fn tail_word(rest: &[u8]) -> u64 {
    let n = rest.len();
    if n >= 4 {
        let first = u32::from_le_bytes([rest[0], rest[1], rest[2], rest[3]]) as u64;
        let last = u32::from_le_bytes([rest[n - 4], rest[n - 3], rest[n - 2], rest[n - 1]]) as u64;
        first | (last << ((n - 4) * 8))
    } else if n >= 2 {
        let first = u16::from_le_bytes([rest[0], rest[1]]) as u64;
        let last = u16::from_le_bytes([rest[n - 2], rest[n - 1]]) as u64;
        first | (last << ((n - 2) * 8))
    } else {
        rest[0] as u64
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
            // The tail's length goes in the top byte so `b"ab"` and `b"ab\0"` differ.
            self.add(tail_word(rest) ^ ((rest.len() as u64) << 56));
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

    #[test]
    fn a_tail_reads_as_a_zero_padded_word() {
        let bytes = *b"\x01\x02\x03\x04\x05\x06\x07";
        for n in 1..=7 {
            let mut padded = [0u8; 8];
            padded[..n].copy_from_slice(&bytes[..n]);
            assert_eq!(
                super::tail_word(&bytes[..n]),
                u64::from_le_bytes(padded),
                "{n}"
            );
        }
    }
}
