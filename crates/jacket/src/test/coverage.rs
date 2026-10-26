//! `jacket test --coverage`: which lines of the code under test the unit tests ran.
//! See docs/tools/jacket-test.md and docs/tools/coverage.md.
//!
//! The unit-test program is built with `jaic -coverage`, which numbers its basic blocks and
//! makes each process that runs it write the blocks it ran at exit (`JAIC_COVERAGE_DIR`). Each
//! test process gets a folder of its own; when it ends its blocks are merged into one table per
//! program and the folder is deleted, so a thousand tests do not leave a thousand files. After
//! the run the DWARF line table of the program (`llvm-dwarfdump --debug-line`) turns blocks
//! into lines: a line is covered when a block that holds it ran. That is a yes or no, not a
//! count. The result is a table per file and an lcov file.
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Whether each block of a program ran, by link-time address.
type Blocks = BTreeMap<u64, bool>;

/// The lines of each source file, and whether any run covered them.
pub type Files = BTreeMap<PathBuf, BTreeMap<u32, bool>>;

pub struct Coverage {
    /// Where the lcov file goes.
    pub out: PathBuf,
    /// A folder per test process is made below this one.
    dir: PathBuf,
    /// Files below these folders are reported, except those below `exclude`.
    include: Vec<PathBuf>,
    exclude: Vec<PathBuf>,
    /// Paths in the table are shown from here.
    base: PathBuf,
    dwarfdump: PathBuf,
    next: AtomicUsize,
    programs: Mutex<BTreeMap<PathBuf, Blocks>>,
}

impl Coverage {
    /// Coverage written to `out`, with run files kept in `dir`. Fails when the tool that reads
    /// debug info is missing.
    pub fn new(
        out: PathBuf,
        dir: PathBuf,
        include: Vec<PathBuf>,
        exclude: Vec<PathBuf>,
        base: PathBuf,
    ) -> Result<Coverage, String> {
        Ok(Coverage {
            out,
            dir,
            include,
            exclude,
            base,
            dwarfdump: find_dwarfdump()?,
            next: AtomicUsize::new(0),
            programs: Mutex::new(BTreeMap::new()),
        })
    }

    /// A fresh folder for one test process to write its blocks to.
    pub fn run_dir(&self) -> PathBuf {
        let n = self.next.fetch_add(1, Ordering::Relaxed);
        let dir = self.dir.join(n.to_string());
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    /// Merges what the process that was given `dir` wrote, and deletes the folder.
    pub fn collect(&self, dir: &Path) {
        for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let Ok(text) = std::fs::read_to_string(entry.path()) else {
                continue;
            };
            let Some((program, blocks)) = parse_run(&text) else {
                continue;
            };
            let mut programs = self.programs.lock().expect("lock");
            let merged = programs.entry(program).or_default();
            for (address, hit) in blocks {
                *merged.entry(address).or_default() |= hit;
            }
        }
        let _ = std::fs::remove_dir_all(dir);
    }

    /// Writes the lcov file and prints the table: to standard output, or to standard error
    /// when that carries machine-readable results. Run before the programs are deleted.
    pub fn report(&self, to_stdout: bool) -> Result<(), String> {
        let files = self.finish()?;
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
        eprintln!("jacket: coverage written to {}", self.out.display());
        Ok(())
    }

    /// The lines of every program, read from its debug info. Run before the programs are deleted.
    pub fn finish(&self) -> Result<Files, String> {
        let programs = self.programs.lock().expect("lock");
        let mut files = Files::new();
        let mut known: BTreeMap<String, Option<PathBuf>> = BTreeMap::new();
        for (program, blocks) in programs.iter() {
            let rows = line_table(&self.dwarfdump, &debug_file(program))?;
            let wanted = |path: &str| {
                known
                    .entry(path.to_string())
                    .or_insert_with(|| self.reported(path))
                    .clone()
            };
            for (path, lines) in block_lines(&rows, blocks, wanted) {
                let covered = files.entry(path).or_default();
                for (line, hit) in lines {
                    *covered.entry(line).or_default() |= hit;
                }
            }
        }
        Ok(files)
    }

    /// The file `path` names when it is code to report on.
    fn reported(&self, path: &str) -> Option<PathBuf> {
        let file = Path::new(path).canonicalize().ok()?;
        let inside = |folders: &[PathBuf]| folders.iter().any(|f| file.starts_with(f));
        (inside(&self.include) && !inside(&self.exclude)).then_some(file)
    }
}

/// The file that holds a program's debug info: the `.dSYM` next to it on macOS.
fn debug_file(program: &Path) -> PathBuf {
    let name = program.file_name().unwrap_or_default();
    let mut dsym = program.as_os_str().to_owned();
    dsym.push(".dSYM/Contents/Resources/DWARF");
    let dsym = PathBuf::from(dsym).join(name);
    if dsym.is_file() {
        dsym
    } else {
        program.to_path_buf()
    }
}

/// `llvm-dwarfdump` from the LLVM jaic is built with (`LLVM_SYS_231_PREFIX`), Homebrew's, or
/// the one on `PATH`.
fn find_dwarfdump() -> Result<PathBuf, String> {
    let prefixes = std::env::var_os("LLVM_SYS_231_PREFIX")
        .map(PathBuf::from)
        .into_iter()
        .chain(["/opt/homebrew/opt/llvm", "/usr/local/opt/llvm"].map(PathBuf::from));
    let path = std::env::var_os("PATH").unwrap_or_default();
    let folders = prefixes
        .map(|p| p.join("bin"))
        .chain(std::env::split_paths(&path));
    for folder in folders {
        for name in ["llvm-dwarfdump", "llvm-dwarfdump-23"] {
            let tool = folder.join(name);
            if tool.is_file() {
                return Ok(tool);
            }
        }
    }
    Err("`--coverage` needs `llvm-dwarfdump` to read debug info; install LLVM or set LLVM_SYS_231_PREFIX".into())
}

/// The program and the blocks of a file `jaic -coverage` programs write: `jaic-coverage 1`,
/// `exe PATH`, then per module `module N` and a `address entry hit` line per block.
fn parse_run(text: &str) -> Option<(PathBuf, Blocks)> {
    let mut lines = text.lines();
    lines.next()?.starts_with("jaic-coverage").then_some(())?;
    let mut program = None;
    let mut blocks = Blocks::new();
    for line in lines {
        if let Some(path) = line.strip_prefix("exe ") {
            program = Some(PathBuf::from(path.trim()));
        } else if !line.starts_with("module ") {
            let mut fields = line.split_whitespace();
            let address = u64::from_str_radix(fields.next()?, 16).ok()?;
            let hit = fields.nth(1)? != "0";
            *blocks.entry(address).or_default() |= hit;
        }
    }
    Some((program?, blocks))
}

/// A row of a DWARF line table.
#[derive(Debug, PartialEq, Eq)]
struct Row {
    address: u64,
    line: u32,
    path: String,
    ends_sequence: bool,
}

fn line_table(dwarfdump: &Path, debug_file: &Path) -> Result<Vec<Row>, String> {
    let output = std::process::Command::new(dwarfdump)
        .arg("--debug-line")
        .arg(debug_file)
        .output()
        .map_err(|e| format!("could not start `{}`: {e}", dwarfdump.display()))?;
    if !output.status.success() {
        return Err(format!(
            "`llvm-dwarfdump` failed on {}: {}",
            debug_file.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(parse_line_table(&String::from_utf8_lossy(&output.stdout)))
}

/// The text between the first and last `"` of `text`.
fn quoted(text: &str) -> Option<&str> {
    let start = text.find('"')? + 1;
    let end = text.rfind('"')?;
    (end >= start).then(|| &text[start..end])
}

/// The number in `name[ 3]` style text, after `prefix`.
fn index_after(text: &str, prefix: &str) -> Option<u32> {
    let rest = text.strip_prefix(prefix)?;
    rest.split(']').next()?.trim().parse().ok()
}

/// The rows of every line table in `llvm-dwarfdump --debug-line` output, by address (a row that
/// ends a sequence before others at its address).
fn parse_line_table(text: &str) -> Vec<Row> {
    let mut rows = Vec::new();
    let mut directories: BTreeMap<u32, String> = BTreeMap::new();
    let mut files: BTreeMap<u32, (String, u32)> = BTreeMap::new();
    let mut pending = None;
    for line in text.lines() {
        let line = line.trim_start();
        if line.starts_with("debug_line[") {
            directories.clear();
            files.clear();
            pending = None;
        } else if let Some(index) = index_after(line, "include_directories[") {
            if let Some(path) = quoted(line) {
                directories.insert(index, path.to_string());
            }
        } else if let Some(index) = index_after(line, "file_names[") {
            pending = Some(index);
            files.entry(index).or_default();
        } else if let Some(name) = line.strip_prefix("name:").and_then(quoted) {
            if let Some(file) = pending.and_then(|p| files.get_mut(&p)) {
                file.0 = name.to_string();
            }
        } else if let Some(dir) = line.strip_prefix("dir_index:") {
            if let Some(file) = pending.and_then(|p| files.get_mut(&p)) {
                file.1 = dir.trim().parse().unwrap_or(0);
            }
        } else if line.starts_with("0x") {
            let fields: Vec<&str> = line.split_whitespace().collect();
            let [
                address,
                number,
                _column,
                file,
                _isa,
                _discriminator,
                flags @ ..,
            ] = &fields[..]
            else {
                continue;
            };
            let (Ok(address), Ok(number), Ok(file)) = (
                u64::from_str_radix(address.trim_start_matches("0x"), 16),
                number.parse(),
                file.parse::<u32>(),
            ) else {
                continue;
            };
            let (name, directory) = files.get(&file).cloned().unwrap_or(("?".into(), 0));
            let path = match directories.get(&directory) {
                Some(base) if !name.starts_with('/') && !base.is_empty() => {
                    format!("{base}/{name}")
                }
                _ => name,
            };
            rows.push(Row {
                address,
                line: number,
                path,
                ends_sequence: flags.contains(&"end_sequence"),
            });
        }
    }
    rows.sort_by_key(|r| (r.address, !r.ends_sequence));
    rows
}

/// The lines each file has that some block holds, covered when a block that ran holds them.
/// A block runs from its address to the next block's; its lines are those of the row in force
/// at its start and of every row that starts inside it. `wanted` maps a path to the file to
/// report, or `None` to skip it.
fn block_lines(
    rows: &[Row],
    blocks: &Blocks,
    mut wanted: impl FnMut(&str) -> Option<PathBuf>,
) -> Files {
    let mut files = Files::new();
    let starts: Vec<u64> = blocks.keys().copied().collect();
    for (i, (&address, &hit)) in blocks.iter().enumerate() {
        let end = starts.get(i + 1).copied().unwrap_or(address + 1);
        let first = rows
            .partition_point(|r| r.address <= address)
            .saturating_sub(1);
        for (n, row) in rows.iter().enumerate().skip(first) {
            if n != first && row.address >= end {
                break;
            }
            if row.line == 0 || row.ends_sequence {
                continue;
            }
            if let Some(file) = wanted(&row.path) {
                *files.entry(file).or_default().entry(row.line).or_default() |= hit;
            }
        }
    }
    files
}

/// The files in lcov's text format, with a `DA:line,hits` per line (hits are 0 or 1).
pub fn lcov(files: &Files) -> String {
    let mut text = String::new();
    for (path, lines) in files {
        let hit = lines.values().filter(|h| **h).count();
        let _ = writeln!(text, "TN:\nSF:{}", path.display());
        for (line, covered) in lines {
            let _ = writeln!(text, "DA:{line},{}", u8::from(*covered));
        }
        let _ = writeln!(text, "LF:{}\nLH:{hit}\nend_of_record", lines.len());
    }
    text
}

/// A table of the files' covered lines, paths shown from `base` when they are below it.
pub fn table(files: &Files, base: &Path) -> String {
    let rows: Vec<(String, usize, usize)> = files
        .iter()
        .map(|(path, lines)| {
            let shown = path
                .strip_prefix(base)
                .unwrap_or(path)
                .display()
                .to_string();
            (shown, lines.values().filter(|h| **h).count(), lines.len())
        })
        .collect();
    let width = rows.iter().map(|r| r.0.len()).max().unwrap_or(0).max(5);
    let percent = |hit: usize, total: usize| {
        if total == 0 {
            100.0
        } else {
            100.0 * hit as f64 / total as f64
        }
    };
    let mut text = format!("{:<width$}  {:>11}  {:>6}\n", "File", "Lines", "Cover");
    let (mut hit_all, mut all) = (0, 0);
    for (shown, hit, total) in &rows {
        let _ = writeln!(
            text,
            "{shown:<width$}  {:>11}  {:>5.1}%",
            format!("{hit}/{total}"),
            percent(*hit, *total)
        );
        hit_all += hit;
        all += total;
    }
    let _ = writeln!(
        text,
        "{:<width$}  {:>11}  {:>5.1}%",
        "Total",
        format!("{hit_all}/{all}"),
        percent(hit_all, all)
    );
    text
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

    const RUN: &str = "jaic-coverage 1\nexe /w/tests\nmodule 3\n1000 1 1\n1010 0 0\n1020 0 1\n";

    #[test]
    fn a_run_file_lists_blocks_and_whether_they_ran() {
        let (program, blocks) = parse_run(RUN).unwrap();
        assert_eq!(program, PathBuf::from("/w/tests"));
        assert_eq!(
            blocks.into_iter().collect::<Vec<_>>(),
            [(0x1000, true), (0x1010, false), (0x1020, true)]
        );
        assert!(parse_run("something else\n").is_none());
        assert!(
            parse_run("jaic-coverage 1\n1000 1 1\n").is_none(),
            "no exe line"
        );
    }

    const TABLE: &str = r#"
debug_line[0x00000000]
Line table prologue:
    version: 5
include_directories[  0] = "/src"
include_directories[  1] = "/other"
file_names[  0]:
           name: "a.jai"
      dir_index: 0
file_names[  1]:
           name: "b.jai"
      dir_index: 1
file_names[  2]:
           name: "/abs/c.jai"
      dir_index: 0
Address            Line   Column File   ISA Discriminator OpIndex Flags
------------------ ------ ------ ------ --- ------------- ------- -------------
0x0000000000001010      7      1      1   0             0       0  is_stmt
0x0000000000001000      3      1      0   0             0       0  is_stmt prologue_end
0x0000000000001020      9      1      2   0             0       0  is_stmt
0x0000000000001030      9      1      2   0             0       0  is_stmt end_sequence
"#;

    #[test]
    fn line_tables_give_paths_lines_and_sequence_ends() {
        let rows = parse_line_table(TABLE);
        let summary: Vec<_> = rows
            .iter()
            .map(|r| (r.address, r.line, r.path.as_str(), r.ends_sequence))
            .collect();
        assert_eq!(
            summary,
            [
                (0x1000, 3, "/src/a.jai", false),
                (0x1010, 7, "/other/b.jai", false),
                (0x1020, 9, "/abs/c.jai", false),
                (0x1030, 9, "/abs/c.jai", true),
            ]
        );
    }

    #[test]
    fn a_line_is_covered_when_a_block_that_holds_it_ran() {
        let rows = parse_line_table(TABLE);
        let blocks: Blocks = [(0x1000, true), (0x1010, false), (0x1020, false)].into();
        let files = block_lines(&rows, &blocks, |path| Some(PathBuf::from(path)));
        let a = &files[Path::new("/src/a.jai")];
        let b = &files[Path::new("/other/b.jai")];
        let c = &files[Path::new("/abs/c.jai")];
        assert_eq!((a[&3], b[&7], c[&9]), (true, false, false));
        // Another run that ran the last two blocks adds to the first.
        let other: Blocks = [(0x1000, false), (0x1010, true), (0x1020, true)].into();
        let more = block_lines(&rows, &other, |path| Some(PathBuf::from(path)));
        assert!(!more[Path::new("/src/a.jai")][&3]);
        assert!(more[Path::new("/other/b.jai")][&7]);
    }

    #[test]
    fn a_block_also_holds_the_rows_that_start_inside_it() {
        let rows = parse_line_table(TABLE);
        // One block from 0x1000 to the end holds the lines of every row after the first.
        let blocks: Blocks = [(0x1000, true), (0x1040, false)].into();
        let files = block_lines(&rows, &blocks, |path| Some(PathBuf::from(path)));
        assert!(files[Path::new("/other/b.jai")][&7]);
        assert!(files[Path::new("/abs/c.jai")][&9]);
    }

    #[test]
    fn unwanted_files_are_left_out() {
        let rows = parse_line_table(TABLE);
        let blocks: Blocks = [(0x1000, true), (0x1040, true)].into();
        let files = block_lines(&rows, &blocks, |path| {
            path.starts_with("/src").then(|| PathBuf::from(path))
        });
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn lcov_and_the_table_report_the_same_counts() {
        let mut files = Files::new();
        files.insert(
            PathBuf::from("/p/src/a.jai"),
            [(1, true), (2, false), (3, true), (4, true)].into(),
        );
        files.insert(PathBuf::from("/p/src/b.jai"), [(5, false)].into());
        let text = lcov(&files);
        assert!(
            text.starts_with("TN:\nSF:/p/src/a.jai\nDA:1,1\nDA:2,0\n"),
            "{text}"
        );
        assert!(text.contains("LF:4\nLH:3\nend_of_record\n"), "{text}");
        let shown = table(&files, Path::new("/p"));
        assert!(shown.contains("src/a.jai"), "{shown}");
        assert!(shown.contains("3/4") && shown.contains("75.0%"), "{shown}");
        assert!(
            shown.contains("Total") && shown.contains("3/5") && shown.contains("60.0%"),
            "{shown}"
        );
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
