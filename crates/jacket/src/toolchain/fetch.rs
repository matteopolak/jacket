//! What `jacket toolchain add` downloads and unpacks. Everything is fetched over https from a
//! pinned URL and checked against a pinned sha256 before it is used; nothing else in jacket
//! reaches the network for a build.
//!
//! The real work is done by `curl` and `tar`, through [`crate::net`] as in `jacket upgrade`.
use crate::net::Curl;
use crate::upgrade::checksum::sha256_file;
use std::path::Path;

/// Saves `url` as `dest`, unless `dest` already holds a file with the sha256 `sha256`. A
/// download whose hash differs is deleted and is an error.
pub fn pinned(url: &str, sha256: &str, dest: &Path) -> Result<(), String> {
    if dest.is_file() && sha256_file(dest).is_ok_and(|hash| hash == sha256) {
        return Ok(());
    }

    let curl = Curl::get(url, "jacket toolchain add").retry(3);

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }

    let partial = dest.with_extension("part");

    if let Err(error) = curl.output(&partial).run() {
        let _ = std::fs::remove_file(&partial);
        return Err(error);
    }

    let hash = sha256_file(&partial).map_err(|e| format!("{}: {e}", partial.display()))?;

    if hash != sha256 {
        let _ = std::fs::remove_file(&partial);

        return Err(format!(
            "{url} has the sha256 {hash}, not the pinned {sha256}; nothing was installed"
        ));
    }

    std::fs::rename(&partial, dest).map_err(|e| format!("{}: {e}", dest.display()))
}

/// Unpacks the tar archive `archive` (`.tar`, `.tar.xz`, `.tar.gz`) into the existing folder
/// `dest`.
pub fn untar(archive: &Path, dest: &Path) -> Result<(), String> {
    crate::net::untar(archive, dest, "jacket toolchain add")
}

/// The contents of the member `data.tar.*` of a Debian package (an `ar` archive), with its
/// file name (`data.tar.xz`). GNU `tar` cannot read `ar`, so the package is read here.
pub fn deb_data(deb: &[u8]) -> Result<(String, &[u8]), String> {
    const MAGIC: &[u8] = b"!<arch>\n";
    let mut rest = deb
        .strip_prefix(MAGIC)
        .ok_or("not a Debian package: no `ar` header")?;

    while rest.len() >= 60 {
        let (header, body) = rest.split_at(60);
        let name = String::from_utf8_lossy(&header[..16])
            .trim_end()
            .to_string();
        let size: usize = String::from_utf8_lossy(&header[48..58])
            .trim()
            .parse()
            .map_err(|_| "not a Debian package: bad member size")?;

        if body.len() < size {
            return Err("not a Debian package: truncated".into());
        }

        let name = name.trim_end_matches('/').to_string();

        if name.starts_with("data.tar") {
            return Ok((name, &body[..size]));
        }

        // Members are padded to an even length.
        let skip = (size + size % 2).min(body.len());

        rest = &body[skip..];
    }

    Err("not a Debian package: it has no data.tar member".into())
}

/// Unpacks the files of the Debian package `deb` into the existing folder `dest`.
pub fn unpack_deb(deb: &Path, scratch: &Path, dest: &Path) -> Result<(), String> {
    let bytes = std::fs::read(deb).map_err(|e| format!("{}: {e}", deb.display()))?;
    let (name, data) = deb_data(&bytes).map_err(|e| format!("{}: {e}", deb.display()))?;

    std::fs::create_dir_all(scratch).map_err(|e| format!("{}: {e}", scratch.display()))?;
    let tar = scratch.join(name);

    std::fs::write(&tar, data).map_err(|e| format!("{}: {e}", tar.display()))?;
    let result = untar(&tar, dest);
    let _ = std::fs::remove_file(&tar);

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn member(name: &str, body: &[u8]) -> Vec<u8> {
        let mut out = format!(
            "{name:<16}{:<12}{:<6}{:<6}{:<8}{:<10}`\n",
            0,
            0,
            0,
            100_644,
            body.len()
        )
        .into_bytes();

        out.extend_from_slice(body);

        if body.len() % 2 == 1 {
            out.push(b'\n');
        }

        out
    }

    fn package(members: &[(&str, &[u8])]) -> Vec<u8> {
        let mut out = b"!<arch>\n".to_vec();

        for (name, body) in members {
            out.extend(member(name, body));
        }

        out
    }

    #[test]
    fn the_data_member_is_found_after_odd_sized_members() {
        let deb = package(&[
            ("debian-binary", b"2.0\n"),
            ("control.tar.xz", b"abc"),
            ("data.tar.xz", b"payload"),
        ]);
        let (name, data) = deb_data(&deb).unwrap();

        assert_eq!(name, "data.tar.xz");
        assert_eq!(data, b"payload");
    }

    #[test]
    fn a_file_that_is_not_a_package_is_refused() {
        assert!(deb_data(b"hello").unwrap_err().contains("no `ar` header"));
        assert!(
            deb_data(&package(&[("debian-binary", b"2.0\n")]))
                .unwrap_err()
                .contains("no data.tar")
        );
        let mut truncated = package(&[("data.tar.xz", b"payload")]);

        truncated.truncate(truncated.len() - 3);
        assert!(deb_data(&truncated).unwrap_err().contains("truncated"));
    }

    #[test]
    fn a_download_is_refused_unless_it_is_https() {
        let dir = test_support::TempDir::new("fetch-https");
        let error = pinned("http://example.com/a", "00", &dir.join("a")).unwrap_err();

        assert!(error.contains("only https"), "{error}");
    }

    #[test]
    fn a_file_with_the_pinned_hash_is_not_fetched_again() {
        let dir = test_support::TempDir::new("fetch-cached");
        let file = dir.write("a", "hello");
        let hash = sha256_file(&file).unwrap();

        // The URL would fail: the file is already there and matches.
        pinned("https://invalid.invalid/a", &hash, &file).unwrap();
    }
}
