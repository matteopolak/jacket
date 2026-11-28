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
/// else the one jaic reports (`jaic -print_stdlib`). Whichever it is must hold a prelude, as
/// jaic decides it ([`jaic::find_preload`]): a folder that does not is an error naming where the
/// path came from.
pub(crate) fn stdlib_dir(jaic: &Path, project: &Path, cwd: &Path) -> Result<PathBuf, String> {
    if let Some(dir) = jacket_config::env::STDLIB.non_empty() {
        let dir = from_variable(&dir, cwd);

        return checked(
            dir,
            &format!("{} is set to it", jacket_config::env::STDLIB.name),
        );
    }

    let own = project.join("stdlib");

    if holds_library(&own) {
        return checked(own, "it is the project's own");
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

    checked(
        PathBuf::from(line),
        &format!("`{} -print_stdlib` named it", jaic.display()),
    )
}

/// Whether `dir` holds a standard library: a prelude jaic would load.
fn holds_library(dir: &Path) -> bool {
    jaic::find_preload(&jaic::filesystem::NativeFs, dir).is_some()
}

/// `dir`, canonical, when it holds a standard library; else jaic's error for it, with `why` it
/// was chosen.
fn checked(dir: PathBuf, why: &str) -> Result<PathBuf, String> {
    if let Some(message) = jaic::missing_stdlib(&dir) {
        return Err(format!(
            "{message}\nnote: jacket uses `{}` because {why}",
            dir.display()
        ));
    }

    dir.canonicalize()
        .map_err(|e| format!("cannot open the standard library `{}`: {e}", dir.display()))
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
    fn a_folder_without_a_prelude_is_an_error_naming_where_it_came_from() {
        let root = std::env::temp_dir().join(format!("jacket-stdlib-bad-{}", std::process::id()));

        std::fs::create_dir_all(&root).unwrap();
        let error = checked(root.clone(), "JAIC_STDLIB is set to it").unwrap_err();

        assert!(
            error.starts_with("cannot find the standard library"),
            "{error}"
        );
        assert!(
            error.ends_with(&format!(
                "\nnote: jacket uses `{}` because JAIC_STDLIB is set to it",
                root.display()
            )),
            "{error}"
        );
        let stdlib = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");

        assert_eq!(
            checked(stdlib.clone(), "-").unwrap(),
            stdlib.canonicalize().unwrap()
        );
        let _ = std::fs::remove_dir_all(&root);
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
