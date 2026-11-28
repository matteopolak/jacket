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
//!   blocks reading its arguments from standard input (`--args-from-stdin`), which jacket writes
//!   when it hands the process its first tests;
//! - `--list` is that first process too, told to name the cases of the `@cases` tests (`--list`)
//!   and not to run anything.
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

use super::deps_store::{Program, Store};
use super::exec::{self, Demux, Invocation, Piece, Started};

pub const PLUGIN: &str = "+Extensions/Testing/Discovery";

const ENTRY_POINT: &str = "__jacket_test_main";

/// How to compile: the compiler, the flags every compile gets, and how tests run.
#[derive(Clone, Debug)]
pub struct Toolchain {
    pub jaic: PathBuf,
    /// The standard library every jaic run uses (`JAIC_STDLIB`).
    pub stdlib: PathBuf,
    /// `-os`, `-cpu`, the user's extra flags.
    pub flags: Vec<String>,
    pub interp: bool,
    /// Starts test programs built for another target (`node tools/wasi_run.ts`).
    pub runner: Vec<String>,
    /// The target OS (`wasm` programs are `.wasm` files).
    pub os: String,
    /// Build every program with `-coverage`.
    pub coverage: bool,
    /// Where the programs of this toolchain write what they ran (`JAIC_COVERAGE_DIR`), when
    /// they are built to run once.
    pub coverage_dir: Option<PathBuf>,
    /// Doc examples are type-checked only, whatever their mode (`--check-only`).
    pub check_only: bool,
    /// Where the source files each compile read are kept for `--changed`; `None` records nothing.
    pub deps: Arc<Store>,
    /// How many of the last `flags` only prepare the environment (the folder of the probe
    /// libraries) and cannot change which files a compile reads: dependency entries leave them out.
    pub environment_flags: usize,
}

impl Toolchain {
    /// A catalogue's compiler view, also used for that catalogue's dependency queries.
    pub fn for_discovery(&self, listing: Option<Listing>) -> Self {
        let mut toolchain = self.clone();

        if listing == Some(Listing::Host) {
            toolchain.flags = host_flags(&self.flags);
            toolchain.environment_flags = 0;
        }

        toolchain
    }

    /// What a jaic run is told of its standard library.
    pub fn jaic_env(&self) -> Vec<(String, String)> {
        vec![(
            jacket_config::env::STDLIB.name.into(),
            self.stdlib.to_string_lossy().into_owned(),
        )]
    }

    /// The flags that decide which files a compile reads.
    pub fn program_flags(&self) -> &[String] {
        &self.flags[..self.flags.len() - self.environment_flags]
    }

    /// Where the list of files a compile of `program` reads is kept.
    pub fn deps_entry(&self, program: Program, root_flags: &[String], plugin: bool) -> PathBuf {
        self.deps.entry(
            program,
            &self.stdlib,
            self.program_flags(),
            root_flags,
            plugin,
        )
    }

    /// The arguments that make a compile of `program` record the files it reads in the store.
    /// Any list left by an earlier compile goes, so a compile that fails leaves none. Put them
    /// before the plugins, which take the arguments that follow them.
    pub fn deps_args(
        &self,
        program: Program,
        root_flags: &[String],
        plugin: bool,
    ) -> Result<Vec<String>, String> {
        let entry = self.deps_entry(program, root_flags, plugin);

        self.deps.prepare(&entry)?;
        Ok(vec!["-deps_file".into(), path_text(&entry)])
    }

    /// A command that runs this toolchain's jaic with its standard library.
    pub fn jaic_command(&self) -> Command {
        let mut command = Command::new(&self.jaic);

        command.env(jacket_config::env::STDLIB.name, &self.stdlib);

        if let Some(dir) = &self.coverage_dir {
            command.env(jacket_config::env::COVERAGE_DIR.name, dir);
        }

        command
    }
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

pub(super) fn path_text(path: &Path) -> String {
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

/// Which compile-time target to use while listing test and subtest declarations.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Listing {
    Host,
    Target,
}

/// Builds the unit tests of `files` in `work` (a fresh folder). A listing discovers declarations
/// for its requested view without building a native program or running compile-time tests.
///
/// With `--interp`, `warm` leaves the compiling process waiting for the first run of the root;
/// without it the process ends once it has found the tests, and the run compiles again. Each
/// waiting process is a compiler held in memory, so the caller bounds how many there are.
pub fn unit(
    files: &[PathBuf],
    root_flags: &[String],
    work: &Path,
    toolchain: &Toolchain,
    listing: Option<Listing>,
    warm: bool,
) -> Result<Built, String> {
    std::fs::create_dir_all(work).map_err(|e| format!("cannot create {}: {e}", work.display()))?;
    let mut loads = String::from("// Written by `jacket test`: every unit test file of a root.\n");

    for file in files {
        loads.push_str(&format!("#load {};\n", literal(file)));
    }

    let root = work.join("tests.jai");

    write(&root, &loads)?;

    if listing.is_some() || toolchain.interp {
        return interpreted(&loads, &root, work, root_flags, toolchain, listing, warm);
    }

    let wasm = targets_wasm(toolchain);
    let exe = work.join(if wasm {
        "tests.wasm"
    } else {
        "tests"
    });
    let report = work.join("exe.txt");
    // A plugin takes the arguments that follow it up to the next flag, so the plugins a flag list
    // names (a board's) move behind everything else.
    let (plain, plugins) = split_plugins(&toolchain.flags);
    let mut command = toolchain.jaic_command();

    command
        .current_dir(work)
        .arg(&root)
        .arg("-quiet")
        .args(&plain)
        .args(root_flags)
        .args(toolchain.deps_args(Program::Text(&loads), root_flags, true)?)
        .arg("-exe")
        .arg(&exe)
        .env(jacket_config::env::REPORT_EXE.name, &report);

    if toolchain.coverage {
        command.arg("-coverage");
    }

    command.args(&plugins).arg(PLUGIN);
    let found = records(&run_jaic(&mut command, toolchain)?);
    let program = crate::compile::reported_executable(&report)?;

    if !wasm && toolchain.runner.is_empty() {
        stage_dlls(&program)?;
    }

    Ok(Built {
        entries: manifest(&found),
        program,
        cases: Vec::new(),
        warm: Mutex::new(None),
    })
}

/// Type-checks every unit test file of a root for the target of `toolchain` (`--check-only`),
/// including the procedures nothing calls, without building or running anything.
pub fn check_unit(
    files: &[PathBuf],
    root_flags: &[String],
    work: &Path,
    toolchain: &Toolchain,
) -> Result<(), String> {
    std::fs::create_dir_all(work).map_err(|e| format!("cannot create {}: {e}", work.display()))?;
    let mut loads = String::from("// Written by `jacket test`: every unit test file of a root.\n");

    for file in files {
        loads.push_str(&format!("#load {};\n", literal(file)));
    }

    let root = work.join("tests.jai");

    write(&root, &loads)?;
    let mut command = toolchain.jaic_command();

    command
        .current_dir(work)
        .arg(&root)
        .args(["-check", "-no_dce", "-quiet"])
        .args(&toolchain.flags)
        .args(root_flags)
        .args(toolchain.deps_args(Program::Text(&loads), root_flags, false)?);
    run_jaic(&mut command, toolchain).map(|_| ())
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
    listing: Option<Listing>,
    warm: bool,
) -> Result<Built, String> {
    let list = listing.is_some();
    // A process can wait for its arguments only if what it prints reaches jacket before it ends
    // and it can read the file: neither holds for a wasm target, whose program runs in the
    // interpreter's sandbox. That one, a listing and a run that measures coverage (each run
    // writes what it ran into a folder of its own) run to their end with `--list` and compile
    // again for each run.
    // A run under `--coverage` needs its own process to report what it ran.
    let waits = warm && !list && !targets_wasm(toolchain) && !toolchain.coverage;
    // The caller resolved one view for declarations and dependency queries together.
    let mut args = vec![path_text(root), "-interp".into(), "-quiet".into()];

    args.extend(toolchain.flags.iter().cloned());
    args.extend(root_flags.iter().cloned());
    args.extend(toolchain.deps_args(Program::Text(loads), root_flags, true)?);
    args.push(PLUGIN.into());

    if list {
        // Nothing runs afterwards, so the plugin leaves the `@compile_time` tests unrun too.
        args.push("list".into());
    }

    args.push("--".into());

    if waits {
        args.push("--args-from-stdin".into());
    } else {
        args.push("--list".into());
    }

    let mut process = exec::start(&Invocation {
        program: toolchain.jaic.clone(),
        args,
        cwd: work.to_path_buf(),
        env: toolchain.jaic_env(),
        pipe_input: waits,
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
        });
    }

    Ok(result)
}

/// `flags` without the plugins (`+Name` and the options after it, up to the next flag), and the
/// plugins.
pub(super) fn split_plugins(flags: &[String]) -> (Vec<String>, Vec<String>) {
    let (mut plain, mut plugins) = (Vec::new(), Vec::new());
    let mut in_plugin = false;

    for flag in flags {
        if flag.starts_with('+') {
            in_plugin = true;
        } else if flag.starts_with('-') {
            in_plugin = false;
        }

        if in_plugin {
            plugins.push(flag.clone());
        } else {
            plain.push(flag.clone());
        }
    }

    (plain, plugins)
}

/// Runs a compile and returns what it printed on standard output.
fn run_jaic(command: &mut Command, toolchain: &Toolchain) -> Result<Vec<u8>, String> {
    let output = command
        .output()
        .map_err(|e| format!("could not start `{}`: {e}", toolchain.jaic.display()))?;

    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(failure_output(&output))
    }
}

/// What a failed compile printed: standard error, then the text on standard output. The plugin's
/// records (the generated test table, the manifest) are protocol for jacket, not for the reader,
/// and would bury the diagnostics under pages of generated code.
fn failure_output(output: &std::process::Output) -> String {
    failure_text(&output.stderr, &output.stdout)
}

fn failure_text(stderr: &[u8], stdout: &[u8]) -> String {
    let mut demux = Demux::default();
    let mut text = String::from_utf8_lossy(stderr).into_owned();

    for piece in demux.feed(stdout) {
        if let Piece::Text(bytes) = piece {
            text.push_str(&String::from_utf8_lossy(&bytes));
        }
    }

    text.push_str(&String::from_utf8_lossy(&demux.finish()));
    text
}

/// The folders of libraries test programs load by name: `JACKET_TEST_DLLS`, a path list.
const DLL_FOLDERS: &str = jacket_config::env::TEST_DLLS.name;

/// Puts the `.dll` files of the folders in `JACKET_TEST_DLLS` next to `exe`, because Windows
/// looks in the executable's folder before the system's: a program built in a temporary folder
/// finds a library that way (SDL3) and gets a driver in place of the system's (Mesa's
/// `opengl32.dll` for the software OpenGL). Hard links where the folders share a volume, copies
/// otherwise; a file that is already there stays, so the folder listed first wins.
pub fn stage_dlls(exe: &Path) -> Result<(), String> {
    match jacket_config::env::TEST_DLLS.os() {
        Some(folders) => stage_dlls_from(exe, &folders),
        None => Ok(()),
    }
}

fn stage_dlls_from(exe: &Path, folders: &std::ffi::OsStr) -> Result<(), String> {
    let Some(target) = exe.parent() else {
        return Ok(());
    };

    for folder in std::env::split_paths(folders) {
        let entries = std::fs::read_dir(&folder)
            .map_err(|e| format!("{DLL_FOLDERS}: cannot read {}: {e}", folder.display()))?;

        for entry in entries.flatten() {
            let from = entry.path();
            let is_dll = from
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("dll"));
            let to = target.join(entry.file_name());

            if !is_dll || to.exists() {
                continue;
            }

            // Another test building in the same folder may have put it there meanwhile.
            let placed =
                std::fs::hard_link(&from, &to).or_else(|_| std::fs::copy(&from, &to).map(|_| ()));

            if let Err(error) = placed
                && !to.exists()
            {
                return Err(format!(
                    "{DLL_FOLDERS}: cannot place {} next to {}: {error}",
                    from.display(),
                    exe.display()
                ));
            }
        }
    }

    Ok(())
}

/// Compiler arguments for host-side discovery. The complete sequence was checked before
/// constructing the toolchain, so this cannot fail for a valid run.
pub(super) fn host_flags(flags: &[String]) -> Vec<String> {
    jaic_cli_args::arguments::target_flags(flags)
        .expect("validated compiler arguments")
        .host
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

        if toolchain.coverage {
            args.push("-coverage".into());
        }

        args.extend(toolchain.flags.iter().cloned());
        args.extend(root_flags.iter().cloned());
        args.push("--".into());
        (toolchain.jaic.clone(), args)
    } else {
        let extra = run_arguments(names, temp, run);
        program_and_args(&built.program, &toolchain.runner, &extra)
    };

    if toolchain.interp {
        args.append(&mut run_arguments(names, temp, run));
    }

    Invocation {
        program,
        args,
        cwd: cwd.to_path_buf(),
        // A test program may start `jaic` itself, from its own folder.
        env: toolchain.jaic_env(),
        ..Invocation::default()
    }
}


/// With `--interp`, the process that compiled the root while jacket planned, told to run `names`
/// (once; later runs compile by themselves). `None` when there is no such process.
pub fn warm_start(built: &Built, names: &[String], temp: &Path, run: &RunFlags) -> Option<Started> {
    let mut warm = built.warm.lock().ok()?.take()?;
    let mut process = warm.process.take()?;
    // One argument per line; the process reads to the end of its input.
    let mut text = run_arguments(names, temp, run).join("\n");

    text.push('\n');

    if process.hand_over(&text).is_err() {
        process.kill();
        return None;
    }

    Some(process)
}

/// What starts `exe` with `extra` as its arguments: `exe` itself, or the runner. A runner word may
/// hold `{exe}`, where the program goes, and be exactly `{args}`, where the arguments go as one
/// word, for an emulator that passes them through an option (`-append {args}`). Without `{exe}`
/// the program follows the runner's words, and without `{args}` the arguments follow it.
pub fn program_and_args(exe: &Path, runner: &[String], extra: &[String]) -> (PathBuf, Vec<String>) {
    let Some((first, rest)) = runner.split_first() else {
        return (exe.to_path_buf(), extra.to_vec());
    };
    let exe_text = path_text(exe);
    let has = |placeholder: &str| runner.iter().any(|word| word.contains(placeholder));
    let mut args: Vec<String> = rest
        .iter()
        .map(|word| {
            if word == "{args}" {
                extra.join(" ")
            } else {
                word.replace("{exe}", &exe_text)
            }
        })
        .collect();

    if !has("{exe}") {
        args.push(exe_text);
    }

    if !has("{args}") {
        args.extend(extra.iter().cloned());
    }

    (PathBuf::from(first.replace("{exe}", &path_text(exe))), args)
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
    fn a_failed_compile_shows_the_diagnostics_without_the_plugins_records() {
        let stdout = b"printed\n\x02source\x1f#scope_file\x03\x02manifest\x1ftest\x1fadds\x03";
        let shown = failure_text(b"error: type mismatch\n", stdout);

        assert_eq!(shown, "error: type mismatch\nprinted\n");
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
    fn dlls_are_staged_next_to_the_executable_first_folder_first() {
        let root = std::env::temp_dir().join(format!("jacket-dlls-{}", std::process::id()));
        let (first, second, out) = (root.join("a"), root.join("b"), root.join("out"));

        for dir in [&first, &second, &out] {
            std::fs::create_dir_all(dir).unwrap();
        }

        std::fs::write(first.join("gl.dll"), "first").unwrap();
        std::fs::write(second.join("gl.dll"), "second").unwrap();
        std::fs::write(second.join("sdl.dll"), "sdl").unwrap();
        std::fs::write(second.join("notes.txt"), "no").unwrap();
        let folders = std::env::join_paths([&first, &second]).unwrap();

        stage_dlls_from(&out.join("tests"), &folders).unwrap();
        stage_dlls_from(&out.join("tests"), &folders).unwrap();
        assert_eq!(std::fs::read_to_string(out.join("sdl.dll")).unwrap(), "sdl");
        assert!(!out.join("notes.txt").exists());
        assert_eq!(
            std::fs::read_to_string(out.join("gl.dll")).unwrap(),
            "first"
        );
        assert!(
            stage_dlls_from(&out.join("tests"), root.join("missing").as_os_str())
                .unwrap_err()
                .contains("cannot read")
        );
        std::fs::remove_dir_all(&root).unwrap();
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
            stdlib: PathBuf::from("/stdlib"),
            flags: Vec::new(),
            interp: false,
            runner: Vec::new(),
            os: "linux".into(),
            coverage: false,
            coverage_dir: None,
            check_only: false,
            deps: super::super::deps_store::Store::scratch(),
            environment_flags: 0,
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
    fn a_native_test_program_is_told_the_standard_library_by_absolute_path() {
        let built = Built {
            entries: Vec::new(),
            program: PathBuf::from("/b/tests"),
            cases: Vec::new(),
            warm: Mutex::new(None),
        };
        let toolchain = Toolchain {
            jaic: PathBuf::from("jaic"),
            stdlib: PathBuf::from("/stdlib"),
            flags: Vec::new(),
            interp: false,
            runner: Vec::new(),
            os: "linux".into(),
            coverage: false,
            coverage_dir: None,
            check_only: false,
            deps: super::super::deps_store::Store::scratch(),
            environment_flags: 0,
        };
        let run = RunFlags {
            seed: 1,
            accept: false,
        };
        let invocation = invocation(
            &built,
            &[],
            &toolchain,
            &[],
            Path::new("/t"),
            Path::new("/r"),
            &run,
        );

        assert_eq!(
            invocation.env,
            [("JAIC_STDLIB".to_string(), "/stdlib".to_string())]
        );
    }

    #[test]
    fn wasm_routing_uses_the_resolved_target() {
        let toolchain = |os: &str, flags: &[&str]| Toolchain {
            jaic: PathBuf::from("jaic"),
            stdlib: PathBuf::from("/stdlib"),
            flags: flags.iter().map(ToString::to_string).collect(),
            interp: true,
            runner: Vec::new(),
            os: os.into(),
            coverage: false,
            coverage_dir: None,
            check_only: false,
            deps: super::super::deps_store::Store::scratch(),
            environment_flags: 0,
        };

        assert!(targets_wasm(&toolchain("wasm", &[])));
        assert!(!targets_wasm(&toolchain("linux", &["-q", "-os", "wasm"])));
        assert!(!targets_wasm(&toolchain(
            "linux",
            &["-target", "wasm64-unknown-wasi"]
        )));
        assert!(!targets_wasm(&toolchain("linux", &["-os", "macos"])));
    }

    #[cfg(unix)]
    #[test]
    fn the_waiting_process_gets_its_arguments_once() {
        let dir = std::env::temp_dir().join(format!("jacket-warm-start-{}", std::process::id()));

        std::fs::create_dir_all(&dir).unwrap();
        let received = dir.join("received.txt");
        let process = exec::start(&Invocation {
            program: PathBuf::from("/bin/sh"),
            args: vec!["-c".into(), format!("cat > '{}'", received.display())],
            cwd: dir.clone(),
            pipe_input: true,
            ..Invocation::default()
        })
        .unwrap();
        let built = Built {
            entries: Vec::new(),
            program: PathBuf::from("/b/program.jai"),
            cases: Vec::new(),
            warm: Mutex::new(Some(Warm {
                process: Some(process),
            })),
        };
        let run = RunFlags {
            seed: 7,
            accept: false,
        };
        let names = ["a".to_string(), "b/c".to_string()];
        let process = warm_start(&built, &names, Path::new("/t"), &run).unwrap();

        // Closing its input is what lets the process go on: it ends once it has read them all.
        process.finish().unwrap();
        assert_eq!(
            std::fs::read_to_string(&received).unwrap(),
            "--seed\n7\n--run\na\nb/c\n--temp\n/t\n"
        );
        // The process is spoken for: the next run starts its own.
        assert!(built.warm.lock().unwrap().is_none());
        assert!(warm_start(&built, &names, Path::new("/t"), &run).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A compiler that prints `discovered` and, told to wait for arguments, does not end.
    #[cfg(unix)]
    fn waiting_jaic(dir: &Path) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        std::fs::create_dir_all(dir).unwrap();
        let jaic = dir.join("jaic");

        std::fs::write(
            &jaic,
            "#!/bin/sh\nprintf '\\002discovered\\003'\ncase \"$*\" in *--args-from*) exec sleep 30;; esac\n",
        )
        .unwrap();
        std::fs::set_permissions(&jaic, std::fs::Permissions::from_mode(0o755)).unwrap();
        jaic
    }

    #[cfg(unix)]
    #[test]
    fn only_a_root_asked_to_keep_its_process_waiting_does() {
        let dir = std::env::temp_dir().join(format!("jacket-warm-limit-{}", std::process::id()));
        let toolchain = Toolchain {
            jaic: waiting_jaic(&dir),
            stdlib: PathBuf::from("/stdlib"),
            flags: Vec::new(),
            interp: true,
            runner: Vec::new(),
            os: "linux".into(),
            coverage: false,
            coverage_dir: None,
            check_only: false,
            deps: super::super::deps_store::Store::scratch(),
            environment_flags: 0,
        };
        let built = |name: &str, warm: bool| {
            unit(
                &[PathBuf::from("/t/a.jai")],
                &[],
                &dir.join(name),
                &toolchain,
                None,
                warm,
            )
            .unwrap()
        };
        let waiting = built("waiting", true);
        let cold = built("cold", false);

        assert!(waiting.warm.lock().unwrap().is_some());
        assert!(cold.warm.lock().unwrap().is_none());
        drop(waiting);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn plugins_move_behind_the_other_flags() {
        let flags: Vec<String> = ["-O2", "+Board", "tests", "-import_dir", "v", "-quiet"]
            .map(String::from)
            .to_vec();
        let (plain, plugins) = split_plugins(&flags);

        assert_eq!(plain, ["-O2", "-import_dir", "v", "-quiet"]);
        assert_eq!(plugins, ["+Board", "tests"]);
    }

    #[test]
    fn a_runner_starts_the_program() {
        let runner = ["node".to_string(), "wasi_run.ts".to_string()];
        let (program, args) = program_and_args(
            Path::new("/b/t.wasm"),
            &runner,
            &["--run".into(), "a".into()],
        );

        assert_eq!(program, PathBuf::from("node"));
        assert_eq!(args, ["wasi_run.ts", "/b/t.wasm", "--run", "a"]);
    }

    #[test]
    fn a_runner_places_the_program_and_the_arguments() {
        let runner: Vec<String> = ["qemu", "-kernel", "{exe}", "-append", "{args}"]
            .map(String::from)
            .to_vec();
        let (program, args) = program_and_args(
            Path::new("/b/t.elf"),
            &runner,
            &["--run".into(), "a/b".into()],
        );

        assert_eq!(program, PathBuf::from("qemu"));
        assert_eq!(args, ["-kernel", "/b/t.elf", "-append", "--run a/b"]);
        let (_, args) = program_and_args(Path::new("/b/t.elf"), &runner, &[]);

        assert_eq!(args, ["-kernel", "/b/t.elf", "-append", ""]);
    }
}
