//! `textDocument/formatting` and `textDocument/rangeFormatting`, answered by running `jaifmt`
//! (the formatter is written in Jai) on the document's text. Whatever goes wrong, the answer is
//! no edits and a log line: an editor shows a formatting error as a popup on every save.
use crate::session::Session;
use crate::{DocumentUri, Error, Range, TextEdit};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// How long `jaifmt` may take before it is killed.
const TIMEOUT: Duration = Duration::from_secs(5);

/// What a finished program left behind.
struct Run {
    success: bool,
    code: Option<i32>,
    stdout: Vec<u8>,
    stderr: String,
}

fn spawn_thread<T: Send + 'static>(
    work: impl FnOnce() -> T + Send + 'static,
) -> Result<std::thread::JoinHandle<T>, String> {
    std::thread::Builder::new()
        .spawn(work)
        .map_err(|e| format!("could not start a thread: {e}"))
}

/// Runs `program args` in `cwd` with `input` on its standard input, and kills it once `timeout`
/// has passed.
fn run(
    program: &Path,
    args: &[String],
    cwd: Option<&Path>,
    input: &[u8],
    timeout: Duration,
) -> Result<Run, String> {
    let mut command = Command::new(program);
    command
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    let mut child = command
        .spawn()
        .map_err(|e| format!("could not start {}: {e}", program.display()))?;
    let (Some(mut stdin), Some(mut stdout), Some(mut stderr)) =
        (child.stdin.take(), child.stdout.take(), child.stderr.take())
    else {
        let _ = child.kill();
        let _ = child.wait();
        return Err("could not open the standard streams".into());
    };
    // Each pipe has a thread of its own, so a full pipe never stalls the child while it is
    // waited on.
    let input = input.to_vec();
    let threads = (
        spawn_thread(move || {
            let _ = stdin.write_all(&input);
        }),
        spawn_thread(move || {
            let mut bytes = Vec::new();
            let _ = stdout.read_to_end(&mut bytes);
            bytes
        }),
        spawn_thread(move || {
            let mut bytes = Vec::new();
            let _ = stderr.read_to_end(&mut bytes);
            bytes
        }),
    );
    let (Ok(writer), Ok(out), Ok(err)) = threads else {
        let _ = child.kill();
        let _ = child.wait();
        return Err("could not start a thread".into());
    };
    let deadline = Instant::now() + timeout;
    let mut pause = Duration::from_micros(100);
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {}
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("could not wait for {}: {e}", program.display()));
            }
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            // Closing the pipes ends the reader threads.
            let _ = writer.join();
            let _ = out.join();
            let _ = err.join();
            return Err(format!(
                "{} took longer than {} s and was stopped",
                program.display(),
                timeout.as_secs_f64()
            ));
        }
        std::thread::sleep(pause);
        pause = (pause * 2).min(Duration::from_millis(2));
    };
    let _ = writer.join();
    Ok(Run {
        success: status.success(),
        code: status.code(),
        stdout: out.join().unwrap_or_default(),
        stderr: String::from_utf8_lossy(&err.join().unwrap_or_default()).into_owned(),
    })
}

/// The smallest run of whole lines that differs: the lines both texts begin with and end with
/// stay, so the cursor, folds and diagnostics outside the change do not move.
fn line_edit(before: &str, after: &str) -> Option<(usize, usize, String)> {
    if before == after {
        return None;
    }
    let old: Vec<&str> = before.split_inclusive('\n').collect();
    let new: Vec<&str> = after.split_inclusive('\n').collect();
    let limit = old.len().min(new.len());
    let mut head = 0;
    while head < limit && old[head] == new[head] {
        head += 1;
    }
    let mut tail = 0;
    while tail < limit - head && old[old.len() - 1 - tail] == new[new.len() - 1 - tail] {
        tail += 1;
    }
    let bytes = |lines: &[&str]| lines.iter().map(|l| l.len()).sum::<usize>();
    let start = bytes(&old[..head]);
    let old_end = start + bytes(&old[head..old.len() - tail]);
    let new_start = start;
    let new_end = new_start + bytes(&new[head..new.len() - tail]);
    Some((start, old_end, after[new_start..new_end].to_string()))
}

impl Session {
    /// Where `jaifmt` is: the configured path (which must exist: a wrong setting is reported,
    /// not worked around), else next to this server, else on `PATH`.
    fn formatter(&self) -> Result<PathBuf, String> {
        if let Some(path) = &self.formatter_path {
            return if path.is_file() {
                Ok(path.clone())
            } else {
                Err(format!(
                    "the configured formatter {} does not exist",
                    path.display()
                ))
            };
        }
        jacket_config::tools::locate("jaifmt").ok_or_else(|| {
            "not found next to the language server or on PATH; install the toolchain or set jai.formatter.path".into()
        })
    }

    /// The edits that format `uri`, or the statements `range` touches. Empty when the text is
    /// already formatted, and also (with a log line) when it cannot be formatted: `jaifmt` is
    /// missing, rejects the text or runs too long. `FormattingOptions` are not read: the
    /// `[format]` table of `jacket.toml` decides.
    pub fn format(&self, uri: &DocumentUri, range: Option<Range>) -> Result<Vec<TextEdit>, Error> {
        let doc = self.document(uri)?;
        let log = |message: String| self.index.borrow_mut().log(&format!("jaifmt: {message}"));
        let program = match self.formatter() {
            Ok(program) => program,
            Err(message) => {
                log(message);
                return Ok(Vec::new());
            }
        };
        // jaifmt finds the `jacket.toml` that applies from where it runs.
        let cwd = Path::new(uri.path())
            .parent()
            .filter(|dir| dir.is_dir())
            .map(Path::to_path_buf)
            .or_else(|| self.workspace_folders.first().cloned());
        let mut args = vec!["--stdin".to_string()];
        if let Some(range) = range {
            let (a, b) = (
                doc.index.byte(&doc.text, range.start),
                doc.index.byte(&doc.text, range.end),
            );
            args.push("--range".into());
            args.push(format!("{}:{}", a.min(b), a.max(b)));
        }
        // A file the config's `ignore` list covers is left alone.
        let ignored = run(
            &program,
            &["--is-ignored".into(), uri.path().into()],
            cwd.as_deref(),
            b"",
            TIMEOUT,
        );
        if matches!(&ignored, Ok(r) if r.code == Some(0)) {
            return Ok(Vec::new());
        }
        let result = match run(
            &program,
            &args,
            cwd.as_deref(),
            doc.text.as_bytes(),
            TIMEOUT,
        ) {
            Ok(result) => result,
            Err(message) => {
                log(message);
                return Ok(Vec::new());
            }
        };
        if !result.success {
            let reason = result.stderr.trim().replace('\n', " ");
            log(if reason.is_empty() {
                format!("exited with status {}", result.code.unwrap_or(-1))
            } else {
                reason
            });
            return Ok(Vec::new());
        }
        let Ok(formatted) = String::from_utf8(result.stdout) else {
            log("printed text that is not UTF-8".into());
            return Ok(Vec::new());
        };
        let Some((start, end, new_text)) = line_edit(&doc.text, &formatted) else {
            return Ok(Vec::new());
        };
        Ok(vec![TextEdit {
            range: Range {
                start: doc.index.position(&doc.text, start)?,
                end: doc.index.position(&doc.text, end)?,
            },
            new_text,
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_changed_lines_are_replaced() {
        let before = "a\nb :=1;\nc\n";
        let after = "a\nb := 1;\nc\n";
        assert_eq!(line_edit(before, after), Some((2, 9, "b := 1;\n".into())));
        assert_eq!(line_edit(before, before), None);
    }

    #[test]
    fn a_change_to_the_last_line_without_a_newline() {
        assert_eq!(line_edit("a\nb", "a\nb\n"), Some((2, 3, "b\n".into())));
    }

    #[test]
    fn inserted_and_removed_lines() {
        assert_eq!(line_edit("a\nc\n", "a\nb\nc\n"), Some((2, 2, "b\n".into())));
        assert_eq!(
            line_edit("a\nb\nc\n", "a\nc\n"),
            Some((2, 4, String::new()))
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_program_that_runs_too_long_is_killed() {
        let started = Instant::now();
        let result = run(
            Path::new("/bin/sh"),
            &["-c".into(), "exec sleep 30".into()],
            None,
            b"",
            Duration::from_millis(200),
        );
        assert!(result.err().is_some_and(|e| e.contains("stopped")));
        assert!(started.elapsed() < Duration::from_secs(5));
    }
}
