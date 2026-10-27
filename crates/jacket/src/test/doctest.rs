//! Doc tests: the ```` ```jai ```` blocks in a library's doc comments, each built and run as a
//! program of its own that imports the library.
//!
//! The doc comments come from `jaic -doc_comments`, which reads the library with the compiler's
//! parser: the docs above each constant declaration (also in `#if` branches) and the comment
//! that opens `module.jai` (the module's own docs). A block with a `main` is used as it is;
//! otherwise its lines are the body of a generated procedure, with its `#import` and `#load`
//! lines moved to file level. Either way the program runs the example through the Testing module,
//! so a failed `assert` or `expect` fails the doc test. Options follow `jai` on the fence:
//! `ignore`, `no_run`, `compile_fail`, `should_fail`, and the notes of a test (`only(linux)`).
use super::build::{Toolchain, compile_output};
use super::exec::{Ending, Live};
use super::golden::{self, Checked};
use super::jobs::ending_text;
use super::options::TestOptions;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

/// How a doc block is checked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Built and run; it must exit cleanly.
    Run,
    /// Type-checked only.
    NoRun,
    /// Must not compile.
    CompileFail,
    /// Built and run; it must fail (a failed check, an assert, a crash or a nonzero exit).
    ShouldFail,
}

/// One example from a doc comment.
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    /// `Module.symbol#n`, or `Module#n` for the module's own docs.
    pub name: String,
    pub file: PathBuf,
    /// The line of the opening fence.
    pub line: u32,
    pub code: String,
    pub mode: Mode,
    /// `ignore` and the test notes on the fence (`only(linux)`).
    pub options: TestOptions,
    /// Fence words that are not options, to warn about.
    pub problems: Vec<String>,
}

const ENTRY_POINT: &str = "__jacket_doc_main";

/// One doc comment as `jaic -doc_comments` prints it.
struct DocComment {
    file: PathBuf,
    line: u32,
    /// `None` for the module's own docs.
    symbol: Option<String>,
    text: String,
}

/// `field` with the escapes of `jaic -doc_comments` undone.
fn unescape(field: &str) -> String {
    let mut out = String::with_capacity(field.len());
    let mut chars = field.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }
    out
}

fn parse_records(text: &str) -> Vec<DocComment> {
    text.lines()
        .filter_map(|record| {
            let mut fields = record.split('\t');
            let file = PathBuf::from(unescape(fields.next()?));
            let line = fields.next()?.parse().ok()?;
            let symbol = unescape(fields.next()?);
            let text = unescape(fields.next()?);
            Some(DocComment {
                file,
                line,
                symbol: (!symbol.is_empty()).then_some(symbol),
                text,
            })
        })
        .collect()
}

/// Every doc block of the library `module` in `dir`, as `jaic -doc_comments` finds its docs. A
/// file in `seen` was read for an earlier library and is not read again.
pub fn blocks(dir: &Path, module: &str, jaic: &Path, seen: &mut BTreeSet<PathBuf>) -> Vec<Block> {
    let output = Command::new(jaic).arg("-doc_comments").arg(dir).output();
    let records = match output {
        Ok(output) if output.status.success() => {
            parse_records(&String::from_utf8_lossy(&output.stdout))
        }
        Ok(output) => {
            eprintln!(
                "jacket: warning: could not read the doc comments of {}: {}",
                dir.display(),
                String::from_utf8_lossy(&output.stderr).trim()
            );
            return Vec::new();
        }
        Err(error) => {
            eprintln!(
                "jacket: warning: could not start `{}`: {error}",
                jaic.display()
            );
            return Vec::new();
        }
    };
    let mut claimed: BTreeSet<PathBuf> = BTreeSet::new();
    let mut counts: BTreeMap<String, u32> = BTreeMap::new();
    let mut out = Vec::new();
    for comment in records {
        if seen.contains(&comment.file) {
            continue;
        }
        claimed.insert(comment.file.clone());
        let name = match &comment.symbol {
            Some(symbol) => format!("{module}.{symbol}"),
            None => module.to_string(),
        };
        let lines: Vec<&str> = comment.text.lines().collect();
        for (offset, block) in fenced(&lines) {
            let n = counts.entry(name.clone()).or_insert(0);
            *n += 1;
            out.push(Block {
                name: format!("{name}#{n}"),
                file: comment.file.clone(),
                line: comment.line + u32::try_from(offset).unwrap_or(0),
                ..block
            });
        }
    }
    seen.extend(claimed);
    out
}

/// The words of a fence's info string, split at commas and spaces outside parentheses:
/// `jai,only(linux, macos) no_run` is `jai`, `only(linux, macos)`, `no_run`.
fn fence_words(info: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut depth = 0usize;
    for c in info.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            _ => {}
        }
        if depth == 0 && (c == ',' || c.is_whitespace()) {
            if !word.is_empty() {
                words.push(std::mem::take(&mut word));
            }
        } else {
            word.push(c);
        }
    }
    if !word.is_empty() {
        words.push(word);
    }
    words
}

/// The name a declaration line declares (`split :: (`), if it is one.
fn declared(line: &str) -> Option<&str> {
    let line = line.trim_start();
    let end = line
        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
        .unwrap_or(line.len());
    let (name, rest) = line.split_at(end);
    let starts_well = name
        .chars()
        .next()
        .is_some_and(|c| c.is_alphabetic() || c == '_');
    (starts_well && rest.trim_start().starts_with("::")).then_some(name)
}

/// The ```` ```jai ```` blocks of a doc comment's lines, with the line each fence opens on.
fn fenced(lines: &[&str]) -> Vec<(usize, Block)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let open = lines[i].trim_start();
        let indent = lines[i].len() - open.len();
        let Some(info) = open.strip_prefix("```") else {
            i += 1;
            continue;
        };
        let mut words = fence_words(info).into_iter();
        let runs = words.next().as_deref() == Some("jai");
        let start = i;
        let mut code = String::new();
        i += 1;
        while i < lines.len() && lines[i].trim() != "```" {
            let line = lines[i];
            let cut = line.len() - line.trim_start().len();
            code.push_str(&line[cut.min(indent)..]);
            code.push('\n');
            i += 1;
        }
        i += 1;
        if !runs {
            continue;
        }
        let mut block = Block {
            name: String::new(),
            file: PathBuf::new(),
            line: 0,
            code,
            mode: Mode::Run,
            options: TestOptions::default(),
            problems: Vec::new(),
        };
        for word in words {
            match word.as_str() {
                "ignore" => block.options.ignore = true,
                "no_run" => block.mode = Mode::NoRun,
                "compile_fail" => block.mode = Mode::CompileFail,
                "should_fail" => block.mode = Mode::ShouldFail,
                // The notes of a test: `only(linux)`, `skip_on(wasm)`, `timeout(5)`, ...
                note => {
                    if let Err(problem) = block.options.apply(note) {
                        block.problems.push(problem.replace("note `@", "option `"));
                    }
                }
            }
        }
        out.push((start, block));
    }
    out
}

/// Whether `code` has a file-level `main`.
fn has_main(code: &str) -> bool {
    code.lines()
        .any(|l| !l.starts_with([' ', '\t']) && declared(l) == Some("main"))
}

fn imports(code: &str, module: &str) -> bool {
    code.lines().any(|l| {
        let l = l.trim();
        l.starts_with("#import") && l.contains(&format!("\"{module}\""))
    })
}

/// The program that runs `block` of the library `module`, and for each of its lines the doc comment line it came from (0 for a line
/// jacket added).
fn generate(block: &Block, module: &str) -> (String, Vec<u32>) {
    let mut out = String::new();
    let mut lines = Vec::new();
    let mut push = |text: &str, from: u32| {
        for line in text.lines() {
            out.push_str(line);
            out.push('\n');
            lines.push(from);
        }
    };
    push(
        &format!(
            "// Written by `jacket test`: the example at {}:{}.",
            block.file.display(),
            block.line
        ),
        0,
    );
    if !imports(&block.code, "Basic") {
        push("#import \"Basic\";", 0);
    }
    if !imports(&block.code, module) {
        push(&format!("#import \"{module}\";"), 0);
    }
    // The block's lines follow its fence.
    let code: Vec<(&str, u32)> = block.code.lines().zip(block.line + 1..).collect();
    let wrapped = !has_main(&block.code);
    if wrapped {
        for &(line, from) in &code {
            let trimmed = line.trim_start();
            if trimmed.starts_with("#import") || trimmed.starts_with("#load") {
                push(trimmed, from);
            }
        }
    } else {
        for &(line, from) in &code {
            push(line, from);
        }
    }
    push("__jacket_doc_testing :: #import \"Extensions/Testing\";", 0);
    push(
        "__jacket_doc_example :: (t: *__jacket_doc_testing.Test) {",
        0,
    );
    if wrapped {
        for &(line, from) in &code {
            let trimmed = line.trim_start();
            if !(trimmed.starts_with("#import") || trimmed.starts_with("#load")) {
                push(&format!("    {line}"), from);
            }
        }
    } else {
        push("    main();", 0);
    }
    push(
        &format!(
            "}}\n\
             {ENTRY_POINT} :: () {{\n    \
             __jacket_doc_testing.run_example(__jacket_doc_example);\n}}\n\
             #run {{\n    \
             C :: #import \"Compiler\";\n    \
             options := C.get_build_options();\n    \
             options.entry_point_name = \"{ENTRY_POINT}\";\n    \
             C.set_build_options(options);\n}}"
        ),
        0,
    );
    (out, lines)
}

/// `text` with each `.../doc.jai:N` that came from the doc comment written as its place there.
fn relabel(text: &str, lines: &[u32], file: &Path) -> String {
    const NAME: &str = "doc.jai:";
    let mut out = String::new();
    let mut rest = text;
    while let Some(at) = rest.find(NAME) {
        let digits_at = at + NAME.len();
        let digits = rest[digits_at..]
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(rest.len() - digits_at);
        let line: usize = rest[digits_at..digits_at + digits].parse().unwrap_or(0);
        // The path before the name runs back to a space or the start of the line.
        let path_at = rest[..at]
            .rfind(|c: char| c.is_whitespace())
            .map_or(0, |n| n + 1);
        match line.checked_sub(1).and_then(|i| lines.get(i)) {
            Some(&from) if from > 0 => {
                out.push_str(&rest[..path_at]);
                out.push_str(&format!("{}:{from}", file.display()));
            }
            _ => out.push_str(&rest[..digits_at + digits]),
        }
        rest = &rest[digits_at + digits..];
    }
    out.push_str(rest);
    out
}

/// Builds `block` in `work` and checks it as its mode says. Diagnostics and stack traces name
/// the doc comment's lines, not the generated program's.
#[allow(clippy::too_many_arguments)]
pub fn check(
    block: &Block,
    module: &str,
    root_flags: &[String],
    toolchain: &Toolchain,
    work: &Path,
    cwd: &Path,
    timeout: Option<Duration>,
    live: &mut dyn FnMut(Live),
) -> Checked {
    let lines = generate(block, module).1;
    let mut checked = check_program(
        block, module, root_flags, toolchain, work, cwd, timeout, live,
    );
    for problem in &mut checked.problems {
        *problem = relabel(problem, &lines, &block.file);
    }
    for stream in [&mut checked.stdout, &mut checked.stderr] {
        let text = relabel(&String::from_utf8_lossy(stream), &lines, &block.file);
        *stream = text.into_bytes();
    }
    checked
}

#[allow(clippy::too_many_arguments)]
fn check_program(
    block: &Block,
    module: &str,
    root_flags: &[String],
    toolchain: &Toolchain,
    work: &Path,
    cwd: &Path,
    timeout: Option<Duration>,
    live: &mut dyn FnMut(Live),
) -> Checked {
    let mut checked = Checked::default();
    let source = work.join("doc.jai");
    let (text, _) = generate(block, module);
    if let Err(error) = std::fs::write(&source, text) {
        checked
            .problems
            .push(format!("cannot write {}: {error}", source.display()));
        return checked;
    }
    if matches!(block.mode, Mode::NoRun | Mode::CompileFail) {
        let output = Command::new(&toolchain.jaic)
            .arg(&source)
            .arg("-check")
            .arg("-quiet")
            .args(&toolchain.flags)
            .args(root_flags)
            .current_dir(work)
            .output();
        match output {
            Err(error) => checked.problems.push(format!(
                "could not start `{}`: {error}",
                toolchain.jaic.display()
            )),
            Ok(output) if block.mode == Mode::NoRun && !output.status.success() => checked
                .problems
                .push(format!("it does not compile:\n{}", compile_output(&output))),
            Ok(output) if block.mode == Mode::CompileFail && output.status.success() => checked
                .problems
                .push("it compiled; `compile_fail` expects it not to".into()),
            Ok(_) => {}
        }
        checked.passed = checked.problems.is_empty();
        return checked;
    }
    let invocation = match golden::invocation(&source, root_flags, toolchain, work, cwd) {
        Ok(invocation) => invocation,
        Err(problem) => {
            checked.problems.push(problem);
            return checked;
        }
    };
    let outcome = golden::run_plain(&invocation, timeout, live);
    checked.stdout = outcome.preamble_stdout;
    checked.stderr = outcome.preamble;
    let clean = outcome.ending == Ending::Exited(0);
    if outcome.ending == Ending::TimedOut {
        checked.timed_out = true;
        checked.problems.push("it timed out".into());
    } else if block.mode == Mode::ShouldFail && clean {
        checked
            .problems
            .push("it passed; `should_fail` expects it to fail".into());
    } else if block.mode == Mode::Run && !clean {
        checked
            .problems
            .push(format!("the example {}", ending_text(&outcome.ending)));
    }
    checked.passed = checked.problems.is_empty();
    checked
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_become_blocks() {
        let text = "/a.jai\t4\tsplit\tSplits.\\n\\n```jai,no_run\\nx := 1;\\n```\\n```\\nnot jai\\n```\\n```jai ignore bogus\\ny := 2;\\n```\n/a.jai\t1\t\tModule.\\n```jai\\nexpect(true);\\n```\n";
        let comments = parse_records(text);
        assert_eq!(comments.len(), 2);
        assert_eq!(comments[0].symbol.as_deref(), Some("split"));
        assert_eq!(comments[1].symbol, None);
        let lines: Vec<&str> = comments[0].text.lines().collect();
        let blocks = fenced(&lines);
        let summary: Vec<(usize, Mode, bool)> = blocks
            .iter()
            .map(|(at, b)| (*at, b.mode, b.options.ignore))
            .collect();
        assert_eq!(summary, [(2, Mode::NoRun, false), (8, Mode::Run, true)]);
        assert_eq!(blocks[1].1.problems, ["unknown option `bogus` (ignored)"]);
    }

    #[test]
    fn fence_words_keep_parentheses_whole() {
        assert_eq!(
            fence_words("jai,only(linux, macos) no_run"),
            ["jai", "only(linux, macos)", "no_run"]
        );
        let blocks = fenced(&["```jai,skip_on(wasm),timeout(3)", "x := 1;", "```"]);
        assert_eq!(blocks[0].1.options.skip_on[0].names, ["wasm"]);
        assert_eq!(blocks[0].1.options.timeout, Some(3));
    }

    #[test]
    fn bare_statements_become_a_procedure_body() {
        let block = Block {
            name: "M.f#1".into(),
            file: "/m/module.jai".into(),
            line: 4,
            code: "#import \"Math\";\nx := 1;\n".into(),
            mode: Mode::Run,
            options: TestOptions::default(),
            problems: Vec::new(),
        };
        let (text, lines) = generate(&block, "M");
        assert!(text.contains("#import \"Basic\";\n#import \"M\";\n#import \"Math\";\n"));
        let x = text.lines().position(|l| l == "    x := 1;").unwrap();
        assert_eq!(lines[x], 6);
        assert_eq!(
            relabel(
                &format!("/w/doc.jai:{}:5: error: no\nat /w/doc.jai:1", x + 1),
                &lines,
                Path::new("/m/module.jai")
            ),
            "/m/module.jai:6:5: error: no\nat /w/doc.jai:1"
        );
        assert!(text.contains("(t: *__jacket_doc_testing.Test) {\n    x := 1;\n}"));
        let with_main = Block {
            code: "#import \"M\";\nmain :: () {}\n".into(),
            ..block
        };
        let text = generate(&with_main, "M").0;
        assert!(!text.contains("#import \"M\";\n#import \"M\""));
        assert!(text.contains("main :: () {}\n"));
        assert!(text.contains("{\n    main();\n}"));
    }
}
