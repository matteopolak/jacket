//! Splitting OS strings (process arguments, environment values) at ASCII characters, so an
//! argument that is not UTF-8 keeps its bytes instead of being decoded or rejected.
//!
//! ```
//! use jacket_config::os_text;
//! use std::ffi::{OsStr, OsString};
//! assert_eq!(
//!     os_text::strip_prefix(OsStr::new("-Wl,-rpath,/x"), "-Wl,"),
//!     Some(OsString::from("-rpath,/x"))
//! );
//! assert_eq!(os_text::split(OsStr::new("a,b"), b','), ["a", "b"]);
//! ```
use std::ffi::{OsStr, OsString};

/// The text after `prefix`, when `arg` starts with it.
pub fn strip_prefix(arg: &OsStr, prefix: &str) -> Option<OsString> {
    if let Some(text) = arg.to_str() {
        return text.strip_prefix(prefix).map(OsString::from);
    }
    let units = units(arg);
    let prefix: Vec<Unit> = prefix.bytes().map(unit).collect();
    units.strip_prefix(prefix.as_slice()).map(from_units)
}

/// The parts before and after the first `separator` (an ASCII character).
pub fn split_once(arg: &OsStr, separator: u8) -> Option<(OsString, OsString)> {
    debug_assert!(separator.is_ascii());
    let units = units(arg);
    let at = units.iter().position(|&u| u == unit(separator))?;
    Some((from_units(&units[..at]), from_units(&units[at + 1..])))
}

/// The parts between each `separator` (an ASCII character).
pub fn split(arg: &OsStr, separator: u8) -> Vec<OsString> {
    debug_assert!(separator.is_ascii());
    units(arg)
        .split(|&u| u == unit(separator))
        .map(from_units)
        .collect()
}

#[cfg(any(unix, not(windows)))]
type Unit = u8;

/// An ASCII character as a unit of the platform's encoding.
#[cfg(any(unix, not(windows)))]
fn unit(byte: u8) -> Unit {
    byte
}

#[cfg(unix)]
fn units(arg: &OsStr) -> Vec<u8> {
    std::os::unix::ffi::OsStrExt::as_bytes(arg).to_vec()
}

#[cfg(unix)]
fn from_units(units: &[u8]) -> OsString {
    <OsStr as std::os::unix::ffi::OsStrExt>::from_bytes(units).to_os_string()
}

#[cfg(windows)]
type Unit = u16;

#[cfg(windows)]
fn unit(byte: u8) -> Unit {
    Unit::from(byte)
}

#[cfg(windows)]
fn units(arg: &OsStr) -> Vec<u16> {
    std::os::windows::ffi::OsStrExt::encode_wide(arg).collect()
}

#[cfg(windows)]
fn from_units(units: &[u16]) -> OsString {
    std::os::windows::ffi::OsStringExt::from_wide(units)
}

// Elsewhere (the browser build) an OS string comes only from Rust text, so it is UTF-8.

#[cfg(not(any(unix, windows)))]
fn units(arg: &OsStr) -> Vec<u8> {
    arg.to_string_lossy().into_owned().into_bytes()
}

#[cfg(not(any(unix, windows)))]
fn from_units(units: &[u8]) -> OsString {
    String::from_utf8_lossy(units).into_owned().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_splits_at_ascii_characters() {
        assert_eq!(
            split_once(OsStr::new("-link_arg=a=b"), b'='),
            Some(("-link_arg".into(), "a=b".into()))
        );
        assert_eq!(split_once(OsStr::new("plain"), b'='), None);
        assert_eq!(split(OsStr::new("-rpath,,/x"), b','), ["-rpath", "", "/x"]);
        assert_eq!(strip_prefix(OsStr::new("-L/y"), "-Wl,"), None);
    }

    #[cfg(unix)]
    #[test]
    fn bytes_that_are_not_utf8_are_kept() {
        use std::os::unix::ffi::OsStrExt;
        let arg = OsStr::from_bytes(b"-Wl,-rpath,/\xff");
        let rest = strip_prefix(arg, "-Wl,").unwrap();
        assert_eq!(rest.as_bytes(), b"-rpath,/\xff");
        let parts = split(&rest, b',');
        assert_eq!(parts[1].as_bytes(), b"/\xff");
        let (name, value) = split_once(OsStr::from_bytes(b"-link_arg=\xfe"), b'=').unwrap();
        assert_eq!(name, "-link_arg");
        assert_eq!(value.as_bytes(), b"\xfe");
    }

    #[cfg(windows)]
    #[test]
    fn unpaired_surrogates_are_kept() {
        use std::os::windows::ffi::{OsStrExt, OsStringExt};
        // `-Wl,` then a lone high surrogate, a comma and `x`: not UTF-8, but a valid Windows
        // argument.
        let mut wide: Vec<u16> = "-Wl,".encode_utf16().collect();
        wide.extend([0xD800, u16::from(b','), u16::from(b'x')]);
        let arg = OsString::from_wide(&wide);
        assert!(arg.to_str().is_none());
        let rest = strip_prefix(&arg, "-Wl,").unwrap();
        let parts = split(&rest, b',');
        assert_eq!(parts[0].encode_wide().collect::<Vec<_>>(), [0xD800]);
        assert_eq!(parts[1], "x");
        let (name, value) = split_once(&arg, b',').unwrap();
        assert_eq!(name, "-Wl");
        assert_eq!(value.encode_wide().next(), Some(0xD800));
    }
}
