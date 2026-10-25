//! Golden-output tests (`tests/name.jai` with `name.stdout`, `name.stderr`, `name.exit`) and
//! compile-fail tests (`tests/compile_fail/name.jai` with `name.stderr`).
//!
//! Output is compared after `\r\n` becomes `\n`. In a compile-fail test's diagnostics the
//! test's folder is written `$DIR` and the standard library's `$STDLIB`, so expected files do not
//! depend on where the repository is. `--accept` writes what the program printed into the
//! expected files instead of comparing (for a golden test, into those that exist, `.stdout` at
//! least).
use super::build::{Toolchain, compile_output, program_and_args, unbuilt_flags};
use super::exec::{self, Ending, Invocation, Limits, Live};
use super::jobs::ending_text;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

/// How a golden or compile-fail test went.
#[derive(Debug, Default)]
pub struct Checked {
    pub passed: bool,
    /// Why it failed: a diff, a status, compile errors.
    pub problems: Vec<String>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub timed_out: bool,
    pub accepted: Vec<PathBuf>,
}

fn normalized(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).replace("\r\n", "\n")
}

/// Compares `got` to the expected file at `path`, or writes it there with `accept`.
fn expect_text(path: &Path, got: &str, what: &str, accept: bool, checked: &mut Checked) {
    if accept {
        if std::fs::write(path, got).is_ok() {
            checked.accepted.push(path.to_path_buf());
        }
        return;
    }
    let want = std::fs::read(path)
        .map(|b| normalized(&b))
        .unwrap_or_default();
    if want != got {
        checked.problems.push(format!(
            "{what} differs from {}:\n{}",
            file_name(path),
            super::diff::lines(&want, got)
        ));
    }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned())
}

/// How to run `source`: `jaic -interp` on it, or the program built from it in `work`. A
/// compile error is the `Err`.
pub(super) fn invocation(
    source: &Path,
    root_flags: &[String],
    toolchain: &Toolchain,
    work: &Path,
    cwd: &Path,
) -> Result<Invocation, String> {
    if toolchain.interp {
        let mut args = vec![
            source.to_string_lossy().into_owned(),
            "-interp".into(),
            "-quiet".into(),
        ];
        args.extend(unbuilt_flags(&toolchain.flags));
        args.extend(root_flags.iter().cloned());
        return Ok(Invocation {
            program: toolchain.jaic.clone(),
            args,
            cwd: cwd.to_path_buf(),
        });
    }
    let stem = file_name(&source.with_extension(""));
    let exe = work.join(format!("golden-{stem}"));
    let report = work.join(format!("golden-{stem}.txt"));
    let output = Command::new(&toolchain.jaic)
        .arg(source)
        .arg("-quiet")
        .args(&toolchain.flags)
        .args(root_flags)
        .arg("-exe")
        .arg(&exe)
        .env("JAIC_REPORT_EXE", &report)
        .current_dir(work)
        .output()
        .map_err(|error| format!("could not start `{}`: {error}", toolchain.jaic.display()))?;
    if !output.status.success() {
        return Err(format!("it does not compile:\n{}", compile_output(&output)));
    }
    let exe = std::fs::read_to_string(&report)
        .ok()
        .and_then(|t| t.lines().last().map(PathBuf::from))
        .filter(|p| p.is_file())
        .unwrap_or(exe);
    let (program, args) = program_and_args(&exe, &toolchain.runner);
    Ok(Invocation {
        program,
        args,
        cwd: cwd.to_path_buf(),
    })
}

/// Runs `invocation` with no records expected: everything it prints is its output.
pub(super) fn run_plain(
    invocation: &Invocation,
    timeout: Option<Duration>,
    live: &mut dyn FnMut(Live),
) -> exec::Outcome {
    let limit = move |_: &str| timeout;
    let limits = Limits {
        timeout: &limit,
        slow: None,
        stream: false,
    };
    exec::run(invocation, &limits, live)
}

/// Builds and runs the golden test `source`.
#[allow(clippy::too_many_arguments)]
pub fn golden(
    source: &Path,
    root_flags: &[String],
    toolchain: &Toolchain,
    work: &Path,
    cwd: &Path,
    timeout: Option<Duration>,
    accept: bool,
    live: &mut dyn FnMut(Live),
) -> Checked {
    let mut checked = Checked::default();
    let invocation = match invocation(source, root_flags, toolchain, work, cwd) {
        Ok(invocation) => invocation,
        Err(problem) => {
            checked.problems.push(problem);
            return checked;
        }
    };
    let outcome = run_plain(&invocation, timeout, live);
    checked.stdout = outcome.preamble_stdout;
    checked.stderr = outcome.preamble;
    if outcome.ending == Ending::TimedOut {
        checked.timed_out = true;
        checked.problems.push("it timed out".into());
        return checked;
    }
    let code = match outcome.ending {
        Ending::Exited(code) => Some(code),
        _ => None,
    };
    let exit_file = source.with_extension("exit");
    let want_code: i32 = std::fs::read_to_string(&exit_file)
        .ok()
        .and_then(|t| t.trim().parse().ok())
        .unwrap_or(0);
    if accept && exit_file.is_file() {
        if let Some(code) = code
            && std::fs::write(&exit_file, format!("{code}\n")).is_ok()
        {
            checked.accepted.push(exit_file.clone());
        }
    } else if code != Some(want_code) {
        checked.problems.push(format!(
            "the program {}, want status {want_code}",
            ending_text(&outcome.ending)
        ));
    }
    let stdout_file = source.with_extension("stdout");
    if stdout_file.is_file() || accept {
        expect_text(
            &stdout_file,
            &normalized(&checked.stdout),
            "standard output",
            accept,
            &mut checked,
        );
    }
    let stderr_file = source.with_extension("stderr");
    if stderr_file.is_file() {
        expect_text(
            &stderr_file,
            &normalized(&checked.stderr),
            "standard error",
            accept,
            &mut checked,
        );
    }
    checked.passed = checked.problems.is_empty();
    checked
}

/// `text` with `dir` written `$DIR` and `stdlib` written `$STDLIB`, separators as `/`.
pub fn normalize_paths(text: &str, dir: &Path, stdlib: &Path) -> String {
    let mut text = text.replace("\r\n", "\n");
    for (path, name) in [(stdlib, "$STDLIB"), (dir, "$DIR")] {
        let plain = path.to_string_lossy().into_owned();
        let forward = plain.replace('\\', "/");
        for form in [plain, forward] {
            if !form.is_empty() {
                text = text.replace(&form, name);
            }
        }
    }
    // Paths after `$DIR` and `$STDLIB` keep their own separators on Windows; make them alike.
    text.lines()
        .map(|line| {
            if line.contains("$DIR\\") || line.contains("$STDLIB\\") {
                line.replace('\\', "/")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + if text.ends_with('\n') {
            "\n"
        } else {
            ""
        }
}

/// Type-checks the compile-fail test `source`: it must fail, printing its `.stderr`.
pub fn compile_fail(
    source: &Path,
    root_flags: &[String],
    toolchain: &Toolchain,
    stdlib: &Path,
    accept: bool,
) -> Checked {
    // From the test's folder, diagnostics name the file without a path.
    let dir = source.parent().unwrap_or(Path::new("."));
    let mut checked = Checked::default();
    let output = Command::new(&toolchain.jaic)
        .arg(source)
        .arg("-check")
        .arg("-quiet")
        .arg("-no_color")
        .args(unbuilt_flags(&toolchain.flags))
        .args(root_flags)
        .env("NO_COLOR", "1")
        .current_dir(dir)
        .output();
    let output = match output {
        Ok(output) => output,
        Err(error) => {
            checked.problems.push(format!(
                "could not start `{}`: {error}",
                toolchain.jaic.display()
            ));
            return checked;
        }
    };
    if output.status.success() {
        checked
            .problems
            .push("it compiled; a compile-fail test must not".into());
        return checked;
    }
    let got = normalize_paths(&compile_output(&output), dir, stdlib);
    let expected = source.with_extension("stderr");
    if expected.is_file() || accept {
        expect_text(&expected, &got, "the diagnostics", accept, &mut checked);
    } else {
        checked.problems.push(format!(
            "{} is missing; `jacket test --accept` writes it from these diagnostics:\n{got}",
            file_name(&expected)
        ));
    }
    checked.passed = checked.problems.is_empty();
    checked
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_become_placeholders() {
        let text = "/r/tests/compile_fail/bad.jai:3:1: error: x\n/s/Basic/module.jai:9: note\n";
        assert_eq!(
            normalize_paths(text, Path::new("/r/tests/compile_fail"), Path::new("/s")),
            "$DIR/bad.jai:3:1: error: x\n$STDLIB/Basic/module.jai:9: note\n"
        );
        assert_eq!(
            normalize_paths(
                "C:\\r\\t\\bad.jai:1: e\r\n",
                Path::new("C:\\r\\t"),
                Path::new("C:\\s")
            ),
            "$DIR/bad.jai:1: e\n"
        );
    }
}
