//! Paths as messages show them.

use std::path::Path;

/// `path` as a message shows it: relative to `base` when it is below it (`.` for `base` itself),
/// else as given, with `/` between folders on every OS.
///
/// ```ignore
/// assert_eq!(path(Path::new("/p/target/a.json"), Path::new("/p")), "target/a.json");
/// ```
pub fn path(path: &Path, base: &Path) -> String {
    let canonical = jaic::canonicalize(base).ok();
    let below = canonical
        .as_deref()
        .and_then(|base| path.strip_prefix(base).ok())
        .or_else(|| path.strip_prefix(base).ok());
    match below {
        Some(below) if below.as_os_str().is_empty() => ".".to_string(),
        Some(below) => below.to_string_lossy().replace('\\', "/"),
        None => path.to_string_lossy().replace('\\', "/"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_below_the_base_are_relative() {
        assert_eq!(
            path(Path::new("/p/target/a.json"), Path::new("/p")),
            "target/a.json"
        );
        assert_eq!(path(Path::new("/p"), Path::new("/p")), ".");
        assert_eq!(path(Path::new("/q/a"), Path::new("/p")), "/q/a");
    }
}
