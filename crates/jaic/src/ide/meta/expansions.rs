//! Recording macro and `#insert` expansions and the output a metaprogram produced.
use super::text::{bounded, dedent, quote};
use super::{IdeExpansion, IdeExpansionKind, MAX_EXPANSIONS, MAX_VARIANTS};
use crate::ast;
use crate::sema::lower::Operand;
use crate::sema::value::CodeId;
use crate::sema::{Compiler, Value};
use crate::source::Span;

impl Compiler {
    // Recording

    pub(super) fn ide_note_expansion(
        &mut self,
        span: Span,
        kind: IdeExpansionKind,
        text: String,
        detail: String,
        output: String,
        literal: Option<String>,
    ) {
        let text = bounded(text);
        let Some(ide) = self.ed.facts_mut() else {
            return;
        };
        if let Some(&at) = ide.expansion_index.get(&(span, kind)) {
            let e = &mut ide.expansions[at];
            if !e.texts.contains(&text) && e.texts.len() < MAX_VARIANTS {
                e.texts.push(text);
            }
            if e.output.is_empty() {
                e.output = output;
            }
            return;
        }
        if ide.expansions.len() >= MAX_EXPANSIONS {
            return;
        }
        ide.expansion_index
            .insert((span, kind), ide.expansions.len());
        ide.expansions.push(IdeExpansion {
            span,
            kind,
            texts: vec![text],
            detail,
            output: bounded(output),
            literal,
        });
    }

    /// Bytes compile-time code has printed so far (to tell what one `#run` printed).
    pub(crate) fn ide_output_mark(&self) -> usize {
        self.ed
            .facts()
            .and_then(|i| i.output.as_ref())
            .map_or(0, |o| {
                o.try_borrow()
                    .map_or(0, |h| h.stdout.len() + h.stderr.len())
            })
    }

    fn ide_output_since(&self, mark: usize) -> String {
        let Some(out) = self.ed.facts().and_then(|i| i.output.as_ref()) else {
            return String::new();
        };
        let Ok(host) = out.try_borrow() else {
            return String::new();
        };
        // Both streams in write order; `mark` counted bytes of both.
        let mut all = Vec::new();
        let (mut o, mut e) = (0, 0);
        for &(to_stderr, n) in &host.order {
            if to_stderr {
                all.extend_from_slice(&host.stderr[e..(e + n).min(host.stderr.len())]);
                e += n;
            } else {
                all.extend_from_slice(&host.stdout[o..(o + n).min(host.stdout.len())]);
                o += n;
            }
        }
        String::from_utf8_lossy(all.get(mark..).unwrap_or(&[])).into_owned()
    }

    /// `span` widened back to the directive (`#insert`, `#if`...) written before it, when one
    /// of `names` is: the operand's span is what the compiler has.
    fn ide_widen(&self, span: Span, names: &[&str]) -> Option<Span> {
        let text = &self.src.sources.try_get(span.file)?.text;
        let start = span.start as usize;
        if start > text.len() {
            return None;
        }
        let from = text.floor_char_boundary(start.saturating_sub(512));
        let before = &text[from..start];
        let mut at = before.len();
        while let Some(hash) = before[..at].rfind('#') {
            let between = &before[hash..];
            if between.contains([';', '{', '}']) {
                return None;
            }
            let word: String = between[1..]
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if names.contains(&word.as_str()) {
                return Some(Span {
                    start: (from + hash) as u32,
                    ..span
                });
            }
            at = hash;
        }
        None
    }

    /// Source text of a `Code` value: an expression, or a block's statements without braces.
    pub(crate) fn ide_code_text(&self, code: CodeId) -> String {
        match &*self.codes[code.0 as usize] {
            ast::CodeBody::Expr(e) => self.ide_snippet(e.span).trim().to_string(),
            ast::CodeBody::Stmt(s) => self.ide_snippet(s.span).trim().to_string(),
            ast::CodeBody::Block(b) => {
                let text = self.ide_snippet(b.span).trim();
                match text.strip_prefix('{').and_then(|t| t.strip_suffix('}')) {
                    Some(inner) => dedent(inner.trim_matches('\n')),
                    None => text.to_string(),
                }
            }
        }
    }

    pub(crate) fn ide_snippet(&self, span: Span) -> &str {
        let Some(file) = self.src.sources.try_get(span.file) else {
            return "";
        };
        let text = &file.text;
        let end = (span.end as usize).min(text.len());
        let start = (span.start as usize).min(end);
        text.get(start..end).unwrap_or("")
    }

    /// `#insert` evaluated `value` (a string or `Code`) to `op`.
    pub(crate) fn ide_note_insert(&mut self, value: &ast::Expr, op: &Operand) {
        if !self.ide_wants_file(value.span.file) {
            return;
        }
        let text = match op {
            Operand::Const {
                value: Value::String(s),
                ..
            } => String::from_utf8_lossy(s).into_owned(),
            Operand::Const {
                value: Value::Code(c),
                ..
            } => self.ide_code_text(*c),
            Operand::Void => String::new(),
            _ => return,
        };
        let span = self
            .ide_widen(value.span, &["insert"])
            .unwrap_or(value.span);
        self.ide_note_expansion(
            span,
            IdeExpansionKind::Insert,
            text,
            String::new(),
            String::new(),
            None,
        );
    }

    /// `#run` at `span` produced `op`; compile-time output past `mark` is what it printed.
    pub(crate) fn ide_note_run(&mut self, span: Span, op: &Operand, mark: usize) {
        if !self.ide_wants_file(span.file) {
            return;
        }
        let output = self.ide_output_since(mark);
        let (text, detail, literal) = match op {
            Operand::Const {
                value,
                ty,
                ..
            } => {
                let literal = self.ide_literal(value);
                let text = literal.clone().unwrap_or_else(|| match value {
                    Value::Proc(p) => self.proc(*p).name.to_string(),
                    Value::Code(c) => format!("#code {}", self.ide_code_text(*c)),
                    Value::Bytes(_) => format!("{}.{{…}}", self.types.name(*ty)),
                    other => other.render(&self.types),
                });
                (text, self.types.name(*ty), literal)
            }
            Operand::Type(t) => {
                let name = self.types.name(*t);
                (name.clone(), "Type".into(), Some(name))
            }
            Operand::Procs(ps) => (
                ps.first()
                    .map(|&p| self.proc(p).name.to_string())
                    .unwrap_or_default(),
                "procedure".into(),
                None,
            ),
            Operand::Void => (String::new(), "void".into(), None),
            _ => return,
        };
        self.ide_note_expansion(span, IdeExpansionKind::Run, text, detail, output, literal);
    }

    /// A constant as Jai source, when it has a literal form.
    pub(super) fn ide_literal(&self, value: &Value) -> Option<String> {
        Some(match value {
            Value::Int(v) => v.to_string(),
            Value::Float(v, _) => {
                let text = format!("{v}");
                if text.contains(['.', 'e', 'i', 'N']) {
                    text
                } else {
                    format!("{text}.0")
                }
            }
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".into(),
            Value::String(s) => quote(&String::from_utf8_lossy(s)),
            Value::Type(t) => self.types.name(*t),
            _ => return None,
        })
    }

    /// A static condition (`#if`, `#ifx`, `#assert`) at `cond` evaluated to `holds`.
    pub(crate) fn ide_note_condition(&mut self, cond: &ast::Expr, holds: bool) {
        if !self.ide_wants_file(cond.span.file) {
            return;
        }
        let Some(span) = self.ide_widen(cond.span, &["if", "ifx", "assert"]) else {
            return;
        };
        self.ide_note_expansion(
            span,
            IdeExpansionKind::If,
            holds.to_string(),
            String::new(),
            String::new(),
            None,
        );
    }
}
