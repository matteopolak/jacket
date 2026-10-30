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
//! read at once, on the thread that ran it. The result is a table per file or module and an lcov
//! or html file, which can also be recorded as a baseline or compared with one.
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

mod baseline;
mod dwarf;
mod html;
mod procedures;
mod report;

use crate::cli::{CoverageBy, CoverageFormat};
use baseline::Baseline;
use report::{Data, Group, lcov, rows, table};

/// A basic block of a program: how often it ran, and whether it is the entry of a function.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Block {
    pub count: u64,
    pub entry: bool,
}

/// The blocks of a program, by link-time address.
type Blocks = BTreeMap<u64, Block>;

/// The lines of each source file, and how often they ran.
pub type Files = BTreeMap<PathBuf, BTreeMap<u32, u64>>;

/// What `--coverage` writes and checks.
pub struct Output {
    /// The file that gets the report, in `format`.
    pub out: PathBuf,
    pub format: CoverageFormat,
    /// What a row of the printed table is.
    pub by: CoverageBy,
    /// A recorded baseline to compare with: a module covering fewer lines by more than `margin`
    /// percentage points fails the run.
    pub baseline: Option<PathBuf>,
    pub margin: f64,
    /// Where to record the baseline of this run.
    pub record: Option<PathBuf>,
    /// A markdown table of the modules is added to this file (`$GITHUB_STEP_SUMMARY`).
    pub summary: Option<PathBuf>,
}

/// What is measured and where it is found.
pub struct Config {
    pub output: Output,
    /// A folder per test process is made below this one.
    pub dir: PathBuf,
    /// Files below these folders are reported, except those below `exclude` and those in a
    /// folder named `tests`.
    pub include: Vec<PathBuf>,
    pub exclude: Vec<PathBuf>,
    /// Paths in the table are shown from here.
    pub base: PathBuf,
    /// Lists the procedures of the reported folders, for the target of the tests.
    pub jaic: PathBuf,
    pub jaic_flags: Vec<String>,
    /// The programs are native, so reading their debug info needs `llvm-dwarfdump`.
    pub native: bool,
}

pub struct Coverage {
    output: Output,
    dir: PathBuf,
    include: Vec<PathBuf>,
    exclude: Vec<PathBuf>,
    base: PathBuf,
    /// Reads the debug info of native programs; interpreted runs need none.
    dwarfdump: Option<PathBuf>,
    jaic: PathBuf,
    jaic_flags: Vec<String>,
    next: AtomicUsize,
    /// The blocks of programs many processes ran, until the run ends.
    programs: Mutex<BTreeMap<PathBuf, Blocks>>,
    /// The lines of the programs already read.
    files: Mutex<Files>,
    /// How often functions were entered, by the line they start on.
    entries: Mutex<Files>,
    /// Which file a path of debug info is, when it is one to report.
    known: Mutex<BTreeMap<String, Option<PathBuf>>>,
    /// The first program whose debug info could not be read.
    failure: Mutex<Option<String>>,
}

/// What a run measured.
pub struct Measured {
    pub data: Data,
    /// The procedures in no program the tests ran, counted as not run.
    pub unbuilt: usize,
}

/// How a finished report went.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Held,
    /// A module (or all of them) covers less than the baseline by more than the margin.
    Dropped,
}

impl Coverage {
    /// Coverage per `config`. For native programs it fails when the tool that reads debug info
    /// is missing.
    pub fn new(config: Config) -> Result<Coverage, String> {
        let dwarfdump = if config.native {
            Some(dwarf::find_dwarfdump()?)
        } else {
            dwarf::find_dwarfdump().ok()
        };
        Ok(Coverage {
            output: config.output,
            dir: config.dir,
            include: config.include,
            exclude: config.exclude,
            base: config.base,
            dwarfdump,
            jaic: config.jaic,
            jaic_flags: config.jaic_flags,
            next: AtomicUsize::new(0),
            programs: Mutex::new(BTreeMap::new()),
            files: Mutex::new(Files::new()),
            entries: Mutex::new(Files::new()),
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
                    for (address, block) in blocks {
                        let into: &mut Block = merged.entry(address).or_default();
                        into.count += block.count;
                        into.entry |= block.entry;
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
            if let Err(error) = self.read(&program, &blocks) {
                self.failure.lock().expect("lock").get_or_insert(error);
            }
        }
        let _ = std::fs::remove_dir_all(dir);
    }

    /// Writes the report and prints the table: to standard output, or to standard error when
    /// that carries machine-readable results. Run before the programs are deleted.
    pub fn report(&self, to_stdout: bool) -> Result<Verdict, String> {
        let measured = self.finish()?;
        let data = &measured.data;
        let out = &self.output.out;
        let text = match self.output.format {
            CoverageFormat::Lcov => lcov(data),
            CoverageFormat::Html => html::render(data, &self.base),
        };
        write_file(out, &text)?;
        let by = match self.output.by {
            CoverageBy::File => Group::File,
            CoverageBy::Module => Group::Module,
        };
        let shown = rows(data, &self.base, by);
        if data.lines.is_empty() {
            eprintln!("jacket: no lines of the tested code ran");
        } else {
            let what = if by == Group::File {
                "File"
            } else {
                "Module"
            };
            let table = table(&shown, what);
            if to_stdout {
                print!("\n{table}");
            } else {
                eprint!("\n{table}");
            }
        }
        if measured.unbuilt > 0 {
            eprintln!(
                "jacket: {} procedures are in no program the tests built; their lines count as not run",
                measured.unbuilt
            );
        }
        eprintln!("jacket: coverage written to {}", out.display());
        self.compare(data)
    }

    /// Records the baseline, writes the summary and compares with the baseline, as asked.
    fn compare(&self, data: &Data) -> Result<Verdict, String> {
        let modules = rows(data, &self.base, Group::Module);
        let now = Baseline::of(&modules);
        if let Some(path) = &self.output.record {
            write_file(path, &now.render())?;
            eprintln!("jacket: coverage baseline recorded in {}", path.display());
        }
        let baseline =
            match &self.output.baseline {
                Some(path) => {
                    let text = std::fs::read_to_string(path)
                        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
                    Some(Baseline::parse(&text).map_err(|e| {
                        format!("{} is not a coverage baseline: {e}", path.display())
                    })?)
                }
                None => None,
            };
        let drops = baseline
            .as_ref()
            .map_or_else(Vec::new, |b| b.drops(&now, self.output.margin));
        for drop in &drops {
            eprintln!(
                "jacket: coverage of {} dropped from {:.1}% to {:.1}% (allowed: {} points)",
                drop.name, drop.before, drop.after, self.output.margin
            );
        }
        if let Some(path) = &self.output.summary {
            let text = baseline::markdown(&modules, baseline.as_ref(), &drops, self.output.margin);
            append_file(path, &text)?;
        }
        Ok(if drops.is_empty() {
            Verdict::Held
        } else {
            Verdict::Dropped
        })
    }

    /// The lines of every program, read from its debug info, with the procedures no program has
    /// code of as lines that never ran. Run before the programs are deleted.
    pub fn finish(&self) -> Result<Measured, String> {
        if let Some(error) = self.failure.lock().expect("lock").take() {
            return Err(error);
        }
        let programs = std::mem::take(&mut *self.programs.lock().expect("lock"));
        for (program, blocks) in &programs {
            self.read(program, blocks)?;
        }
        let mut lines = self.files.lock().expect("lock").clone();
        let entries = self.entries.lock().expect("lock").clone();
        let listed = procedures::list(&self.jaic, &self.include, &self.jaic_flags)?;
        let listed: Vec<&procedures::Procedure> = listed
            .iter()
            .filter(|p| self.reported(&p.file.to_string_lossy()).is_some())
            .collect();
        let unbuilt = procedures::unreferenced(&lines, &listed);
        let functions = procedures::functions(&listed, &lines, &entries);
        procedures::add_never_run(&mut lines, &unbuilt);
        Ok(Measured {
            unbuilt: unbuilt.len(),
            data: Data {
                lines,
                functions,
            },
        })
    }

    /// Adds the lines an interpreted run reported, those of the files to report on.
    fn add_lines(&self, lines: Files) {
        let wanted: Files = lines
            .into_iter()
            .filter_map(|(path, lines)| Some((self.reported(&path.to_string_lossy())?, lines)))
            .collect();
        merge(&mut self.files.lock().expect("lock"), wanted);
    }

    /// Adds the reported lines `program` ran, from its debug info.
    fn read(&self, program: &Path, blocks: &Blocks) -> Result<(), String> {
        let dwarfdump = self
            .dwarfdump
            .as_ref()
            .ok_or("reading debug info needs `llvm-dwarfdump`")?;
        let rows = dwarf::line_table(dwarfdump, &dwarf::debug_file(program))?;
        let counted = dwarf::block_lines(&rows, blocks, |path| {
            self.known
                .lock()
                .expect("lock")
                .entry(path.to_string())
                .or_insert_with(|| self.reported(path))
                .clone()
        });
        merge(&mut self.files.lock().expect("lock"), counted.lines);
        merge(&mut self.entries.lock().expect("lock"), counted.entries);
        Ok(())
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

fn write_file(path: &Path, text: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    std::fs::write(path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))
}

/// Adds `text` to the end of `path`, which may hold what others wrote.
fn append_file(path: &Path, text: &str) -> Result<(), String> {
    use std::io::Write as _;
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut file| file.write_all(text.as_bytes()))
        .map_err(|e| format!("cannot write {}: {e}", path.display()))
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
            let entry = fields.next()? != "0";
            let count: u64 = fields.next()?.parse().ok()?;
            let block: &mut Block = blocks.entry(address).or_default();
            block.count += count;
            block.entry |= entry;
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
            [
                (
                    0x1000,
                    Block {
                        count: 7,
                        entry: true
                    }
                ),
                (
                    0x1010,
                    Block {
                        count: 0,
                        entry: false
                    }
                ),
                (
                    0x1020,
                    Block {
                        count: 1,
                        entry: false
                    }
                )
            ]
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
            output: Output {
                out: PathBuf::new(),
                format: CoverageFormat::Lcov,
                by: CoverageBy::File,
                baseline: None,
                margin: 0.0,
                record: None,
                summary: None,
            },
            entries: Mutex::default(),
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
