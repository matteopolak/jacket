//! `console` code blocks in the guides: commands and what they print, checked by running them.
//!
//! A block is lines that start with `$ ` (a command) and the lines after each (its output):
//!
//! ````text
//! ```console cwd=../examples/testing/slugs
//! $ jacket test
//!     Starting 4 tests
//! ```
//! ````
//!
//! `jacket doc` runs every command in the block's folder (`cwd=`, from the guide's folder; the
//! guide's own folder without it) and fails when the output differs from the block. The markdown
//! holds the text alone; the colours the commands printed are kept beside the guide in
//! `<guide>.console.json`, which the site renders them from. `--accept` writes both.
//!
//! A `cache` block is run only when its capture is missing or no longer matches its text, for
//! commands whose output depends on the machine (timings, installed cross tools).

use super::markdown::esc;
use jaic::fxhash::HashMap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ops::Range;
use std::path::{Path, PathBuf};

#[cfg(test)]
#[path = "console_tests.rs"]
mod tests;

/// One command of a block and the lines it printed, without the line break after the last.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Run {
    pub command: String,
    pub output: String,
}

/// A `console` fence.
#[derive(Debug, PartialEq, Eq)]
pub struct Block {
    /// The opening fence's line, from 1.
    pub line: usize,
    /// The folder the commands run in, from the guide's folder.
    pub cwd: Option<String>,
    /// Whether a capture that matches the text stands for running the commands.
    pub cache: bool,
    /// The text between the fences, in the markdown.
    pub body: Range<usize>,
    pub runs: Vec<Run>,
}

/// What the commands printed, colours and all: one list of runs per block, in order.
#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
struct Capture {
    blocks: Vec<Vec<Run>>,
}

/// The programs a block may run.
pub struct Tools<'a> {
    pub jacket: &'a Path,
    pub jaic: &'a Path,
    pub stdlib: &'a Path,
}

/// The `console` fences of `md`, parsed. A fence that does not start with a command, or has a
/// setting other than `cwd=` and `cache`, is an error.
pub fn blocks(md: &str) -> Result<Vec<Block>, String> {
    use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
    let options = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_GFM;
    let mut out = Vec::new();
    // The open fence as a block with no runs yet, and its text so far.
    let mut open: Option<(Block, Option<Range<usize>>)> = None;
    for (event, range) in Parser::new_ext(md, options).into_offset_iter() {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) => {
                let mut words = info.split([' ', ',']).filter(|w| !w.is_empty());
                if words.next() != Some("console") {
                    continue;
                }
                let line = md[..range.start].matches('\n').count() + 1;
                let mut cwd = None;
                let mut cache = false;
                for word in words {
                    match word.split_once('=') {
                        Some(("cwd", dir)) => cwd = Some(dir.to_owned()),
                        None if word == "cache" => cache = true,
                        _ => {
                            return Err(format!(
                                "line {line}: unknown console setting `{word}`; the settings are `cwd=FOLDER` and `cache`"
                            ));
                        }
                    }
                }
                let block = Block {
                    line,
                    cwd,
                    cache,
                    body: 0..0,
                    runs: Vec::new(),
                };
                open = Some((block, None));
            }
            Event::Text(_) => {
                if let Some((_, body)) = &mut open {
                    let start = body.as_ref().map_or(range.start, |b| b.start);
                    *body = Some(start..range.end);
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                let Some((mut block, body)) = open.take() else {
                    continue;
                };
                block.body = body.unwrap_or(range.start..range.start);
                block.runs = runs(&md[block.body.clone()])
                    .map_err(|e| format!("line {}: {e}", block.line))?;
                out.push(block);
            }
            _ => {}
        }
    }
    Ok(out)
}

/// A block's text as commands and their output.
fn runs(text: &str) -> Result<Vec<Run>, String> {
    let mut runs: Vec<Run> = Vec::new();
    let mut lines: Vec<Vec<&str>> = Vec::new();
    for line in text.lines() {
        if let Some(command) = line.strip_prefix("$ ") {
            runs.push(Run {
                command: command.trim().to_owned(),
                output: String::new(),
            });
            lines.push(Vec::new());
        } else if let Some(last) = lines.last_mut() {
            last.push(line);
        } else {
            return Err(
                "a console block starts with a command: a line that starts with `$ `".into(),
            );
        }
    }
    if runs.is_empty() {
        return Err("a console block needs a command: a line that starts with `$ `".into());
    }
    for (run, lines) in runs.iter_mut().zip(lines) {
        run.output = trim_end_lines(&lines.join("\n"));
    }
    Ok(runs)
}

/// `text` without the blank lines at its end.
fn trim_end_lines(text: &str) -> String {
    let mut lines: Vec<&str> = text.lines().collect();
    while lines.last().is_some_and(|l| l.trim().is_empty()) {
        lines.pop();
    }
    lines.join("\n")
}

/// The text of a block that holds `runs`, as it goes between the fences.
fn body(runs: &[Run]) -> String {
    let mut out = String::new();
    for run in runs {
        out.push_str("$ ");
        out.push_str(&run.command);
        out.push('\n');
        if !run.output.is_empty() {
            out.push_str(&run.output);
            out.push('\n');
        }
    }
    out
}

/// `text` without its terminal escape sequences (`ESC [ ... letter`).
pub fn strip(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            let mut rest = chars.clone();
            if rest.next() == Some('[') {
                for c in rest.by_ref() {
                    if c.is_ascii_alphabetic() {
                        break;
                    }
                }
                chars = rest;
                continue;
            }
        }
        out.push(c);
    }
    out
}

/// What two outputs are compared as: line breaks as `\n`, no spaces at line ends, and times
/// (`[   0.057s]`, `[  12ms]`) as `[time]`, since they differ from run to run. The `slowest:`
/// list `jacket test` prints for tests over a second is left out for the same reason.
pub fn normal(text: &str) -> String {
    static TIME: std::sync::LazyLock<regex_lite::Regex> = std::sync::LazyLock::new(|| {
        regex_lite::Regex::new(r"\[ *[0-9]+(\.[0-9]+)?m?s\]").expect("valid pattern")
    });
    static SLOW: std::sync::LazyLock<regex_lite::Regex> = std::sync::LazyLock::new(|| {
        regex_lite::Regex::new(r"^ +[0-9]+\.[0-9]+s ").expect("valid pattern")
    });
    let text = text.replace("\r\n", "\n");
    let mut in_slowest = false;
    let lines: Vec<String> = text
        .lines()
        .filter(|l| {
            in_slowest = if *l == "slowest:" {
                true
            } else {
                in_slowest && SLOW.is_match(l)
            };
            !in_slowest
        })
        .map(|l| TIME.replace_all(l.trim_end(), "[time]").into_owned())
        .collect();
    trim_end_lines(&lines.join("\n"))
}

/// `command` split into words the way a shell would: spaces separate them, and `'...'` or
/// `"..."` keep spaces in one. No variables, globs or pipes.
pub fn words(command: &str) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut word = String::new();
    let mut started = false;
    let mut chars = command.chars();
    while let Some(c) = chars.next() {
        match c {
            '\'' | '"' => {
                started = true;
                loop {
                    match chars.next() {
                        Some(q) if q == c => break,
                        Some(other) => word.push(other),
                        None => return Err(format!("`{command}` has an unclosed {c}")),
                    }
                }
            }
            c if c.is_whitespace() => {
                if started {
                    out.push(std::mem::take(&mut word));
                    started = false;
                }
            }
            '|' | '>' | '<' | '&' | ';' | '$' | '*' | '`' => {
                return Err(format!(
                    "`{command}` uses `{c}`; console commands run without a shell"
                ));
            }
            c => {
                started = true;
                word.push(c);
            }
        }
    }
    if started {
        out.push(word);
    }
    Ok(out)
}

/// The files below `dir` and what they hold.
fn files(dir: &Path) -> Result<BTreeMap<PathBuf, Vec<u8>>, String> {
    fn walk(dir: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) -> Result<(), String> {
        let entries =
            std::fs::read_dir(dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out)?;
            } else {
                let bytes = std::fs::read(&path)
                    .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
                out.insert(path, bytes);
            }
        }
        Ok(())
    }
    let mut out = BTreeMap::new();
    walk(dir, &mut out)?;
    Ok(out)
}

/// The folders below `dir`.
fn folders(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.push(path.clone());
            folders(&path, out);
        }
    }
}

/// Runs one command in `cwd` and returns what it printed to standard output and error, in the
/// order it printed it, with colour on.
fn run_command(command: &str, cwd: &Path, tools: &Tools<'_>) -> Result<String, String> {
    use std::io::Read;
    let words = words(command)?;
    let Some((program, args)) = words.split_first() else {
        return Err("an empty command".into());
    };
    let program = match program.as_str() {
        "jacket" => tools.jacket,
        "jaic" => tools.jaic,
        "cat" => {
            let mut out = String::new();
            for file in args {
                let path = cwd.join(file);
                out.push_str(
                    &std::fs::read_to_string(&path)
                        .map_err(|e| format!("`{command}`: cannot read {}: {e}", path.display()))?,
                );
            }
            return Ok(out);
        }
        other => {
            return Err(format!(
                "`{command}` runs `{other}`; console blocks run `jacket`, `jaic` and `cat`"
            ));
        }
    };
    let (mut reader, writer) =
        std::io::pipe().map_err(|e| format!("`{command}`: cannot make a pipe: {e}"))?;
    let mut child = {
        let mut cmd = std::process::Command::new(program);
        cmd.args(args)
            .current_dir(cwd)
            .stdin(std::process::Stdio::null())
            .stdout(
                writer
                    .try_clone()
                    .map_err(|e| format!("`{command}`: cannot share a pipe: {e}"))?,
            )
            .stderr(writer)
            .env(jacket_config::env::STDLIB.name, tools.stdlib)
            .env(jacket_config::env::FORCE_COLOR.name, "1")
            // The status lines a terminal shows, without the line redrawn in place.
            .env(jacket_config::env::PROGRESS.name, "plain");
        // What a reader's terminal would not have, or would set differently.
        for var in [
            &jacket_config::env::CI,
            &jacket_config::env::NO_COLOR,
            &jacket_config::env::DIAGNOSTICS,
            &jacket_config::env::COLUMNS,
        ] {
            cmd.env_remove(var.name);
        }
        cmd.spawn()
            .map_err(|e| format!("cannot run `{command}`: {e}"))?
        // `cmd` and its copies of the pipe's write end are dropped here, so the read below ends
        // when the child exits.
    };
    let mut out = Vec::new();
    let read = reader.read_to_end(&mut out);
    child
        .wait()
        .map_err(|e| format!("`{command}` did not finish: {e}"))?;
    read.map_err(|e| format!("cannot read what `{command}` printed: {e}"))?;
    Ok(String::from_utf8_lossy(&out).into_owned())
}

/// Runs a block's commands in turn, in its folder, and returns their output. Files they make are
/// removed afterwards; changing a file that was there is an error, so a block leaves its folder
/// as it found it.
fn run_block(block: &Block, dir: &Path, tools: &Tools<'_>) -> Result<Vec<Run>, String> {
    if !dir.is_dir() {
        return Err(format!("the folder {} does not exist", dir.display()));
    }
    let before = files(dir)?;
    let mut had = Vec::new();
    folders(dir, &mut had);
    let mut out = Vec::new();
    let mut failure = None;
    for run in &block.runs {
        match run_command(&run.command, dir, tools) {
            Ok(output) => out.push(Run {
                command: run.command.clone(),
                output: trim_end_lines(&output.replace("\r\n", "\n")),
            }),
            Err(e) => {
                failure = Some(e);
                break;
            }
        }
    }
    let after = files(dir)?;
    let mut now = Vec::new();
    folders(dir, &mut now);
    for path in after.keys().filter(|p| !before.contains_key(*p)) {
        let _ = std::fs::remove_file(path);
    }
    for path in now.iter().rev().filter(|p| !had.contains(p)) {
        let _ = std::fs::remove_dir(path);
    }
    if let Some(e) = failure {
        return Err(e);
    }
    let changed: Vec<String> = before
        .iter()
        .filter(|(path, bytes)| after.get(*path) != Some(*bytes))
        .map(|(path, _)| path.display().to_string())
        .collect();
    if !changed.is_empty() {
        return Err(format!(
            "the commands changed {}; a console block must leave its folder as it found it",
            changed.join(", ")
        ));
    }
    Ok(out)
}

/// The colour capture kept beside `guide`.
pub fn capture_path(guide: &Path) -> PathBuf {
    guide.with_extension("console.json")
}

fn read_capture(path: &Path) -> Result<Option<Capture>, String> {
    match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).map(Some).map_err(|e| {
            format!(
                "{}: {e}; `jacket doc --accept` writes it again",
                path.display()
            )
        }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("cannot read {}: {e}", path.display())),
    }
}

/// Whether `shot`, a captured block, is what `block` shows: the same commands, and the same
/// text once the colours are gone.
fn matches(block: &Block, shot: &[Run]) -> bool {
    block.runs.len() == shot.len()
        && block
            .runs
            .iter()
            .zip(shot)
            .all(|(b, s)| b.command == s.command && b.output == strip(&s.output))
}

/// A diff of what the guide shows against what the command printed.
fn diff(want: &str, got: &str, command: &str) -> String {
    similar::TextDiff::from_lines(&format!("{want}\n"), &format!("{got}\n"))
        .unified_diff()
        .header("the guide", command)
        .to_string()
}

/// Runs the console blocks of the guide at `path` and reports each one whose output differs from
/// what it printed, or whose colour capture is missing or stale; with `accept`, writes the guide
/// and the capture instead. `shown` is the guide's path in messages.
pub fn check(
    path: &Path,
    shown: &str,
    tools: &Tools<'_>,
    accept: bool,
) -> Result<Vec<String>, String> {
    let md = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let blocks = blocks(&md).map_err(|e| format!("{shown}: {e}"))?;
    let capture_file = capture_path(path);
    let stored = read_capture(&capture_file)?;
    let capture_shown = format!("{}", capture_file.display());
    if blocks.is_empty() {
        if stored.is_none() {
            return Ok(Vec::new());
        }
        if accept {
            std::fs::remove_file(&capture_file)
                .map_err(|e| format!("cannot remove {capture_shown}: {e}"))?;
            return Ok(Vec::new());
        }
        return Ok(vec![format!(
            "{capture_shown}: {shown} has no console blocks any more; `jacket doc --accept` removes it"
        )]);
    }
    let stored = stored.unwrap_or_default();
    let here = path.parent().unwrap_or(Path::new("."));
    let mut problems = Vec::new();
    let mut fresh = Capture::default();
    let mut rewrites: Vec<(Range<usize>, String)> = Vec::new();
    for (n, block) in blocks.iter().enumerate() {
        let old = stored.blocks.get(n).filter(|shot| matches(block, shot));
        if block.cache
            && let Some(old) = old
        {
            fresh.blocks.push(old.clone());
            continue;
        }
        let dir = here.join(block.cwd.as_deref().unwrap_or("."));
        let got =
            run_block(block, &dir, tools).map_err(|e| format!("{shown}:{}: {e}", block.line))?;
        let same = block
            .runs
            .iter()
            .zip(&got)
            .all(|(want, got)| normal(&want.output) == normal(&strip(&got.output)));
        if same && let Some(old) = old {
            fresh.blocks.push(old.clone());
            continue;
        }
        if accept {
            let text: Vec<Run> = got
                .iter()
                .map(|r| Run {
                    command: r.command.clone(),
                    output: strip(&r.output),
                })
                .collect();
            rewrites.push((block.body.clone(), body(&text)));
            fresh.blocks.push(got);
            continue;
        }
        if same {
            problems.push(format!(
                "{shown}:{}: the colours in {capture_shown} are missing or out of date; `jacket doc --accept` writes them",
                block.line
            ));
            continue;
        }
        for (want, got) in block.runs.iter().zip(&got) {
            let (want, got) = (normal(&want.output), normal(&strip(&got.output)));
            if want != got {
                problems.push(format!(
                    "{shown}:{}: `{}` printed something else:\n{}`jacket doc --accept` writes what it printed into the guide",
                    block.line,
                    block.runs[0].command,
                    diff(&want, &got, &format!("`{}`", block.runs[0].command))
                ));
                break;
            }
        }
    }
    if stored.blocks.len() > blocks.len() && !accept && problems.is_empty() {
        problems.push(format!(
            "{capture_shown}: it has more blocks than {shown}; `jacket doc --accept` writes it again"
        ));
    }
    if accept {
        if !rewrites.is_empty() {
            let mut md = md;
            for (range, text) in rewrites.into_iter().rev() {
                md.replace_range(range, &text);
            }
            std::fs::write(path, md)
                .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
        }
        if fresh != stored {
            let json = serde_json::to_string_pretty(&fresh).map_err(|e| e.to_string())?;
            std::fs::write(&capture_file, format!("{json}\n"))
                .map_err(|e| format!("cannot write {capture_shown}: {e}"))?;
        }
    }
    Ok(problems)
}

/// The coloured HTML of each console block of `md` whose capture beside `path` is current,
/// keyed by the block's text. Blocks without one are left out and render as plain text.
pub fn rendered(path: &Path, md: &str) -> HashMap<String, String> {
    let (Ok(blocks), Ok(Some(capture))) = (blocks(md), read_capture(&capture_path(path))) else {
        return HashMap::default();
    };
    blocks
        .iter()
        .zip(&capture.blocks)
        .filter(|(block, shot)| matches(block, shot))
        .map(|(block, shot)| (md[block.body.clone()].to_owned(), html(shot)))
        .collect()
}

/// A captured block as HTML: each command after a prompt, then its output in the colours it
/// was printed in.
fn html(runs: &[Run]) -> String {
    let mut out = String::new();
    for (n, run) in runs.iter().enumerate() {
        if n > 0 {
            out.push('\n');
        }
        out.push_str("<span class=\"prompt\">$ </span>");
        out.push_str(&esc(&run.command));
        if !run.output.is_empty() {
            out.push('\n');
            out.push_str(&ansi_html(&run.output));
        }
    }
    out
}

/// Text with SGR escape sequences as HTML spans: bold, dim, italic, underline and the 16
/// colours, as `ansi-*` classes the theme styles. Other sequences are dropped.
pub fn ansi_html(text: &str) -> String {
    #[derive(Clone, Copy, Default, PartialEq)]
    struct Pen {
        bold: bool,
        dim: bool,
        italic: bool,
        underline: bool,
        color: Option<u8>,
    }
    const NAMES: [&str; 8] = [
        "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
    ];
    let classes = |pen: Pen| {
        let mut c = Vec::new();
        if pen.bold {
            c.push("ansi-bold".to_owned());
        }
        if pen.dim {
            c.push("ansi-dim".to_owned());
        }
        if pen.italic {
            c.push("ansi-italic".to_owned());
        }
        if pen.underline {
            c.push("ansi-underline".to_owned());
        }
        if let Some(n) = pen.color {
            let bright = if n >= 8 {
                "bright-"
            } else {
                ""
            };
            c.push(format!("ansi-{bright}{}", NAMES[usize::from(n % 8)]));
        }
        c.join(" ")
    };
    let mut out = String::new();
    let mut pen = Pen::default();
    let mut chars = text.chars().peekable();
    let mut run = String::new();
    let flush = |out: &mut String, run: &mut String, pen: Pen| {
        if run.is_empty() {
            return;
        }
        if pen == Pen::default() {
            out.push_str(&esc(run));
        } else {
            out.push_str(&format!(
                "<span class=\"{}\">{}</span>",
                classes(pen),
                esc(run)
            ));
        }
        run.clear();
    };
    while let Some(c) = chars.next() {
        if c != '\x1b' || chars.peek() != Some(&'[') {
            run.push(c);
            continue;
        }
        chars.next();
        let mut params = String::new();
        let mut last = None;
        for c in chars.by_ref() {
            if c.is_ascii_alphabetic() {
                last = Some(c);
                break;
            }
            params.push(c);
        }
        if last != Some('m') {
            continue;
        }
        flush(&mut out, &mut run, pen);
        let mut next = pen;
        let codes: Vec<u8> = if params.is_empty() {
            vec![0]
        } else {
            params.split(';').filter_map(|p| p.parse().ok()).collect()
        };
        for code in codes {
            match code {
                0 => next = Pen::default(),
                1 => next.bold = true,
                2 => next.dim = true,
                3 => next.italic = true,
                4 => next.underline = true,
                22 => (next.bold, next.dim) = (false, false),
                23 => next.italic = false,
                24 => next.underline = false,
                30..=37 => next.color = Some(code - 30),
                39 => next.color = None,
                90..=97 => next.color = Some(code - 90 + 8),
                _ => {}
            }
        }
        pen = next;
    }
    flush(&mut out, &mut run, pen);
    out
}
