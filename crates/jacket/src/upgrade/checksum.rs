//! SHA-256 of a downloaded archive and the release's `SHA256SUMS` lines. The line formats
//! match `install.sh` and `tests/installers/vectors.json`: `<hash>  <name>`, `<hash> *<name>`,
//! any case.
use sha2::{Digest, Sha256};
use std::path::Path;

/// The lower-case hex SHA-256 of `bytes`.
#[cfg(test)]
pub fn sha256_hex(bytes: &[u8]) -> String {
    hex(Sha256::digest(bytes).as_slice())
}

/// The lower-case hex SHA-256 of a file, read in chunks.
pub fn sha256_file(path: &Path) -> std::io::Result<String> {
    let mut hash = Sha256::new();
    std::io::copy(&mut std::fs::File::open(path)?, &mut hash)?;
    Ok(hex(hash.finalize().as_slice()))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
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
