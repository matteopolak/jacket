//! Procedures no program was built with. A procedure nothing references is not compiled into
//! any program, so debug info has no line of it and it would not be in the totals at all; a
//! module whose public procedures no test calls would read 100%. `jaic -procedures` lists every
//! procedure with a body in the reported folders, and the ones with no line in the lines the
//! programs ran are added as lines that never ran.
use std::path::{Path, PathBuf};
use std::process::Command;

use super::Files;

/// One procedure `jaic -procedures` listed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Procedure {
    pub file: PathBuf,
    pub name: String,
    pub line: u32,
    pub end: u32,
    /// The lines of its statements, which a call would run.
    pub lines: Vec<u32>,
}

/// The records `jaic -procedures` prints: `procedure`, file, name, first line, last line and the
/// running lines, separated by tabs.
pub fn parse(text: &str) -> Vec<Procedure> {
    text.lines().filter_map(parse_record).collect()
}

fn parse_record(line: &str) -> Option<Procedure> {
    let mut fields = line.split('\t');
    (fields.next()? == "procedure").then_some(())?;
    let file = PathBuf::from(fields.next()?);
    let name = fields.next()?.to_string();
    let first = fields.next()?.parse().ok()?;
    let end = fields.next()?.parse().ok()?;
    let lines = fields
        .next()?
        .split(',')
        .filter_map(|n| n.parse().ok())
        .collect();
    Some(Procedure {
        file,
        name,
        line: first,
        end,
        lines,
    })
}

/// Asks `jaic` for the procedures declared below `folders`.
pub fn list(jaic: &Path, folders: &[PathBuf], flags: &[String]) -> Result<Vec<Procedure>, String> {
    let dirs = std::env::join_paths(folders)
        .map_err(|e| format!("cannot pass the coverage folders to jaic: {e}"))?;
    let output = Command::new(jaic)
        .arg("-procedures")
        .arg(dirs)
        .args(flags)
        .output()
        .map_err(|e| format!("could not start `{}`: {e}", jaic.display()))?;
    if !output.status.success() {
        return Err(format!(
            "`{} -procedures` failed: {}",
            jaic.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(parse(&String::from_utf8_lossy(&output.stdout)))
}

/// The procedures none of whose lines are among the lines of `files`: no program has code of
/// them.
pub fn unreferenced<'a>(
    files: &Files,
    procedures: &'a [Procedure],
    mut reported: impl FnMut(&Path) -> bool,
) -> Vec<&'a Procedure> {
    procedures
        .iter()
        .filter(|p| reported(&p.file))
        .filter(|p| {
            let seen = files.get(&p.file);
            !p.lines
                .iter()
                .any(|l| seen.is_some_and(|f| f.contains_key(l)))
        })
        .collect()
}

/// Adds the lines of `procedures` to `files` as lines that never ran.
pub fn add_never_run(files: &mut Files, procedures: &[&Procedure]) {
    for p in procedures {
        let lines = files.entry(p.file.clone()).or_default();
        for line in &p.lines {
            lines.entry(*line).or_insert(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn procedure(file: &str, name: &str, lines: &[u32]) -> Procedure {
        Procedure {
            file: PathBuf::from(file),
            name: name.into(),
            line: lines[0],
            end: lines[lines.len() - 1] + 1,
            lines: lines.to_vec(),
        }
    }

    #[test]
    fn records_become_procedures() {
        let found =
            parse("procedure\t/m/a.jai\tadd\t3\t9\t3,4,8\nsomething else\nprocedure\tbad\n");
        let expected = Procedure {
            file: PathBuf::from("/m/a.jai"),
            name: "add".into(),
            line: 3,
            end: 9,
            lines: vec![3, 4, 8],
        };
        assert_eq!(found, [expected]);
    }

    #[test]
    fn a_procedure_with_no_line_in_any_program_is_unreferenced() {
        let mut files = Files::new();
        files.insert(PathBuf::from("/m/a.jai"), [(3, 4), (4, 0), (20, 1)].into());
        let procedures = [
            procedure("/m/a.jai", "ran", &[3, 4]),
            // Built into a program but never called: its lines are there with no runs.
            procedure("/m/a.jai", "built", &[4, 5]),
            procedure("/m/a.jai", "unused", &[10, 11]),
            procedure("/m/b.jai", "other_file", &[1]),
            procedure("/elsewhere/c.jai", "not_reported", &[1]),
        ];
        let found = unreferenced(&files, &procedures, |file| !file.starts_with("/elsewhere"));
        let names: Vec<_> = found.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["unused", "other_file"]);
    }

    #[test]
    fn unreferenced_procedures_add_lines_that_never_ran() {
        let mut files = Files::new();
        files.insert(PathBuf::from("/m/a.jai"), [(3, 4)].into());
        let a = procedure("/m/a.jai", "unused", &[10, 11, 3]);
        let b = procedure("/m/b.jai", "unused_too", &[1]);
        add_never_run(&mut files, &[&a, &b]);
        let a_lines: Vec<_> = files[Path::new("/m/a.jai")]
            .iter()
            .map(|(l, n)| (*l, *n))
            .collect();
        // A line that some program ran keeps its count.
        assert_eq!(a_lines, [(3, 4), (10, 0), (11, 0)]);
        assert_eq!(files[Path::new("/m/b.jai")][&1], 0);
    }
}
