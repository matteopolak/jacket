//! Which identifiers of a project get no hover or no go to definition from jailsp.
//!
//! `cargo run --release -p jai-language-server --example ide_coverage -- [--list] [--errors] [--stats] [--stdlib DIR] DIR...`
//!
//! Each DIR is opened as a workspace folder with all of its `.jai` files open, as an editor
//! would. Every identifier and directive outside comments and strings is asked for hover, and
//! every use (not a declaration) for go to definition. A hover that is only the syntax layer's
//! fallback counts as missing. Files with a type-checker error are reported apart: past the error some
//! facts are expected to be missing. See docs/tools/ide-coverage.md.
use jai_language_server::{
    DiagnosticCode, DiagnosticSeverity, DocumentUri, Environment, Limits, Position, Session,
};
use jaic::lexer::{P, Tok};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const KEYWORDS: &[&str] = &[
    "if",
    "else",
    "then",
    "ifx",
    "for",
    "while",
    "break",
    "continue",
    "return",
    "defer",
    "using",
    "struct",
    "union",
    "enum",
    "enum_flags",
    "case",
    "cast",
    "xx",
    "inline",
    "no_inline",
    "push_context",
    "remove",
    "it",
    "it_index",
    "null",
    "true",
    "false",
    "size_of",
    "type_of",
    "type_info",
    "is_constant",
    "initializer_of",
    "operator",
    "interface",
    "context",
    "Type",
    "Code",
    "void",
    "bool",
    "int",
    "u8",
    "u16",
    "u32",
    "u64",
    "s8",
    "s16",
    "s32",
    "s64",
    "float",
    "float32",
    "float64",
    "string",
    "Any",
];

/// What an identifier is doing, from its neighbours.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Role {
    /// `name ::`, `name :=`, `name:` (and parameters).
    Declaration,
    /// `f(name = 1)`, `.{name = 1}`.
    Named,
    /// `a.name`, `.NAME`.
    Member,
    Use,
    /// `#name` (hover only).
    Directive,
}

struct Gap {
    line: usize,
    column: usize,
    name: String,
    role: Role,
    hover: bool,
    definition: bool,
    context: String,
}

fn environment(stdlib: PathBuf) -> Environment {
    Environment {
        fs: std::rc::Rc::new(jaic::sema::NativeFs),
        options: Box::new(move |_| {
            let mut options = jaic::sema::Options::host();
            options.import_paths = vec![stdlib.clone()];
            options.preload = Some(stdlib.join("prelude/module.jai"));
            options
        }),
    }
}

fn jai_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().map(|e| e.path()).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            jai_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "jai") {
            out.push(path);
        }
    }
}

/// Byte offsets where each line starts, for `position`.
fn line_starts(text: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(text.match_indices('\n').map(|(i, _)| i + 1))
        .collect()
}

fn position(text: &str, starts: &[usize], byte: usize) -> Position {
    let line = starts.partition_point(|&s| s <= byte) - 1;
    let character = text[starts[line]..byte].encode_utf16().count();
    Position {
        line: line as u32,
        character: character as u32,
    }
}

fn role(tokens: &[jaic::lexer::Token], i: usize) -> Role {
    let next = tokens.get(i + 1).map(|t| &t.tok);
    let prev = i.checked_sub(1).map(|j| &tokens[j].tok);
    if matches!(prev, Some(Tok::Punct(P::Dot))) {
        return Role::Member;
    }
    match next {
        Some(Tok::Punct(P::ColonColon | P::Colon | P::ColonEq)) => Role::Declaration,
        Some(Tok::Punct(P::Eq))
            if matches!(
                prev,
                Some(Tok::Punct(P::LParen | P::Comma | P::LBrace | P::DotBrace))
            ) =>
        {
            Role::Named
        }
        _ => Role::Use,
    }
}

fn main() {
    let mut list = false;
    let mut show_errors = false;
    let mut stats = false;
    let mut stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut dirs = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--list" => list = true,
            "--errors" => show_errors = true,
            "--stats" => stats = true,
            "--stdlib" => stdlib = PathBuf::from(args.next().expect("--stdlib DIR")),
            _ => dirs.push(PathBuf::from(arg)),
        }
    }
    let mut totals = BTreeMap::<(&str, Role), usize>::new();
    for dir in dirs {
        let dir = std::fs::canonicalize(&dir).expect("a directory");
        let mut files = Vec::new();
        jai_files(&dir, &mut files);
        let mut session = Session::with_environment(Limits::default(), environment(stdlib.clone()));
        session.set_workspace_folders(vec![dir.clone()]);
        let mut opened = Vec::new();
        for path in &files {
            let Ok(text) = std::fs::read_to_string(path) else {
                continue;
            };
            // `16.4_check_#as.jai`: a `#` or `?` would start a fragment or query.
            let encoded: String = path
                .display()
                .to_string()
                .chars()
                .map(|c| match c {
                    '%' => "%25".into(),
                    '#' => "%23".into(),
                    '?' => "%3F".into(),
                    ' ' => "%20".into(),
                    c => c.to_string(),
                })
                .collect();
            let uri = DocumentUri::parse(&format!("file://{encoded}")).unwrap();
            if session.open(uri.clone(), 1, text.clone()).is_ok() {
                opened.push((path.clone(), uri, text));
            }
        }
        for (path, uri, text) in &opened {
            let started = std::time::Instant::now();
            let starts = line_starts(text);
            let compiles = session.compile_counts().0;
            let errors: Vec<_> = session
                .diagnostics(uri)
                .unwrap_or_default()
                .into_iter()
                .filter(|d| {
                    d.code == DiagnosticCode::Check && d.severity == DiagnosticSeverity::Error
                })
                .collect();
            let errored = !errors.is_empty();
            let Ok(tokens) = jaic::lexer::lex(jaic::source::FileId(0), text) else {
                continue;
            };
            let mut gaps = Vec::new();
            let mut idents = 0;
            for (i, token) in tokens.iter().enumerate() {
                let (name, role) = match &token.tok {
                    // `#import,file`, `#type,isa`: a directive's flags.
                    Tok::Ident(_)
                        if i >= 2
                            && matches!(tokens[i - 1].tok, Tok::Punct(P::Comma))
                            && matches!(tokens[i - 2].tok, Tok::Directive(_)) =>
                    {
                        continue;
                    }
                    Tok::Ident(name) if !KEYWORDS.contains(&name.as_str()) => {
                        (name.to_string(), role(&tokens, i))
                    }
                    Tok::Directive(name) => (format!("#{name}"), Role::Directive),
                    _ => continue,
                };
                idents += 1;
                let start = token.span.start as usize;
                let at = position(text, &starts, start + 1);
                let hover = session
                    .hover(uri, at)
                    .ok()
                    .flatten()
                    .is_some_and(|h| !h.contents.value.contains("Source syntax declaration"));
                let definition = matches!(role, Role::Declaration | Role::Directive)
                    || session.definition(uri, at).is_ok_and(|d| !d.is_empty());
                if hover && definition {
                    continue;
                }
                let line_start = text[..start].rfind('\n').map_or(0, |n| n + 1);
                let line_end = text[start..].find('\n').map_or(text.len(), |n| start + n);
                gaps.push(Gap {
                    line: at.line as usize + 1,
                    column: start - line_start + 1,
                    name,
                    role,
                    hover,
                    definition,
                    context: text[line_start..line_end].trim().to_string(),
                });
            }
            let shown = path.strip_prefix(&dir).unwrap_or(path).display();
            let kind = if errored {
                "errored"
            } else {
                "clean"
            };
            println!(
                "{}/{shown}: {idents} identifiers, {} without hover, {} without definition{}",
                dir.file_name().unwrap_or_default().to_string_lossy(),
                gaps.iter().filter(|g| !g.hover).count(),
                gaps.iter().filter(|g| !g.definition).count(),
                if errored {
                    " (has a type-checker error)"
                } else {
                    ""
                },
            );
            if stats {
                println!(
                    "  {} compiles, {} ms",
                    session.compile_counts().0 - compiles,
                    started.elapsed().as_millis()
                );
            }
            if show_errors {
                for e in &errors {
                    let first = e.message.lines().next().unwrap_or_default();
                    println!(
                        "  {}:{} error: {first}",
                        e.range.start.line + 1,
                        e.range.start.character + 1
                    );
                }
            }
            for g in &gaps {
                if !g.hover {
                    *totals.entry((kind, g.role)).or_default() += 1;
                }
                if list {
                    println!(
                        "  {}:{} {:?} `{}`{}{}  | {}",
                        g.line,
                        g.column,
                        g.role,
                        g.name,
                        if g.hover {
                            ""
                        } else {
                            " no-hover"
                        },
                        if g.definition {
                            ""
                        } else {
                            " no-definition"
                        },
                        g.context
                    );
                }
            }
        }
    }
    println!("\nidentifiers without hover, by file kind and role:");
    for ((kind, role), n) in totals {
        println!("  {kind:8} {role:?}: {n}");
    }
}
