//! SHA-256 of a downloaded archive and the release's `SHA256SUMS` lines. The line formats
//! match `install.sh` and `tests/installers/vectors.json`: `<hash>  <name>`, `<hash> *<name>`,
//! any case.
use std::io::Read;
use std::path::Path;

#[rustfmt::skip]
const K: [u32; 64] = [
    0x428a_2f98, 0x7137_4491, 0xb5c0_fbcf, 0xe9b5_dba5, 0x3956_c25b, 0x59f1_11f1, 0x923f_82a4, 0xab1c_5ed5,
    0xd807_aa98, 0x1283_5b01, 0x2431_85be, 0x550c_7dc3, 0x72be_5d74, 0x80de_b1fe, 0x9bdc_06a7, 0xc19b_f174,
    0xe49b_69c1, 0xefbe_4786, 0x0fc1_9dc6, 0x240c_a1cc, 0x2de9_2c6f, 0x4a74_84aa, 0x5cb0_a9dc, 0x76f9_88da,
    0x983e_5152, 0xa831_c66d, 0xb003_27c8, 0xbf59_7fc7, 0xc6e0_0bf3, 0xd5a7_9147, 0x06ca_6351, 0x1429_2967,
    0x27b7_0a85, 0x2e1b_2138, 0x4d2c_6dfc, 0x5338_0d13, 0x650a_7354, 0x766a_0abb, 0x81c2_c92e, 0x9272_2c85,
    0xa2bf_e8a1, 0xa81a_664b, 0xc24b_8b70, 0xc76c_51a3, 0xd192_e819, 0xd699_0624, 0xf40e_3585, 0x106a_a070,
    0x19a4_c116, 0x1e37_6c08, 0x2748_774c, 0x34b0_bcb5, 0x391c_0cb3, 0x4ed8_aa4a, 0x5b9c_ca4f, 0x682e_6ff3,
    0x748f_82ee, 0x78a5_636f, 0x84c8_7814, 0x8cc7_0208, 0x90be_fffa, 0xa450_6ceb, 0xbef9_a3f7, 0xc671_78f2,
];

struct Sha256 {
    state: [u32; 8],
    block: [u8; 64],
    filled: usize,
    length: u64,
}

impl Sha256 {
    fn new() -> Sha256 {
        Sha256 {
            state: [
                0x6a09_e667,
                0xbb67_ae85,
                0x3c6e_f372,
                0xa54f_f53a,
                0x510e_527f,
                0x9b05_688c,
                0x1f83_d9ab,
                0x5be0_cd19,
            ],
            block: [0; 64],
            filled: 0,
            length: 0,
        }
    }

    fn update(&mut self, mut data: &[u8]) {
        self.length += data.len() as u64;
        while !data.is_empty() {
            let take = (64 - self.filled).min(data.len());
            self.block[self.filled..self.filled + take].copy_from_slice(&data[..take]);
            self.filled += take;
            data = &data[take..];
            if self.filled == 64 {
                let block = self.block;
                self.compress(&block);
                self.filled = 0;
            }
        }
    }

    fn compress(&mut self, block: &[u8; 64]) {
        let mut w = [0u32; 64];
        for (i, word) in block.as_chunks::<4>().0.iter().enumerate() {
            w[i] = u32::from_be_bytes(*word);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let mut v = self.state;
        for i in 0..64 {
            let s1 = v[4].rotate_right(6) ^ v[4].rotate_right(11) ^ v[4].rotate_right(25);
            let ch = (v[4] & v[5]) ^ (!v[4] & v[6]);
            let t1 = v[7]
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = v[0].rotate_right(2) ^ v[0].rotate_right(13) ^ v[0].rotate_right(22);
            let maj = (v[0] & v[1]) ^ (v[0] & v[2]) ^ (v[1] & v[2]);
            let t2 = s0.wrapping_add(maj);
            v = [
                t1.wrapping_add(t2),
                v[0],
                v[1],
                v[2],
                v[3].wrapping_add(t1),
                v[4],
                v[5],
                v[6],
            ];
        }
        for (s, x) in self.state.iter_mut().zip(v) {
            *s = s.wrapping_add(x);
        }
    }

    fn hex(mut self) -> String {
        let bits = self.length * 8;
        self.update(&[0x80]);
        while self.filled != 56 {
            self.update(&[0]);
        }
        self.update(&bits.to_be_bytes());
        self.state.iter().map(|w| format!("{w:08x}")).collect()
    }
}

/// The lower-case hex SHA-256 of `bytes`.
#[cfg(test)]
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hash = Sha256::new();
    hash.update(bytes);
    hash.hex()
}

/// The lower-case hex SHA-256 of a file, read in chunks.
pub fn sha256_file(path: &Path) -> std::io::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = vec![0u8; 1 << 16];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            return Ok(hash.hex());
        }
        hash.update(&buffer[..n]);
    }
}

/// The hash `sums` (a `SHA256SUMS` file) lists for `asset`, lower-cased. The last entry wins.
pub fn expected(sums: &str, asset: &str) -> Option<String> {
    let mut found = None;
    for line in sums.lines() {
        let Some((hash, name)) = line.trim().split_once(char::is_whitespace) else {
            continue;
        };
        let name = name.trim_start();
        let name = name.strip_prefix('*').unwrap_or(name);
        if name == asset && hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()) {
            found = Some(hash.to_ascii_lowercase());
        }
    }
    found
}

/// Checks `archive` against the hash `sums` lists for `asset`.
pub fn verify(archive: &Path, asset: &str, sums: &str) -> Result<(), String> {
    let Some(want) = expected(sums, asset) else {
        return Err(format!("the release's SHA256SUMS has no entry for {asset}"));
    };
    let got =
        sha256_file(archive).map_err(|e| format!("could not read {}: {e}", archive.display()))?;
    if got == want {
        Ok(())
    } else {
        Err(format!(
            "checksum mismatch for {asset}: expected {want}, got {got}"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_digests() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        // Longer than one block, with the padding spilling into a second.
        assert_eq!(
            sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
    }

    #[test]
    fn a_file_hashes_like_its_bytes() {
        let dir = std::env::temp_dir().join(format!("jacket-sha-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let data: Vec<u8> = (0..200_000u32).map(|i| (i % 251) as u8).collect();
        let path = dir.join("blob");
        std::fs::write(&path, &data).unwrap();
        assert_eq!(sha256_file(&path).unwrap(), sha256_hex(&data));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    const HASH: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

    #[test]
    fn the_sums_formats_the_installers_accept() {
        let upper = HASH.to_ascii_uppercase();
        for line in [
            format!("{HASH}  a.tar.gz"),
            format!("{HASH} *a.tar.gz"),
            format!("{upper}  a.tar.gz"),
        ] {
            assert_eq!(expected(&line, "a.tar.gz").as_deref(), Some(HASH), "{line}");
        }
        assert_eq!(expected(&format!("{HASH}  b.tar.gz"), "a.tar.gz"), None);
        assert_eq!(expected("", "a.tar.gz"), None);
        assert_eq!(expected("short  a.tar.gz", "a.tar.gz"), None);
    }

    #[test]
    fn a_mismatch_names_both_hashes() {
        let dir = std::env::temp_dir().join(format!("jacket-sum-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("a.tar.gz");
        std::fs::write(&path, b"abc").unwrap();
        verify(&path, "a.tar.gz", &format!("{HASH}  a.tar.gz")).unwrap();
        let wrong = "0".repeat(64);
        let error = verify(&path, "a.tar.gz", &format!("{wrong}  a.tar.gz")).unwrap_err();
        assert!(error.contains("checksum mismatch"), "{error}");
        assert!(error.contains(&wrong) && error.contains(HASH), "{error}");
        let error = verify(&path, "a.tar.gz", "").unwrap_err();
        assert!(error.contains("no entry"), "{error}");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
