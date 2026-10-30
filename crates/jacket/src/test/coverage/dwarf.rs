//! Reading debug info: which lines a block of a `jaic -coverage` program holds. See `mod.rs`.
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use super::{Blocks, Files};


/// The file that holds a program's debug info: the `.dSYM` next to it on macOS.
pub(super) fn debug_file(program: &Path) -> PathBuf {
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
pub(super) fn find_dwarfdump() -> Result<PathBuf, String> {
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

/// A row of a DWARF line table.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Row {
    pub address: u64,
    pub line: u32,
    pub path: String,
    pub ends_sequence: bool,
}

pub(super) fn line_table(dwarfdump: &Path, debug_file: &Path) -> Result<Vec<Row>, String> {
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
pub(super) fn parse_line_table(text: &str) -> Vec<Row> {
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

/// How often the lines of the code that ran ran, and the function entries.
pub(super) struct Counted {
    /// The most any block of a function that holds a line ran, added up over the functions that
    /// hold it (the instances of a polymorphic procedure, a macro used twice).
    pub lines: Files,
    /// How often the functions whose entry is at a line were called, by that line.
    pub entries: Files,
}

/// The lines each file has that some block holds. A block runs from its address to the next
/// block's; its lines are those of the row in force at its start and of every row that starts
/// inside it. A function runs from one entry block to the next, and holds a line as often as
/// its busiest block that holds it. `wanted` maps a path to the file to report, or `None` to
/// skip it.
pub(super) fn block_lines(
    rows: &[Row],
    blocks: &Blocks,
    mut wanted: impl FnMut(&str) -> Option<PathBuf>,
) -> Counted {
    let mut paths: Vec<PathBuf> = Vec::new();
    // Which file each path of the line table is, found once.
    let by_path: Vec<Option<usize>> = {
        let mut names: HashMap<&str, Option<usize>> = HashMap::new();
        rows.iter()
            .map(|row| {
                *names.entry(row.path.as_str()).or_insert_with(|| {
                    wanted(&row.path).map(|file| {
                        paths.push(file);
                        paths.len() - 1
                    })
                })
            })
            .collect()
    };
    // The lines of the function being read, by file and line.
    let mut function: BTreeMap<(usize, u32), u64> = BTreeMap::new();
    let mut lines: BTreeMap<(usize, u32), u64> = BTreeMap::new();
    let mut entries: BTreeMap<(usize, u32), u64> = BTreeMap::new();
    let starts: Vec<u64> = blocks.keys().copied().collect();
    for (i, (&address, block)) in blocks.iter().enumerate() {
        let end = starts.get(i + 1).copied().unwrap_or(address + 1);
        let first = rows
            .partition_point(|r| r.address <= address)
            .saturating_sub(1);
        if block.entry {
            for (key, count) in std::mem::take(&mut function) {
                *lines.entry(key).or_default() += count;
            }
            if let Some(row) = rows.get(first)
                && row.line != 0
                && !row.ends_sequence
                && let Some(file) = by_path[first]
            {
                *entries.entry((file, row.line)).or_default() += block.count;
            }
        }
        for (n, row) in rows.iter().enumerate().skip(first) {
            if n != first && row.address >= end {
                break;
            }
            if row.line == 0 || row.ends_sequence {
                continue;
            }
            if let Some(file) = by_path[n] {
                let seen = function.entry((file, row.line)).or_default();
                *seen = (*seen).max(block.count);
            }
        }
    }
    for (key, count) in function {
        *lines.entry(key).or_default() += count;
    }
    let collect = |counted: BTreeMap<(usize, u32), u64>| {
        let mut files = Files::new();
        for ((file, line), count) in counted {
            *files
                .entry(paths[file].clone())
                .or_default()
                .entry(line)
                .or_default() += count;
        }
        files
    };
    Counted {
        lines: collect(lines),
        entries: collect(entries),
    }
}

#[cfg(test)]
mod tests {
    use super::super::Block;
    use super::*;

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

    fn everything(path: &str) -> Option<PathBuf> {
        Some(PathBuf::from(path))
    }

    fn block(count: u64, entry: bool) -> Block {
        Block {
            count,
            entry,
        }
    }

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
    fn a_line_has_the_count_of_the_block_that_holds_it() {
        let rows = parse_line_table(TABLE);
        let blocks: Blocks = [
            (0x1000, block(4, true)),
            (0x1010, block(0, false)),
            (0x1020, block(0, false)),
        ]
        .into();
        let counted = block_lines(&rows, &blocks, everything);
        let a = &counted.lines[Path::new("/src/a.jai")];
        let b = &counted.lines[Path::new("/other/b.jai")];
        let c = &counted.lines[Path::new("/abs/c.jai")];
        assert_eq!((a[&3], b[&7], c[&9]), (4, 0, 0));
        // The one function was entered four times, at the line of its first row.
        assert_eq!(counted.entries[Path::new("/src/a.jai")][&3], 4);
        assert_eq!(counted.entries.len(), 1);
    }

    #[test]
    fn a_function_holds_a_line_as_often_as_its_busiest_block_and_functions_add_up() {
        let rows = parse_line_table(TABLE);
        // Two functions hold the lines of b.jai: one runs them 2 and 5 times in two blocks of
        // its own, the other 3 times.
        let blocks: Blocks = [
            (0x1000, block(1, true)),
            (0x1010, block(2, false)),
            (0x1018, block(5, false)),
            (0x1020, block(3, true)),
        ]
        .into();
        let rows: Vec<Row> = rows
            .into_iter()
            .chain([Row {
                address: 0x1018,
                line: 7,
                path: "/other/b.jai".into(),
                ends_sequence: false,
            }])
            .collect();
        let mut rows = rows;
        rows.sort_by_key(|r| (r.address, !r.ends_sequence));
        let counted = block_lines(&rows, &blocks, everything);
        assert_eq!(counted.lines[Path::new("/other/b.jai")][&7], 5);
        assert_eq!(counted.lines[Path::new("/abs/c.jai")][&9], 3);
    }

    #[test]
    fn a_block_also_holds_the_rows_that_start_inside_it() {
        let rows = parse_line_table(TABLE);
        // One block from 0x1000 to the end holds the lines of every row after the first.
        let blocks: Blocks = [(0x1000, block(2, true)), (0x1040, block(0, false))].into();
        let counted = block_lines(&rows, &blocks, everything);
        assert_eq!(counted.lines[Path::new("/other/b.jai")][&7], 2);
        assert_eq!(counted.lines[Path::new("/abs/c.jai")][&9], 2);
    }

    #[test]
    fn unwanted_files_are_left_out() {
        let rows = parse_line_table(TABLE);
        let blocks: Blocks = [(0x1000, block(1, true)), (0x1040, block(1, false))].into();
        let counted = block_lines(&rows, &blocks, |path| {
            path.starts_with("/src").then(|| PathBuf::from(path))
        });
        assert_eq!(counted.lines.len(), 1);
    }
}
