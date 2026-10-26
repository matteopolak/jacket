//! Building a root's unit tests: one program from all of its unit test files.
//!
//! jacket writes a root file that `#load`s them and compiles it with the Testing module's
//! Discovery plugin (`+Extensions/Testing/Discovery`), which prints the tests it found and the
//! code it adds (the table of tests and the entry point) as records on standard output. One
//! compile finds the tests and builds them, for every mode:
//!
//! - a build (native or `-os wasm`) writes the program to an executable or module;
//! - `--interp` has no program to build, and every process compiles by itself. A plugin makes
//!   the compiler check all of the root, which costs about three times as much as running it
//!   does, so a second root file loads the tests and the code the plugin printed and names the
//!   entry point in a `#run`: `program.jai`, which all processes but the first run. The first
//!   is started at once with the plugin: it compiles while jacket reads what it printed, then
//!   waits for a file with its arguments (`--args-from`), which jacket writes when it hands the
//!   process its first tests;
//! - `--list` is that first process too, told to name the cases of the `@cases` tests (`--list`)
//!   and not to run anything.
use super::exec::{self, Demux, Invocation, Piece, Started};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

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
    /// With `--interp`, the process that compiled while its records were read, waiting for the
    /// tests it runs. The first run of the root takes it.
    pub warm: Mutex<Option<Warm>>,
}

/// A test program that is already compiling and waits for its arguments.
#[derive(Debug)]
pub struct Warm {
    process: Option<Started>,
    plan: PathBuf,
}

impl Drop for Warm {
    fn drop(&mut self) {
        // Nothing took it: stop it and what it started.
        if let Some(process) = self.process.as_mut() {
            process.kill();
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

/// What the plugin printed: the records of standard output, the fields of each (the kind first).
pub fn records(output: &[u8]) -> Vec<Vec<String>> {
    Demux::default()
        .feed(output)
        .into_iter()
        .filter_map(|piece| match piece {
            Piece::Record(fields) => Some(fields),
            Piece::Text(_) => None,
        })
        .collect()
}

/// The tests and benches among the plugin's `manifest` records.
fn manifest(records: &[Vec<String>]) -> Vec<Entry> {
    records
        .iter()
        .filter(|fields| fields[0] == "manifest")
        .filter_map(|fields| {
            let line = fields.get(4)?.parse().ok()?;
            Some(Entry {
                kind: fields.get(1)?.clone(),
                name: fields.get(2)?.clone(),
                file: PathBuf::from(fields.get(3)?),
                line,
                notes: fields[5..].to_vec(),
            })
        })
        .collect()
}

/// The code the plugin added, from its `source` records.
fn added_source(records: &[Vec<String>]) -> String {
    let lines: Vec<&str> = records
        .iter()
        .filter(|fields| fields[0] == "source")
        .map(|fields| fields.get(1).map_or("", String::as_str))
        .collect();
    lines.join("\n")
}

/// `(test, case)` per `case` record the test program wrote.
fn cases(records: &[Vec<String>]) -> Vec<(String, String)> {
    records
        .iter()
        .filter(|fields| fields[0] == "case" && fields.len() == 3)
        .map(|fields| (fields[1].clone(), fields[2].clone()))
        .collect()
}

/// Whether the plugin ran to its end.
fn discovered(records: &[Vec<String>]) -> bool {
    records.iter().any(|fields| fields[0] == "discovered")
}

/// The output of a compile that failed, for the user.
pub fn compile_output(output: &std::process::Output) -> String {
    let mut text = String::from_utf8_lossy(&output.stderr).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stdout));
    text
}

/// Builds the unit tests of `files` in `work` (a fresh folder). With `list` nothing is built:
/// the tests and the cases of the `@cases` tests are found.
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
    write(&root, &loads)?;
    if list || toolchain.interp {
        return interpreted(&loads, &root, work, root_flags, toolchain, list);
    }
    let wasm = targets_wasm(toolchain);
    let exe = work.join(if wasm {
        "tests.wasm"
    } else {
        "tests"
    });
    let report = work.join("exe.txt");
    let mut command = Command::new(&toolchain.jaic);
    command
        .current_dir(work)
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
    let found = records(&run_jaic(&mut command, toolchain)?);
    Ok(Built {
        entries: manifest(&found),
        program: reported_exe(&report, exe),
        cases: Vec::new(),
        warm: Mutex::new(None),
    })
}

/// The unit tests of a root under `--interp`, or listed. Without a program to build, jacket
/// needs the tests to plan the run: the process that will run the first tests is started now and
/// prints them as soon as the plugin has found them.
fn interpreted(
    loads: &str,
    root: &Path,
    work: &Path,
    root_flags: &[String],
    toolchain: &Toolchain,
    list: bool,
) -> Result<Built, String> {
    let plan = work.join("plan.txt");
    // A process can wait for its arguments only if what it prints reaches jacket before it ends
    // and it can read the file: neither holds for a wasm target, whose program runs in the
    // interpreter's sandbox. That one, and a listing, run to their end with `--list` and
    // compile again for each run.
    let waits = !list && !targets_wasm(toolchain);
    // A listing is of the tests, not of a target: the host's view of them.
    let flags = if list {
        host_flags(&toolchain.flags)
    } else {
        toolchain.flags.clone()
    };
    let mut args = vec![path_text(root), "-interp".into(), "-quiet".into()];
    args.extend(flags);
    args.extend(root_flags.iter().cloned());
    args.extend([PLUGIN.into(), "--".into()]);
    if waits {
        args.extend(["--args-from".into(), path_text(&plan)]);
    } else {
        args.push("--list".into());
    }
    let mut process = exec::start(&Invocation {
        program: toolchain.jaic.clone(),
        args,
        cwd: work.to_path_buf(),
        env: Vec::new(),
    })
    .map_err(|e| format!("could not start `{}`: {e}", toolchain.jaic.display()))?;
    let found = process.records(waits.then_some("discovered"));
    // A process that ends first did not compile.
    if !discovered(&found) {
        return Err(process.failure());
    }
    let mut result = Built {
        entries: manifest(&found),
        program: PathBuf::new(),
        cases: Vec::new(),
        warm: Mutex::new(None),
    };
    let warm = if waits {
        Some(process)
    } else {
        process.finish()?;
        None
    };
    if list {
        result.cases = cases(&found);
        return Ok(result);
    }
    let registry = work.join("registry.jai");
    result.program = work.join("program.jai");
    write(&registry, &added_source(&found))?;
    write(
        &result.program,
        &format!(
            "{loads}#load {};\n#run {{\n    C :: #import \"Compiler\";\n    options := C.get_build_options();\n    options.entry_point_name = \"{ENTRY_POINT}\";\n    C.set_build_options(options);\n}}\n",
            literal(&registry)
        ),
    )?;
    if warm.is_some() {
        *result.warm.lock().expect("lock") = Some(Warm {
            process: warm,
            plan,
        });
    }
    Ok(result)
}

/// Hands the process waiting for `--args-from plan` its arguments, one per line. The file is
/// moved into place whole, for a process that waits for it.
fn write_plan(plan: &Path, arguments: &[String]) -> Result<(), String> {
    let mut text = arguments.join("\n");
    text.push('\n');
    let partial = plan.with_extension("partial");
    write(&partial, &text)?;
    std::fs::rename(&partial, plan).map_err(|e| format!("cannot write {}: {e}", plan.display()))
}

/// Runs a compile and returns what it printed on standard output.
fn run_jaic(command: &mut Command, toolchain: &Toolchain) -> Result<Vec<u8>, String> {
    let output = command
        .output()
        .map_err(|e| format!("could not start `{}`: {e}", toolchain.jaic.display()))?;
    if output.status.success() {
        Ok(output.stdout)
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
pub fn warm_start(built: &Built, names: &[String], temp: &Path, run: &RunFlags) -> Option<Started> {
    let mut warm = built.warm.lock().ok()?.take()?;
    write_plan(&warm.plan, &run_arguments(names, temp, run)).ok()?;
    warm.process.take()
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

    fn record(fields: &[&str]) -> Vec<String> {
        fields.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn manifest_records_are_entries() {
        let entries = manifest(&[
            record(&[
                "manifest",
                "test",
                "adds",
                "/a/t.jai",
                "4",
                "timeout(5)",
                "serial",
            ]),
            record(&["source", "x"]),
            record(&["manifest", "bench", "b", "/a/t.jai", "9"]),
            record(&["manifest", "test", "short"]),
        ]);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].notes, ["timeout(5)", "serial"]);
        assert_eq!((entries[1].kind.as_str(), entries[1].line), ("bench", 9));
    }

    #[test]
    fn the_plugins_output_is_records_among_noise() {
        let output = b"noise\n\x02source\x1f#scope_file\x03\x02source\x1f\x03\x02source\x1fx :: 1;\x03more\x02manifest\x1ftest\x1fadds\x1f/a/t.jai\x1f4\x03\x02discovered\x03";
        let found = records(output);
        assert_eq!(manifest(&found).len(), 1);
        assert_eq!(added_source(&found), "#scope_file\n\nx :: 1;");
        assert!(discovered(&found));
        assert!(!discovered(&records(b"nothing here\n")));
    }

    #[test]
    fn case_records_pair_a_test_with_a_case() {
        let named = cases(&[
            record(&["case", "lengths", "\"\",0"]),
            record(&["case", "lengths", "a,1"]),
            record(&["start", "noise"]),
            record(&["case", "colors", "RED"]),
        ]);
        assert_eq!(
            named,
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
                process: None,
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
