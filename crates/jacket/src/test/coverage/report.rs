//! What was measured, as lcov and as a table. See `mod.rs`.
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use super::Files;

/// A procedure of a reported file and how often it was called.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    /// The line it is declared on.
    pub line: u32,
    pub hits: u64,
}

/// The procedures of each file.
pub type Functions = BTreeMap<PathBuf, Vec<Function>>;

/// Everything `--coverage` measured.
#[derive(Clone, Debug, Default)]
pub struct Data {
    /// How often each line ran.
    pub lines: Files,
    pub functions: Functions,
}

/// The files in lcov's text format: `FN`, `FNDA`, `FNF` and `FNH` for the procedures, `DA`,
/// `LF` and `LH` for the lines.
pub fn lcov(data: &Data) -> String {
    let mut text = String::new();
    let none = Vec::new();
    for (path, lines) in &data.lines {
        let functions = data.functions.get(path).unwrap_or(&none);
        let _ = writeln!(text, "TN:\nSF:{}", path.display());
        for function in functions {
            let _ = writeln!(text, "FN:{},{}", function.line, function.name);
        }
        for function in functions {
            let _ = writeln!(text, "FNDA:{},{}", function.hits, function.name);
        }
        let called = functions.iter().filter(|f| f.hits > 0).count();
        let _ = writeln!(text, "FNF:{}\nFNH:{called}", functions.len());
        for (line, count) in lines {
            let _ = writeln!(text, "DA:{line},{count}");
        }
        let hit = lines.values().filter(|n| **n > 0).count();
        let _ = writeln!(text, "LF:{}\nLH:{hit}\nend_of_record", lines.len());
    }
    text
}

/// How much of something ran.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tally {
    pub hit: usize,
    pub total: usize,
}

impl Tally {
    /// The share that ran, in percent; a tally of nothing is complete.
    pub fn percent(self) -> f64 {
        if self.total == 0 {
            100.0
        } else {
            100.0 * self.hit as f64 / self.total as f64
        }
    }

    fn add(&mut self, other: Tally) {
        self.hit += other.hit;
        self.total += other.total;
    }
}

/// A row of the table: a file or a module.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub name: String,
    pub lines: Tally,
    pub functions: Tally,
}

/// What a row of the table is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    File,
    /// The nearest folder holding a `module.jai`, else the file's folder.
    Module,
}

/// The module `file` belongs to, as a path from `base` when it is below it.
pub fn module_of(file: &Path, base: &Path, known: &mut HashMap<PathBuf, PathBuf>) -> String {
    let dir = file.parent().unwrap_or(file);
    let module = known
        .entry(dir.to_path_buf())
        .or_insert_with(|| {
            // A folder above `base` is not a module of the project.
            let inside = dir.starts_with(base);
            dir.ancestors()
                .take_while(|d| !inside || d.starts_with(base))
                .find(|d| d.join("module.jai").is_file())
                .unwrap_or(dir)
                .to_path_buf()
        })
        .clone();
    shown(&module, base)
}

fn shown(path: &Path, base: &Path) -> String {
    let below = path.strip_prefix(base).unwrap_or(path);
    if below.as_os_str().is_empty() {
        ".".to_string()
    } else {
        below.display().to_string()
    }
}

/// One row per file or module, by name.
pub fn rows(data: &Data, base: &Path, group: Group) -> Vec<Row> {
    let mut by_name: BTreeMap<String, Row> = BTreeMap::new();
    let mut modules = HashMap::new();
    for (path, lines) in &data.lines {
        let name = match group {
            Group::File => shown(path, base),
            Group::Module => module_of(path, base, &mut modules),
        };
        let row = by_name.entry(name.clone()).or_insert_with(|| Row {
            name,
            lines: Tally::default(),
            functions: Tally::default(),
        });
        row.lines.add(Tally {
            hit: lines.values().filter(|n| **n > 0).count(),
            total: lines.len(),
        });
        let functions = data.functions.get(path).map_or(&[][..], Vec::as_slice);
        row.functions.add(Tally {
            hit: functions.iter().filter(|f| f.hits > 0).count(),
            total: functions.len(),
        });
    }
    by_name.into_values().collect()
}

/// The lines and procedures of all the rows.
pub fn total(rows: &[Row]) -> (Tally, Tally) {
    let (mut lines, mut functions) = (Tally::default(), Tally::default());
    for row in rows {
        lines.add(row.lines);
        functions.add(row.functions);
    }
    (lines, functions)
}

/// A table of the rows, with a total.
pub fn table(rows: &[Row], what: &str) -> String {
    let width = rows.iter().map(|r| r.name.len()).max().unwrap_or(0).max(5);
    let cell = |tally: Tally| {
        format!(
            "{:>11}  {:>5.1}%",
            format!("{}/{}", tally.hit, tally.total),
            tally.percent()
        )
    };
    let mut text = format!(
        "{:<width$}  {:>11}  {:>6}  {:>11}  {:>6}\n",
        what, "Lines", "Cover", "Functions", "Cover"
    );
    for row in rows {
        let _ = writeln!(
            text,
            "{:<width$}  {}  {}",
            row.name,
            cell(row.lines),
            cell(row.functions)
        );
    }
    let (lines, functions) = total(rows);
    let _ = writeln!(
        text,
        "{:<width$}  {}  {}",
        "Total",
        cell(lines),
        cell(functions)
    );
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data() -> Data {
        let mut data = Data::default();
        data.lines.insert(
            PathBuf::from("/p/src/a.jai"),
            [(1, 5), (2, 0), (3, 1), (4, 2)].into(),
        );
        data.lines
            .insert(PathBuf::from("/p/src/b.jai"), [(5, 0)].into());
        data.functions.insert(
            PathBuf::from("/p/src/a.jai"),
            vec![
                Function {
                    name: "run".into(),
                    line: 1,
                    hits: 5,
                },
                Function {
                    name: "idle".into(),
                    line: 3,
                    hits: 0,
                },
            ],
        );
        data
    }

    #[test]
    fn lcov_has_procedures_and_line_counts() {
        let text = lcov(&data());
        assert!(
            text.starts_with("TN:\nSF:/p/src/a.jai\nFN:1,run\nFN:3,idle\nFNDA:5,run\nFNDA:0,idle\nFNF:2\nFNH:1\nDA:1,5\nDA:2,0\n"),
            "{text}"
        );
        assert!(
            text.contains(
                "DA:4,2\nLF:4\nLH:3\nend_of_record\nTN:\nSF:/p/src/b.jai\nFNF:0\nFNH:0\nDA:5,0\n"
            ),
            "{text}"
        );
    }

    #[test]
    fn the_table_and_lcov_report_the_same_counts() {
        let rows = rows(&data(), Path::new("/p"), Group::File);
        let shown = table(&rows, "File");
        assert!(shown.contains("src/a.jai"), "{shown}");
        assert!(shown.contains("3/4") && shown.contains("75.0%"), "{shown}");
        assert!(shown.contains("1/2") && shown.contains("50.0%"), "{shown}");
        let total_row = shown.lines().last().unwrap();
        assert!(
            total_row.starts_with("Total")
                && total_row.contains("3/5")
                && total_row.contains("60.0%"),
            "{shown}"
        );
    }

    #[test]
    fn modules_are_the_nearest_folder_with_a_module_file() {
        let dir = std::env::temp_dir().join(format!("jacket-cov-modules-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("Sort/deep")).unwrap();
        std::fs::create_dir_all(dir.join("loose")).unwrap();
        std::fs::write(dir.join("Sort/module.jai"), "").unwrap();
        let mut known = HashMap::new();
        let name = |file: &str, known: &mut HashMap<PathBuf, PathBuf>| {
            module_of(&dir.join(file), &dir, known)
        };
        assert_eq!(name("Sort/module.jai", &mut known), "Sort");
        assert_eq!(name("Sort/deep/more.jai", &mut known), "Sort");
        assert_eq!(name("loose/x.jai", &mut known), "loose");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
