//! Refactoring code actions (`refactor.extract`, `refactor.inline`, `refactor.rewrite`).
//!
//! Every action is worked out from the parse tree of the document, plus a few facts of the
//! type-checked compile: the declared types of locals, the references of a name, and the members
//! of an enum or struct. An action is offered only when its edit is safe; anything unclear (a
//! variable another statement modifies, a type nobody knows, a selection that is not whole
//! statements) is declined rather than guessed.
//!
//! The text an action writes follows jaifmt's canonical layout: one statement per line, braces
//! joined to their headers, and one indentation unit (the one the document uses) per level.
mod exprs;
mod fill;
mod ifx;
mod procedure;
mod text;
mod tree;
mod variables;

use self::text::indent_unit;
pub(crate) use self::text::{extent, stmt_extent, trim};
use self::tree::proc_bodies;
use crate::analysis::Span;
use crate::session::Session;
use crate::{CodeAction, DocumentUri, TextEdit};
use jaic::ast::walk::{Node, walk};
use jaic::ast::{Expr, File, Stmt};
use jaic::fxhash::HashMap;
use jaic::intern::Sym;
use jaic::source::{DiagnosticKind, FileId};
use std::cell::OnceCell;

type Sp = jaic::source::Span;

// The refactoring kinds.

pub(crate) const EXTRACT: &str = "refactor.extract";
pub(crate) const INLINE: &str = "refactor.inline";
pub(crate) const REWRITE: &str = "refactor.rewrite";

impl Session {
    /// "Add missing cases" as the preferred `quickfix` of a `#complete` switch's error, when the
    /// error touches `start..end` (bytes) of `uri` or the client named it.
    pub(crate) fn complete_switch_actions(
        &self,
        uri: &DocumentUri,
        start: usize,
        end: usize,
        context: &crate::lints::ActionContext,
    ) -> Vec<CodeAction> {
        if !context.wants("quickfix") {
            return Vec::new();
        }
        let Ok(doc) = self.document(uri) else {
            return Vec::new();
        };
        let incomplete: Vec<_> = self
            .with_semantic(uri, &doc.text, |analysis, path| {
                Some(analysis.check_error_spans(path, DiagnosticKind::IncompleteSwitch))
            })
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(start, end)| self.range_of(uri, Span::new(start, end)))
            .collect();
        let mut out = Vec::new();
        for diagnostic in self.check_diagnostics(uri) {
            if !incomplete.contains(&diagnostic.range) {
                continue;
            }
            let a = doc.index.byte(&doc.text, diagnostic.range.start);
            let b = doc.index.byte(&doc.text, diagnostic.range.end);
            let named = context.diagnostics.iter().any(|(code, range)| {
                code == crate::suggestion_fixes::CHECK_CODE && *range == diagnostic.range
            });
            if !named && (a > end || start > b) {
                continue;
            }
            // The error is on the switched value, which the refactoring reads as "in the switch".
            let fill = self
                .refactor_actions(uri, a, a, Pick::FillSwitch)
                .into_iter()
                .next();
            if let Some(mut fix) = fill {
                fix.kind = Some("quickfix");
                fix.is_preferred = true;
                fix.diagnostics = vec![diagnostic];
                out.push(fix);
            }
        }
        out
    }

    /// The refactorings of the `pick` kinds that apply to the selection `start..end` (bytes) of
    /// `uri`.
    pub(crate) fn refactor_actions(
        &self,
        uri: &DocumentUri,
        start: usize,
        end: usize,
        pick: Pick,
    ) -> Vec<CodeAction> {
        let Ok(doc) = self.document(uri) else {
            return Vec::new();
        };
        if !self.analysis_for(uri).is_some_and(|a| a.complete) {
            return Vec::new();
        }
        let Ok(file) = jaic::parser::parse_file(FileId(0), &doc.text) else {
            return Vec::new();
        };
        let mut bodies = Vec::new();
        proc_bodies(&file.stmts, &mut bodies);
        let mut roots: Vec<Root> = vec![Root {
            stmts: &file.stmts,
            in_proc: false,
        }];
        roots.extend(bodies.iter().map(|b| Root {
            stmts: &b.1.stmts,
            in_proc: true,
        }));
        let cx = Cx {
            session: self,
            uri,
            text: &doc.text,
            file: &file,
            roots,
            sel: trim(&doc.text, start, end),
            cursor: start,
            unit: indent_unit(&doc.text),
            nl: if doc.text.contains("\r\n") {
                "\r\n"
            } else {
                "\n"
            },
            types: OnceCell::new(),
        };
        let mut out = Vec::new();
        if pick != Pick::FillSwitch {
            cx.extract_variable(&mut out);
            cx.extract_procedure(&mut out);
            cx.inline_variable(&mut out);
        }
        if pick != Pick::AllButFillSwitch {
            cx.fill_switch(&mut out);
        }
        if pick != Pick::FillSwitch {
            cx.fill_struct(&mut out);
            cx.ifx_to_if(&mut out);
            cx.if_to_ifx(&mut out);
        }
        out
    }
}

/// Which refactorings `refactor_actions` offers.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Pick {
    All,
    /// Only "Add missing cases" of a switch.
    FillSwitch,
    /// Everything but "Add missing cases", when the quick fix already offers it.
    AllButFillSwitch,
}

struct Root<'a> {
    stmts: &'a [Stmt],
    /// The statements are a procedure's body (a place for a local to live).
    in_proc: bool,
}

struct Cx<'a> {
    session: &'a Session,
    uri: &'a DocumentUri,
    text: &'a str,
    file: &'a File,
    roots: Vec<Root<'a>>,
    /// The selection, trimmed of whitespace.
    sel: (usize, usize),
    cursor: usize,
    unit: String,
    nl: &'static str,
    /// Declared types of locals, by the start of their names.
    types: OnceCell<HashMap<usize, String>>,
}

/// A local variable that is visible where a selection starts.
struct Local {
    name: Sym,
    ty: Option<String>,
    constant: bool,
}

// Text helpers

impl<'a> Cx<'a> {
    fn ex(&self, e: &Expr) -> (usize, usize) {
        extent(self.text, e.span)
    }

    fn slice(&self, (a, b): (usize, usize)) -> &'a str {
        self.text.get(a..b).unwrap_or("")
    }

    fn edit(&self, start: usize, end: usize, new_text: String) -> Option<TextEdit> {
        Some(TextEdit {
            range: self.session.range_of(self.uri, Span::new(start, end))?,
            new_text,
        })
    }

    fn action(
        &self,
        title: String,
        kind: &'static str,
        edits: Vec<Option<TextEdit>>,
    ) -> Option<CodeAction> {
        let edits: Option<Vec<TextEdit>> = edits.into_iter().collect();
        Some(CodeAction {
            title,
            kind: Some(kind),
            edit: Some((self.uri.as_str().into(), edits?)),
            ..CodeAction::default()
        })
    }

    /// The declared type of every local (`x := 1` is `s64`), from the type-checked compile.
    fn declared(&self, at: usize) -> Option<String> {
        self.types
            .get_or_init(|| {
                self.session
                    .checked(self.uri, |a, f| a.compiler.ide_declared_types(f))
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(span, ty)| (span.start as usize, ty))
                    .collect()
            })
            .get(&at)
            .cloned()
    }

    /// The type of a declared name: as written, else as the compiler worked it out.
    fn type_of_decl(&self, d: &jaic::ast::Decl, index: usize) -> Option<String> {
        if let Some(ty) = &d.ty
            && d.names.len() == 1
        {
            return Some(self.slice(self.ex(ty)).to_string());
        }
        self.declared(d.names.get(index)?.span.start as usize)
    }

    /// The first expression (parents first) in the procedure bodies, or in every tree when
    /// `anywhere`, for which `pick` answers; with the nodes above it.
    fn find<T>(
        &self,
        anywhere: bool,
        mut pick: impl FnMut(Node<'a>, &[Node<'a>]) -> Option<T>,
    ) -> Option<(T, Vec<Node<'a>>)> {
        for root in self.roots.iter().filter(|r| anywhere || r.in_proc) {
            let mut found = None;
            walk(root.stmts, &mut |n, parents| {
                if found.is_some() {
                    return false;
                }
                if let Some(t) = pick(n, parents) {
                    found = Some((t, parents.to_vec()));
                    return false;
                }
                true
            });
            if found.is_some() {
                return found;
            }
        }
        None
    }

    /// The smallest node (an expression when `exprs`) around the cursor that `pick` accepts.
    fn innermost<T>(
        &self,
        exprs: bool,
        mut pick: impl FnMut(Node<'a>) -> Option<(T, (usize, usize))>,
    ) -> Option<(T, Vec<Node<'a>>)> {
        let mut best: Option<(usize, T, Vec<Node<'a>>)> = None;
        for root in &self.roots {
            walk(root.stmts, &mut |n, parents| {
                if n.expr().is_some() != exprs {
                    return true;
                }
                if let Some((t, (a, b))) = pick(n)
                    && a <= self.cursor
                    && self.cursor <= b
                    && best.as_ref().is_none_or(|(size, ..)| b - a <= *size)
                {
                    best = Some((b - a, t, parents.to_vec()));
                }
                true
            });
        }
        best.map(|(_, t, p)| (t, p))
    }
}
