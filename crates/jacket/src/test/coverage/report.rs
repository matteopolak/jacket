//! The coverage report: the lcov file and the table. See `mod.rs`.
use std::fmt::Write as _;
use std::path::Path;

use super::Files;

/// The files in lcov's text format, with a `DA:line,count` per line.
pub fn lcov(files: &Files) -> String {
    let mut text = String::new();
    for (path, lines) in files {
        let hit = lines.values().filter(|n| **n > 0).count();
        let _ = writeln!(text, "TN:\nSF:{}", path.display());
        for (line, count) in lines {
            let _ = writeln!(text, "DA:{line},{count}");
        }
        let _ = writeln!(text, "LF:{}\nLH:{hit}\nend_of_record", lines.len());
    }
    text
}

fn percent(hit: usize, total: usize) -> f64 {
    if total == 0 {
        100.0
    } else {
        100.0 * hit as f64 / total as f64
    }
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
            (
                shown,
                lines.values().filter(|n| **n > 0).count(),
                lines.len(),
            )
        })
        .collect();
    let width = rows.iter().map(|r| r.0.len()).max().unwrap_or(0).max(5);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn lcov_and_the_table_report_the_same_counts() {
        let mut files = Files::new();
        files.insert(
            PathBuf::from("/p/src/a.jai"),
            [(1, 5), (2, 0), (3, 1), (4, 2)].into(),
        );
        files.insert(PathBuf::from("/p/src/b.jai"), [(5, 0)].into());
        let text = lcov(&files);
        assert!(
            text.starts_with("TN:\nSF:/p/src/a.jai\nDA:1,5\nDA:2,0\n"),
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
}
