//! The file-level structure the declaration rows do not hold: scope sections, `#if`/`#else`
//! blocks, `#run`, `#insert`, `#placeholder` and `#load`. `Session::document_symbols` nests the
//! rows and these sections by the spans they cover.
use super::{Analysis, Span, span_of};
use crate::SymbolKind;
use jaic::ast::{File, ScopeKind, Stmt, StmtKind};

/// A part of the outline that is not a declaration.
#[derive(Clone, Debug)]
pub(crate) struct Section {
    pub name: String,
    pub detail: String,
    pub kind: SymbolKind,
    /// Everything the section covers: the declarations inside lie within it.
    pub location: Span,
    /// The directive or keyword that starts it (always within `location`).
    pub selection: Span,
}

/// An expression's source on one line, cut at `max` characters.
fn excerpt(text: &str, span: jaic::source::Span, max: usize) -> String {
    let source = text
        .get(span.start as usize..span.end as usize)
        .unwrap_or("");
    let mut out = String::new();
    for word in source.split_whitespace() {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(word);
        if out.chars().count() > max {
            break;
        }
    }
    if out.chars().count() > max {
        out = out.chars().take(max).collect();
        out.push('…');
    }
    out
}

impl Analysis {
    pub(super) fn outline_of(&mut self, file: &File, text: &str) {
        let mut sections = Vec::new();
        collect(&file.stmts, text.len(), text, &mut sections);
        self.outline = sections;
    }
}

/// The sections of `stmts`, a list that ends at `end`: a scope directive holds until the next
/// one in the list.
fn collect(stmts: &[Stmt], end: usize, text: &str, out: &mut Vec<Section>) {
    for (at, stmt) in stmts.iter().enumerate() {
        let span = span_of(stmt.span);
        match &stmt.kind {
            StmtKind::Scope(kind) => {
                let closes = stmts[at + 1..]
                    .iter()
                    .find(|s| matches!(s.kind, StmtKind::Scope(_)))
                    .map_or(end, |s| s.span.start as usize);
                let name = match kind {
                    ScopeKind::File => "#scope_file",
                    ScopeKind::Export => "#scope_export",
                    ScopeKind::Module => "#scope_module",
                };
                out.push(Section {
                    name: name.into(),
                    detail: String::new(),
                    kind: SymbolKind::Namespace,
                    location: Span::new(span.start, closes.max(span.end)),
                    selection: span,
                });
            }
            StmtKind::StaticIf(branches) => {
                let then = span_of(branches.then_arm);
                let condition = span_of(branches.cond.span);
                out.push(Section {
                    name: format!("#if {}", excerpt(text, branches.cond.span, 48)),
                    detail: String::new(),
                    kind: SymbolKind::Boolean,
                    location: Span::new(span.start.min(then.start), span.end.max(then.end)),
                    selection: Span::new(
                        then.start,
                        condition.end.clamp(then.start, span.end.max(then.end)),
                    ),
                });
                collect(&branches.then_branch, then.end, text, out);
                if let Some(arm) = branches.else_arm {
                    let arm = span_of(arm);
                    out.push(Section {
                        name: "#else".into(),
                        detail: String::new(),
                        kind: SymbolKind::Boolean,
                        location: arm,
                        selection: Span::new(arm.start, (arm.start + 4).min(arm.end)),
                    });
                    collect(&branches.else_branch, arm.end, text, out);
                }
            }
            StmtKind::Run(expression) => out.push(Section {
                name: format!("#run {}", excerpt(text, expression.span, 48)),
                detail: String::new(),
                kind: SymbolKind::Event,
                location: span,
                selection: Span::new(span.start, (span.start + 4).min(span.end)),
            }),
            StmtKind::Insert(insert) => out.push(Section {
                name: format!("#insert {}", excerpt(text, insert.value.span, 48)),
                detail: String::new(),
                kind: SymbolKind::Event,
                location: span,
                selection: Span::new(span.start, (span.start + 7).min(span.end)),
            }),
            StmtKind::Placeholder(names) => {
                for name in names {
                    out.push(Section {
                        name: name.name.as_str().into(),
                        detail: "#placeholder".into(),
                        kind: SymbolKind::Variable,
                        location: span_of(name.span),
                        selection: span_of(name.span),
                    });
                }
            }
            StmtKind::Load {
                path,
                span: at,
            } => out.push(Section {
                name: format!("#load \"{path}\""),
                detail: String::new(),
                kind: SymbolKind::File,
                location: Span::new(
                    span.start.min(span_of(*at).start),
                    span.end.max(span_of(*at).end),
                ),
                selection: span_of(*at),
            }),
            _ => {}
        }
    }
}
