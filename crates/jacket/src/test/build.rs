//! Building a root's unit tests: one program from all of its unit test files.
//!
//! jacket writes a root file that `#load`s them and compiles it with the Testing module's
//! Discovery plugin (`+Extensions/Testing/Discovery`), which writes the manifest of tests and
//! adds the table of tests and the entry point. One compile does both, for every mode:
//!
//! - a native build writes the manifest to a file and the program to an executable;
//! - a wasm build prints the manifest on standard output (`-print-manifest`), since compile-time
//!   code of a wasm target cannot write files, and writes the module;
//! - `--interp` has no program to build, and every process compiles by itself. A plugin makes
//!   the compiler check all of the root, which costs about three times as much as running it
//!   does, so the plugin also writes the added code to a file, and a second root file loads the
//!   tests and that code and names the entry point in a `#run`: `program.jai`, which the
//!   processes run. The process that will run the first tests is the one with the plugin: it is
//!   started at once and compiles while jacket reads the manifest, then waits for a file with
//!   its arguments (`--args-from`), which jacket writes when it hands the process its first
//!   tests.
use super::exec::{Invocation, kill_tree};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::Mutex;
use std::time::Duration;

pub const PLUGIN: &str = "+Extensions/Testing/Discovery";

/// The line of standard output that carries a manifest line (`-print-manifest`).
const ENTRY_POINT: &str = "__jacket_test_main";

const MANIFEST_LINE: &str = "jacket-manifest: ";

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
    /// Build the unit-test program with `-coverage`.
    pub coverage: bool,
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
    /// With `--interp`, the process that compiled while the manifest was read, waiting for the
    /// tests it runs. The first run of the root takes it.
    pub warm: Mutex<Option<Warm>>,
}

/// A test program that is already compiling and waits for its arguments.
#[derive(Debug)]
pub struct Warm {
    child: Option<Child>,
    plan: PathBuf,
}

impl Drop for Warm {
    fn drop(&mut self) {
        // Nothing took it: stop it and what it started.
        if let Some(child) = self.child.as_mut() {
            kill_tree(child);
            let _ = child.wait();
        }
    }
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
    write(&root, &loads)?;
    // Compile-time code of a wasm target runs in the interpreter's sandbox, whose files are not
    // the disk's: its manifest goes to standard output.
    let wasm = targets_wasm(toolchain);
    let built = |entries: Vec<Entry>, program: PathBuf| Built {
        entries,
        program,
        cases: Vec::new(),
        warm: Mutex::new(None),
    };
    let mut command = Command::new(&toolchain.jaic);
    command.current_dir(work);
    if list {
        // Finds the tests (and the cases of `@cases` tests, which a wasm target could not write
        // out): the host's view of them.
        let case_file = work.join("cases.tsv");
        let flags = if wasm {
            host_flags(&toolchain.flags)
        } else {
            toolchain.flags.clone()
        };
        command
            .arg(&root)
            .arg("-quiet")
            .args(&flags)
            .args(root_flags)
            .args(["-check", PLUGIN, "-manifest"])
            .arg(&manifest)
            .arg("-cases")
            .arg(&case_file)
            .arg("-no-compile-time");
        run_jaic(&mut command, toolchain)?;
        return Ok(Built {
            cases: parse_cases(&std::fs::read_to_string(&case_file).unwrap_or_default()),
            ..built(
                parse_manifest(&std::fs::read_to_string(&manifest).unwrap_or_default()),
                PathBuf::new(),
            )
        });
    }
    if toolchain.interp {
        return interpreted(&loads, &root, work, root_flags, toolchain, built);
    }
    let exe = work.join(if wasm {
        "tests.wasm"
    } else {
        "tests"
    });
    let report = work.join("exe.txt");
    command
        .arg(&root)
        .arg("-quiet")
        .args(&toolchain.flags)
        .args(root_flags)
        .arg("-exe")
        .arg(&exe)
        .env("JAIC_REPORT_EXE", &report);
    if toolchain.coverage {
        command.arg("-coverage");
    }
    command.arg(PLUGIN);
    let entries = if wasm {
        parse_manifest(&manifest_lines(&run_jaic_output(
            command.arg("-print-manifest"),
            toolchain,
        )?))
    } else {
        run_jaic(command.arg("-manifest").arg(&manifest), toolchain)?;
        parse_manifest(&std::fs::read_to_string(&manifest).unwrap_or_default())
    };
    Ok(built(entries, reported_exe(&report, exe)))
}

/// The unit tests of a root under `--interp`. Without a program to build, jacket needs the
/// manifest to plan the run: the process that will run the first tests is started now and
/// writes it as soon as the plugin has found the tests.
fn interpreted(
    loads: &str,
    root: &Path,
    work: &Path,
    root_flags: &[String],
    toolchain: &Toolchain,
    built: impl Fn(Vec<Entry>, PathBuf) -> Built,
) -> Result<Built, String> {
    let manifest = work.join("manifest.tsv");
    let registry = work.join("registry.jai");
    let program = work.join("program.jai");
    let source = format!(
        "{loads}#load {};\n#run {{\n    C :: #import \"Compiler\";\n    options := C.get_build_options();\n    options.entry_point_name = \"{ENTRY_POINT}\";\n    C.set_build_options(options);\n}}\n",
        literal(&registry)
    );
    let wasm = targets_wasm(toolchain);
    if wasm || toolchain.coverage {
        // Every run gets its own coverage folder in its environment, which a process started now
        // would not have; compile-time code of a wasm target cannot write the files. A check for
        // the host finds the tests and writes the code, and each run compiles by itself.
        let flags = if wasm {
            host_flags(&toolchain.flags)
        } else {
            toolchain.flags.clone()
        };
        let mut command = Command::new(&toolchain.jaic);
        command
            .arg(root)
            .arg("-quiet")
            .args(&flags)
            .args(root_flags)
            .args(["-check", PLUGIN, "-no-compile-time", "-manifest"])
            .arg(&manifest)
            .arg("-registry")
            .arg(&registry)
            .current_dir(work);
        run_jaic(&mut command, toolchain)?;
        write(&program, &source)?;
        if wasm {
            // The check was for the host: check the program for its target too, so a compile
            // error is reported once and not by every test.
            let mut command = Command::new(&toolchain.jaic);
            command
                .arg(&program)
                .arg("-check")
                .arg("-quiet")
                .args(&toolchain.flags)
                .args(root_flags)
                .current_dir(work);
            run_jaic(&mut command, toolchain)?;
        }
        let entries = parse_manifest(&std::fs::read_to_string(&manifest).unwrap_or_default());
        return Ok(built(entries, program));
    }
    let plan = work.join("plan.txt");
    let mut args = vec![path_text(root), "-interp".into(), "-quiet".into()];
    args.extend(toolchain.flags.iter().cloned());
    args.extend(root_flags.iter().cloned());
    args.extend([PLUGIN.into(), "-manifest".into(), path_text(&manifest)]);
    args.extend(["-registry".into(), path_text(&registry)]);
    args.extend(["--".into(), "--args-from".into(), path_text(&plan)]);
    let mut child = super::exec::spawn(&Invocation {
        program: toolchain.jaic.clone(),
        args,
        cwd: work.to_path_buf(),
        env: Vec::new(),
    })
    .map_err(|e| format!("could not start `{}`: {e}", toolchain.jaic.display()))?;
    // The plugin moves the manifest into place whole, after the code it adds, once it has found
    // the tests; a process that ends first did not compile.
    let entries = loop {
        if let Ok(text) = std::fs::read_to_string(&manifest) {
            break parse_manifest(&text);
        }
        match child.try_wait() {
            Ok(Some(_)) => {
                let output = child.wait_with_output().map_err(|e| e.to_string())?;
                return Err(compile_output(&output));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(3)),
            Err(e) => return Err(e.to_string()),
        }
    };
    write(&program, &source)?;
    let result = built(entries, program);
    *result.warm.lock().expect("lock") = Some(Warm {
        child: Some(child),
        plan,
    });
    Ok(result)
}

/// The manifest lines a compile printed (`-print-manifest`).
fn manifest_lines(output: &str) -> String {
    output
        .lines()
        .filter_map(|line| line.strip_prefix(MANIFEST_LINE))
        .fold(String::new(), |mut text, line| {
            text.push_str(line);
            text.push('\n');
            text
        })
}

fn run_jaic(command: &mut Command, toolchain: &Toolchain) -> Result<(), String> {
    run_jaic_output(command, toolchain).map(|_| ())
}

/// Runs a compile and returns what it printed on standard output.
fn run_jaic_output(command: &mut Command, toolchain: &Toolchain) -> Result<String, String> {
    let output = command
        .output()
        .map_err(|e| format!("could not start `{}`: {e}", toolchain.jaic.display()))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
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

fn write(path: &Path, text: &str) -> Result<(), String> {
    std::fs::write(path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))
}

/// What every run of a test program is told besides the tests to run.
#[derive(Clone, Copy, Debug)]
pub struct RunFlags {
    /// The seed property tests start from.
    pub seed: u64,
    /// Snapshots that differ or are new replace the stored ones.
    pub accept: bool,
}

/// Whether the tests are built for wasm: `--os wasm`, or jaic flags that say so.
fn targets_wasm(toolchain: &Toolchain) -> bool {
    toolchain.os == "wasm"
        || toolchain.flags.windows(2).any(|pair| {
            (pair[0] == "-os" && pair[1] == "wasm")
                || (pair[0] == "-target" && pair[1].starts_with("wasm"))
        })
}

/// The arguments that follow the program's own: the seed, `--accept`, and `--run` with `names`,
/// each with its temporary folder under `temp` (`temp/0`, `temp/1`, ...).
fn run_arguments(names: &[String], temp: &Path, run: &RunFlags) -> Vec<String> {
    let mut tail = vec!["--seed".to_string(), run.seed.to_string()];
    if run.accept {
        tail.push("--accept".to_string());
    }
    tail.push("--run".to_string());
    tail.extend(names.iter().cloned());
    tail.extend(["--temp".to_string(), path_text(temp)]);
    tail
}

/// The command that runs `names` of a built program.
pub fn invocation(
    built: &Built,
    root_flags: &[String],
    toolchain: &Toolchain,
    names: &[String],
    temp: &Path,
    cwd: &Path,
    run: &RunFlags,
) -> Invocation {
    let (program, mut args) = if toolchain.interp {
        let mut args = vec![path_text(&built.program), "-interp".into(), "-quiet".into()];
        args.extend(toolchain.flags.iter().cloned());
        args.extend(root_flags.iter().cloned());
        args.push("--".into());
        (toolchain.jaic.clone(), args)
    } else {
        program_and_args(&built.program, &toolchain.runner)
    };
    args.append(&mut run_arguments(names, temp, run));
    Invocation {
        program,
        args,
        cwd: cwd.to_path_buf(),
        env: Vec::new(),
    }
}

/// With `--interp`, the process that compiled the root while jacket planned, told to run `names`
/// (once; later runs compile by themselves). `None` when there is no such process.
pub fn warm_start(built: &Built, names: &[String], temp: &Path, run: &RunFlags) -> Option<Child> {
    let mut warm = built.warm.lock().ok()?.take()?;
    let mut text = run_arguments(names, temp, run).join("\n");
    text.push('\n');
    // Moved into place whole, for a process that waits for the file.
    let partial = warm.plan.with_extension("partial");
    std::fs::write(&partial, text).ok()?;
    std::fs::rename(&partial, &warm.plan).ok()?;
    warm.child.take()
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
    fn the_seed_and_accept_reach_the_program() {
        let built = Built {
            entries: Vec::new(),
            program: PathBuf::from("/b/tests"),
            cases: Vec::new(),
            warm: Mutex::new(None),
        };
        let toolchain = Toolchain {
            jaic: PathBuf::from("jaic"),
            flags: Vec::new(),
            interp: false,
            runner: Vec::new(),
            os: "linux".into(),
            coverage: false,
        };
        let names = ["a".to_string()];
        let run = |accept| {
            invocation(
                &built,
                &[],
                &toolchain,
                &names,
                Path::new("/t"),
                Path::new("/r"),
                &RunFlags {
                    seed: 42,
                    accept,
                },
            )
            .args
        };
        assert_eq!(run(false), ["--seed", "42", "--run", "a", "--temp", "/t"]);
        assert_eq!(
            run(true),
            ["--seed", "42", "--accept", "--run", "a", "--temp", "/t"]
        );
    }

    #[test]
    fn printed_manifest_lines_are_picked_out_of_the_output() {
        let output = "noise\njacket-manifest: test\tadds\t/a/t.jai\t4\nmore noise\njacket-manifest: bench\tb\t/a/t.jai\t9\t\n";
        let entries = parse_manifest(&manifest_lines(output));
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "adds");
        assert_eq!(entries[1].kind, "bench");
        assert!(manifest_lines("nothing here\n").is_empty());
    }

    #[test]
    fn a_wasm_target_is_found_in_the_flags_too() {
        let toolchain = |os: &str, flags: &[&str]| Toolchain {
            jaic: PathBuf::from("jaic"),
            flags: flags.iter().map(ToString::to_string).collect(),
            interp: true,
            runner: Vec::new(),
            os: os.into(),
            coverage: false,
        };
        assert!(targets_wasm(&toolchain("wasm", &[])));
        assert!(targets_wasm(&toolchain("linux", &["-q", "-os", "wasm"])));
        assert!(targets_wasm(&toolchain(
            "linux",
            &["-target", "wasm64-unknown-wasi"]
        )));
        assert!(!targets_wasm(&toolchain("linux", &["-os", "macos"])));
    }

    #[test]
    fn the_waiting_process_gets_its_arguments_once() {
        let dir = std::env::temp_dir().join(format!("jacket-warm-start-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let plan = dir.join("plan.txt");
        let _ = std::fs::remove_file(&plan);
        let built = Built {
            entries: Vec::new(),
            program: PathBuf::from("/b/program.jai"),
            cases: Vec::new(),
            warm: Mutex::new(Some(Warm {
                child: None,
                plan: plan.clone(),
            })),
        };
        let run = RunFlags {
            seed: 7,
            accept: false,
        };
        let names = ["a".to_string(), "b/c".to_string()];
        assert!(warm_start(&built, &names, Path::new("/t"), &run).is_none());
        assert_eq!(
            std::fs::read_to_string(&plan).unwrap(),
            "--seed\n7\n--run\na\nb/c\n--temp\n/t\n"
        );
        // The process is spoken for: the next run starts its own.
        assert!(built.warm.lock().unwrap().is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_runner_starts_the_program() {
        let runner = ["node".to_string(), "wasi_run.mjs".to_string()];
        let (program, args) = program_and_args(Path::new("/b/t.wasm"), &runner);
        assert_eq!(program, PathBuf::from("node"));
        assert_eq!(args, ["wasi_run.mjs", "/b/t.wasm"]);
    }
}
