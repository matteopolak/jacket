use crate::{Error, Range};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DocumentUri {
    uri: String,
    path: String,
}

impl DocumentUri {
    pub fn parse(uri: &str) -> Result<Self, Error> {
        Self::parse_platform(uri, cfg!(windows))
    }

    fn parse_platform(uri: &str, windows: bool) -> Result<Self, Error> {
        if uri.len() > 4096 {
            return Err(Error::Limit("document URI exceeds 4096 bytes"));
        }
        let encoded = uri
            .strip_prefix("file://")
            .ok_or(Error::Uri("only absolute local file URIs are supported"))?;
        if !encoded.starts_with('/') || encoded.contains(['?', '#', '\\', '\0']) {
            return Err(Error::Uri(
                "file URI must have no authority, query or fragment",
            ));
        }
        let mut bytes = Vec::with_capacity(encoded.len());
        let input = encoded.as_bytes();
        let mut at = 0;
        while at < input.len() {
            if input[at] == b'%' {
                let value = input
                    .get(at + 1..at + 3)
                    .ok_or(Error::Uri("incomplete URI escape"))?;
                let high = hex(value[0]).ok_or(Error::Uri("invalid URI escape"))?;
                let low = hex(value[1]).ok_or(Error::Uri("invalid URI escape"))?;
                bytes.push(high * 16 + low);
                at += 3;
            } else {
                bytes.push(input[at]);
                at += 1;
            }
        }
        let decoded = String::from_utf8(bytes).map_err(|_| Error::Uri("URI path is not UTF-8"))?;
        Self::from_uri_path(&decoded, windows)
    }

    /// A native filesystem path is data, not an already encoded URI component.
    /// Drive paths use the host's Windows rules; filenames retain their case.
    pub fn from_path(path: &Path) -> Result<Self, Error> {
        let text = path.to_str().ok_or(Error::Uri("file path is not UTF-8"))?;
        Self::from_native(text, cfg!(windows))
    }

    fn from_native(path: &str, windows: bool) -> Result<Self, Error> {
        if path.len() > 4096 {
            return Err(Error::Limit("document path exceeds 4096 bytes"));
        }
        let mut path = native_spelling(path, windows)?;
        if windows {
            if path.starts_with("//") {
                return Err(Error::Uri(
                    "only local file paths without an authority are supported",
                ));
            }
            if drive(&path) {
                if !path[2..].starts_with('/') {
                    return Err(Error::Uri("drive-relative paths are unsupported"));
                }
                path.insert(0, '/');
            }
        }
        Self::from_uri_path(&path, windows)
    }

    fn from_uri_path(path: &str, windows: bool) -> Result<Self, Error> {
        let normalized = normalize(path, windows)?;
        let uri = format!("file://{}", encode(&normalized));
        if uri.len() > 4096 {
            return Err(Error::Limit("canonical document URI exceeds 4096 bytes"));
        }
        Ok(Self {
            uri,
            // `/C:/...` is URI syntax. A leading slash makes it a rooted path on the
            // current Windows drive instead of the absolute path named by the client.
            path: if windows && drive(&normalized[1..]) {
                normalized[1..].to_owned()
            } else {
                normalized
            },
        })
    }

    pub fn as_str(&self) -> &str {
        &self.uri
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub(crate) fn load(&self, target: &str) -> Result<Self, Error> {
        if target.contains('\0') || target.len() > 4096 {
            return Err(Error::Uri("unsupported #load target"));
        }
        let parent = Path::new(&self.path)
            .parent()
            .ok_or(Error::Uri("document has no parent folder"))?;
        Self::from_path(&parent.join(target))
    }
}

fn native_spelling(path: &str, windows: bool) -> Result<String, Error> {
    let mut result = path.to_owned();
    if windows {
        if result.starts_with(r"\\?\") {
            result = jaic::strip_verbatim(&result)
                .ok_or(Error::Uri("unsupported verbatim file path"))?;
        }
        result = result.replace('\\', "/");
        if drive(&result) {
            result.replace_range(0..1, &result[..1].to_ascii_uppercase());
        }
    }
    Ok(result)
}

/// Normalize native spelling after filesystem canonicalization. Do not casefold filenames:
/// canonicalization knows actual file identity, including case-sensitive Windows folders.
pub(crate) fn path_key(path: &Path) -> PathBuf {
    path_key_on(path, cfg!(windows))
}

fn path_key_on(path: &Path, windows: bool) -> PathBuf {
    if windows
        && let Some(text) = path.to_str()
        && let Ok(key) = native_spelling(text, true)
    {
        return PathBuf::from(key);
    }
    path.to_owned()
}

/// The one spelling of a file the server compares, stores and hands the compiler: `fs`'s
/// canonical path in `path_key` form. Paths from the client, the settings and the compiler
/// all meet here, so a mix of separators, drive-letter case or verbatim prefixes still match.
pub(crate) fn file_key(fs: &dyn jaic::sema::FileSystem, path: &Path) -> PathBuf {
    file_key_on(fs, path, cfg!(windows))
}

fn file_key_on(fs: &dyn jaic::sema::FileSystem, path: &Path, windows: bool) -> PathBuf {
    path_key_on(&fs.canonical(path), windows)
}

/// `dir`'s key as a string prefix of the keys of the files under it.
pub(crate) fn dir_prefix(fs: &dyn jaic::sema::FileSystem, dir: &Path) -> String {
    dir_prefix_on(fs, dir, cfg!(windows))
}

fn dir_prefix_on(fs: &dyn jaic::sema::FileSystem, dir: &Path, windows: bool) -> String {
    let mut prefix = file_key_on(fs, dir, windows).to_string_lossy().into_owned();
    if !prefix.ends_with('/') {
        prefix.push('/');
    }
    prefix
}

fn hex(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

fn encode(path: &str) -> String {
    let mut out = String::new();
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || b"/-._~:".contains(&byte) {
            out.push(byte as char);
        } else {
            use std::fmt::Write;
            let _ = write!(out, "%{byte:02X}");
        }
    }
    out
}

fn drive(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

fn normalize(path: &str, windows: bool) -> Result<String, Error> {
    if !path.starts_with('/') || path.contains('\0') || (windows && path.contains('\\')) {
        return Err(Error::Uri(
            "document URI path must be absolute with valid separators and no NUL",
        ));
    }
    if windows && path.starts_with("//") {
        return Err(Error::Uri("file URI must have no authority"));
    }
    let has_drive = windows && drive(&path[1..]);
    if has_drive && !path[3..].starts_with('/') {
        return Err(Error::Uri("drive URI must name an absolute path"));
    }
    let mut parts = vec![];
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if parts.len() <= usize::from(has_drive) {
                    return Err(Error::Uri("document path escapes its root"));
                }
                parts.pop();
            }
            value => parts.push(value),
        }
    }
    let mut result = format!("/{}", parts.join("/"));
    if has_drive {
        result.replace_range(1..2, &result[1..2].to_ascii_uppercase());
        if parts.len() == 1 {
            result.push('/');
        }
    }
    Ok(result)
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextChange {
    pub range: Option<Range>,
    pub range_length: Option<u32>,
    pub text: String,
}

/// A closed immutable snapshot of the open documents. It never consults an OS filesystem.
#[derive(Clone, Debug, Default)]
pub struct VirtualSources {
    pub(crate) files: BTreeMap<String, String>,
}

impl VirtualSources {
    /// Text of an open document by absolute path; unopened or escaping paths are `None`.
    pub fn read(&self, path: &str) -> Option<&str> {
        self.files
            .get(DocumentUri::from_path(Path::new(path)).ok()?.path())
            .map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_drive_uri_maps_to_native_absolute_path() {
        let uri = DocumentUri::parse_platform(
            "file:///c%3A/Users/Name/../jai%20%23%25%C3%A9/module.jai",
            true,
        )
        .unwrap();
        assert_eq!(uri.path(), "C:/Users/jai #%é/module.jai");
        assert_eq!(
            uri.as_str(),
            "file:///C:/Users/jai%20%23%25%C3%A9/module.jai"
        );
        assert_eq!(
            DocumentUri::parse_platform(uri.as_str(), true).unwrap(),
            uri
        );
    }

    #[test]
    fn windows_native_separators_drive_case_and_verbatim_paths_share_uri_identity() {
        let a = DocumentUri::from_native(r"c:\Users\jai # % é\Basic\module.jai", true).unwrap();
        for spelling in [
            "C:/Users/jai # % é/Basic/module.jai",
            r"\\?\c:\Users\jai # % é\Basic\module.jai",
        ] {
            assert_eq!(DocumentUri::from_native(spelling, true).unwrap(), a);
        }
        assert_eq!(
            a.as_str(),
            "file:///C:/Users/jai%20%23%20%25%20%C3%A9/Basic/module.jai"
        );
        assert_ne!(
            a,
            DocumentUri::from_native("C:/Users/jai # % é/basic/module.jai", true).unwrap(),
            "filesystem identity must decide filename case equivalence"
        );
    }

    #[test]
    fn local_uri_boundaries_reject_ambiguous_drives_authorities_escapes_and_nuls() {
        for uri in [
            "file:///C:/../outside.jai",
            "file:///C:relative.jai",
            "file://remote/share/main.jai",
            "file:////remote/share/main.jai",
            "file:///C:/bad%00name.jai",
            r"file:///C:\main.jai",
            "file:///C:/bad%GGname.jai",
        ] {
            assert!(DocumentUri::parse_platform(uri, true).is_err(), "{uri}");
        }
        for path in ["C:relative.jai", r"\\remote\share\main.jai", r"\\.\device"] {
            assert!(DocumentUri::from_native(path, true).is_err(), "{path}");
        }
        let root = DocumentUri::parse_platform("file:///c%3A/", true).unwrap();
        assert_eq!(root.path(), "C:/");
        assert_eq!(root.as_str(), "file:///C:/");
        assert_eq!(
            DocumentUri::parse_platform("file:///", false)
                .unwrap()
                .path(),
            "/"
        );
    }

    #[test]
    fn posix_names_and_literal_percent_bytes_are_encoded_once() {
        let uri = DocumentUri::from_native(r"/case/c:/a\b # %23 é.jai", false).unwrap();
        assert_eq!(
            uri.as_str(),
            "file:///case/c:/a%5Cb%20%23%20%2523%20%C3%A9.jai"
        );
        assert_eq!(
            DocumentUri::parse_platform(uri.as_str(), false).unwrap(),
            uri
        );
        assert!(DocumentUri::from_native("relative.jai", false).is_err());
    }

    /// Canonicalizes the way Windows does: native separators, sometimes a verbatim prefix.
    struct WindowsFs {
        verbatim: bool,
    }

    impl jaic::sema::FileSystem for WindowsFs {
        fn read(&self, _: &Path) -> Option<Vec<u8>> {
            None
        }

        fn is_file(&self, _: &Path) -> bool {
            true
        }

        fn is_dir(&self, _: &Path) -> bool {
            true
        }

        fn canonical(&self, path: &Path) -> PathBuf {
            let native = path.to_str().unwrap().replace('/', "\\");
            PathBuf::from(if self.verbatim {
                format!(r"\\?\{native}")
            } else {
                native
            })
        }
    }

    #[test]
    fn windows_root_prefix_matches_the_keys_of_its_files() {
        for verbatim in [false, true] {
            let fs = WindowsFs {
                verbatim,
            };
            let prefix = dir_prefix_on(&fs, Path::new(r"c:\Users\x\proj"), true);
            assert_eq!(prefix, "C:/Users/x/proj/");
            for file in [r"C:\Users\x\proj\main.jai", "c:/Users/x/proj/src/a.jai"] {
                let key = file_key_on(&fs, Path::new(file), true);
                assert!(key.to_str().unwrap().starts_with(&prefix), "{key:?}");
            }
        }
    }

    #[test]
    fn windows_stdlib_setting_and_client_uri_name_the_same_file() {
        let fs = WindowsFs {
            verbatim: true,
        };
        let stdlib = Path::new(r"C:\Users\x\jai-toolchain\stdlib");
        let key = file_key_on(&fs, &stdlib.join("Basic").join("Print.jai"), true);
        assert_eq!(
            key.to_str().unwrap(),
            "C:/Users/x/jai-toolchain/stdlib/Basic/Print.jai"
        );
        let from_compiler = DocumentUri::from_native(key.to_str().unwrap(), true).unwrap();
        let from_client = DocumentUri::parse_platform(
            "file:///c%3A/Users/x/jai-toolchain/stdlib/Basic/Print.jai",
            true,
        )
        .unwrap();
        assert_eq!(from_compiler, from_client);
        assert_eq!(
            from_compiler.as_str(),
            "file:///C:/Users/x/jai-toolchain/stdlib/Basic/Print.jai"
        );
    }
}
