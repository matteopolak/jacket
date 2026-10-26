//! Building a root's unit tests: one program from all of its unit test files.
//!
//! jacket writes a root file that `#load`s them and compiles it with the Testing module's
//! Discovery plugin (`+Extensions/Testing/Discovery`), which writes the manifest of tests and
//! adds the table of tests and the entry point. With `--interp`, plugins do not run, so the
//! plugin runs under `-check` instead and also writes the code it adds to a file; a second root
//! file loads that and names the entry point itself, and each worker runs it with `-interp`.
//! A wasm build goes the same way (compile-time code of a wasm target cannot write files), so
//! its tests are found as the host sees them: keep tests per platform with `@only`, not `#if`.
use super::exec::Invocation;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const PLUGIN: &str = "+Extensions/Testing/Discovery";
const ENTRY_POINT: &str = "__jacket_test_main";

/// How to compile: the compiler, the flags every compile gets, and how tests run.
#[derive(Clone, Debug)]
pub struct Toolchain {
    pub jaic: PathBuf,
    /// `-os`, `-cpu`, the user's extra flags.
    pub flags: Vec<String>,
    pub interp: bool,
    /// Starts test programs built for another target (`node tools/wasi_run.mjs`).
    pub runner: Vec<String>,
    /// The target OS (`wasm` programs are `.wasm` files).
    pub os: String,
}

/// One `@test` or `@bench` the plugin found.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub kind: String,
    pub name: String,
    pub file: PathBuf,
    pub line: u32,
    pub notes: Vec<String>,
}

/// A built unit-test program.
#[derive(Debug)]
pub struct Built {
    pub entries: Vec<Entry>,
    /// The executable, or (with `--interp`) the root file `jaic -interp` runs.
    pub program: PathBuf,
    /// `(test, case)` per case of the `@cases` tests, found when only listing.
    pub cases: Vec<(String, String)>,
}

/// Flags of a root: its import folder when it is a library.
pub fn root_flags(dir: &Path, library: bool) -> Vec<String> {
    match dir.parent() {
        Some(parent) if library => vec!["-import_dir".into(), path_text(parent)],
        _ => Vec::new(),
    }
}

fn path_text(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

/// `path` as a Jai string literal, with `/` separators.
fn literal(path: &Path) -> String {
    let text = jacket_config::slashed(path);
    format!("\"{}\"", text.replace('"', "\\\""))
}

pub fn parse_manifest(text: &str) -> Vec<Entry> {
    text.lines()
        .filter_map(|line| {
            let mut fields = line.split('\t');
            let kind = fields.next()?.to_string();
            let name = fields.next()?.to_string();
            let file = PathBuf::from(fields.next()?);
            let line = fields.next()?.parse().ok()?;
            Some(Entry {
                kind,
                name,
                file,
                line,
                notes: fields.map(String::from).collect(),
            })
        })
        .collect()
}

/// `test<TAB>case` per line.
pub fn parse_cases(text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter_map(|line| line.split_once('\t'))
        .map(|(test, case)| (test.to_string(), case.to_string()))
        .collect()
}

/// The output of a compile that failed, for the user.
pub fn compile_output(output: &std::process::Output) -> String {
    let mut text = String::from_utf8_lossy(&output.stderr).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stdout));
    text
}

/// Builds the unit tests of `files` in `work` (a fresh folder). `list` only type-checks: the
/// manifest is all that is needed.
pub fn unit(
    files: &[PathBuf],
    root_flags: &[String],
    work: &Path,
    toolchain: &Toolchain,
    list: bool,
) -> Result<Built, String> {
    std::fs::create_dir_all(work).map_err(|e| format!("cannot create {}: {e}", work.display()))?;
    let mut loads = String::from("// Written by `jacket test`: every unit test file of a root.\n");
    for file in files {
        loads.push_str(&format!("#load {};\n", literal(file)));
    }
    let root = work.join("tests.jai");
    let manifest = work.join("manifest.tsv");
    let registry = work.join("registry.jai");
    write(&root, &loads)?;
    // Compile-time code of a wasm target runs in the interpreter's sandbox, whose files are not
    // the disk's: the plugin runs in a check for the host instead, and the program is built from
    // the code it wrote.
    let wasm = toolchain.os == "wasm";
    let from_registry = toolchain.interp || wasm;
    let discovery_flags = if wasm {
        host_flags(&toolchain.flags)
    } else {
        toolchain.flags.clone()
    };
    let discovery_flags = if list || from_registry {
        unbuilt_flags(&discovery_flags)
    } else {
        discovery_flags
    };
    let mut command = Command::new(&toolchain.jaic);
    command
        .arg(&root)
        .arg("-quiet")
        .args(&discovery_flags)
        .args(root_flags);
    let exe = work.join(if wasm {
        "tests.wasm"
    } else {
        "tests"
    });
    let report = work.join("exe.txt");
    let case_file = work.join("cases.tsv");
    if list || from_registry {
        command.arg("-check");
    } else {
        command
            .arg("-exe")
            .arg(&exe)
            .env("JAIC_REPORT_EXE", &report);
    }
    command
        .arg(PLUGIN)
        .arg("-manifest")
        .arg(&manifest)
        .arg("-registry")
        .arg(&registry)
        .current_dir(work);
    if list {
        command.arg("-cases").arg(&case_file);
    }
    run_jaic(&mut command, toolchain)?;
    let entries = parse_manifest(&std::fs::read_to_string(&manifest).unwrap_or_default());
    if list {
        return Ok(Built {
            entries,
            program: PathBuf::new(),
            cases: parse_cases(&std::fs::read_to_string(&case_file).unwrap_or_default()),
        });
    }
    let program = if from_registry {
        let program_root = work.join("program.jai");
        let source = format!(
            "{loads}#load {};\n#run {{\n    C :: #import \"Compiler\";\n    options := C.get_build_options();\n    options.entry_point_name = \"{ENTRY_POINT}\";\n    C.set_build_options(options);\n}}\n",
            literal(&registry)
        );
        write(&program_root, &source)?;
        if toolchain.interp {
            if wasm {
                // The discovery check was for the host: check the program for its target too,
                // so a compile error is reported once and not by every test.
                let mut command = Command::new(&toolchain.jaic);
                command
                    .arg(&program_root)
                    .arg("-check")
                    .arg("-quiet")
                    .args(unbuilt_flags(&toolchain.flags))
                    .args(root_flags)
                    .current_dir(work);
                run_jaic(&mut command, toolchain)?;
            }
            program_root
        } else {
            let mut command = Command::new(&toolchain.jaic);
            command
                .arg(&program_root)
                .arg("-quiet")
                .args(&toolchain.flags)
                .args(root_flags)
                .arg("-exe")
                .arg(&exe)
                .env("JAIC_REPORT_EXE", &report)
                .current_dir(work);
            run_jaic(&mut command, toolchain)?;
            reported_exe(&report, exe)
        }
    } else {
        reported_exe(&report, exe)
    };
    Ok(Built {
        entries,
        program,
        cases: Vec::new(),
    })
}

fn run_jaic(command: &mut Command, toolchain: &Toolchain) -> Result<(), String> {
    let output = command
        .output()
        .map_err(|e| format!("could not start `{}`: {e}", toolchain.jaic.display()))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(compile_output(&output))
    }
}

/// The executable jaic reported writing, or `fallback`.
fn reported_exe(report: &Path, fallback: PathBuf) -> PathBuf {
    std::fs::read_to_string(report)
        .ok()
        .and_then(|t| t.lines().last().map(PathBuf::from))
        .filter(|p| p.is_file())
        .unwrap_or(fallback)
}

/// `flags` without the target (`-os X`, `-cpu X`).
fn host_flags(flags: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut skip = false;
    for flag in flags {
        if skip {
            skip = false;
        } else if flag == "-os" || flag == "-cpu" {
            skip = true;
        } else {
            out.push(flag.clone());
        }
    }
    out
}

/// jaic's options that only a build takes (`-check` and `-interp` reject them), and whether
/// each takes a value.
const BUILD_ONLY: [(&str, bool); 17] = [
    ("-exe", true),
    ("-output_path", true),
    ("-emit_ir", true),
    ("-sanitize", true),
    ("-o", false),
    ("-optimized", false),
    ("-od", false),
    ("-optimized_debug", false),
    ("-very_debug", false),
    ("-O0", false),
    ("-O1", false),
    ("-O2", false),
    ("-O3", false),
    ("-no_inline", false),
    ("-no_split", false),
    ("-no_debug_info", false),
    ("-coverage", false),
];

/// `flags` for a `-check` or `-interp` run: without the options only a build takes, so
/// `jacket test -- -O2` still lists, checks compile-fail tests and runs interpreted.
pub fn unbuilt_flags(flags: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut flags = flags.iter();
    while let Some(flag) = flags.next() {
        match BUILD_ONLY.iter().find(|(name, _)| name == flag) {
            Some((_, true)) => {
                flags.next();
            }
            Some((_, false)) => {}
            None => out.push(flag.clone()),
        }
    }
    out
}

fn write(path: &Path, text: &str) -> Result<(), String> {
    std::fs::write(path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))
}

/// The command that runs `names` of a built program, each with its temporary folder under
/// `temp` (`temp/0`, `temp/1`, ...).
pub fn invocation(
    built: &Built,
    root_flags: &[String],
    toolchain: &Toolchain,
    names: &[String],
    temp: &Path,
    cwd: &Path,
) -> Invocation {
    let mut tail = vec!["--run".to_string()];
    tail.extend(names.iter().cloned());
    tail.extend(["--temp".to_string(), path_text(temp)]);
    let (program, mut args) = if toolchain.interp {
        let mut args = vec![path_text(&built.program), "-interp".into(), "-quiet".into()];
        args.extend(unbuilt_flags(&toolchain.flags));
        args.extend(root_flags.iter().cloned());
        args.push("--".into());
        (toolchain.jaic.clone(), args)
    } else {
        program_and_args(&built.program, &toolchain.runner)
    };
    args.append(&mut tail);
    Invocation {
        program,
        args,
        cwd: cwd.to_path_buf(),
    }
}

/// `exe`, or the runner with `exe` as its first argument.
pub fn program_and_args(exe: &Path, runner: &[String]) -> (PathBuf, Vec<String>) {
    match runner.split_first() {
        Some((first, rest)) => {
            let mut args = rest.to_vec();
            args.push(path_text(exe));
            (PathBuf::from(first), args)
        }
        None => (exe.to_path_buf(), Vec::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checks_leave_out_build_only_flags() {
        let flags: Vec<String> = ["-O2", "-os", "wasm", "-exe", "a", "-import_dir", "x"]
            .map(String::from)
            .into();
        assert_eq!(unbuilt_flags(&flags), ["-os", "wasm", "-import_dir", "x"]);
    }

    #[test]
    fn manifest_lines_are_entries() {
        let entries = parse_manifest(
            "test\tadds\t/a/t.jai\t4\ttimeout(5)\tserial\nbench\tb\t/a/t.jai\t9\n\n",
        );
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].notes, ["timeout(5)", "serial"]);
        assert_eq!((entries[1].kind.as_str(), entries[1].line), ("bench", 9));
    }

    #[test]
    fn case_lines_pair_a_test_with_a_case() {
        let cases = parse_cases("lengths\t\"\",0\nlengths\ta,1\nnoise\ncolors\tRED\n");
        assert_eq!(
            cases,
            [
                ("lengths".to_string(), "\"\",0".to_string()),
                ("lengths".to_string(), "a,1".to_string()),
                ("colors".to_string(), "RED".to_string()),
            ]
        );
    }

    #[test]
    fn a_runner_starts_the_program() {
        let runner = ["node".to_string(), "wasi_run.mjs".to_string()];
        let (program, args) = program_and_args(Path::new("/b/t.wasm"), &runner);
        assert_eq!(program, PathBuf::from("node"));
        assert_eq!(args, ["wasi_run.mjs", "/b/t.wasm"]);
    }
}
