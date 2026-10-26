//! `jacket test --review`: the snapshots a run left as `NAME.snap.new`, shown as diffs against
//! `NAME.snap` and accepted or rejected one by one.
use super::diff;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const PENDING: &str = ".snap.new";

/// A snapshot waiting for a decision.
#[derive(Debug, PartialEq)]
pub struct Pending {
    /// `NAME.snap.new`.
    pub new: PathBuf,
    /// `NAME.snap`, which may not exist yet.
    pub stored: PathBuf,
}

/// The `.snap.new` files below each root's `tests/` folder, in path order.
pub fn pending(roots: &[PathBuf]) -> Vec<Pending> {
    let mut found = Vec::new();
    for root in roots {
        collect(&root.join("tests"), &mut found);
    }
    found.sort();
    found.dedup();
    found
        .into_iter()
        .map(|new| {
            let text = new.to_string_lossy();
            let stored = PathBuf::from(&text[..text.len() - ".new".len()]);
            Pending {
                new,
                stored,
            }
        })
        .collect()
}

fn collect(dir: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, found);
        } else if path.to_string_lossy().ends_with(PENDING) {
            found.push(path);
        }
    }
}

/// Text as a snapshot compares it: no carriage returns, no line breaks at the end.
fn normalized(text: &str) -> String {
    text.replace('\r', "").trim_end_matches('\n').to_string()
}

/// What `a` and `b` say, as the diff the prompt shows.
fn describe(pending: &Pending) -> String {
    let new = normalized(&std::fs::read_to_string(&pending.new).unwrap_or_default());
    if let Ok(stored) = std::fs::read_to_string(&pending.stored) {
        return diff::lines(&normalized(&stored), &new);
    }
    let body: Vec<String> = new.lines().map(|l| format!("+ {l}")).collect();
    format!("(no stored snapshot yet)\n{}", body.join("\n"))
}

fn accept(pending: &Pending) -> std::io::Result<()> {
    // Renaming over a file fails on Windows.
    let _ = std::fs::remove_file(&pending.stored);
    std::fs::rename(&pending.new, &pending.stored)
}

fn shown(path: &Path, cwd: &Path) -> String {
    path.strip_prefix(cwd).unwrap_or(path).display().to_string()
}

/// Lists the pending snapshots of `roots`; in a terminal, asks about each.
/// Exits 0 when none are left, 1 when some are (always the case when only listing).
pub fn run(
    roots: &[PathBuf],
    cwd: &Path,
    interactive: bool,
    input: &mut dyn BufRead,
    out: &mut dyn Write,
) -> ExitCode {
    let found = pending(roots);
    if found.is_empty() {
        let _ = writeln!(out, "no pending snapshots");
        return ExitCode::SUCCESS;
    }
    if !interactive {
        for item in &found {
            let _ = writeln!(out, "{}", shown(&item.new, cwd));
        }
        let _ = writeln!(
            out,
            "{} pending snapshot(s); `jacket test --review` in a terminal goes through them, `jacket test --accept` keeps them all",
            found.len()
        );
        return ExitCode::from(1);
    }
    let (mut accepted, mut rejected, mut left) = (0, 0, 0);
    let total = found.len();
    let mut quit = false;
    for (n, item) in found.iter().enumerate() {
        if quit {
            left += 1;
            continue;
        }
        let _ = writeln!(
            out,
            "\n[{}/{total}] {}\n{}",
            n + 1,
            shown(&item.stored, cwd),
            describe(item)
        );
        loop {
            let _ = write!(out, "[a]ccept, [r]eject, [s]kip, [q]uit? ");
            let _ = out.flush();
            let mut answer = String::new();
            if input.read_line(&mut answer).unwrap_or(0) == 0 {
                quit = true;
                left += 1;
                break;
            }
            match answer.trim().to_ascii_lowercase().as_str() {
                "a" | "accept" => {
                    match accept(item) {
                        Ok(()) => accepted += 1,
                        Err(error) => {
                            let _ = writeln!(out, "cannot accept: {error}");
                            left += 1;
                        }
                    }
                    break;
                }
                "r" | "reject" => {
                    let _ = std::fs::remove_file(&item.new);
                    rejected += 1;
                    break;
                }
                "s" | "skip" => {
                    left += 1;
                    break;
                }
                "q" | "quit" => {
                    quit = true;
                    left += 1;
                    break;
                }
                _ => {}
            }
        }
    }
    let _ = writeln!(
        out,
        "\n{accepted} accepted, {rejected} rejected, {left} left pending"
    );
    if left == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("jacket-review-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let folder = dir.join("tests/__snapshots__/file");
        std::fs::create_dir_all(&folder).expect("folder");
        dir
    }

    fn snapshots(root: &Path) -> PathBuf {
        root.join("tests/__snapshots__/file")
    }

    #[test]
    fn pending_snapshots_pair_with_their_stored_file() {
        let root = scratch("pending");
        let folder = snapshots(&root);
        std::fs::write(folder.join("a.snap.new"), "x\n").expect("write");
        std::fs::write(folder.join("b.snap"), "kept\n").expect("write");
        let found = pending(std::slice::from_ref(&root));
        assert_eq!(
            found,
            [Pending {
                new: folder.join("a.snap.new"),
                stored: folder.join("a.snap"),
            }]
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn without_a_terminal_they_are_only_listed() {
        let root = scratch("list");
        let folder = snapshots(&root);
        std::fs::write(folder.join("a.snap.new"), "x\n").expect("write");
        let mut out = Vec::new();
        run(
            std::slice::from_ref(&root),
            &root,
            false,
            &mut std::io::empty(),
            &mut out,
        );
        let text = String::from_utf8(out).expect("utf-8");
        assert!(
            text.contains("tests/__snapshots__/file/a.snap.new"),
            "{text}"
        );
        assert!(folder.join("a.snap.new").exists());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn answers_accept_reject_and_skip() {
        let root = scratch("answers");
        let folder = snapshots(&root);
        for name in ["a", "b", "c"] {
            std::fs::write(folder.join(format!("{name}.snap.new")), "new\n").expect("write");
        }
        std::fs::write(folder.join("a.snap"), "old\n").expect("write");
        let mut input = std::io::Cursor::new("?\na\nr\ns\n");
        let mut out = Vec::new();
        run(
            std::slice::from_ref(&root),
            &root,
            true,
            &mut input,
            &mut out,
        );
        let text = String::from_utf8(out).expect("utf-8");
        assert!(text.contains('-') && text.contains("+ new"), "{text}");
        assert_eq!(
            std::fs::read_to_string(folder.join("a.snap")).expect("accepted"),
            "new\n"
        );
        assert!(!folder.join("a.snap.new").exists());
        assert!(!folder.join("b.snap.new").exists());
        assert!(!folder.join("b.snap").exists());
        assert!(folder.join("c.snap.new").exists());
        assert!(
            text.contains("1 accepted, 1 rejected, 1 left pending"),
            "{text}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_diff_ignores_line_endings() {
        let root = scratch("endings");
        let folder = snapshots(&root);
        std::fs::write(folder.join("a.snap"), "one\r\ntwo\r\n").expect("write");
        std::fs::write(folder.join("a.snap.new"), "one\ntwo\n").expect("write");
        let item = &pending(std::slice::from_ref(&root))[0];
        assert_eq!(describe(item), "--- want\n+++ got\n  one\n  two");
        let _ = std::fs::remove_dir_all(&root);
    }
}
