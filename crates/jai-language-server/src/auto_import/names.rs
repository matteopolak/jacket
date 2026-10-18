//! Module names and ranks for auto-import.
use jaic::sema::TargetOs;
use std::path::Path;

/// Whether a module asserting `only_os` builds for `os`.
pub(super) fn builds_for(only_os: Option<&[String]>, os: TargetOs) -> bool {
    let name = jaic::target::os_member(os);
    only_os.is_none_or(|targets| targets.iter().any(|t| t == name))
}

/// The name of the module of `dir` that `file` is part of: its folder (`Basic/x.jai`, and
/// `Extensions/Name/x.jai` in the stdlib) or the file itself (`Name.jai`).
pub(super) fn module_of(file: &Path, dir: &Path) -> Option<String> {
    let parts: Vec<String> = file
        .strip_prefix(dir)
        .ok()?
        .iter()
        .map(|c| c.to_string_lossy().into_owned())
        .collect();
    match parts.as_slice() {
        [extensions, name, _, ..] if extensions == jaic::STDLIB_EXTENSIONS_DIR => {
            Some(format!("{extensions}/{name}"))
        }
        [folder, _, ..] => Some(folder.clone()),
        [file] => file.strip_suffix(".jai").map(str::to_string),
        [] => None,
    }
}

/// Ranks of stdlib modules: common ones (`Basic`, `String`, ...) first, `Extensions/` last.
pub(super) fn stdlib_rank(module: &str) -> u32 {
    match jaic::sema::COMMON_MODULES.iter().position(|m| *m == module) {
        Some(at) => 10 + at as u32,
        None if module.contains('/') => 2000,
        None => 1000,
    }
}
