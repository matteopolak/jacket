//! Golden-output tests (`tests/name.jai` with `name.stdout`, `name.stderr`, `name.exit`) and
//! compile-fail tests (`tests/compile_fail/name.jai` with `name.stderr`).
//!
//! Output is compared after `\r\n` becomes `\n`. In a compile-fail test's diagnostics the
//! test's folder is written `$DIR`, the standard library's `$STDLIB` and the project folder's
//! `$ROOT`, so expected files do not depend on where the repository is. `--accept` writes what the program printed into the
//! expected files instead of comparing (for a golden test, into those that exist, `.stdout` at
//! least).
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::build::{Toolchain, compile_output, path_text, program_and_args, stage_dlls};
use super::deps_store::Program;
use super::exec::{self, Ending, Invocation, Limits, Live};
use super::jobs::ending_text;
use super::options::TestOptions;

/// How a golden or compile-fail test went.
#[derive(Debug, Default)]
pub(crate) struct Checked {
    pub passed: bool,
    /// Why it failed: a diff, a status, compile errors.
    pub problems: Vec<String>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub timed_out: bool,
    /// The program died of a fault.
    pub crashed: bool,
    pub accepted: Vec<PathBuf>,
    /// Failed checks the program reported, shown like a unit test's.
    pub failures: Vec<exec::Failure>,
    /// Expected-output files written because they did not exist yet.
    pub created: Vec<PathBuf>,
}

fn normalized(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).replace("\r\n", "\n")
}

/// The other outputs a test may print instead of the one in `path`: `path.2`, `path.3`, ... in
/// order, for output that depends on the host (a sound device that is or is not present).
fn alternatives(path: &Path) -> Vec<PathBuf> {
    (2..100)
        .map(|n| PathBuf::from(format!("{}.{n}", path.display())))
        .take_while(|p| p.is_file())
        .collect()
}

/// Compares `got` to the expected file at `path` (or to any of its `alternatives`), or writes it
/// there with `accept`; with alternatives, `accept` writes only a new output that none of them has.
fn expect_text(path: &Path, got: &str, what: &str, accept: bool, checked: &mut Checked) {
    let others = alternatives(path);
    let matches = |file: &Path| std::fs::read(file).is_ok_and(|b| normalized(&b) == got);

    if accept {
        if !others.is_empty() && (matches(path) || others.iter().any(|f| matches(f))) {
            return;
        }

        if std::fs::write(path, got).is_ok() {
            checked.accepted.push(path.to_path_buf());
        }

        return;
    }

    let want = std::fs::read(path)
        .map(|b| normalized(&b))
        .unwrap_or_default();

    if want != got && !others.iter().any(|f| matches(f)) {
        let more = if others.is_empty() {
            String::new()
        } else {
            format!(" and {} alternative output(s)", others.len())
        };

        checked.problems.push(format!(
            "{what} differs from {}{more}:\n{}",
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
    built(source, root_flags, toolchain, work, cwd, true).map(|built| built.invocation)
}

/// A golden program ready to run, and what the compiler printed building it.
struct Built {
    invocation: Invocation,
    /// The compiler's standard output and error (the warnings); empty when interpreted.
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    /// Building made no executable: the program did all its work at compile time (a metaprogram
    /// with `#run` and no `main`), so what the compiler printed is its output.
    compile_time_only: bool,
}

/// Where the program reports what it ran, when coverage is measured.
fn coverage_env(toolchain: &Toolchain) -> Vec<(String, String)> {
    toolchain
        .coverage_dir
        .iter()
        .map(|dir| {
            (
                jacket_config::env::COVERAGE_DIR.name.to_string(),
                path_text(dir),
            )
        })
        .chain(toolchain.jaic_env())
        .collect()
}

/// `invocation`, with the compiler's output when the program was built. `quiet` is jaic's
/// `-quiet`, which also hides warnings.
fn built(
    source: &Path,
    root_flags: &[String],
    toolchain: &Toolchain,
    work: &Path,
    cwd: &Path,
    quiet: bool,
) -> Result<Built, String> {
    let quiet: Vec<&str> = if quiet {
        vec!["-quiet"]
    } else {
        Vec::new()
    };

    if toolchain.interp {
        let mut args = vec![source.to_string_lossy().into_owned(), "-interp".into()];

        args.extend(quiet.iter().map(ToString::to_string));

        if toolchain.coverage {
            args.push("-coverage".into());
        }

        args.extend(toolchain.flags.iter().cloned());
        args.extend(root_flags.iter().cloned());
        args.extend(toolchain.deps_args(Program::File(source), root_flags, false)?);

        return Ok(Built {
            invocation: Invocation {
                program: toolchain.jaic.clone(),
                args,
                cwd: cwd.to_path_buf(),
                env: coverage_env(toolchain)
                    .into_iter()
                    .chain(toolchain.jaic_env())
                    .collect(),
                ..Invocation::default()
            },
            stdout: Vec::new(),
            stderr: Vec::new(),
            compile_time_only: false,
        });
    }

    let stem = file_name(&source.with_extension(""));
    let exe = work.join(format!("golden-{stem}"));
    let report = work.join(format!("golden-{stem}.txt"));
    let mut command = toolchain.jaic_command();

    command
        .arg(source)
        .args(&quiet)
        .args(&toolchain.flags)
        .args(root_flags)
        .args(toolchain.deps_args(Program::File(source), root_flags, false)?)
        .arg("-exe")
        .arg(&exe)
        .env(jacket_config::env::REPORT_EXE.name, &report)
        .current_dir(work);

    if toolchain.coverage {
        command.arg("-coverage");
    }

    // A program that builds and runs others at compile time reports what they ran too.
    if let Some(dir) = &toolchain.coverage_dir {
        command.env(jacket_config::env::COVERAGE_DIR.name, dir);
    }

    let output = command
        .output()
        .map_err(|error| format!("could not start `{}`: {error}", toolchain.jaic.display()))?;

    if !output.status.success() {
        let text = compile_output(&output);

        // A program without `main` that does its work at compile time: it only has to
        // type-check, and what the compiler printed is its output.
        if text.contains("has no `main` procedure") || text.contains("no exported 'main'") {
            let checked = toolchain
                .jaic_command()
                .arg(source)
                .args(&quiet)
                .args(&toolchain.flags)
                .args(root_flags)
                .arg("-check")
                .current_dir(work)
                .output()
                .map_err(|error| {
                    format!("could not start `{}`: {error}", toolchain.jaic.display())
                })?;

            if checked.status.success() {
                return Ok(Built {
                    invocation: Invocation {
                        program: exe.clone(),
                        args: Vec::new(),
                        cwd: cwd.to_path_buf(),
                        ..Invocation::default()
                    },
                    stdout: checked.stdout,
                    stderr: checked.stderr,
                    compile_time_only: true,
                });
            }

            return Err(format!(
                "it does not type-check:\n{}",
                compile_output(&checked)
            ));
        }

        return Err(format!("it does not compile:\n{text}"));
    }

    let exe = std::fs::read_to_string(&report)
        .ok()
        .and_then(|t| t.lines().last().map(PathBuf::from))
        .filter(|p| p.is_file())
        .unwrap_or(exe);
    let compile_time_only = !exe.is_file();

    if !compile_time_only && toolchain.runner.is_empty() {
        stage_dlls(&exe)?;
    }

    let (program, args) = program_and_args(&exe, &toolchain.runner, &[]);
    let env = coverage_env(toolchain);

    Ok(Built {
        invocation: Invocation {
            program,
            args,
            cwd: cwd.to_path_buf(),
            env,
            ..Invocation::default()
        },
        stdout: output.stdout,
        stderr: output.stderr,
        compile_time_only,
    })
}

/// Runs `invocation` with no records expected: everything it prints is its output.
/// Its supplied timeout covers the whole process from launch, without a Testing start record.
pub(super) fn run_plain(
    invocation: &Invocation,
    timeout: Option<Duration>,
    live: &mut dyn FnMut(Live),
) -> exec::Outcome {
    let limit = |_: &str| None;
    let limits = Limits {
        process_timeout: timeout,
        timeout: &limit,
        slow: None,
        stream: false,
    };

    exec::run(invocation, &limits, live)
}

/// Type-checks the golden test `source` (`check_only`): it must pass.
fn check_only(source: &Path, root_flags: &[String], toolchain: &Toolchain, cwd: &Path) -> Checked {
    let mut checked = Checked::default();
    let output = toolchain
        .jaic_command()
        .arg(source)
        .arg("-check")
        .arg("-quiet")
        .args(&toolchain.flags)
        .args(root_flags)
        .args(
            match toolchain.deps_args(Program::File(source), root_flags, false) {
                Ok(args) => args,
                Err(error) => {
                    checked.problems.push(error);
                    return checked;
                }
            },
        )
        .current_dir(cwd)
        .output();

    match output {
        Ok(output) if output.status.success() => {}
        Ok(output) => checked.problems.push(format!(
            "it does not type-check:\n{}",
            compile_output(&output)
        )),
        Err(error) => checked.problems.push(format!(
            "could not start `{}`: {error}",
            toolchain.jaic.display()
        )),
    }

    checked.passed = checked.problems.is_empty();
    checked
}

/// The program's arguments: `$TEMP` is a fresh folder, `$DIR` the test's folder (for fixture
/// files beside it) and `${NAME}` the environment variable `NAME`, which must be set: a test
/// that needs a file the machine provides (a font) fails, instead of running without it.
fn expand_args(
    args: &[String],
    temp: &str,
    dir: &str,
    env: &dyn Fn(&str) -> Option<String>,
) -> Result<Vec<String>, String> {
    let mut expanded = Vec::new();

    for arg in args {
        let mut text = arg.replace("$TEMP", temp).replace("$DIR", dir);

        while let Some(start) = text.find("${") {
            let Some(length) = text[start..].find('}') else {
                return Err(format!(
                    "the argument `{arg}` has `${{` without a closing `}}`"
                ));
            };
            let name = &text[start + 2..start + length];
            let Some(value) = env(name).filter(|v| !v.is_empty()) else {
                return Err(format!(
                    "the argument `{arg}` needs the environment variable {name}, which is not set"
                ));
            };

            text.replace_range(start..=start + length, &value);
        }

        expanded.push(text);
    }

    Ok(expanded)
}

/// Builds and runs the golden test `source`; `options` set its arguments and the warning it
/// must produce.
#[allow(clippy::too_many_arguments)]
pub(super) fn golden(
    source: &Path,
    options: &TestOptions,
    root_flags: &[String],
    toolchain: &Toolchain,
    work: &Path,
    cwd: &Path,
    timeout: Option<Duration>,
    accept: bool,
    live: &mut dyn FnMut(Live),
) -> Checked {
    if options.check_only || toolchain.check_only {
        return check_only(source, root_flags, toolchain, cwd);
    }

    let mut checked = Checked::default();
    let Built {
        mut invocation,
        stdout: build_stdout,
        stderr: build_stderr,
        compile_time_only,
    } = match built(
        source,
        root_flags,
        toolchain,
        work,
        cwd,
        options.warns.is_none(),
    ) {
        Ok(built) => built,
        Err(problem) => {
            checked.problems.push(problem);
            return checked;
        }
    };
    let build_output = format!(
        "{}{}",
        String::from_utf8_lossy(&build_stderr),
        String::from_utf8_lossy(&build_stdout)
    );

    if !options.args.is_empty() {
        let temp = work.join("temp");
        let _ = std::fs::create_dir_all(&temp);
        let temp = temp.to_string_lossy().into_owned();
        let dir = source.parent().unwrap_or(Path::new("."));
        let dir = std::path::absolute(dir).unwrap_or_else(|_| dir.to_path_buf());
        let args = match expand_args(
            &options.args,
            &temp,
            &dir.to_string_lossy(),
            &jacket_config::env::for_program,
        ) {
            Ok(args) => args,
            Err(problem) => {
                checked.problems.push(problem);
                return checked;
            }
        };

        if toolchain.interp {
            invocation.args.push("--".into());
        }

        invocation.args.extend(args);
    }

    let outcome = if compile_time_only {
        exec::Outcome {
            ending: Ending::Exited(0),
            preamble_stdout: build_stdout,
            preamble: build_stderr,
            tests: Vec::new(),
            missing: Vec::new(),
        }
    } else {
        run_plain(&invocation, timeout, live)
    };

    checked.stdout = outcome.preamble_stdout;
    checked.stderr = outcome.preamble;

    if outcome.ending == Ending::TimedOut {
        checked.timed_out = true;
        checked.problems.push("it timed out".into());

        return checked;
    }

    checked.crashed = outcome.ending.crashed();
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

    if let Some(warning) = &options.warns {
        let seen = format!("{build_output}{}", normalized(&checked.stderr));

        if !seen.contains(warning.as_str()) {
            checked.problems.push(format!(
                "no warning contains `{warning}`; the compiler said:\n{seen}"
            ));
        }
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

/// `text` with `dir` written `$DIR`, `stdlib` `$STDLIB`, `project` (the folder with
/// `jacket.toml`) `$ROOT` and the `imports` folders `$IMPORT1`, `$IMPORT2`, ... in the order given,
/// separators as `/`. Paths outside all of them stay as printed.
pub(super) fn normalize_paths(
    text: &str,
    dir: &Path,
    stdlib: &Path,
    project: &Path,
    imports: &[PathBuf],
) -> String {
    let mut text = text.replace("\r\n", "\n");
    let imports: Vec<(&Path, String)> = imports
        .iter()
        .enumerate()
        .filter(|(_, path)| path.is_absolute())
        .map(|(i, path)| (path.as_path(), format!("$IMPORT{}", i + 1)))
        .collect();
    let named = [(stdlib, "$STDLIB"), (dir, "$DIR"), (project, "$ROOT")]
        .into_iter()
        .map(|(path, name)| (path, name.to_string()))
        .chain(imports);

    for (path, name) in named {
        let name = name.as_str();
        // The compiler prints resolved paths, so `a/../stdlib` must match its canonical form too.
        let real = path.canonicalize().ok();

        for path in [Some(path), real.as_deref()].into_iter().flatten() {
            let plain = path.to_string_lossy().into_owned();
            let plain = plain.strip_prefix(r"\\?\").unwrap_or(&plain).to_string();
            let forward = jacket_config::to_slash(&plain);

            for form in [plain, forward] {
                if !form.is_empty() {
                    text = replace_path(&text, &form, name);
                }
            }
        }
    }

    // Paths after `$DIR`, `$STDLIB` and `$ROOT` keep their own separators on Windows; make them alike.
    text.lines()
        .map(|line| {
            if ["$DIR\\", "$STDLIB\\", "$ROOT\\", "$IMPORT"]
                .iter()
                .any(|p| line.contains(p))
            {
                return jacket_config::to_slash(line);
            }

            // A relative location such as `..\..\layout.jai:67:13:` gets `/` in its path only.
            match location_end(line) {
                Some(end) if line[..end].contains('\\') => {
                    format!("{}{}", jacket_config::to_slash(&line[..end]), &line[end..])
                }
                _ => line.to_string(),
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

/// `text` with each whole-path occurrence of `path` replaced by `name`: `/s` is not found in
/// `/src`, only where a path ends (a separator, the end, or punctuation) and starts (not inside a
/// longer name).
fn replace_path(text: &str, path: &str, name: &str) -> String {
    let name_char = |c: char| c.is_alphanumeric() || matches!(c, '_' | '-' | '.');
    let mut out = String::with_capacity(text.len());
    let mut last = 0;

    for (at, _) in text.match_indices(path) {
        let whole = !text[..at].chars().next_back().is_some_and(name_char)
            && !text[at + path.len()..]
                .chars()
                .next()
                .is_some_and(name_char);

        if whole && at >= last {
            out.push_str(&text[last..at]);
            out.push_str(name);
            last = at + path.len();
        }
    }

    out.push_str(&text[last..]);
    out
}

/// Where the path of the location starting `line` ends (`path:line:`): the first `:` followed by a
/// digit, with no space before it. `None` when the line does not start with a location.
fn location_end(line: &str) -> Option<usize> {
    let end = line
        .char_indices()
        .find(|&(i, c)| c == ':' && line[i + 1..].starts_with(|d: char| d.is_ascii_digit()))?
        .0;
    (end > 0 && !line[..end].contains(' ')).then_some(end)
}

/// The folders compile-fail output names as `$STDLIB` and `$ROOT`.
pub(super) struct Folders<'a> {
    pub stdlib: &'a Path,
    pub project: &'a Path,
}

/// Type-checks the compile-fail test `source`: it must fail, printing its `.stderr` (or, with
/// `compile_fail(text)`, diagnostics that contain the text). A missing `.stderr` is written from
/// what the compiler printed when `create_missing` (`--create-missing`), else it fails.
pub(super) fn compile_fail(
    source: &Path,
    options: &TestOptions,
    root_flags: &[String],
    toolchain: &Toolchain,
    folders: &Folders,
    accept: bool,
    create_missing: bool,
) -> Checked {
    // From the test's folder, diagnostics name the file without a path.
    let dir = source.parent().unwrap_or(Path::new("."));
    let mut checked = Checked::default();
    let output = toolchain
        .jaic_command()
        .arg(source)
        .arg("-check")
        .arg("-quiet")
        .arg("-no_color")
        .args(&toolchain.flags)
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

    // The import folders the build named (`-import_dir`), each written `$IMPORTn`.
    let flags: Vec<String> = toolchain.flags.iter().chain(root_flags).cloned().collect();
    let imports = jaic_cli_args::arguments::import_dirs(&flags);
    let got = normalize_paths(
        &compile_output(&output),
        dir,
        folders.stdlib,
        folders.project,
        &imports,
    );
    let expected = source.with_extension("stderr");

    if let Some(part) = &options.diagnostic_contains {
        if !got.contains(part.as_str()) {
            checked
                .problems
                .push(format!("the diagnostics do not contain `{part}`:\n{got}"));
        }
    } else if expected.is_file() || accept {
        expect_text(
            &expected,
            &got,
            "the diagnostic output",
            accept,
            &mut checked,
        );
    } else if create_missing && std::fs::write(&expected, &got).is_ok() {
        checked.created.push(expected);
    } else {
        checked.problems.push(format!(
            "{} is missing; `jacket test --create-missing` writes it from these diagnostics:\n{got}",
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
            normalize_paths(
                text,
                Path::new("/r/tests/compile_fail"),
                Path::new("/s"),
                Path::new("/r"),
                &[]
            ),
            "$DIR/bad.jai:3:1: error: x\n$STDLIB/Basic/module.jai:9: note\n"
        );
        assert_eq!(
            normalize_paths(
                "C:\\r\\t\\bad.jai:1: e\r\n",
                Path::new("C:\\r\\t"),
                Path::new("C:\\s"),
                Path::new("C:\\r"),
                &[]
            ),
            "$DIR/bad.jai:1: e\n"
        );
        assert_eq!(
            normalize_paths(
                "..\\..\\layout.jai:67:13: error: a\\b\n",
                Path::new("C:\\r\\t"),
                Path::new("C:\\s"),
                Path::new("C:\\r"),
                &[]
            ),
            "../../layout.jai:67:13: error: a\\b\n"
        );
        // Any file a location names, not only Jai sources (a metaprogram reports on its inputs).
        assert_eq!(
            normalize_paths(
                "..\\fixtures/clash.proto:3:15: error: a\\b\nnote: C:\\x 1:2\n",
                Path::new("C:\\r\\t"),
                Path::new("C:\\s"),
                Path::new("C:\\r"),
                &[]
            ),
            "../fixtures/clash.proto:3:15: error: a\\b\nnote: C:\\x 1:2\n"
        );
    }

    #[test]
    fn project_files_outside_the_test_folder_are_relative_to_root() {
        let dir = Path::new("/r/tests/compile_fail");
        let text = "/r/src/lib.jai:4:2: error: x\n/r/tests/other/a.jai:1:1: note\n/r/tests/compile_fail/bad.jai:3:1: e\n/s/Basic/module.jai:9: note\n/elsewhere/f.jai:1:1: e\n";

        assert_eq!(
            normalize_paths(text, dir, Path::new("/s"), Path::new("/r"), &[]),
            "$ROOT/src/lib.jai:4:2: error: x\n$ROOT/tests/other/a.jai:1:1: note\n$DIR/bad.jai:3:1: e\n$STDLIB/Basic/module.jai:9: note\n/elsewhere/f.jai:1:1: e\n"
        );
        // The stdlib inside the project stays `$STDLIB`, and `\` after `$ROOT` becomes `/`.
        assert_eq!(
            normalize_paths(
                "C:\\r\\stdlib\\B\\m.jai:1: e\nC:\\r\\src\\lib.jai:2: e\n",
                Path::new("C:\\r\\t"),
                Path::new("C:\\r\\stdlib"),
                Path::new("C:\\r"),
                &[]
            ),
            "$STDLIB/B/m.jai:1: e\n$ROOT/src/lib.jai:2: e\n"
        );
    }

    #[test]
    fn import_folders_are_numbered_in_flag_order() {
        let dir = Path::new("/r/tests/compile_fail");
        let imports = [PathBuf::from("/libs/a"), PathBuf::from("/libs/b")];
        let text =
            "/libs/b/x.jai:1:1: error: x\n/libs/a/y/z.jai:2:2: note\n/libs/ab/q.jai:3:3: e\n";

        assert_eq!(
            normalize_paths(text, dir, Path::new("/s"), Path::new("/r"), &imports),
            "$IMPORT2/x.jai:1:1: error: x\n$IMPORT1/y/z.jai:2:2: note\n/libs/ab/q.jai:3:3: e\n"
        );
    }

    #[cfg(unix)]
    #[test]
    fn the_library_is_found_by_any_spelling_of_its_path() {
        let base = std::env::temp_dir().join(format!("jacket-norm-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let real = base.join("real/stdlib");

        std::fs::create_dir_all(real.join("Basic")).unwrap();
        std::fs::create_dir_all(base.join("real/case")).unwrap();
        std::os::unix::fs::symlink(base.join("real"), base.join("link")).unwrap();
        let canonical = real.canonicalize().unwrap();
        let dir = base.join("real/case").canonicalize().unwrap();
        let printed = format!("{}/Basic/module.jai:9: note\n", canonical.display());
        let want = "$STDLIB/Basic/module.jai:9: note\n";
        // The library as given: canonical, through a symlink, or with a `..`; and the compiler
        // printing the symlink's spelling for a library named through it.
        let linked = base.join("link/stdlib");
        let dotted = base.join("real/case/../stdlib");

        for stdlib in [&canonical, &linked, &dotted] {
            assert_eq!(normalize_paths(&printed, &dir, stdlib, &base, &[]), want);
        }

        let through_link = format!("{}/Basic/module.jai:9: note\n", linked.display());

        assert_eq!(
            normalize_paths(&through_link, &dir, &linked, &base, &[]),
            want
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn arguments_expand_folders_and_environment_variables() {
        let env = |name: &str| (name == "FONT").then(|| "/fonts/a.ttf".to_string());
        let args = [
            "$TEMP/data".to_string(),
            "$DIR/x.bmp".into(),
            "${FONT}".into(),
        ];

        assert_eq!(
            expand_args(&args, "/t", "/d", &env).unwrap(),
            ["/t/data", "/d/x.bmp", "/fonts/a.ttf"]
        );
        let missing = expand_args(&["${NOPE}".to_string()], "/t", "/d", &env).unwrap_err();

        assert!(missing.contains("NOPE"), "{missing}");
        assert!(expand_args(&["${NOPE".to_string()], "/t", "/d", &env).is_err());
    }

    #[test]
    fn any_alternative_output_passes() {
        let dir = std::env::temp_dir().join(format!("jacket-alt-{}", std::process::id()));

        std::fs::create_dir_all(&dir).unwrap();
        let expected = dir.join("t.stdout");

        std::fs::write(&expected, "opened\n").unwrap();
        std::fs::write(dir.join("t.stdout.2"), "unavailable\n").unwrap();

        for (got, passes) in [
            ("opened\n", true),
            ("unavailable\n", true),
            ("other\n", false),
        ] {
            let mut checked = Checked::default();

            expect_text(&expected, got, "standard output", false, &mut checked);
            assert_eq!(checked.problems.is_empty(), passes, "{got}");
        }

        // Accepting keeps the outputs that exist and adds nothing for them.
        let mut checked = Checked::default();

        expect_text(
            &expected,
            "unavailable\n",
            "standard output",
            true,
            &mut checked,
        );
        assert!(checked.accepted.is_empty());
        assert_eq!(std::fs::read_to_string(&expected).unwrap(), "opened\n");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
