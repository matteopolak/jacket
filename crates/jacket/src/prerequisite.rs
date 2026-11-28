//! The error for something jacket needs from the machine and cannot find: what is missing, which
//! command needs it, and the one note that says how to get it. Every check runs before the
//! command changes anything.
use jaic_toolchain::package::{Need, PackageManager};
use jaic_toolchain::{Lookup, System};
use std::path::{Path, PathBuf};

/// The text of the error for `program` missing when `command` needs it. The caller prints it
/// after `error:`.
///
/// ```text
/// `curl` is not installed, and `jacket upgrade` needs it
/// note: install curl with `apt install curl`
/// ```
pub fn missing(need: &Need, program: &str, command: &str) -> String {
    let lookup = Lookup::new(&System);
    let note = need.note(lookup.host.os(), PackageManager::detect(&lookup));

    format!("`{program}` is not installed, and `{command}` needs it\nnote: {note}")
}

/// The error for `program` that would not start: missing, or present but not runnable.
pub fn spawn_error(error: &std::io::Error, need: &Need, program: &str, command: &str) -> String {
    if error.kind() == std::io::ErrorKind::NotFound {
        missing(need, program, command)
    } else {
        format!(
            "could not run `{program}`: {}, and `{command}` needs it\nnote: check that `{program}` on PATH is a working program",
            jaic::io_reason(error)
        )
    }
}

/// `program` on `PATH`, or the error for it missing.
pub fn require(need: &Need, program: &str, command: &str) -> Result<PathBuf, String> {
    Lookup::new(&System)
        .which(program)
        .ok_or_else(|| missing(need, program, command))
}

/// The program a runner starts, checked before anything is built for it. A word with a folder in
/// it is a path; a placeholder (`{exe}`) or an empty runner starts the program itself.
pub fn runner(words: &[String], command: &str) -> Result<(), String> {
    let Some(program) = words.first() else {
        return Ok(());
    };

    if program.starts_with('{') {
        return Ok(());
    }

    if program.contains(['/', '\\']) {
        return if Path::new(program).is_file() {
            Ok(())
        } else {
            Err(format!(
                "the runner `{program}` does not exist, and `{command}` starts it\nnote: fix the path in `--runner` or in the target's `runner` in jacket.toml"
            ))
        };
    }

    require(&Need::for_program(program), program, command).map(|_| ())
}

/// Makes sure `dir` exists and can be written to, before `command` starts filling it.
pub fn writable_dir(dir: &Path, what: &str, command: &str) -> Result<(), String> {
    let probe = dir.join(format!(".jacket-write-test-{}", std::process::id()));
    let result = std::fs::create_dir_all(dir)
        .and_then(|()| std::fs::write(&probe, b""))
        .map(|()| {
            let _ = std::fs::remove_file(&probe);
        });

    result.map_err(|error| {
        format!(
            "cannot write to {what} `{}`: {}, and `{command}` needs it\nnote: make it writable, or move the cache folder with XDG_CACHE_HOME (LOCALAPPDATA on Windows)",
            dir.display(),
            jaic::io_reason(&error)
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_program_names_the_command_and_the_next_step() {
        let text = missing(&Need::Curl, "curl", "jacket upgrade");

        assert!(text.starts_with("`curl` is not installed, and `jacket upgrade` needs it\nnote: "));
    }

    #[test]
    fn a_runner_that_is_not_installed_names_itself_and_the_command() {
        let error = runner(&["no-such-runner-9f3".to_string()], "jacket test").unwrap_err();

        assert!(
            error.starts_with(
                "`no-such-runner-9f3` is not installed, and `jacket test` needs it\nnote: "
            ),
            "{error}"
        );
        assert!(runner(&[], "jacket test").is_ok());
        assert!(runner(&["{exe}".to_string()], "jacket test").is_ok());
        let error = runner(&["/no/such/dir/run".to_string()], "jacket test").unwrap_err();

        assert!(
            error.contains("does not exist") && error.contains("\nnote: "),
            "{error}"
        );
    }

    #[test]
    fn a_folder_that_cannot_be_made_names_itself_the_command_and_the_variable() {
        let file = test_support::TempDir::new("jacket-prerequisite");

        std::fs::create_dir_all(&file).unwrap();
        let blocker = file.join("file");

        std::fs::write(&blocker, "").unwrap();
        let error = writable_dir(
            &blocker.join("cache"),
            "the cache folder",
            "jacket toolchain add",
        )
        .unwrap_err();

        assert!(
            error.contains("cannot write to the cache folder"),
            "{error}"
        );
        assert!(error.contains("`jacket toolchain add` needs it"), "{error}");
        assert!(
            error.contains("note: ") && error.contains("XDG_CACHE_HOME"),
            "{error}"
        );
    }
}
