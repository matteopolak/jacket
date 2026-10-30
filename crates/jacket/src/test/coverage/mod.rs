//! `jacket test --coverage`: which lines of the code under test the tests ran.
//! See docs/tools/jacket-test.md and docs/tools/coverage.md.
//!
//! Every program the tests run (the unit-test program of a root, each golden program and each
//! doc example) is built with `jaic -coverage`, which numbers its basic blocks and makes each
//! process that runs it write the blocks it ran at exit (`JAIC_COVERAGE_DIR`). Each process gets
//! a folder of its own; when it ends its blocks are read and the folder is deleted, so a thousand
//! tests do not leave a thousand files. The DWARF line table of the program
//! (`llvm-dwarfdump --debug-line`) turns blocks into lines. The unit-test program is run by many
//! processes, so its blocks are merged and read after the run; any other program is run once and
//! read at once, on the thread that ran it. The result is a table per file and an lcov file.
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

mod dwarf;
mod procedures;
mod report;

pub use report::{lcov, table};

/// How often each block of a program ran, by link-time address.
type Blocks = BTreeMap<u64, u64>;

/// The lines of each source file, and how often they ran.
pub type Files = BTreeMap<PathBuf, BTreeMap<u32, u64>>;

pub struct Coverage {
    /// Where the lcov file goes.
    pub out: PathBuf,
    /// A folder per test process is made below this one.
    dir: PathBuf,
    /// Files below these folders are reported, except those below `exclude` and those in a
    /// folder named `tests`.
    include: Vec<PathBuf>,
    exclude: Vec<PathBuf>,
    /// Paths in the table are shown from here.
    base: PathBuf,
    /// Reads the debug info of native programs; interpreted runs need none.
    dwarfdump: Option<PathBuf>,
    /// Lists the procedures of the reported folders, for the target of the tests.
    jaic: PathBuf,
    jaic_flags: Vec<String>,
    next: AtomicUsize,
    /// The blocks of programs many processes ran, until the run ends.
    programs: Mutex<BTreeMap<PathBuf, Blocks>>,
    /// The lines of the programs already read.
    files: Mutex<Files>,
    /// Which file a path of debug info is, when it is one to report.
    known: Mutex<BTreeMap<String, Option<PathBuf>>>,
    /// The first program whose debug info could not be read.
    failure: Mutex<Option<String>>,
}

impl Coverage {
    /// Coverage written to `out`, with run files kept in `dir`. For native programs (`native`) it
    /// fails when the tool that reads debug info is missing.
    pub fn new(
        out: PathBuf,
        dir: PathBuf,
        include: Vec<PathBuf>,
        exclude: Vec<PathBuf>,
        base: PathBuf,
        jaic: PathBuf,
        jaic_flags: Vec<String>,
        native: bool,
    ) -> Result<Coverage, String> {
        let dwarfdump = if native {
            Some(dwarf::find_dwarfdump()?)
        } else {
            dwarf::find_dwarfdump().ok()
        };
        Ok(Coverage {
            out,
            dir,
            include,
            exclude,
            base,
            dwarfdump,
            jaic,
            jaic_flags,
            next: AtomicUsize::new(0),
            programs: Mutex::new(BTreeMap::new()),
            files: Mutex::new(Files::new()),
            known: Mutex::new(BTreeMap::new()),
            failure: Mutex::new(None),
        })
    }

    /// A fresh folder for one test process to write its blocks to.
    pub fn run_dir(&self) -> PathBuf {
        let n = self.next.fetch_add(1, Ordering::Relaxed);
        let dir = self.dir.join(n.to_string());
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    /// Merges what the processes that were given `dir` wrote into the blocks of their program,
    /// to be read after the run, and deletes the folder. For a program many processes run. What
    /// an interpreted run wrote is lines already, and is added at once.
    pub fn collect(&self, dir: &Path) {
        for run in runs(dir) {
            match run {
                Run::Blocks(program, blocks) => {
                    let mut programs = self.programs.lock().expect("lock");
                    let merged = programs.entry(program).or_default();
                    for (address, count) in blocks {
                        *merged.entry(address).or_default() += count;
                    }
                }
                Run::Lines(lines) => self.add_lines(lines),
            }
        }
        let _ = std::fs::remove_dir_all(dir);
    }

    /// Reads the lines of the programs the processes that were given `dir` ran now, and deletes
    /// the folder. For a program built to run once, which is read while it still exists.
    pub fn collect_once(&self, dir: &Path) {
        for run in runs(dir) {
            let (program, blocks) = match run {
                Run::Blocks(program, blocks) => (program, blocks),
                Run::Lines(lines) => {
                    self.add_lines(lines);
                    continue;
                }
            };
            // A program a test built and deleted again cannot be read.
            if !program.exists() {
                continue;
            }
            match self.lines(&program, &blocks) {
                Ok(lines) => merge(&mut self.files.lock().expect("lock"), lines),
                Err(error) => {
                    self.failure.lock().expect("lock").get_or_insert(error);
                }
            }
        }
        let _ = std::fs::remove_dir_all(dir);
    }

    /// Writes the lcov file and prints the table: to standard output, or to standard error
    /// when that carries machine-readable results. Run before the programs are deleted.
    pub fn report(&self, to_stdout: bool) -> Result<(), String> {
        let (files, unbuilt) = self.finish()?;
        if let Some(parent) = self.out.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
        }
        std::fs::write(&self.out, lcov(&files))
            .map_err(|e| format!("cannot write {}: {e}", self.out.display()))?;
        let table = table(&files, &self.base);
        if files.is_empty() {
            eprintln!("jacket: no lines of the tested code ran");
        } else if to_stdout {
            print!("\n{table}");
        } else {
            eprint!("\n{table}");
        }
        if unbuilt > 0 {
            eprintln!(
                "jacket: {unbuilt} procedures are in no program the tests built; their lines count as not run"
            );
        }
        eprintln!("jacket: coverage written to {}", self.out.display());
        Ok(())
    }

    /// The lines of every program, read from its debug info, with the lines of the procedures
    /// no program has code of as lines that never ran; and how many such procedures there are.
    /// Run before the programs are deleted.
    pub fn finish(&self) -> Result<(Files, usize), String> {
        if let Some(error) = self.failure.lock().expect("lock").take() {
            return Err(error);
        }
        let programs = std::mem::take(&mut *self.programs.lock().expect("lock"));
        for (program, blocks) in &programs {
            let lines = self.lines(program, blocks)?;
            merge(&mut self.files.lock().expect("lock"), lines);
        }
        let mut files = self.files.lock().expect("lock").clone();
        let listed = procedures::list(&self.jaic, &self.include, &self.jaic_flags)?;
        let unbuilt = procedures::unreferenced(&files, &listed, |file| {
            self.reported(&file.to_string_lossy()).is_some()
        });
        procedures::add_never_run(&mut files, &unbuilt);
        Ok((files, unbuilt.len()))
    }

    /// Adds the lines an interpreted run reported, those of the files to report on.
    fn add_lines(&self, lines: Files) {
        let wanted: Files = lines
            .into_iter()
            .filter_map(|(path, lines)| Some((self.reported(&path.to_string_lossy())?, lines)))
            .collect();
        merge(&mut self.files.lock().expect("lock"), wanted);
    }

    /// The reported lines `program` ran, from its debug info.
    fn lines(&self, program: &Path, blocks: &Blocks) -> Result<Files, String> {
        let dwarfdump = self
            .dwarfdump
            .as_ref()
            .ok_or("reading debug info needs `llvm-dwarfdump`")?;
        let rows = dwarf::line_table(dwarfdump, &dwarf::debug_file(program))?;
        Ok(dwarf::block_lines(&rows, blocks, |path| {
            self.known
                .lock()
                .expect("lock")
                .entry(path.to_string())
                .or_insert_with(|| self.reported(path))
                .clone()
        }))
    }

    /// The file `path` names when it is code to report on.
    fn reported(&self, path: &str) -> Option<PathBuf> {
        let file = Path::new(path).canonicalize().ok()?;
        let inside = |folders: &[PathBuf]| folders.iter().any(|f| file.starts_with(f));
        let in_tests = self
            .include
            .iter()
            .filter_map(|f| file.strip_prefix(f).ok())
            .any(|below| below.components().any(|c| c.as_os_str() == "tests"));
        (inside(&self.include) && !inside(&self.exclude) && !in_tests).then_some(file)
    }
}

/// Adds the counts of `more` to `files`.
fn merge(files: &mut Files, more: Files) {
    for (path, lines) in more {
        let counts = files.entry(path).or_default();
        for (line, count) in lines {
            *counts.entry(line).or_default() += count;
        }
    }
}

/// What a process ran: the blocks of a native program, or the lines of an interpreted one.
#[derive(Debug, PartialEq, Eq)]
enum Run {
    Blocks(PathBuf, Blocks),
    Lines(Files),
}

/// What every run file in `dir` holds.
fn runs(dir: &Path) -> Vec<Run> {
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| parse_run(&std::fs::read_to_string(entry.path()).ok()?))
        .collect()
}

/// The file a process wrote on exit. A program built with `jaic -coverage` writes
/// `jaic-coverage 1`, `exe PATH`, then per module `module N` and an `address entry count` line per
/// block. An interpreted program (`jaic -interp -coverage`) writes `jaic-coverage 1`, `interp`,
/// then a `file PATH` line and a `line count` line for each line of it.
fn parse_run(text: &str) -> Option<Run> {
    let mut lines = text.lines();
    lines.next()?.starts_with("jaic-coverage").then_some(())?;
    let mut program = None;
    let mut blocks = Blocks::new();
    let mut files = Files::new();
    let mut interp = false;
    let mut file: Option<PathBuf> = None;
    for line in lines {
        if let Some(path) = line.strip_prefix("exe ") {
            program = Some(PathBuf::from(path.trim()));
        } else if line == "interp" {
            interp = true;
        } else if let Some(path) = line.strip_prefix("file ") {
            file = Some(PathBuf::from(path));
        } else if interp {
            let (number, count) = line.split_once(' ')?;
            let counts = files.entry(file.clone()?).or_default();
            *counts.entry(number.parse().ok()?).or_default() += count.parse::<u64>().ok()?;
        } else if !line.starts_with("module ") {
            let mut fields = line.split_whitespace();
            let address = u64::from_str_radix(fields.next()?, 16).ok()?;
            let count: u64 = fields.nth(1)?.parse().ok()?;
            *blocks.entry(address).or_default() += count;
        }
    }
    if interp {
        Some(Run::Lines(files))
    } else {
        Some(Run::Blocks(program?, blocks))
    }
}

/// The folders below which sources are reported on, as a set (a folder inside another is
/// already covered by it).
pub fn outermost(folders: impl IntoIterator<Item = PathBuf>) -> Vec<PathBuf> {
    let all: BTreeSet<PathBuf> = folders.into_iter().collect();
    all.iter()
        .filter(|f| !all.iter().any(|other| other != *f && f.starts_with(other)))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const RUN: &str = "jaic-coverage 1\nexe /w/tests\nmodule 3\n1000 1 7\n1010 0 0\n1020 0 1\n";

    #[test]
    fn a_run_file_lists_blocks_and_how_often_they_ran() {
        let Some(Run::Blocks(program, blocks)) = parse_run(RUN) else {
            panic!("not a native run");
        };
        assert_eq!(program, PathBuf::from("/w/tests"));
        assert_eq!(
            blocks.into_iter().collect::<Vec<_>>(),
            [(0x1000, 7), (0x1010, 0), (0x1020, 1)]
        );
        assert!(parse_run("something else\n").is_none());
        assert!(
            parse_run("jaic-coverage 1\n1000 1 1\n").is_none(),
            "no exe line"
        );
    }

    #[test]
    fn an_interpreted_run_lists_the_lines_of_its_files() {
        let text = "jaic-coverage 1\ninterp\nfile /p/a.jai\n3 2\n4 0\nfile /p/b.jai\n1 7\n";
        let Some(Run::Lines(files)) = parse_run(text) else {
            panic!("not an interpreted run");
        };
        assert_eq!(
            files[Path::new("/p/a.jai")],
            BTreeMap::from([(3, 2), (4, 0)])
        );
        assert_eq!(files[Path::new("/p/b.jai")], BTreeMap::from([(1, 7)]));
        assert!(
            parse_run("jaic-coverage 1\ninterp\n3 2\n").is_none(),
            "a line before any file"
        );
    }

    #[test]
    fn counts_of_separate_runs_add_up() {
        let mut files = Files::new();
        let a = PathBuf::from("/p/a.jai");
        merge(&mut files, [(a.clone(), [(1, 2), (2, 0)].into())].into());
        merge(&mut files, [(a.clone(), [(1, 3), (3, 1)].into())].into());
        assert_eq!(
            files[&a].iter().map(|(l, n)| (*l, *n)).collect::<Vec<_>>(),
            [(1, 5), (2, 0), (3, 1)]
        );
    }

    #[test]
    fn files_in_tests_folders_are_not_reported() {
        let coverage = Coverage {
            out: PathBuf::new(),
            dir: PathBuf::new(),
            include: vec![PathBuf::from("/")],
            exclude: Vec::new(),
            base: PathBuf::new(),
            dwarfdump: None,
            jaic: PathBuf::new(),
            jaic_flags: Vec::new(),
            next: AtomicUsize::new(0),
            programs: Mutex::default(),
            files: Mutex::default(),
            known: Mutex::default(),
            failure: Mutex::default(),
        };
        let dir = std::env::temp_dir().join(format!("jacket-cov-reported-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("tests")).unwrap();
        for name in ["module.jai", "tests/t.jai"] {
            std::fs::write(dir.join(name), "").unwrap();
        }
        let include = dir.canonicalize().unwrap();
        let coverage = Coverage {
            include: vec![include.clone()],
            ..coverage
        };
        assert!(
            coverage
                .reported(&dir.join("module.jai").to_string_lossy())
                .is_some()
        );
        assert!(
            coverage
                .reported(&dir.join("tests/t.jai").to_string_lossy())
                .is_none()
        );
        assert!(coverage.reported("/nowhere/at/all.jai").is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn folders_inside_others_are_dropped() {
        let folders = ["/a/b", "/a", "/c"].map(PathBuf::from);
        assert_eq!(
            outermost(folders),
            [PathBuf::from("/a"), PathBuf::from("/c")]
        );
    }
}
