//! Language-server oracle checks: the compiler is the oracle for what the server answers on files
//! `jaic -check` accepts (see `docs/tools/lsp-oracle.md`; `tools/lsp/oracle.py` picks the files).
//!
//! ```sh
//! cargo run --release -p jai-language-server --example lsp_oracle -- [--jaifmt PATH] FILE...
//! ```
//!
//! For each file: (a) no error diagnostics, and its unreachable regions listed for a run check;
//! (b) every identifier the compiler resolved gets a hover and, where it is declared in a file, a
//! definition there; (c) blank lines and `jaifmt` change nothing beyond positions, and editing the
//! session into such a text equals opening it fresh; (d) completion at member accesses and struct
//! literals lists what the code uses. One JSON object per file on stdout.
use jai_language_server::{DocumentUri, Limits, Position, Session};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
#[path = "support/typing_files.rs"]
mod typing_files;

#[path = "lsp_oracle/checks.rs"]
mod checks;
#[path = "lsp_oracle/metamorphic.rs"]
mod metamorphic;
#[path = "lsp_oracle/oracle.rs"]
mod oracle;

#[derive(Serialize)]
pub struct Finding {
    pub check: &'static str,
    /// 1-based, as editors show it.
    pub line: u32,
    pub character: u32,
    pub name: String,
    pub detail: String,
}

impl Finding {
    pub fn new(check: &'static str, at: Position, name: &str, detail: String) -> Self {
        Self {
            check,
            line: at.line + 1,
            character: at.character,
            name: name.to_owned(),
            detail,
        }
    }
}

#[derive(Serialize, Default)]
struct Counts {
    resolved: usize,
    probed_resolutions: usize,
    probed_completions: usize,
    literals: usize,
    variants: usize,
}

#[derive(Serialize)]
struct FileReport {
    path: String,
    /// `checked`, or why the file was left out (`compile`: the oracle compile failed).
    status: String,
    detail: Option<String>,
    findings: Vec<Finding>,
    dead: Vec<checks::Dead>,
    counts: Counts,
    millis: u128,
}

pub fn pos_of(text: &str, byte: usize) -> Position {
    let before = &text[..byte];
    Position {
        line: before.bytes().filter(|&b| b == b'\n').count() as u32,
        character: before
            .rsplit('\n')
            .next()
            .map_or(0, jaic::source::utf16_len),
    }
}

pub fn byte_of(text: &str, position: Position) -> Option<usize> {
    let mut start = 0;
    for _ in 0..position.line {
        start += text[start..].find('\n')? + 1;
    }
    let line = &text[start..];
    let line = &line[..line.find('\n').unwrap_or(line.len())];
    let mut units = 0;
    for (byte, ch) in line.char_indices() {
        if units == position.character {
            return Some(start + byte);
        }
        units += ch.len_utf16() as u32;
    }
    (units == position.character).then_some(start + line.len())
}

struct Args {
    stdlib: PathBuf,
    jaifmt: Option<PathBuf>,
    checks: String,
    probes: usize,
    /// `LINE:COL` (1-based line, as findings print): print what the server answers there.
    probe: Option<(u32, u32)>,
    files: Vec<PathBuf>,
}

fn parse_args() -> Result<Args, String> {
    let mut parsed = Args {
        stdlib: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib"),
        jaifmt: None,
        checks: "abcd".into(),
        probes: 300,
        probe: None,
        files: Vec::new(),
    };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or_else(|| format!("{arg} needs a value"));
        match arg.as_str() {
            "--stdlib" => parsed.stdlib = PathBuf::from(value()?),
            "--jaifmt" => parsed.jaifmt = Some(PathBuf::from(value()?)),
            "--checks" => parsed.checks = value()?,
            "--probes" => parsed.probes = value()?.parse().map_err(|_| "--probes N")?,
            "--probe" => {
                let v = value()?;
                let (l, c) = v.split_once(':').ok_or("--probe LINE:COL")?;
                parsed.probe = Some((
                    l.parse().map_err(|_| "--probe LINE:COL")?,
                    c.parse().map_err(|_| "--probe LINE:COL")?,
                ));
            }
            "--list" => {
                let list = std::fs::read_to_string(value()?).map_err(|e| e.to_string())?;
                parsed
                    .files
                    .extend(list.lines().filter(|l| !l.is_empty()).map(PathBuf::from));
            }
            "--help" => {
                return Err("lsp_oracle [--stdlib DIR] [--jaifmt PATH] [--checks abcd] [--probes N] [--probe LINE:COL] [--list FILE] FILE...".into());
            }
            _ if arg.starts_with('-') => return Err(format!("unknown option {arg}")),
            _ => parsed.files.push(PathBuf::from(arg)),
        }
    }
    parsed.stdlib = parsed
        .stdlib
        .canonicalize()
        .map_err(|e| format!("stdlib: {e}"))?;
    Ok(parsed)
}

/// A session with `text` open as `path`, made the entry by a `jacket.toml` beside it, as an editor
/// opening a project's main file has it.
fn open(args: &Args, path: &Path, uri: &DocumentUri, text: &str) -> Option<Session> {
    let mut session = Session::with_environment(
        Limits::default(),
        typing_files::native_environment(args.stdlib.clone()),
    );
    let config = DocumentUri::from_path(&path.parent()?.join("jacket.toml")).ok()?;
    let entry = serde_json::to_string(path.file_name()?.to_str()?).ok()?;
    session
        .open(config, 1, format!("[project]\nentry = [{entry}]\n"))
        .ok()?;
    session.set_auto_import(false);
    session.open(uri.clone(), 1, text.to_owned()).ok()?;
    Some(session)
}

fn check_file(args: &Args, path: &Path) -> FileReport {
    let started = std::time::Instant::now();
    let mut report = FileReport {
        path: path.display().to_string(),
        status: "checked".into(),
        detail: None,
        findings: Vec::new(),
        dead: Vec::new(),
        counts: Counts::default(),
        millis: 0,
    };
    let skip = |mut report: FileReport, status: &str, detail: String| {
        report.status = status.into();
        report.detail = Some(detail);
        report.millis = started.elapsed().as_millis();
        report
    };
    let Ok(path) = path.canonicalize() else {
        return skip(report, "missing", String::new());
    };
    let Ok(text) = std::fs::read_to_string(&path) else {
        return skip(report, "unreadable", String::new());
    };
    let env = typing_files::native_environment(args.stdlib.clone());
    let oracle = match oracle::compile((env.options)(&path), &path, &text) {
        Ok(o) => o,
        Err(e) => return skip(report, "compile", e),
    };
    report.counts.resolved = oracle.refs.len();
    report.counts.literals = oracle.literals.len();
    let Ok(uri) = DocumentUri::from_path(&path) else {
        return skip(report, "uri", String::new());
    };
    let Some(mut session) = open(args, &path, &uri, &text) else {
        return skip(report, "open", String::new());
    };
    if let Some((line, character)) = args.probe {
        let at = Position {
            line: line.saturating_sub(1),
            character,
        };
        let shown = format!(
            "hover: {:?}\ndefinition: {:?}\ncompletion: {:?}",
            session.hover(&uri, at),
            session.definition(&uri, at),
            session.completion(&uri, at).map(|l| l
                .items
                .into_iter()
                .map(|i| i.label)
                .collect::<Vec<_>>()),
        );
        return skip(report, "probe", shown);
    }
    let has = |c: char| args.checks.contains(c);
    if has('a') {
        checks::diagnostics(
            &session,
            &uri,
            &text,
            &mut report.findings,
            &mut report.dead,
        );
    }
    if has('b') {
        report.counts.probed_resolutions = checks::resolutions(
            &session,
            &uri,
            &text,
            &oracle,
            args.probes,
            &mut report.findings,
        );
    }
    if has('d') {
        report.counts.probed_completions = checks::completions(
            &session,
            &uri,
            &text,
            &oracle,
            args.probes / 3,
            &mut report.findings,
        );
    }
    if has('c') {
        let fresh = |t: &str| open(args, &path, &uri, t);
        let cx = metamorphic::Context {
            uri: &uri,
            text: &text,
            fresh: &fresh,
            probes: args.probes / 2,
        };
        let mut version = 1;
        let blank = metamorphic::with_blank_lines(&text);
        if metamorphic::compare(
            &cx,
            "c-blank",
            &blank,
            &mut session,
            &mut version,
            &mut report.findings,
            false,
        ) {
            report.counts.variants += 1;
        }
        if let Some(formatted) = args
            .jaifmt
            .as_deref()
            .and_then(|j| metamorphic::formatted(j, &text))
            && let Some(mut session) = open(args, &path, &uri, &text)
        {
            let mut version = 1;
            if metamorphic::compare(
                &cx,
                "c-format",
                &formatted,
                &mut session,
                &mut version,
                &mut report.findings,
                true,
            ) {
                report.counts.variants += 1;
            }
        }
    }
    report.millis = started.elapsed().as_millis();
    report
}

fn main() {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };
    let mut failed = false;
    for path in &args.files {
        let report = check_file(&args, path);
        failed |= !report.findings.is_empty();
        println!("{}", serde_json::to_string(&report).expect("JSON report"));
    }
    std::process::exit(i32::from(failed));
}
