//! Procedures no program was built with. A procedure nothing references is not compiled into
//! any program, so debug info has no line of it and it would not be in the totals at all; a
//! module whose public procedures no test calls would read 100%. `jaic -procedures` lists every
//! procedure with a body in the reported folders, and the ones with no line in the lines the
//! programs ran are added as lines that never ran.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::Files;
use super::report::{Function, Functions};

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
pub fn list(
    (jaic, stdlib): (&Path, &Path),
    folders: &[PathBuf],
    flags: &[String],
) -> Result<Vec<Procedure>, String> {
    let dirs = std::env::join_paths(folders)
        .map_err(|e| format!("cannot pass the coverage folders to jaic: {e}"))?;
    let output = Command::new(jaic)
        .env("JAIC_STDLIB", stdlib)
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
pub fn unreferenced<'a>(files: &Files, procedures: &[&'a Procedure]) -> Vec<&'a Procedure> {
    procedures
        .iter()
        .copied()
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

/// How often each procedure was called. Native programs count the entries of functions at the
/// line they start on; a call counts for the innermost procedure that line is in. A procedure
/// none of those reached is as called as its declaration line ran (an interpreted program counts
/// the entry of a procedure on that line). Procedures that share a name in a file are told
/// apart by their line.
pub fn functions(procedures: &[&Procedure], lines: &Files, entries: &Files) -> Functions {
    let mut by_file: BTreeMap<&Path, Vec<&Procedure>> = BTreeMap::new();
    for p in procedures {
        by_file.entry(p.file.as_path()).or_default().push(p);
    }
    let mut out = Functions::new();
    for (file, mut procs) in by_file {
        procs.sort_by_key(|p| (p.line, p.end));
        let mut called = vec![0u64; procs.len()];
        for (&line, &count) in entries.get(file).into_iter().flatten() {
            let inside = procs
                .iter()
                .enumerate()
                .filter(|(_, p)| p.line <= line && line <= p.end)
                .min_by_key(|(_, p)| p.end - p.line);
            if let Some((i, _)) = inside {
                called[i] += count;
            }
        }
        let own = lines.get(file);
        let mut names: BTreeMap<&str, usize> = BTreeMap::new();
        for p in &procs {
            *names.entry(p.name.as_str()).or_default() += 1;
        }
        let list = procs
            .iter()
            .zip(called)
            .map(|(p, from_entries)| Function {
                name: if names[p.name.as_str()] > 1 {
                    format!("{}:{}", p.name, p.line)
                } else {
                    p.name.clone()
                },
                line: p.line,
                hits: if from_entries > 0 {
                    from_entries
                } else {
                    own.and_then(|l| l.get(&p.line)).copied().unwrap_or(0)
                },
            })
            .collect();
        out.insert(file.to_path_buf(), list);
    }
    out
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
        let reported: Vec<&Procedure> = procedures
            .iter()
            .filter(|p| !p.file.starts_with("/elsewhere"))
            .collect();
        let found = unreferenced(&files, &reported);
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

    #[test]
    fn procedures_are_as_called_as_their_entries_say() {
        let a = "/m/a.jai";
        let mut lines = Files::new();
        lines.insert(PathBuf::from(a), [(1, 9), (2, 9), (5, 0), (8, 3)].into());
        let mut entries = Files::new();
        entries.insert(PathBuf::from(a), [(1, 4), (2, 5)].into());
        let mut outer = procedure(a, "outer", &[1, 2]);
        outer.end = 4;
        let mut inner = procedure(a, "inner", &[2]);
        inner.end = 3;
        let idle = procedure(a, "idle", &[5]);
        // Never entered natively, but its line ran (an interpreted program).
        let other = procedure(a, "other", &[8]);
        let found = functions(&[&outer, &inner, &idle, &other], &lines, &entries);
        let list: Vec<_> = found[Path::new(a)]
            .iter()
            .map(|f| (f.name.as_str(), f.line, f.hits))
            .collect();
        // The entry at line 2 is the inner procedure's, the one at line 1 the outer's.
        assert_eq!(
            list,
            [
                ("outer", 1, 4),
                ("inner", 2, 5),
                ("idle", 5, 0),
                ("other", 8, 3)
            ]
        );
    }

    #[test]
    fn procedures_with_one_name_are_told_apart_by_line() {
        let a = "/m/a.jai";
        let first = procedure(a, "get", &[1]);
        let second = procedure(a, "get", &[7]);
        let found = functions(&[&second, &first], &Files::new(), &Files::new());
        let names: Vec<_> = found[Path::new(a)]
            .iter()
            .map(|f| f.name.as_str())
            .collect();
        assert_eq!(names, ["get:1", "get:7"]);
    }
}
