//! Locations inside the repository, independent of the crate whose test asks.

use std::path::{Path, PathBuf};

/// The repository root, canonical.
///
/// # Panics
/// When the checkout is missing (the crate was built from a published archive).
#[must_use]
pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the repository root exists")
}

/// The canonical path of `relative` inside the repository (`tests/compiler/x.jai`).
///
/// # Panics
/// When the file does not exist.
#[must_use]
pub fn path(relative: &str) -> PathBuf {
    let path = root().join(relative);
    path.canonicalize()
        .unwrap_or_else(|error| panic!("{} does not exist: {error}", path.display()))
}

/// The text of the repository file `relative`.
///
/// # Panics
/// When the file cannot be read.
#[must_use]
pub fn read(relative: &str) -> String {
    let path = path(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()))
}

/// A file or directory of `tests/fixtures` (`plugin-registration/echo.jai`), not required to
/// exist.
#[must_use]
pub fn fixture(relative: &str) -> PathBuf {
    root().join("tests/fixtures").join(relative)
}

/// The repository's stdlib directory, canonical.
#[must_use]
pub fn stdlib() -> PathBuf {
    path("stdlib")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_repository() {
        assert!(root().join("Cargo.toml").is_file());
        assert!(stdlib().is_dir());
        assert!(read("Cargo.toml").contains("[workspace]"));
    }
}
