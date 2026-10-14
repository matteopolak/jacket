//! Deterministic correctness checks shared by the corpus runner and fixture tests.
use jai_language_server::{
    DiagnosticSeverity, DocumentUri, Environment, Limits, Position, Range, Session, TextChange,
};
use jaic::{ast::StmtKind, source::FileId};
use serde::Serialize;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

// Context is application-defined and may itself be edited. These independent
// probes explicitly have no context dependency.
const HEAD: &str = "__lsp_edit_head :: () #no_context {\n    __head_local := 17;\n    __head_result := __head_local + 1;\n}\n";
const TAIL: &str = "\n__lsp_edit_tail :: () #no_context {\n    __tail_local := 23;\n    __tail_result := __tail_local + 1;\n    for __tail_index: 0..1 { __tail_loop := __tail_index; }\n}\n";

#[derive(Default, Serialize, Debug)]
pub struct Report {
    pub file: String,
    pub baseline_syntax_errors: usize,
    pub baseline_semantic_errors: usize,
    pub utf8_lossy: bool,
    pub edits: usize,
    pub assertions: usize,
    pub scope_changes: Vec<String>,
    pub failures: Vec<String>,
}

static PROGRESS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn set_progress(enabled: bool) {
    PROGRESS.store(enabled, std::sync::atomic::Ordering::Relaxed);
}

fn progress(phase: &str) {
    if PROGRESS.load(std::sync::atomic::Ordering::Relaxed) {
        use std::io::Write;
        println!("{}", serde_json::json!({"progress": phase}));
        std::io::stdout().flush().unwrap();
    }
}

fn position(text: &str, byte: usize) -> Position {
    let before = &text[..byte];
    Position {
        line: before.matches('\n').count() as u32,
        character: before.rsplit('\n').next().unwrap().encode_utf16().count() as u32,
    }
}

fn file_uri(path: &Path) -> DocumentUri {
    let mut encoded = String::new();
    for byte in path.to_string_lossy().bytes() {
        if byte.is_ascii_alphanumeric() || b"/-._~:".contains(&byte) {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    DocumentUri::parse(&format!("file://{encoded}")).unwrap()
}

fn environment(stdlib: Option<PathBuf>) -> Environment {
    Environment {
        fs: std::rc::Rc::new(jaic::sema::NativeFs),
        options: Box::new(move |_| {
            let mut options = jaic::sema::Options::host();
            // No generated executable is needed. The producer's canonical prelude is
            // optional, and the harness never reconstructs a legacy bootstrap.
            options.runtime_support = false;
            options.stack_trace = false;
            if let Some(dir) = &stdlib {
                options.import_paths = vec![dir.clone()];
                options.preload = Some(dir.join("prelude/module.jai"));
            }
            options
        }),
    }
}

/// Cut points include byte-oriented fractions, scalar boundaries, and the closest
/// valid boundary to the UTF-16 midpoint. No invalid UTF-8 or split surrogate is sent.
pub fn cuts(line: &str) -> BTreeSet<usize> {
    let mut cuts = BTreeSet::new();
    for byte in [line.len() / 3, line.len() / 2, line.len().saturating_sub(1)] {
        cuts.insert(line.floor_char_boundary(byte));
    }
    cuts.insert(line.len() - line.trim_start().len());
    let middle = line.encode_utf16().count() / 2;
    let mut units = 0;
    for (byte, ch) in line.char_indices() {
        if units >= middle {
            cuts.insert(byte);
            break;
        }
        units += ch.len_utf16();
    }
    cuts.retain(|&byte| byte < line.len());
    cuts
}

fn require(report: &mut Report, yes: bool, context: &str, message: &str) {
    report.assertions += 1;
    if !yes {
        report.failures.push(format!("{context}: {message}"));
    }
}

fn anchors(s: &Session, uri: &DocumentUri, text: &str, report: &mut Report, context: &str) {
    progress(&format!("{context}: document symbols"));
    let symbols = s.document_symbols(uri).unwrap();
    for (name, local, value) in [
        ("__lsp_edit_head", "__head_local", 17),
        ("__lsp_edit_tail", "__tail_local", 23),
    ] {
        require(
            report,
            symbols.iter().any(|sym| sym.name == name),
            context,
            &format!("lost document symbol {name}"),
        );
        let use_byte = text.find(&format!("{local} + 1")).unwrap();
        let pos = position(text, use_byte);
        progress(&format!("{context}: hover {local}"));
        let hover = s.hover(uri, pos).unwrap();
        require(
            report,
            hover
                .as_ref()
                .is_some_and(|h| h.contents.value == format!("{local}: s64")),
            context,
            &format!("lost typed hover {local}: {hover:?}"),
        );
        progress(&format!("{context}: definition {local}"));
        let defs = s.definition(uri, pos).unwrap();
        let expected = position(text, text.find(&format!("{local} := {value}")).unwrap());
        require(
            report,
            defs.len() == 1 && defs[0].uri == uri.as_str() && defs[0].range.start == expected,
            context,
            &format!("lost or wrong definition {local}: {defs:?}"),
        );
        progress(&format!("{context}: completion {local}"));
        let completions = s
            .completion(uri, position(text, use_byte + local.len()))
            .unwrap();
        require(
            report,
            completions
                .items
                .iter()
                .any(|i| i.label == local && i.detail.contains("s64")),
            context,
            &format!("lost typed completion {local}"),
        );
        progress(&format!("{context}: selection {local}"));
        let selections = s.selection_ranges(uri, &[pos]).unwrap();
        require(
            report,
            selections[0]
                .parent
                .as_ref()
                .is_some_and(|p| p.parent.is_some()),
            context,
            &format!("lost syntax selection ancestors {local}"),
        );
    }
    progress(&format!("{context}: semantic tokens"));
    let tokens = s.semantic_tokens(uri).unwrap();
    let expected_token = position(text, text.find("__tail_local :=").unwrap());
    require(
        report,
        tokens.iter().any(|t| t.position == expected_token),
        context,
        "lost semantic tokens after edit",
    );
    progress(&format!("{context}: folding"));
    let folds = s.folding_ranges(uri).unwrap();
    let tail_line = position(text, text.find("__lsp_edit_tail").unwrap()).line;
    require(
        report,
        folds.iter().any(|f| f.start_line == tail_line),
        context,
        "lost folding range after edit",
    );
    progress(&format!("{context}: ordinary-loop definition"));
    require(
        report,
        s.definition(uri, position(text, text.find("for __tail_index").unwrap()))
            .unwrap()
            .is_empty(),
        context,
        "ordinary loop gained an expansion definition",
    );
    progress(&format!("{context}: inlay hints"));
    let hints = s
        .inlay_hints(
            uri,
            Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: position(text, text.len()),
            },
        )
        .unwrap();
    let expected_hint = position(
        text,
        text.find("__tail_local :=").unwrap() + "__tail_local".len(),
    );
    require(
        report,
        hints
            .iter()
            .any(|hint| hint.position == expected_hint && hint.label == ": s64"),
        context,
        "lost inferred type hint after edit",
    );
}

/// Exercise each selected source line in a real Session, restoring between edits.
/// Baseline errors are reported separately; no missing anchor is treated as success.
pub fn check(path: &Path, source: &str, stdlib: Option<PathBuf>, max_lines: usize) -> Report {
    progress("parse baseline");
    let uri = file_uri(path);
    let text = format!("{HEAD}{source}{TAIL}");
    let parsed = jaic::parser::parse_file_ide(FileId(0), &text);
    let mut report = Report {
        file: path.display().to_string(),
        baseline_syntax_errors: parsed.diagnostics.len(),
        ..Report::default()
    };
    let mut s = Session::with_environment(Limits::default(), environment(stdlib));
    // The corpus worker checks this source file, including its real imports/loads.
    // An inferred application entry may deliberately exclude it on the host target.
    let config = file_uri(&path.parent().unwrap().join("jai.toml"));
    s.open(
        config,
        1,
        format!(
            "build_files = [{}]\n",
            serde_json::to_string(path.file_name().unwrap().to_str().unwrap()).unwrap()
        ),
    )
    .unwrap();
    s.set_auto_import(false);
    progress("open baseline");
    s.open(uri.clone(), 1, text.clone()).unwrap();
    progress("baseline diagnostics");
    report.baseline_semantic_errors = s
        .diagnostics(&uri)
        .unwrap()
        .iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .count();
    anchors(&s, &uri, &text, &mut report, "baseline");
    progress("baseline complete");
    if !report.failures.is_empty() {
        return report;
    }
    let lines: Vec<_> = source
        .split_inclusive('\n')
        .scan(HEAD.len(), |offset, line| {
            let start = *offset;
            *offset += line.len();
            Some((start, line.trim_end_matches(['\r', '\n'])))
        })
        .filter(|(_, line)| !line.trim().is_empty())
        .collect();
    let stride = if max_lines == 0 {
        1
    } else {
        lines.len().div_ceil(max_lines).max(1)
    };
    let mut version = 1;
    for &(start, line) in lines.iter().step_by(stride) {
        for cut in cuts(line) {
            let a = start + cut;
            let b = start + line.len();
            let mut edited = text.clone();
            edited.replace_range(a..b, "");
            version += 1;
            progress(&format!(
                "edit line {} cut {cut}",
                position(&text, start).line + 1
            ));
            s.change(
                &uri,
                version,
                &[TextChange {
                    range: Some(Range {
                        start: position(&text, a),
                        end: position(&text, b),
                    }),
                    range_length: Some(text[a..b].encode_utf16().count() as u32),
                    text: String::new(),
                }],
            )
            .unwrap();
            report.edits += 1;
            let context = format!(
                "line {} byte {cut} utf16 {}",
                position(&text, start).line + 1,
                line[..cut].encode_utf16().count()
            );
            anchors(&s, &uri, &edited, &mut report, &context);
            // Unaffected top-level declarations must remain in the recovered tree.
            let names: BTreeSet<_> = s
                .document_symbols(&uri)
                .unwrap()
                .into_iter()
                .map(|sym| sym.name)
                .collect();
            progress(&format!("{context}: strict scope classification"));
            let strict_edit = jaic::parser::parse_file(FileId(0), &edited).ok();
            for stmt in &parsed.file.stmts {
                if stmt.span.start as usize <= b && a <= stmt.span.end as usize {
                    continue;
                }
                if let StmtKind::Decl(d) = &stmt.kind {
                    for name in &d.names {
                        let mapped = if name.span.start as usize >= b {
                            name.span.start as usize - (b - a)
                        } else {
                            name.span.start as usize
                        };
                        // A suffix deletion can produce valid *different* syntax, e.g.
                        // `#run f();` -> `#run` followed by a declaration becomes a run
                        // statement containing that declaration. Strictly valid current
                        // scopes are the oracle; do not pretend the old scope survives.
                        if !names.contains(name.name.as_str())
                            && strict_edit.as_ref().is_some_and(|file| {
                                file.stmts.iter().any(|s| {
                                    s.span.start as usize <= a
                                        && a <= s.span.end as usize
                                        && s.span.start as usize <= mapped
                                        && mapped < s.span.end as usize
                                })
                            })
                        {
                            report.scope_changes.push(format!("{context}: {} is inside the edited statement in valid current syntax", name.name));
                            continue;
                        }
                        require(
                            &mut report,
                            names.contains(name.name.as_str()),
                            &context,
                            &format!("lost unaffected declaration {}", name.name),
                        );
                    }
                }
            }
            progress(&format!("{context}: diagnostics"));
            let diagnostics = s.diagnostics(&uri).unwrap();
            let independent_tail_line =
                position(&edited, edited.find("__lsp_edit_tail").unwrap()).line;
            for d in diagnostics
                .iter()
                .filter(|d| d.severity == DiagnosticSeverity::Error)
            {
                require(
                    &mut report,
                    d.range.start.line < independent_tail_line,
                    &context,
                    &format!("nonlocal diagnostic in independent tail: {d:?}"),
                );
            }
            version += 1;
            progress(&format!("{context}: restore source"));
            s.change(
                &uri,
                version,
                &[TextChange {
                    range: None,
                    range_length: None,
                    text: text.clone(),
                }],
            )
            .unwrap();
            anchors(&s, &uri, &text, &mut report, "restored");
            let restored: BTreeSet<_> = s
                .document_symbols(&uri)
                .unwrap()
                .into_iter()
                .map(|sym| sym.name)
                .collect();
            for stmt in &parsed.file.stmts {
                if let StmtKind::Decl(d) = &stmt.kind {
                    for name in &d.names {
                        require(
                            &mut report,
                            restored.contains(name.name.as_str()),
                            "restored",
                            &format!("lost original declaration {}", name.name),
                        );
                    }
                }
            }
        }
    }
    report
}
