//! Which standard library a test run uses.

use std::path::{Path, PathBuf};

/// `JAIC_STDLIB` as `value`, made absolute from the working directory `cwd` when it is relative.
/// Every jaic that jacket starts runs in another folder, so each is handed this path.
fn from_variable(value: &std::ffi::OsStr, cwd: &Path) -> PathBuf {
    let dir = cwd.join(value);
    dir.canonicalize().unwrap_or(dir)
}

/// The standard library the tests use, which `$STDLIB` stands for and which every jaic that
/// jacket starts is told through `JAIC_STDLIB`. `JAIC_STDLIB` as jacket was started wins; else
/// the project's own `stdlib/` when it holds the library (the Jacket repository, or a checkout
/// of it such as a git worktree, tests itself and not the checkout its binaries were built in);
/// else the one jaic reports (`jaic -print_stdlib`).
pub(crate) fn stdlib_dir(jaic: &Path, project: &Path, cwd: &Path) -> Result<PathBuf, String> {
    if let Some(dir) = jacket_config::env::STDLIB.non_empty() {
        return Ok(from_variable(&dir, cwd));
    }
    let canonical = |dir: PathBuf| dir.canonicalize().unwrap_or(dir);
    let own = project.join("stdlib");
    if own.join("prelude/module.jai").is_file() || own.join("Preload.jai").is_file() {
        return Ok(canonical(own));
    }
    let output = std::process::Command::new(jaic)
        .arg("-print_stdlib")
        .output()
        .map_err(|e| format!("could not start `{}`: {e}", jaic.display()))?;
    let text = String::from_utf8_lossy(&output.stdout);
    let line = text.lines().next().unwrap_or("").trim();
    if !output.status.success() || line.is_empty() {
        return Err(format!(
            "`{} -print_stdlib` failed; is this jaic too old for this jacket?",
            jaic.display()
        ));
    }
    Ok(canonical(PathBuf::from(line)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_project_holding_the_library_tests_its_own_copy() {
        if jacket_config::env::STDLIB.os().is_some() {
            return;
        }
        let project = std::env::temp_dir().join(format!("jacket-stdlib-{}", std::process::id()));
        std::fs::create_dir_all(project.join("stdlib/prelude")).unwrap();
        std::fs::write(project.join("stdlib/prelude/module.jai"), "").unwrap();
        // The compiler path is never run: the project's library is found first.
        let found = stdlib_dir(Path::new("/no/such/jaic"), &project, &project).unwrap();
        assert_eq!(found, project.join("stdlib").canonicalize().unwrap());
        // Without one, the compiler is asked, and a missing compiler is an error.
        let bare = project.join("stdlib/prelude");
        assert!(stdlib_dir(Path::new("/no/such/jaic"), &bare, &bare).is_err());
        let _ = std::fs::remove_dir_all(&project);
    }

    #[test]
    fn a_relative_variable_is_taken_from_the_working_directory() {
        let root = std::env::temp_dir().join(format!("jacket-stdlib-var-{}", std::process::id()));
        std::fs::create_dir_all(root.join("lib")).unwrap();
        std::fs::create_dir_all(root.join("module")).unwrap();
        let want = root.join("lib").canonicalize().unwrap();
        // From the folder holding it, and from a folder beside it.
        assert_eq!(from_variable("lib".as_ref(), &root), want);
        assert_eq!(from_variable("../lib".as_ref(), &root.join("module")), want);
        assert_eq!(from_variable(want.as_os_str(), &root.join("module")), want);
        // Where there is no such folder it is still absolute, so a jaic started elsewhere names
        // the same place.
        assert_eq!(
            from_variable("lib".as_ref(), &root.join("module")),
            root.join("module/lib")
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}
